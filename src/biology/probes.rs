//! Mechanics diagnosis probes (TCK-172): measurement only, no model change.
//! Every probe is deterministic for a given text seed (all randomness from `SimRng` sub-streams)
//! and returns a JSON report with its parameters and numbers. Run them via `trainer_runner probe`.

use std::collections::{BTreeMap, BTreeSet};

use rand::Rng;
use rayon::prelude::*;
use serde::Serialize;

use crate::biology::creature_factory::{spawn_creature, SpawnOptions};
use crate::biology::dna::{
    extract_raw_gene_payloads, generate_random_genome, generate_structured_genome, mutate_genome, mutate_genome_with_kind, normalize_genome,
    parse_genome, read_evolution_rates, CreaturePhenotype, MutationKind, MutationOperators, NeuronRole, OrganSide, CHANNELS_PER_ORGANELLE,
    OUTPUT_MOTOR_NODES_COUNT, WILDTYPE_GENOME,
};
use crate::biology::scenarios::{Scenario, TrainingScenarioPlugin};
use crate::biology::trainer_engine::{init_rust_sandbox, step_trainer_sandbox_physics_with_config, SandboxOrigin};
use crate::biology::trainer_sim::{TrainerEvent, TrainerParams, TrainerSim, IMMOBILE_DISTANCE_PX};
use crate::shared::brain::{compute_sensory_inputs_into, execute_brain_with_learning};
use crate::shared::fmath;
use crate::shared::physics::step_creature_kinematics;
use crate::shared::rng::{derive, seed_from_str, SimRng};
use crate::shared::types::{AppConfig, CreatureAgent, FoodSpore};

pub const PROBE_NAMES: &[&str] = &["heredity", "speed", "equipment", "steering", "wall", "hebbian", "topology", "metabolism"];

/// Length of the letter-soup genomes (the random immigrants before TCK-184).
const LETTER_SOUP_LENGTH: usize = 256;
/// Organs with a spectral affinity from here on detect food (light/chemical branch of `compute_sensory_inputs`).
const FOOD_SENSING_AFFINITY: f32 = 0.25;
/// Arena edge for wall-free kinematics (creature starts in the centre and never reaches a wall).
const OPEN_ARENA: f32 = 10_000_000.0;
/// Ticks of an exploration epoch; also the settle time of the speed probe.
const LONG_EPOCH_TICKS: u32 = 1800;

/// Sub-stream ids per probe, so every probe is independent of the others (`probe all` == single runs).
const STREAM_RANDOM_GENOMES: u64 = 1;
const STREAM_HEREDITY: u64 = 2;
const STREAM_EQUIPMENT: u64 = 3;
const STREAM_WALL: u64 = 5;
const STREAM_HEBBIAN: u64 = 6;
const STREAM_STRUCTURED_GENOMES: u64 = 7;
const STREAM_METABOLISM: u64 = 8;

/// Population labels: random letters (no gene structure) vs `generate_structured_genome` (trainer generation 1).
const LETTER_SOUP: &str = "letter_soup";
const STRUCTURED: &str = "structured";

/// Inputs shared by all probes.
#[derive(Debug, Clone)]
pub struct ProbeOptions {
    pub seed: String,
    /// Overrides the probe's default number of random genomes
    pub samples: Option<usize>,
    /// Hall-of-fame genomes (best first), empty without a DB
    pub hof: Vec<HofGenome>,
    /// Where the HoF genomes come from (DB path), for the report
    pub hof_source: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HofGenome {
    pub genome: String,
    pub fitness: f32,
    pub run_id: String,
    pub generation: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProbeReport<T: Serialize> {
    pub probe: String,
    pub seed: String,
    pub hof_source: Option<String>,
    pub hof_genomes: usize,
    pub result: T,
}

/// Runs one probe by name and returns its JSON report.
pub fn run_probe(name: &str, opts: &ProbeOptions) -> Result<serde_json::Value, String> {
    let result = match name {
        "heredity" => serde_json::to_value(probe_heredity(opts)),
        "speed" => serde_json::to_value(probe_speed(opts)),
        "equipment" => serde_json::to_value(probe_equipment(opts)),
        "steering" => serde_json::to_value(probe_steering(opts)),
        "wall" => serde_json::to_value(probe_wall(opts)),
        "hebbian" => serde_json::to_value(probe_hebbian(opts)),
        "topology" => serde_json::to_value(probe_topology(opts)),
        "metabolism" => serde_json::to_value(probe_metabolism(opts)),
        other => return Err(format!("unknown probe '{}' (known: {}, all)", other, PROBE_NAMES.join(", "))),
    }
    .map_err(|e| e.to_string())?;
    let report = ProbeReport {
        probe: name.to_string(),
        seed: opts.seed.clone(),
        hof_source: opts.hof_source.clone(),
        hof_genomes: opts.hof.len(),
        result,
    };
    serde_json::to_value(report).map_err(|e| e.to_string())
}

/// Best distinct trainer genomes of all runs, opened strictly read-only.
pub fn load_hof_genomes(db_path: &str, limit: usize) -> Result<Vec<HofGenome>, String> {
    use rusqlite::{Connection, OpenFlags};
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX)
        .map_err(|e| format!("cannot open DB {} read-only: {}", db_path, e))?;
    let mut stmt = conn
        .prepare("SELECT genome, fitness, run_id, generation FROM trainer_genomes ORDER BY fitness DESC, id ASC")
        .map_err(|e| format!("cannot read trainer_genomes: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok(HofGenome {
                genome: row.get(0)?,
                fitness: row.get::<_, f64>(1)? as f32,
                run_id: row.get(2)?,
                generation: row.get::<_, i64>(3)? as u32,
            })
        })
        .map_err(|e| format!("cannot read trainer_genomes: {}", e))?;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for row in rows {
        let record = row.map_err(|e| e.to_string())?;
        if !record.genome.is_empty() && seen.insert(record.genome.clone()) {
            out.push(record);
            if out.len() >= limit {
                break;
            }
        }
    }
    Ok(out)
}

// ==========================================================================
// Shared helpers
// ==========================================================================

/// p10/p25/p50/p75/p90 (linear interpolation), mean, min, max.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Distribution {
    pub n: usize,
    pub mean: f32,
    pub min: f32,
    pub p10: f32,
    pub p25: f32,
    pub p50: f32,
    pub p75: f32,
    pub p90: f32,
    pub max: f32,
}

impl Distribution {
    pub fn of(values: &[f32]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let q = |q: f32| {
            let pos = q * (sorted.len() - 1) as f32;
            let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
            sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f32)
        };
        Some(Self {
            n: sorted.len(),
            mean: sorted.iter().map(|&v| v as f64).sum::<f64>() as f32 / sorted.len() as f32,
            min: sorted[0],
            p10: q(0.1),
            p25: q(0.25),
            p50: q(0.5),
            p75: q(0.75),
            p90: q(0.9),
            max: sorted[sorted.len() - 1],
        })
    }
}

fn share(count: usize, total: usize) -> f32 {
    if total == 0 { 0.0 } else { count as f32 / total as f32 }
}

fn share_where<T>(items: &[T], pred: impl Fn(&T) -> bool) -> f32 {
    share(items.iter().filter(|i| pred(i)).count(), items.len())
}

/// Pearson correlation; `None` for fewer than 2 pairs or zero variance.
pub fn pearson(pairs: &[(f32, f32)]) -> Option<f32> {
    if pairs.len() < 2 {
        return None;
    }
    let n = pairs.len() as f64;
    let (mx, my) = pairs.iter().fold((0.0, 0.0), |(sx, sy), &(x, y)| (sx + x as f64, sy + y as f64));
    let (mx, my) = (mx / n, my / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for &(x, y) in pairs {
        let (dx, dy) = (x as f64 - mx, y as f64 - my);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 { None } else { Some((sxy / (sxx * syy).sqrt()) as f32) }
}

/// The same letter-soup genomes for every probe: `derive(seed, [STREAM_RANDOM_GENOMES, i])`.
pub fn letter_soup_genomes(seed: u64, n: usize) -> Vec<String> {
    (0..n).map(|i| generate_random_genome(LETTER_SOUP_LENGTH, &mut derive(seed, &[STREAM_RANDOM_GENOMES, i as u64]))).collect()
}

/// The same structured genomes (config `genomeComposition`, as trainer generation 1) for every probe.
pub fn structured_genomes(seed: u64, n: usize) -> Vec<String> {
    let composition = &AppConfig::global().genome_composition;
    (0..n).map(|i| generate_structured_genome(composition, &mut derive(seed, &[STREAM_STRUCTURED_GENOMES, i as u64]))).collect()
}

/// Both random populations, letter soup first.
fn random_populations(seed: u64, n: usize) -> [(&'static str, Vec<String>); 2] {
    [(LETTER_SOUP, letter_soup_genomes(seed, n)), (STRUCTURED, structured_genomes(seed, n))]
}

fn agent_at(genome: &str, px: f32, py: f32, heading_angle: f32) -> CreatureAgent {
    spawn_creature(genome, SpawnOptions { id: 1, px, py, heading_angle, generation: 1, methylations: None, synapse_weights: None })
}

fn speed_of(agent: &CreatureAgent) -> f32 {
    (agent.vx * agent.vx + agent.vy * agent.vy).sqrt()
}

fn wrap_angle(mut a: f32) -> f32 {
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

/// One tick of sensing, brain (with the config's Hebbian rate) and kinematics, food fixed.
/// Mirrors `scenarios::step_physics` without food drift; returns the motor outputs and the wall flag.
fn brain_tick(agent: &mut CreatureAgent, foods: &[FoodSpore], size: f32, config: &AppConfig, inputs: &mut Vec<f32>) -> ([f32; OUTPUT_MOTOR_NODES_COUNT], bool) {
    let clock_val = 0.5 + 0.5 * fmath::sin((agent.age as f32) * 0.1);
    compute_sensory_inputs_into(agent, clock_val, foods, &[], size, size, inputs);
    let outputs = execute_brain_with_learning(
        &agent.phenotype.brain,
        inputs,
        &mut agent.neuron_states,
        &mut agent.neuron_activations,
        &mut agent.synapse_weights,
        &mut agent.brain_runtime,
        config.rules.hebbian_learning_rate_base,
    );
    let hit_wall = step_creature_kinematics(agent, outputs[0], outputs[1], config, &[], size, size);
    (outputs, hit_wall)
}

/// Whether any light/chemical organ registers food in `inputs` (from `compute_sensory_inputs`).
fn senses_food(phenotype: &CreaturePhenotype, inputs: &[f32]) -> bool {
    phenotype.organelles.iter().enumerate().any(|(idx, organ)| {
        organ.spectral_affinity >= FOOD_SENSING_AFFINITY && inputs[idx * CHANNELS_PER_ORGANELLE..(idx + 1) * CHANNELS_PER_ORGANELLE].iter().any(|&v| v > 0.0)
    })
}

/// Limbs as counted by `physics::locomotion_drive` (part of the speed band).
fn limb_count(phenotype: &CreaturePhenotype) -> usize {
    phenotype.organelles.iter().filter(|o| o.expression_style >= 0.72).count()
}

// ==========================================================================
// 1. Heredity
// ==========================================================================

/// Body traits compared parent vs child (all inputs of the thrust/mass/drag formula plus stomach and diet).
pub const HEREDITY_TRAITS: &[&str] = &[
    "pulse_speed", "stiffness", "mean_radius", "base_length", "wave_phase", "parapodia_amp", "limbs", "organs",
    "stomach_capacity", "carnivory", "terminal_speed",
];
pub const HEREDITY_MUTATION_COUNTS: &[usize] = &[1, 5, 15];

/// Phenotype features one heredity comparison needs.
struct HeredityProfile {
    /// Non-reflex synapses (source is not a sensor): key -> (weight, owned by an `SY` gene)
    synapses: BTreeMap<u64, (f32, bool)>,
    traits: Vec<f32>,
    neurons: usize,
    organs: usize,
}

fn heredity_profile(genome: &str) -> HeredityProfile {
    let phenotype = parse_genome(genome, None, None);
    let synapses = phenotype
        .brain
        .synapses
        .iter()
        .filter(|s| !matches!(phenotype.brain.neurons[s.from_node].role, NeuronRole::Sensor { .. }))
        .map(|s| (s.key, (s.weight, phenotype.genes[s.owner].gene.promoter == "SY")))
        .collect();
    let sh = &phenotype.spinal_harmonics;
    let traits = vec![
        phenotype.pulse_speed,
        phenotype.stiffness,
        sh.mean_radius,
        sh.base_length,
        phenotype.wave_phase,
        sh.parapodia_amp,
        limb_count(&phenotype) as f32,
        phenotype.organelles.len() as f32,
        phenotype.stomach_capacity,
        phenotype.carnivory,
        straight_run(genome).terminal_speed,
    ];
    HeredityProfile { synapses, traits, neurons: phenotype.brain.neurons.len(), organs: phenotype.organelles.len() }
}

#[derive(Debug, Default)]
struct SynapseTally {
    pairs: Vec<(f32, f32)>,
    parent_synapses: usize,
    retained: usize,
    unchanged: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SynapseHeredity {
    /// Synapses of all parents (summed over children)
    pub parent_synapses: usize,
    /// Pearson correlation of parent vs child weight over homologous synapses (same key), pooled
    pub pearson: Option<f32>,
    /// Share of parent synapses with the same key in the child
    pub retained_share: f32,
    /// Share of parent synapses with the same key and the identical weight in the child
    pub unchanged_share: f32,
}

impl SynapseTally {
    fn summary(&self) -> SynapseHeredity {
        SynapseHeredity {
            parent_synapses: self.parent_synapses,
            pearson: pearson(&self.pairs),
            retained_share: share(self.retained, self.parent_synapses),
            unchanged_share: share(self.unchanged, self.parent_synapses),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TraitChange {
    pub name: String,
    /// Mean |child - parent| / |parent| over children whose parent value is non-zero
    pub mean_abs_rel_change: Option<f32>,
    pub share_changed: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HeredityCell {
    /// A forced `MutationKind` label, mixed (`mutate_genome`, the trainer's operator mix) or legacy (the pre-TCK-184 mix)
    pub kind: String,
    pub mutations: usize,
    pub children: usize,
    /// Mean number of mutation ops actually applied (forced kinds skip ops outside the length limits)
    pub mean_applied_ops: f32,
    pub exuberant: SynapseHeredity,
    pub explicit: SynapseHeredity,
    pub traits: Vec<TraitChange>,
    /// Share of children with |terminal speed ratio| >= 2 (either direction)
    pub terminal_speed_changed_2x_share: f32,
    pub neuron_count_changed_share: f32,
    pub mean_abs_neuron_count_delta: f32,
    pub organ_count_changed_share: f32,
    pub genome_identical_share: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HeredityGroup {
    pub parents: String,
    pub parent_count: usize,
    pub children_per_parent_and_cell: usize,
    pub cells: Vec<HeredityCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HeredityReport {
    pub traits: Vec<String>,
    pub notes: Vec<String>,
    /// Kind composition of `mutate_genome` ops over all mixed cells
    pub mixed_op_shares: BTreeMap<String, f32>,
    /// Kind composition of the legacy character mix over all legacy cells
    pub legacy_op_shares: BTreeMap<String, f32>,
    pub groups: Vec<HeredityGroup>,
}

#[derive(Clone, Copy)]
enum HeredityKind {
    Forced(MutationKind),
    Mixed,
    Legacy,
}

impl HeredityKind {
    fn label(self) -> &'static str {
        match self {
            HeredityKind::Forced(kind) => kind.label(),
            HeredityKind::Mixed => "mixed",
            HeredityKind::Legacy => "legacy",
        }
    }
}

/// Per-kind op counts in `MutationKind::ALL` order.
type OpCounts = [usize; MutationKind::ALL.len()];

fn kind_index(kind: MutationKind) -> usize {
    MutationKind::ALL.iter().position(|&k| k == kind).expect("kind in ALL")
}

/// The pre-TCK-184 operator mix of `mutate_genome`: deletion and insertion with probability EVO rate x 0.5 each
/// (within the length bounds), otherwise a random point mutation.
fn legacy_mutate(genome: &str, ops: &MutationOperators, rng: &mut SimRng) -> Option<(String, MutationKind)> {
    let (insertion_rate, deletion_rate) = read_evolution_rates(genome);
    let roll = rng.gen_range(0.0..1.0);
    let kind = if roll < deletion_rate * 0.5 && genome.len() > 128 {
        MutationKind::Deletion
    } else if roll < (deletion_rate + insertion_rate) * 0.5 && genome.len() < 512 {
        MutationKind::Insertion
    } else {
        MutationKind::Point
    };
    mutate_genome_with_kind(genome, kind, ops, rng).map(|g| (g, kind))
}

/// Applies `count` ops; returns the child, the applied op count and (mixed/legacy only) per-kind op counts.
fn mutate_n(parent: &str, kind: HeredityKind, count: usize, rng: &mut SimRng) -> (String, usize, OpCounts) {
    let ops_config = &AppConfig::global().mutation_operators;
    let mut genome = parent.to_string();
    let (mut applied, mut ops) = (0, OpCounts::default());
    for _ in 0..count {
        let next = match kind {
            HeredityKind::Forced(k) => mutate_genome_with_kind(&genome, k, ops_config, rng).map(|g| (g, k)),
            HeredityKind::Mixed => mutate_genome(&genome, ops_config, rng).map(|m| (m.genome, m.kind)),
            HeredityKind::Legacy => legacy_mutate(&genome, ops_config, rng),
        };
        if let Some((next, k)) = next {
            if !matches!(kind, HeredityKind::Forced(_)) {
                ops[kind_index(k)] += 1;
            }
            genome = next;
            applied += 1;
        }
    }
    (genome, applied, ops)
}

/// Child statistics against one parent.
struct ChildComparison {
    exuberant: SynapseTally,
    explicit: SynapseTally,
    traits: Vec<(f32, f32)>,
    neuron_delta: usize,
    organ_changed: bool,
    identical: bool,
    applied: usize,
    ops: OpCounts,
}

fn compare(parent: &HeredityProfile, parent_genome: &str, child_genome: &str, applied: usize, ops: OpCounts) -> ChildComparison {
    let child = heredity_profile(child_genome);
    let (mut exuberant, mut explicit) = (SynapseTally::default(), SynapseTally::default());
    for (key, &(pw, is_explicit)) in &parent.synapses {
        let tally = if is_explicit { &mut explicit } else { &mut exuberant };
        tally.parent_synapses += 1;
        if let Some(&(cw, _)) = child.synapses.get(key) {
            tally.retained += 1;
            tally.pairs.push((pw, cw));
            if pw == cw {
                tally.unchanged += 1;
            }
        }
    }
    ChildComparison {
        exuberant,
        explicit,
        traits: parent.traits.iter().copied().zip(child.traits.iter().copied()).collect(),
        neuron_delta: parent.neurons.abs_diff(child.neurons),
        organ_changed: parent.organs != child.organs,
        identical: parent_genome == child_genome,
        applied,
        ops,
    }
}

fn heredity_cell(kind: HeredityKind, mutations: usize, comparisons: Vec<ChildComparison>) -> HeredityCell {
    let n = comparisons.len();
    let (mut exuberant, mut explicit) = (SynapseTally::default(), SynapseTally::default());
    for c in &comparisons {
        for (total, part) in [(&mut exuberant, &c.exuberant), (&mut explicit, &c.explicit)] {
            total.pairs.extend_from_slice(&part.pairs);
            total.parent_synapses += part.parent_synapses;
            total.retained += part.retained;
            total.unchanged += part.unchanged;
        }
    }
    let traits = HEREDITY_TRAITS
        .iter()
        .enumerate()
        .map(|(t, name)| {
            let rel: Vec<f32> = comparisons.iter().map(|c| c.traits[t]).filter(|(p, _)| *p != 0.0).map(|(p, c)| (c - p).abs() / p.abs()).collect();
            TraitChange {
                name: name.to_string(),
                mean_abs_rel_change: if rel.is_empty() { None } else { Some(rel.iter().sum::<f32>() / rel.len() as f32) },
                share_changed: share_where(&comparisons, |c| c.traits[t].0 != c.traits[t].1),
            }
        })
        .collect();
    let speed_idx = HEREDITY_TRAITS.len() - 1;
    HeredityCell {
        kind: kind.label().to_string(),
        mutations,
        children: n,
        mean_applied_ops: comparisons.iter().map(|c| c.applied as f32).sum::<f32>() / n.max(1) as f32,
        exuberant: exuberant.summary(),
        explicit: explicit.summary(),
        traits,
        terminal_speed_changed_2x_share: share_where(&comparisons, |c| {
            let (p, ch) = c.traits[speed_idx];
            p > 0.0 && ch > 0.0 && (ch / p).ln().abs() >= std::f32::consts::LN_2
        }),
        neuron_count_changed_share: share_where(&comparisons, |c| c.neuron_delta > 0),
        mean_abs_neuron_count_delta: comparisons.iter().map(|c| c.neuron_delta as f32).sum::<f32>() / n.max(1) as f32,
        organ_count_changed_share: share_where(&comparisons, |c| c.organ_changed),
        genome_identical_share: share_where(&comparisons, |c| c.identical),
    }
}

/// Op counts of all mixed and all legacy cells.
#[derive(Default)]
struct MixTally {
    mixed: OpCounts,
    legacy: OpCounts,
}

/// Heredity of `parents` for the given mutation kinds and counts; `children` children per parent and cell.
fn heredity_group(label: &str, parents: &[String], children: usize, counts: &[usize], seed: u64, group_id: u64, mix: &mut MixTally) -> HeredityGroup {
    let kinds: Vec<HeredityKind> =
        MutationKind::ALL.iter().map(|&k| HeredityKind::Forced(k)).chain([HeredityKind::Mixed, HeredityKind::Legacy]).collect();
    let profiles: Vec<HeredityProfile> = parents.par_iter().map(|g| heredity_profile(g)).collect();
    let mut cells = Vec::new();
    for (kind_idx, &kind) in kinds.iter().enumerate() {
        for &count in counts {
            let jobs: Vec<(usize, usize)> = (0..parents.len()).flat_map(|p| (0..children).map(move |c| (p, c))).collect();
            let comparisons: Vec<ChildComparison> = jobs
                .par_iter()
                .map(|&(p, c)| {
                    let mut rng = derive(seed, &[STREAM_HEREDITY, group_id, kind_idx as u64, count as u64, p as u64, c as u64]);
                    let (child, applied, ops) = mutate_n(&parents[p], kind, count, &mut rng);
                    compare(&profiles[p], &parents[p], &child, applied, ops)
                })
                .collect();
            let tally = match kind {
                HeredityKind::Mixed => Some(&mut mix.mixed),
                HeredityKind::Legacy => Some(&mut mix.legacy),
                HeredityKind::Forced(_) => None,
            };
            if let Some(tally) = tally {
                for c in &comparisons {
                    for (t, n) in tally.iter_mut().zip(c.ops) {
                        *t += n;
                    }
                }
            }
            cells.push(heredity_cell(kind, count, comparisons));
        }
    }
    HeredityGroup { parents: label.to_string(), parent_count: parents.len(), children_per_parent_and_cell: children, cells }
}

/// Parent-child similarity after 1/5/15 mutations per kind, for letter-soup, structured, wildtype and HoF parents.
pub fn probe_heredity(opts: &ProbeOptions) -> HeredityReport {
    heredity_with_counts(opts, HEREDITY_MUTATION_COUNTS)
}

fn heredity_with_counts(opts: &ProbeOptions, counts: &[usize]) -> HeredityReport {
    let seed = seed_from_str(&opts.seed);
    let n = opts.samples.unwrap_or(200);
    let mut mix = MixTally::default();
    let mut groups = vec![
        heredity_group(LETTER_SOUP, &letter_soup_genomes(seed, n), 4, counts, seed, 0, &mut mix),
        heredity_group("wildtype", &[WILDTYPE_GENOME.to_string()], 400, counts, seed, 1, &mut mix),
        heredity_group(STRUCTURED, &structured_genomes(seed, n), 4, counts, seed, 3, &mut mix),
    ];
    if !opts.hof.is_empty() {
        let hof: Vec<String> = opts.hof.iter().map(|h| h.genome.clone()).collect();
        let per_parent = (800 / hof.len()).clamp(4, 400);
        groups.push(heredity_group("hof", &hof, per_parent, counts, seed, 2, &mut mix));
    }
    let shares = |ops: OpCounts| -> BTreeMap<String, f32> {
        let total: usize = ops.iter().sum();
        MutationKind::ALL.iter().zip(ops).map(|(k, n)| (k.label().to_string(), share(n, total))).collect()
    };
    HeredityReport {
        traits: HEREDITY_TRAITS.iter().map(|t| t.to_string()).collect(),
        notes: vec![
            "Homologous synapse = same synapse key (source and target role) in parent and child; reflex synapses (sensor sources) are excluded.".to_string(),
            "explicit = synapse owned by an SY gene; exuberant = all other non-reflex synapses (gene-tape weights).".to_string(),
            "terminal_speed: straight run at full thrust for 1800 ticks (see speed probe).".to_string(),
            "The trainer applies 1 to mutationOpsMax mixed ops per mutant child (config mutationOpsMin/mutationOpsMax, adapted by the 1/5 rule; TCK-174).".to_string(),
            "mixed = mutate_genome (TCK-184 operator mix, config mutationOperators; EVO scales the indel weights); legacy = the pre-TCK-184 character mix (EVO-rate indels, otherwise a random point mutation).".to_string(),
            "Forced kinds skip ops that are not applicable (length bounds, no suitable gene); mixed falls back to creep instead.".to_string(),
        ],
        mixed_op_shares: shares(mix.mixed),
        legacy_op_shares: shares(mix.legacy),
        groups,
    }
}

// ==========================================================================
// 2. Speed
// ==========================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StraightRun {
    /// Speed after `LONG_EPOCH_TICKS` at full thrust from rest
    pub terminal_speed: f32,
    /// Path length after 300 / 1800 ticks from rest
    pub distance_300: f32,
    pub distance_1800: f32,
    /// Relative speed change over the last 100 ticks is below 1 %
    pub settled: bool,
}

/// Full constant thrust, no turn, from rest in an open arena.
pub fn straight_run(genome: &str) -> StraightRun {
    let config = AppConfig::global();
    let mut agent = agent_at(genome, OPEN_ARENA / 2.0, OPEN_ARENA / 2.0, 0.0);
    let (mut distance, mut distance_300, mut speed_at_1700) = (0.0, 0.0, 0.0);
    for tick in 1..=LONG_EPOCH_TICKS {
        step_creature_kinematics(&mut agent, 1.0, 0.0, config, &[], OPEN_ARENA, OPEN_ARENA);
        distance += speed_of(&agent);
        if tick == 300 {
            distance_300 = distance;
        }
        if tick == LONG_EPOCH_TICKS - 100 {
            speed_at_1700 = speed_of(&agent);
        }
    }
    let terminal_speed = speed_of(&agent);
    StraightRun {
        terminal_speed,
        distance_300,
        distance_1800: distance,
        settled: (terminal_speed - speed_at_1700).abs() <= 0.01 * terminal_speed.max(1e-6),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TurnStats {
    pub band: String,
    pub genomes: usize,
    /// Steady-state radius v / omega at full thrust and full turn (px)
    pub radius_px: Option<Distribution>,
    /// Heading change per tick (degrees)
    pub turn_rate_deg_per_tick: Option<Distribution>,
    /// Ticks for a 90 degree turn at that rate
    pub ticks_per_90_deg: Option<Distribution>,
    /// Speed while turning / straight terminal speed
    pub speed_ratio_turning: Option<Distribution>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpeedSet {
    pub genomes: String,
    pub n: usize,
    pub terminal_speed: Option<Distribution>,
    pub p90_p10_ratio: Option<f32>,
    pub share_below_0_2: f32,
    /// Cannot cover 1200 px in 1800 ticks, straight line from rest (fixed speed yardstick of the TCK-188 target; 1200 px was the pre-P7 large-room re-trial distance, the room now uses 500 px)
    pub share_below_1200px_in_1800_ticks: f32,
    /// Cannot cover 200 px (standard trial min distance) in 300 ticks, straight line from rest
    pub share_below_200px_in_300_ticks: f32,
    pub share_not_settled: f32,
    pub turning: Vec<TurnStats>,
    /// Thrust 0, full bend, from rest (TCK-150)
    pub turn_at_rest: TurnAtRest,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TurnAtRest {
    /// Ticks until the heading has changed by 90 degrees (genomes that got there)
    pub ticks_to_90_deg: Option<Distribution>,
    /// Share of genomes turning 90 degrees within 1800 ticks
    pub share_reached: f32,
}

/// Thrust 0 and full bend from rest: ticks until the heading has turned 90 degrees (None within 1800 ticks).
fn turn_from_rest(genome: &str) -> Option<u32> {
    let config = AppConfig::global();
    let mut agent = agent_at(genome, OPEN_ARENA / 2.0, OPEN_ARENA / 2.0, 0.0);
    let mut turned = 0.0f32;
    for tick in 1..=LONG_EPOCH_TICKS {
        let before = agent.heading_angle;
        step_creature_kinematics(&mut agent, 0.0, 1.0, config, &[], OPEN_ARENA, OPEN_ARENA);
        turned += wrap_angle(agent.heading_angle - before).abs();
        if turned >= std::f32::consts::FRAC_PI_2 {
            return Some(tick);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpeedReport {
    pub notes: Vec<String>,
    pub sets: Vec<SpeedSet>,
}

/// Steady turn at full thrust and full bend: (radius px, deg/tick, speed while turning).
fn full_turn(genome: &str) -> (f32, f32, f32) {
    let config = AppConfig::global();
    let mut agent = agent_at(genome, OPEN_ARENA / 2.0, OPEN_ARENA / 2.0, 0.0);
    let (window_start, ticks) = (LONG_EPOCH_TICKS - 200, LONG_EPOCH_TICKS);
    let (mut turned, mut path) = (0.0f32, 0.0f32);
    for tick in 1..=ticks {
        let before = agent.heading_angle;
        step_creature_kinematics(&mut agent, 1.0, 1.0, config, &[], OPEN_ARENA, OPEN_ARENA);
        if tick > window_start {
            turned += wrap_angle(agent.heading_angle - before).abs();
            path += speed_of(&agent);
        }
    }
    let window = (ticks - window_start) as f32;
    let omega = turned / window;
    let speed = path / window;
    (if omega > 0.0 { speed / omega } else { f32::INFINITY }, omega.to_degrees(), speed)
}

fn speed_set(label: &str, genomes: &[String]) -> SpeedSet {
    let runs: Vec<StraightRun> = genomes.par_iter().map(|g| straight_run(g)).collect();
    let speeds: Vec<f32> = runs.iter().map(|r| r.terminal_speed).collect();
    let dist = Distribution::of(&speeds);

    // Turning per speed band (genomes between the given terminal-speed quantiles)
    let mut order: Vec<usize> = (0..genomes.len()).collect();
    order.sort_by(|&a, &b| speeds[a].partial_cmp(&speeds[b]).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(&b)));
    let turning = [("p05-p15 (slow)", 0.05, 0.15), ("p45-p55 (median)", 0.45, 0.55), ("p85-p95 (fast)", 0.85, 0.95)]
        .iter()
        .map(|&(band, lo, hi)| {
            let n = order.len();
            let members: Vec<usize> = order[((lo * n as f32) as usize).min(n)..((hi * n as f32).ceil() as usize).min(n)].to_vec();
            let turns: Vec<(usize, (f32, f32, f32))> = members.par_iter().map(|&i| (i, full_turn(&genomes[i]))).collect();
            let finite: Vec<&(usize, (f32, f32, f32))> = turns.iter().filter(|(_, t)| t.0.is_finite()).collect();
            TurnStats {
                band: band.to_string(),
                genomes: members.len(),
                radius_px: Distribution::of(&finite.iter().map(|(_, t)| t.0).collect::<Vec<_>>()),
                turn_rate_deg_per_tick: Distribution::of(&turns.iter().map(|(_, t)| t.1).collect::<Vec<_>>()),
                ticks_per_90_deg: Distribution::of(&turns.iter().filter(|(_, t)| t.1 > 0.0).map(|(_, t)| 90.0 / t.1).collect::<Vec<_>>()),
                speed_ratio_turning: Distribution::of(&turns.iter().filter(|(i, _)| speeds[*i] > 0.0).map(|(i, t)| t.2 / speeds[*i]).collect::<Vec<_>>()),
            }
        })
        .collect();

    SpeedSet {
        genomes: label.to_string(),
        n: genomes.len(),
        p90_p10_ratio: dist.as_ref().filter(|d| d.p10 > 0.0).map(|d| d.p90 / d.p10),
        terminal_speed: dist,
        share_below_0_2: share_where(&runs, |r| r.terminal_speed < 0.2),
        share_below_1200px_in_1800_ticks: share_where(&runs, |r| r.distance_1800 < 1200.0),
        share_below_200px_in_300_ticks: share_where(&runs, |r| r.distance_300 < 200.0),
        share_not_settled: share_where(&runs, |r| !r.settled),
        turning,
        turn_at_rest: {
            let ticks: Vec<Option<u32>> = genomes.par_iter().map(|g| turn_from_rest(g)).collect();
            TurnAtRest {
                ticks_to_90_deg: Distribution::of(&ticks.iter().flatten().map(|&t| t as f32).collect::<Vec<_>>()),
                share_reached: share_where(&ticks, |t| t.is_some()),
            }
        },
    }
}

/// Real terminal speed at forced full thrust (thrust = 1, turn = 0) and turning at full bend.
pub fn probe_speed(opts: &ProbeOptions) -> SpeedReport {
    let seed = seed_from_str(&opts.seed);
    let mut sets: Vec<SpeedSet> = random_populations(seed, opts.samples.unwrap_or(1000)).iter().map(|(label, genomes)| speed_set(label, genomes)).collect();
    if !opts.hof.is_empty() {
        sets.push(speed_set("hof", &opts.hof.iter().map(|h| h.genome.clone()).collect::<Vec<_>>()));
    }
    SpeedReport {
        notes: vec![
            "step_creature_kinematics with forced outputs (thrust 1, turn 0) from rest, open arena, 1800 ticks; terminal speed = speed at tick 1800 (px/tick).".to_string(),
            "turning: thrust 1, turn 1 for 1800 ticks, measured over the last 200 ticks; bands by straight terminal speed.".to_string(),
            "turn_at_rest: thrust 0, turn 1 from rest; ticks until the heading has changed by 90 degrees (1800 ticks max).".to_string(),
        ],
        sets,
    }
}

// ==========================================================================
// 3. Equipment
// ==========================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpawnSensing {
    pub scenario: Scenario,
    pub chamber_size: f32,
    /// Some visible food lies within the range (scale * 550) of a light/chemical organ
    pub share_food_in_range: f32,
    /// Some light/chemical organ input is non-zero at spawn (range and cone)
    pub share_food_stimulus: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EquipmentReport {
    pub notes: Vec<String>,
    pub sets: Vec<EquipmentSet>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EquipmentSet {
    pub genomes: String,
    pub n: usize,
    pub organ_count: Option<Distribution>,
    pub share_no_organ: f32,
    /// Organ with spectral affinity >= 0.80 (vision labels)
    pub share_with_eye: f32,
    /// Organ with spectral affinity in 0.25..=0.65 (smell labels)
    pub share_with_nose: f32,
    /// Organ with spectral affinity >= 0.25 (any food detection in `compute_sensory_inputs`)
    pub share_with_food_sensor: f32,
    pub share_with_hox_gene: f32,
    /// Thalamus difference nodes present (a HOX-paired organ)
    pub share_with_hox_pair: f32,
    /// A HOX pair of light/chemical mirror organs (left/right food comparison possible)
    pub share_with_food_sensing_pair: f32,
    /// A thalamus pair whose two organs are not mirror images (a midline organ shifted the pairing)
    pub share_with_misaligned_pair: f32,
    pub sensor_range_px: Option<Distribution>,
    pub spawn: Vec<SpawnSensing>,
}

/// Organ equipment of letter-soup and structured random genomes and food in sensor range at spawn (real scenario initialisation).
pub fn probe_equipment(opts: &ProbeOptions) -> EquipmentReport {
    let seed = seed_from_str(&opts.seed);
    EquipmentReport {
        notes: vec![
            "Sensors see foods[0] (plant) and foods[1] (meat) of the scenario's visible foods only.".to_string(),
            "eye/nose by affinity (OrganKind Eye >= 0.80, Nose 0.25..=0.65); food detection uses affinity >= 0.25, so 0.65..0.80 organs (OrganKind Thermal) also see food.".to_string(),
            "letter_soup = random letters; structured = generate_structured_genome (config genomeComposition, trainer generation 1 and immigrants).".to_string(),
        ],
        sets: random_populations(seed, opts.samples.unwrap_or(1000)).iter().enumerate().map(|(set_id, (label, genomes))| equipment_set(label, genomes, seed, set_id as u64)).collect(),
    }
}

fn equipment_set(label: &str, genomes: &[String], seed: u64, set_id: u64) -> EquipmentSet {
    let phenotypes: Vec<CreaturePhenotype> = genomes.par_iter().map(|g| parse_genome(g, None, None)).collect();
    let food_organs = |p: &CreaturePhenotype| p.organelles.iter().filter(|o| o.spectral_affinity >= FOOD_SENSING_AFFINITY).count();
    let ranges: Vec<f32> = phenotypes.iter().flat_map(|p| p.organelles.iter().filter(|o| o.spectral_affinity >= FOOD_SENSING_AFFINITY).map(|o| o.scale * 550.0)).collect();

    let spawn = [Scenario::Standard, Scenario::Exploration]
        .iter()
        .enumerate()
        .map(|(s_idx, &scenario)| {
            let plugin = scenario.plugin();
            let size = plugin.chamber_size();
            let results: Vec<(bool, bool)> = genomes
                .par_iter()
                .enumerate()
                .map(|(i, genome)| {
                    // Letter soup keeps its pre-TCK-184 stream (set 0)
                    let mut rng = if set_id == 0 { derive(seed, &[STREAM_EQUIPMENT, s_idx as u64, i as u64]) } else { derive(seed, &[STREAM_EQUIPMENT, s_idx as u64, i as u64, set_id]) };
                    let sb = init_rust_sandbox(1, genome, 1, SandboxOrigin::Random, scenario, None, None, &mut rng);
                    let mut scratch = Vec::new();
                    let visible = plugin.visible_foods(&sb.agent, &sb.foods, &mut scratch).to_vec();
                    let agent = &sb.agent;
                    let in_range = agent.phenotype.organelles.iter().filter(|o| o.spectral_affinity >= FOOD_SENSING_AFFINITY).any(|o| {
                        visible.iter().take(2).any(|f| ((f.x - agent.px).powi(2) + (f.y - agent.py).powi(2)).sqrt() <= o.scale * 550.0)
                    });
                    let mut inputs = Vec::new();
                    compute_sensory_inputs_into(agent, 0.5, &visible, &[], size, size, &mut inputs);
                    (in_range, senses_food(&agent.phenotype, &inputs))
                })
                .collect();
            SpawnSensing {
                scenario,
                chamber_size: size,
                share_food_in_range: share_where(&results, |r| r.0),
                share_food_stimulus: share_where(&results, |r| r.1),
            }
        })
        .collect();

    // A HOX pair is the left and right organ of one gene (consecutive in organelles); its thalamus has Diff nodes
    let pair_organs = |p: &CreaturePhenotype| -> Vec<(f32, f32, f32)> {
        p.organelles
            .windows(2)
            .filter(|w| w[0].side == OrganSide::Left)
            .map(|w| (w[0].spectral_affinity.min(w[1].spectral_affinity), w[0].angle, w[1].angle))
            .collect()
    };
    let thalamus_pairs = |p: &CreaturePhenotype| pair_organs(p).len();
    EquipmentSet {
        genomes: label.to_string(),
        n: genomes.len(),
        organ_count: Distribution::of(&phenotypes.iter().map(|p| p.organelles.len() as f32).collect::<Vec<_>>()),
        share_no_organ: share_where(&phenotypes, |p| p.organelles.is_empty()),
        share_with_eye: share_where(&phenotypes, |p| p.organelles.iter().any(|o| o.spectral_affinity >= 0.80)),
        share_with_nose: share_where(&phenotypes, |p| p.organelles.iter().any(|o| (0.25..=0.65).contains(&o.spectral_affinity))),
        share_with_food_sensor: share_where(&phenotypes, |p| food_organs(p) > 0),
        share_with_hox_gene: share_where(genomes, |g| !extract_raw_gene_payloads(&normalize_genome(g), "HOX", "EN").is_empty()),
        share_with_hox_pair: share_where(&phenotypes, |p| thalamus_pairs(p) > 0),
        share_with_food_sensing_pair: share_where(&phenotypes, |p| pair_organs(p).iter().any(|&(aff, l, r)| aff >= FOOD_SENSING_AFFINITY && l == -r)),
        share_with_misaligned_pair: share_where(&phenotypes, |p| pair_organs(p).iter().any(|&(_, l, r)| l != -r)),
        sensor_range_px: Distribution::of(&ranges),
        spawn,
    }
}

// ==========================================================================
// 4. Steering
// ==========================================================================

const STEERING_DISTANCE: f32 = 300.0;
const STEERING_BEARING_DEG: f32 = 45.0;
const STEERING_TURN_WINDOW: u32 = 100;
const STEERING_TICKS: u32 = 300;
/// Heading difference (degrees) that counts as a response
const STEERING_RESPONSE_DEG: f32 = 5.0;

/// One run: creature in the standard chamber centre, heading 0, plant and meat at the same fixed point.
struct SteeringRun {
    /// Heading change after `STEERING_TURN_WINDOW` ticks (degrees, positive = towards +bearing)
    turn_deg: f32,
    distance: f32,
    sees_food_at_start: bool,
    reached: bool,
    min_abs_bearing_deg: f32,
    mean_thrust: f32,
}

fn steering_run(genome: &str, bearing_deg: Option<f32>, config: &AppConfig) -> SteeringRun {
    let size = Scenario::Standard.plugin().chamber_size();
    let (cx, cy) = (size / 2.0, size / 2.0);
    let mut agent = agent_at(genome, cx, cy, 0.0);
    let (fx, fy) = match bearing_deg {
        Some(b) => (cx + STEERING_DISTANCE * b.to_radians().cos(), cy + STEERING_DISTANCE * b.to_radians().sin()),
        None => (-99999.0, -99999.0),
    };
    let foods = [
        FoodSpore { id: 1, type_id: 1, x: fx, y: fy, amount: 15.0, vx: 0.0, vy: 0.0 },
        FoodSpore { id: 2, type_id: 2, x: fx, y: fy, amount: 15.0, vx: 0.0, vy: 0.0 },
    ];
    let mut inputs = Vec::new();
    let (mut heading_change, mut turn_deg, mut distance, mut thrust_sum) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let (mut sees_food_at_start, mut reached, mut min_abs_bearing) = (false, false, f32::MAX);
    let reach_dist = agent.phenotype.spinal_harmonics.mean_radius + 10.0;
    for tick in 1..=STEERING_TICKS {
        let before = agent.heading_angle;
        let (outputs, _) = brain_tick(&mut agent, &foods, size, config, &mut inputs);
        if tick == 1 {
            sees_food_at_start = senses_food(&agent.phenotype, &inputs);
        }
        thrust_sum += outputs[0];
        heading_change += wrap_angle(agent.heading_angle - before);
        distance += speed_of(&agent);
        if tick == STEERING_TURN_WINDOW {
            turn_deg = heading_change.to_degrees();
        }
        if bearing_deg.is_some() && !reached {
            let (dx, dy) = (fx - agent.px, fy - agent.py);
            if (dx * dx + dy * dy).sqrt() <= reach_dist {
                reached = true;
            }
            min_abs_bearing = min_abs_bearing.min(wrap_angle(fmath::atan2(dy, dx) - agent.heading_angle).abs().to_degrees());
        }
    }
    SteeringRun { turn_deg, distance, sees_food_at_start, reached, min_abs_bearing_deg: min_abs_bearing, mean_thrust: thrust_sum / STEERING_TICKS as f32 }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SteeringSet {
    pub genomes: String,
    pub n: usize,
    /// Distance travelled >= 5 px within 300 ticks (no-food control)
    pub share_moves: f32,
    /// Mean thrust output over 300 ticks (no-food control)
    pub mean_thrust_output: Option<Distribution>,
    pub share_mean_thrust_negative: f32,
    /// A light/chemical organ registers the food at 300 px on tick 1 (either side)
    pub share_sees_food_plus: f32,
    pub share_sees_food_minus: f32,
    /// (turn(+45) - turn(-45)) / 2 after 100 ticks, degrees; > 0 = towards the food
    pub directional_turn_deg: Option<Distribution>,
    pub share_directional_toward: f32,
    pub share_directional_away: f32,
    /// Turns toward the food on both sides relative to the no-food control (> 5 degrees each)
    pub share_toward_both_sides: f32,
    /// As above, among movers that see the food on both sides
    pub share_toward_both_sides_of_seeing_movers: Option<f32>,
    pub seeing_movers: usize,
    /// |bearing| drops below 35 degrees at some tick on both sides
    pub share_bearing_reduced_both_sides: f32,
    pub share_reached_plus: f32,
    pub share_reached_minus: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SteeringReport {
    pub notes: Vec<String>,
    pub sets: Vec<SteeringSet>,
}

fn steering_set(label: &str, genomes: &[String]) -> SteeringSet {
    let config = AppConfig::global();
    let runs: Vec<[SteeringRun; 3]> = genomes
        .par_iter()
        .map(|g| [steering_run(g, Some(STEERING_BEARING_DEG), config), steering_run(g, Some(-STEERING_BEARING_DEG), config), steering_run(g, None, config)])
        .collect();
    let directional: Vec<f32> = runs.iter().map(|[p, m, _]| (p.turn_deg - m.turn_deg) / 2.0).collect();
    let toward_both = |[p, m, c]: &[SteeringRun; 3]| p.turn_deg - c.turn_deg > STEERING_RESPONSE_DEG && m.turn_deg - c.turn_deg < -STEERING_RESPONSE_DEG;
    let seeing_movers: Vec<&[SteeringRun; 3]> = runs.iter().filter(|[p, m, c]| c.distance >= IMMOBILE_DISTANCE_PX && p.sees_food_at_start && m.sees_food_at_start).collect();
    let thrust: Vec<f32> = runs.iter().map(|r| r[2].mean_thrust).collect();
    SteeringSet {
        genomes: label.to_string(),
        n: genomes.len(),
        share_moves: share_where(&runs, |r| r[2].distance >= IMMOBILE_DISTANCE_PX),
        share_mean_thrust_negative: share_where(&thrust, |&t| t < 0.0),
        mean_thrust_output: Distribution::of(&thrust),
        share_sees_food_plus: share_where(&runs, |r| r[0].sees_food_at_start),
        share_sees_food_minus: share_where(&runs, |r| r[1].sees_food_at_start),
        share_directional_toward: share_where(&directional, |&d| d > STEERING_RESPONSE_DEG),
        share_directional_away: share_where(&directional, |&d| d < -STEERING_RESPONSE_DEG),
        directional_turn_deg: Distribution::of(&directional),
        share_toward_both_sides: share_where(&runs, toward_both),
        share_toward_both_sides_of_seeing_movers: if seeing_movers.is_empty() { None } else { Some(share_where(&seeing_movers, |r| toward_both(r))) },
        seeing_movers: seeing_movers.len(),
        share_bearing_reduced_both_sides: share_where(&runs, |[p, m, _]| p.min_abs_bearing_deg < STEERING_BEARING_DEG - 10.0 && m.min_abs_bearing_deg < STEERING_BEARING_DEG - 10.0),
        share_reached_plus: share_where(&runs, |r| r[0].reached),
        share_reached_minus: share_where(&runs, |r| r[1].reached),
    }
}

/// Food fixed at +45 / -45 degrees and 300 px: does the brain turn the creature towards it?
pub fn probe_steering(opts: &ProbeOptions) -> SteeringReport {
    let seed = seed_from_str(&opts.seed);
    // Runs are fully determined by the genomes (fixed start, no RNG)
    let mut sets: Vec<SteeringSet> = random_populations(seed, opts.samples.unwrap_or(500)).iter().map(|(label, genomes)| steering_set(label, genomes)).collect();
    if !opts.hof.is_empty() {
        sets.push(steering_set("hof", &opts.hof.iter().map(|h| h.genome.clone()).collect::<Vec<_>>()));
    }
    SteeringReport {
        notes: vec![
            "Standard chamber (1000 px), creature at the centre, heading 0, at rest; plant and meat on the same fixed point at 300 px and +/-45 degrees; real brain with Hebbian learning (config default); food does not drift.".to_string(),
            "Each genome runs three times: food at +45, at -45 and no food (control); turn = heading change after 100 ticks.".to_string(),
            "Positive bearing = positive heading change (the brain must output bend > 0; turning also works at rest since P5).".to_string(),
        ],
        sets,
    }
}

// ==========================================================================
// 5. Wall
// ==========================================================================

const WALL_ANGLES_DEG: &[f32] = &[30.0, 60.0, 90.0];
const WALL_TICKS: u32 = 600;
/// Gap between body edge and every wall that counts as free (px)
const WALL_FREE_GAP: f32 = 50.0;

struct WallRun {
    freed_at: Option<u32>,
    contact_ticks: u32,
    stalled_contact_ticks: u32,
    contact_speed_sum: f32,
    reached_corner: bool,
}

/// Creature at the top wall of the standard chamber, heading into it at `angle_deg` to the wall surface.
/// `brain`: real brain with the scenario's food layout, else forced outputs (thrust 1, turn 0).
fn wall_run(genome: &str, angle_deg: f32, brain: bool, rng: &mut SimRng, config: &AppConfig) -> WallRun {
    let scenario = Scenario::Standard;
    let size = scenario.plugin().chamber_size();
    let mut sb = init_rust_sandbox(1, genome, 1, SandboxOrigin::Random, scenario, None, None, rng);
    let r = sb.agent.phenotype.spinal_harmonics.mean_radius;
    sb.agent.px = size / 2.0;
    sb.agent.py = r + 2.0;
    sb.agent.heading_angle = -angle_deg.to_radians();
    let foods = sb.foods.clone();
    let mut agent = sb.agent;
    let mut inputs = Vec::new();
    let mut run = WallRun { freed_at: None, contact_ticks: 0, stalled_contact_ticks: 0, contact_speed_sum: 0.0, reached_corner: false };
    for tick in 1..=WALL_TICKS {
        let hit_wall = if brain {
            brain_tick(&mut agent, &foods, size, config, &mut inputs).1
        } else {
            step_creature_kinematics(&mut agent, 1.0, 0.0, config, &[], size, size)
        };
        let speed = speed_of(&agent);
        if hit_wall {
            run.contact_ticks += 1;
            run.contact_speed_sum += speed;
            if speed < 0.05 {
                run.stalled_contact_ticks += 1;
            }
        }
        let gaps = [agent.px - r, size - r - agent.px, agent.py - r, size - r - agent.py];
        if gaps.iter().filter(|&&g| g < 1.0).count() >= 2 {
            run.reached_corner = true;
        }
        if run.freed_at.is_none() && gaps.iter().all(|&g| g > WALL_FREE_GAP) {
            run.freed_at = Some(tick);
        }
    }
    run
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WallCell {
    pub genomes: String,
    pub mode: String,
    /// Angle between heading and wall surface (90 = head-on)
    pub angle_deg: f32,
    pub n: usize,
    /// Body edge more than 50 px from every wall at some tick within 600 ticks
    pub share_freed: f32,
    pub ticks_to_free: Option<Distribution>,
    /// Share of the 600 ticks with wall contact
    pub contact_tick_share: Option<Distribution>,
    /// Mean speed during contact ticks / straight terminal speed (forced mode) or absolute (brain)
    pub contact_speed: Option<Distribution>,
    /// Share of contact ticks with speed < 0.05 px/tick
    pub stalled_share_of_contact: f32,
    pub share_reached_corner: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WallReport {
    pub notes: Vec<String>,
    pub cells: Vec<WallCell>,
}

fn wall_cells(label: &str, genomes: &[String], seed: u64, set_id: u64) -> Vec<WallCell> {
    let config = AppConfig::global();
    let mut cells = Vec::new();
    for (mode_idx, &(mode, brain)) in [("forced", false), ("brain", true)].iter().enumerate() {
        for (a_idx, &angle) in WALL_ANGLES_DEG.iter().enumerate() {
            let runs: Vec<WallRun> = genomes
                .par_iter()
                .enumerate()
                .map(|(i, g)| wall_run(g, angle, brain, &mut derive(seed, &[STREAM_WALL, set_id, mode_idx as u64, a_idx as u64, i as u64]), config))
                .collect();
            let contact_total: u32 = runs.iter().map(|r| r.contact_ticks).sum();
            let stalled_total: u32 = runs.iter().map(|r| r.stalled_contact_ticks).sum();
            cells.push(WallCell {
                genomes: label.to_string(),
                mode: mode.to_string(),
                angle_deg: angle,
                n: runs.len(),
                share_freed: share_where(&runs, |r| r.freed_at.is_some()),
                ticks_to_free: Distribution::of(&runs.iter().filter_map(|r| r.freed_at).map(|t| t as f32).collect::<Vec<_>>()),
                contact_tick_share: Distribution::of(&runs.iter().map(|r| r.contact_ticks as f32 / WALL_TICKS as f32).collect::<Vec<_>>()),
                contact_speed: Distribution::of(&runs.iter().filter(|r| r.contact_ticks > 0).map(|r| r.contact_speed_sum / r.contact_ticks as f32).collect::<Vec<_>>()),
                stalled_share_of_contact: share(stalled_total as usize, contact_total as usize),
                share_reached_corner: share_where(&runs, |r| r.reached_corner),
            });
        }
    }
    cells
}

/// Thrust into the top wall at 30/60/90 degrees: does the creature get free within 600 ticks?
pub fn probe_wall(opts: &ProbeOptions) -> WallReport {
    let seed = seed_from_str(&opts.seed);
    let [(soup_label, soup), (structured_label, structured)] = random_populations(seed, opts.samples.unwrap_or(300));
    let mut cells = wall_cells(soup_label, &soup, seed, 0);
    if !opts.hof.is_empty() {
        cells.extend(wall_cells("hof", &opts.hof.iter().map(|h| h.genome.clone()).collect::<Vec<_>>(), seed, 1));
    }
    cells.extend(wall_cells(structured_label, &structured, seed, 2));
    WallReport {
        notes: vec![
            "Standard chamber (1000 px); start at the top wall centre, 2 px gap, at rest, heading into the wall at the given angle to the wall surface.".to_string(),
            "forced: thrust 1, turn 0 (the heading changes only through wall contact, which reflects it, TCK-180); brain: real brain with Hebbian learning, food from the seeded standard initialisation, food fixed.".to_string(),
            "contact = step_creature_kinematics reports a wall hit that tick; contact_speed in px/tick.".to_string(),
        ],
        cells,
    }
}

// ==========================================================================
// 6. Hebbian
// ==========================================================================

const HEBBIAN_GENOMES: usize = 32;
const HEBBIAN_TRIALS: u32 = 3;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HebbianCondition {
    pub scenario: Scenario,
    pub hebbian: bool,
    /// Multi-trial fitness with the trainer's viability gate (x0.1 without a catch, +150 per successful trial)
    pub fitness: Option<Distribution>,
    /// Share of trials with at least one catch
    pub success_rate: f32,
    pub mean_catches_per_trial: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HebbianPaired {
    pub scenario: Scenario,
    /// Fitness(on) - fitness(off) per genome
    pub fitness_delta: Option<Distribution>,
    pub share_on_better: f32,
    pub share_off_better: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HebbianReport {
    pub genome_source: String,
    pub genomes: usize,
    pub trials: u32,
    pub hebbian_learning_rate_base: f32,
    pub notes: Vec<String>,
    pub conditions: Vec<HebbianCondition>,
    pub paired: Vec<HebbianPaired>,
    /// Learned weight distribution with learning on (TCK-152): after the evaluated trials (trainer lifetime) and
    /// after a long life of `HEBBIAN_LONG_LIFE_TICKS` in the standard room
    pub weights: Vec<WeightCondition>,
}

/// Ticks of the long-life weight measurement (a long Ocean life, TCK-152 / RV-S04).
const HEBBIAN_LONG_LIFE_TICKS: u32 = 36_000;
/// Weights from here on count as "at the clamp" (clamp ±4).
const CLAMP_SHARE_THRESHOLD: f32 = 3.9;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeightCondition {
    pub scenario: Scenario,
    /// Ticks per genome (trials × epoch, or the long life)
    pub ticks: u32,
    /// Synapses other than sensor → thalamus reflexes
    pub learned: WeightStats,
    /// Sensor → thalamus reflex synapses (genome weight ±1)
    pub reflex: WeightStats,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WeightStats {
    pub synapses: usize,
    pub mean_abs_weight: f32,
    /// Share with |w| ≥ 3.9
    pub share_at_clamp: f32,
    /// Share pruned to exactly 0
    pub share_pruned: f32,
    /// Mean |learned − genome weight|
    pub mean_abs_change: f32,
    pub abs_weight: Option<Distribution>,
}

/// (is reflex, genome weight, learned weight) per synapse of an agent.
type WeightSample = (bool, f32, f32);

fn weight_samples(agent: &CreatureAgent) -> Vec<WeightSample> {
    let brain = &agent.phenotype.brain;
    brain
        .synapses
        .iter()
        .zip(&agent.synapse_weights)
        .map(|(s, &w)| {
            let reflex = matches!((&brain.neurons[s.from_node].role, &brain.neurons[s.to_node].role), (NeuronRole::Sensor { .. }, NeuronRole::Thalamus { .. }));
            (reflex, s.weight, w)
        })
        .collect()
}

fn weight_stats(samples: &[WeightSample], reflex: bool) -> WeightStats {
    let class: Vec<&WeightSample> = samples.iter().filter(|s| s.0 == reflex).collect();
    let n = class.len().max(1) as f32;
    let abs: Vec<f32> = class.iter().map(|s| s.2.abs()).collect();
    WeightStats {
        synapses: class.len(),
        mean_abs_weight: abs.iter().sum::<f32>() / n,
        share_at_clamp: share(abs.iter().filter(|&&w| w >= CLAMP_SHARE_THRESHOLD).count(), class.len()),
        share_pruned: share(class.iter().filter(|s| s.2 == 0.0).count(), class.len()),
        mean_abs_change: class.iter().map(|s| (s.2 - s.1).abs()).sum::<f32>() / n,
        abs_weight: Distribution::of(&abs),
    }
}

/// Learned weights of one genome after `ticks` in `scenario` with consecutive trials (weights carried over).
fn long_life_weights(genome: &str, scenario: Scenario, config: &AppConfig, seed: u64, stream: &[u64], ticks: u32) -> Vec<WeightSample> {
    let epoch = scenario.plugin().epoch_ticks();
    let stream_for = |trial: u32| -> Vec<u64> { stream.iter().copied().chain([trial as u64]).collect() };
    let mut rng = derive(seed, &stream_for(0));
    let mut sb = init_rust_sandbox(1, genome, 1, SandboxOrigin::Elite, scenario, None, None, &mut rng);
    for trial in 0..ticks.div_ceil(epoch) {
        if trial > 0 {
            rng = derive(seed, &stream_for(trial));
            sb.reset_trial(&mut rng);
        }
        for _ in 0..epoch {
            step_trainer_sandbox_physics_with_config(&mut sb, config);
        }
    }
    weight_samples(&sb.agent)
}

/// (fitness, successful trials, catches, learned weights) of one genome in one scenario, as the trainer scores it
/// (mean of the trial fitness, `scenarios::calculate_fitness`).
fn evaluate_genome(genome: &str, scenario: Scenario, config: &AppConfig, seed: u64, stream: &[u64]) -> (f32, u32, u32, Vec<WeightSample>) {
    let epoch = scenario.plugin().epoch_ticks();
    let stream_for = |trial: u32| -> Vec<u64> { stream.iter().copied().chain([trial as u64]).collect() };
    let mut rng = derive(seed, &stream_for(0));
    let mut sb = init_rust_sandbox(1, genome, 1, SandboxOrigin::Elite, scenario, None, None, &mut rng);
    let (mut acc_fitness, mut eaten, mut catches) = (0.0f32, 0u32, 0u32);
    for trial in 0..HEBBIAN_TRIALS {
        if trial > 0 {
            rng = derive(seed, &stream_for(trial));
            sb.reset_trial(&mut rng); // learned weights are kept across trials, as in the trainer
        }
        for _ in 0..epoch {
            step_trainer_sandbox_physics_with_config(&mut sb, config);
        }
        acc_fitness += crate::biology::scenarios::calculate_fitness(&sb, epoch, &config.trainer_fitness);
        if sb.finished {
            eaten += 1;
        }
        catches += sb.consumed_count;
    }
    let fitness = acc_fitness / HEBBIAN_TRIALS as f32;
    (fitness, eaten, catches, weight_samples(&sb.agent))
}

/// Best distinct genomes of a short seeded trainer run (standard room, population 32, 5 generations).
fn trained_genomes(seed: u64, count: usize) -> Vec<String> {
    let params = TrainerParams { population: 32, ..TrainerParams::default() };
    let mut sim = TrainerSim::new(params, seed ^ STREAM_HEBBIAN, 200);
    sim.track_live_fitness = false;
    let mut elites = Vec::new();
    let mut generations = 0;
    while generations < 5 {
        if let Some(TrainerEvent::GenerationCompleted { elites: e, .. }) = sim.step(true) {
            elites.extend(e);
            generations += 1;
        }
    }
    elites.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
    let mut seen = BTreeSet::new();
    elites.into_iter().filter(|e| seen.insert(e.genome.clone())).take(count).map(|e| e.genome).collect()
}

/// Same genomes with Hebbian learning on (config default) vs off (rate 0), standard and exploration room.
pub fn probe_hebbian(opts: &ProbeOptions) -> HebbianReport {
    let seed = seed_from_str(&opts.seed);
    let count = opts.samples.unwrap_or(HEBBIAN_GENOMES);
    let (genomes, source) = if opts.hof.is_empty() {
        (trained_genomes(seed, count), "best elites of a seeded TrainerSim run (standard, population 32, 5 generations)".to_string())
    } else {
        (opts.hof.iter().take(count).map(|h| h.genome.clone()).collect::<Vec<_>>(), "hof (DB)".to_string())
    };
    let on = AppConfig::global().clone();
    let mut off = on.clone();
    off.rules.hebbian_learning_rate_base = 0.0;

    let mut conditions = Vec::new();
    let mut paired = Vec::new();
    let mut weights = Vec::new();
    for (s_idx, &scenario) in [Scenario::Standard, Scenario::Exploration].iter().enumerate() {
        let mut per_condition = Vec::new();
        for (hebbian, config) in [(true, &on), (false, &off)] {
            // Identical RNG streams for on and off: the conditions differ only in the learning rate
            let results: Vec<(f32, u32, u32, Vec<WeightSample>)> = genomes
                .par_iter()
                .enumerate()
                .map(|(g, genome)| evaluate_genome(genome, scenario, config, seed, &[STREAM_HEBBIAN, s_idx as u64, g as u64]))
                .collect();
            let trials = (genomes.len() as u32 * HEBBIAN_TRIALS).max(1) as f32;
            conditions.push(HebbianCondition {
                scenario,
                hebbian,
                fitness: Distribution::of(&results.iter().map(|r| r.0).collect::<Vec<_>>()),
                success_rate: results.iter().map(|r| r.1).sum::<u32>() as f32 / trials,
                mean_catches_per_trial: results.iter().map(|r| r.2).sum::<u32>() as f32 / trials,
            });
            if hebbian {
                let samples: Vec<WeightSample> = results.iter().flat_map(|r| r.3.iter().copied()).collect();
                let ticks = HEBBIAN_TRIALS * scenario.plugin().epoch_ticks();
                weights.push(WeightCondition { scenario, ticks, learned: weight_stats(&samples, false), reflex: weight_stats(&samples, true) });
            }
            per_condition.push(results);
        }
        let deltas: Vec<f32> = per_condition[0].iter().zip(&per_condition[1]).map(|(a, b)| a.0 - b.0).collect();
        paired.push(HebbianPaired {
            scenario,
            share_on_better: share_where(&deltas, |&d| d > 0.0),
            share_off_better: share_where(&deltas, |&d| d < 0.0),
            fitness_delta: Distribution::of(&deltas),
        });
    }
    // Long life (learning on, standard room): bounded learning must not drift to the clamp (RV-S04)
    let long: Vec<WeightSample> = genomes
        .par_iter()
        .enumerate()
        .flat_map(|(g, genome)| long_life_weights(genome, Scenario::Standard, &on, seed, &[STREAM_HEBBIAN, 2, g as u64], HEBBIAN_LONG_LIFE_TICKS))
        .collect();
    weights.push(WeightCondition { scenario: Scenario::Standard, ticks: HEBBIAN_LONG_LIFE_TICKS, learned: weight_stats(&long, false), reflex: weight_stats(&long, true) });
    HebbianReport {
        genome_source: source,
        genomes: genomes.len(),
        trials: HEBBIAN_TRIALS,
        hebbian_learning_rate_base: on.rules.hebbian_learning_rate_base,
        notes: vec![
            "Each genome is evaluated like a trainer sandbox (multi-trial, weights carried across trials, viability gate); genome only, no Lamarckian methylations/weights.".to_string(),
            "off = hebbian_learning_rate_base 0 (no Oja update, no pruning); everything else identical, same RNG streams.".to_string(),
        ],
        conditions,
        paired,
        weights,
    }
}

// ==========================================================================
// 7. Topology
// ==========================================================================

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TopologyReport {
    pub notes: Vec<String>,
    pub sets: Vec<TopologySet>,
    pub wildtype_synapses: usize,
    pub wildtype_neurons: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TopologySet {
    pub genomes: String,
    pub n: usize,
    pub synapses: Option<Distribution>,
    pub neurons: Option<Distribution>,
    pub organs: Option<Distribution>,
    pub genome_length: Option<Distribution>,
}

/// Synapse and neuron counts of letter-soup and structured random genomes (brain size per creature, TCK-182).
pub fn probe_topology(opts: &ProbeOptions) -> TopologyReport {
    let seed = seed_from_str(&opts.seed);
    let sets = random_populations(seed, opts.samples.unwrap_or(1000))
        .iter()
        .map(|(label, genomes)| {
            let counts: Vec<[f32; 4]> = genomes
                .par_iter()
                .map(|g| {
                    let p = parse_genome(g, None, None);
                    [p.brain.synapses.len() as f32, p.brain.neurons.len() as f32, p.organelles.len() as f32, g.len() as f32]
                })
                .collect();
            let column = |i: usize| Distribution::of(&counts.iter().map(|c| c[i]).collect::<Vec<_>>());
            TopologySet { genomes: label.to_string(), n: genomes.len(), synapses: column(0), neurons: column(1), organs: column(2), genome_length: column(3) }
        })
        .collect();
    let wildtype = parse_genome(WILDTYPE_GENOME, None, None);
    TopologyReport {
        notes: vec!["letter_soup = random 256-base genomes, structured = generate_structured_genome (same sets as the other probes); counts from parse_genome.".to_string()],
        sets,
        wildtype_synapses: wildtype.brain.synapses.len(),
        wildtype_neurons: wildtype.brain.neurons.len(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetabolismSet {
    pub genomes: String,
    pub n: usize,
    /// Body mass `r^1.5 · L / 25`
    pub body_mass: Option<Distribution>,
    /// `Σ scale² · (1.1 − bandwidth)`
    pub organ_load: Option<Distribution>,
    /// Thalamus + hidden + motor neurons
    pub neurons: Option<Distribution>,
    /// Mean `Σ |w · a_source|` per tick over one standard-room trial
    pub activity: Option<Distribution>,
    /// Resting cost per tick at the genome's own trial activity (`metabolism::tick_cost`, no movement)
    pub resting_cost: Option<Distribution>,
    /// Brain + senses share of the resting cost per genome
    pub brain_senses_share: Option<Distribution>,
    /// Movement cost at full thrust / resting cost per genome
    pub full_thrust_ratio: Option<Distribution>,
    /// Mean |out_thrust| per tick over the same trial
    pub thrust: Option<Distribution>,
    /// Typical total cost per tick: resting at the trial activity + movement at the trial mean |thrust|
    pub typical_total_cost: Option<Distribution>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetabolismReport {
    pub notes: Vec<String>,
    pub sets: Vec<MetabolismSet>,
}

/// Inputs of the metabolic cost calibration (TCK-151): distributions of the cost terms over random genomes.
pub fn probe_metabolism(opts: &ProbeOptions) -> MetabolismReport {
    use crate::biology::metabolism::{body_mass, brain_neurons, mean_trial_activity_and_thrust, organ_load, tick_cost};
    let seed = seed_from_str(&opts.seed);
    let config = AppConfig::global();
    let sets = random_populations(seed, opts.samples.unwrap_or(1000))
        .iter()
        .enumerate()
        .map(|(set_id, (label, genomes))| {
            let rows: Vec<[f32; 9]> = genomes
                .par_iter()
                .enumerate()
                .map(|(i, g)| {
                    let p = parse_genome(g, None, None);
                    let (activity, thrust) = mean_trial_activity_and_thrust(g, Scenario::Standard, config, &mut derive(seed, &[STREAM_METABOLISM, set_id as u64, i as u64]));
                    let cost = tick_cost(&p, activity, 1.0, &config.metabolism);
                    let share = (cost.senses + cost.brain()) / cost.resting();
                    let typical_total = tick_cost(&p, activity, thrust, &config.metabolism).total();
                    [body_mass(&p), organ_load(&p), brain_neurons(&p) as f32, activity, cost.resting(), share, cost.movement / cost.resting(), thrust, typical_total]
                })
                .collect();
            let column = |i: usize| Distribution::of(&rows.iter().map(|c| c[i]).collect::<Vec<_>>());
            MetabolismSet {
                genomes: label.to_string(),
                n: genomes.len(),
                body_mass: column(0),
                organ_load: column(1),
                neurons: column(2),
                activity: column(3),
                resting_cost: column(4),
                brain_senses_share: column(5),
                full_thrust_ratio: column(6),
                thrust: column(7),
                typical_total_cost: column(8),
            }
        })
        .collect();
    MetabolismReport {
        notes: vec![
            "activity / thrust = mean Σ|w·a| / mean |out_thrust| per tick over one standard-room trial (fresh sandbox, config learning rate, no drain).".to_string(),
            "Pre-P6 median Ocean drain of the structured set (basal_metabolic_rate · 0.002 · 1.0 · 0.45): 0.0243 per tick (BMR 27); derivation in src/biology/metabolism.rs.".to_string(),
        ],
        sets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(samples: usize) -> ProbeOptions {
        ProbeOptions { seed: "probe-test".to_string(), samples: Some(samples), hof: Vec::new(), hof_source: None }
    }

    #[test]
    fn test_heredity_without_mutation_is_identical() {
        let report = heredity_with_counts(&opts(6), &[0]);
        for group in &report.groups {
            for cell in &group.cells {
                assert_eq!(cell.genome_identical_share, 1.0, "{} {}", group.parents, cell.kind);
                assert_eq!(cell.exuberant.pearson, Some(1.0));
                assert_eq!((cell.exuberant.retained_share, cell.exuberant.unchanged_share), (1.0, 1.0));
                assert!(cell.traits.iter().all(|t| t.share_changed == 0.0 && t.mean_abs_rel_change.unwrap_or(0.0) == 0.0));
                assert_eq!((cell.neuron_count_changed_share, cell.organ_count_changed_share), (0.0, 0.0));
            }
        }
    }

    #[test]
    fn test_forced_mutation_kinds_change_length_as_named() {
        let mut rng = derive(1, &[]);
        let ops = &AppConfig::global().mutation_operators;
        let len = WILDTYPE_GENOME.len();
        assert_eq!(mutate_genome_with_kind(WILDTYPE_GENOME, MutationKind::Point, ops, &mut rng).unwrap().len(), len);
        assert_eq!(mutate_genome_with_kind(WILDTYPE_GENOME, MutationKind::Creep, ops, &mut rng).unwrap().len(), len);
        assert_eq!(mutate_genome_with_kind(WILDTYPE_GENOME, MutationKind::Insertion, ops, &mut rng).unwrap().len(), len + 1);
        assert_eq!(mutate_genome_with_kind(WILDTYPE_GENOME, MutationKind::Deletion, ops, &mut rng).unwrap().len(), len - 1);
        assert!(mutate_genome_with_kind(WILDTYPE_GENOME, MutationKind::GeneDuplication, ops, &mut rng).unwrap().len() > len);
        assert!(mutate_genome_with_kind(&"A".repeat(128), MutationKind::Deletion, ops, &mut rng).is_none());
        assert!(mutate_genome_with_kind(&"A".repeat(128), MutationKind::GeneDuplication, ops, &mut rng).is_none());
    }

    #[test]
    fn test_speed_probe_is_deterministic() {
        let a = serde_json::to_string(&probe_speed(&opts(40))).unwrap();
        let b = serde_json::to_string(&probe_speed(&opts(40))).unwrap();
        assert_eq!(a, b);
        let report = probe_speed(&opts(40));
        let speed = report.sets[0].terminal_speed.as_ref().unwrap();
        assert!(speed.min > 0.0 && speed.p10 <= speed.p50 && speed.p50 <= speed.p90);
    }

    #[test]
    fn test_pearson_and_distribution() {
        assert_eq!(pearson(&[(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)]), Some(1.0));
        assert_eq!(pearson(&[(1.0, 1.0), (1.0, 2.0)]), None);
        let d = Distribution::of(&[0.0, 10.0, 20.0, 30.0, 40.0]).unwrap();
        assert_eq!((d.p50, d.p10, d.mean), (20.0, 4.0, 20.0));
    }
}
