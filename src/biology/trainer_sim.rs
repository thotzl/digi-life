//! IO-free trainer: population of sandboxes, epoch/trial loop and breeding.
//! The caller (engine or headless runner) persists the returned events.
//!
//! Shared layouts (TCK-174): every sandbox of a generation faces the same task. Trial `t` of layout `layout_id` uses
//! one RNG stream `derive(seed, [LAYOUT_STREAM, layout_id, t])`, replayed for every sandbox, from which the sandbox
//! draws (in this order) its start heading, its food positions (the creature always starts in the chamber centre) and
//! its respawn seed. Shared therefore: start position, start heading, initial food positions and respawn seed.
//! The respawn after the k-th catch of a trial draws its candidate points from `derive(respawn_seed, [k])`; the
//! accepted point also depends on the creature's position at the catch (minimum respawn distance), so two candidates
//! get the same respawn when they make their k-th catch at the same position, not in general. Not shared: food drift
//! caused by the creature pushing it. The layout is kept across generations and advances (`layout_id + 1`) once every
//! elite of a generation caught food in every trial (`elites_solved_layout`).
//!
//! Fitness (TCK-175): every trial is scored by `scenarios::calculate_fitness` in catch units (1.0 = one catch, same
//! scale in both rooms), per tick for the UI telemetry and at trial end; the generation fitness of a sandbox is the
//! aggregate of its trial fitness (`aggregate_trials`), without a gate or bonus per successful trial.
//!
//! Mutation channels (TCK-181): every mutated child (mutant, hof) records the gene channels its operator calls touched
//! (`dna::classify_mutation`: body / brain / both / neutral); elites carry the channel of the last mutation in their
//! lineage. `GenerationStats::mutation_channels` covers the mutant slots (origin `Mutant`, the population of the 1/5
//! rule); `elite_last_mutation` covers the elites. Mode `TrainerParams::mutation_channels` (config `mutationChannels`).

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biology::creature_factory::{inherit_learned_weights, spawn_creature, SpawnOptions};
use crate::biology::dna::{parse_genome, CTRNNSynapse, MutationChannel, MutationTouch, OrganKind, WILDTYPE_GENOME};
use crate::biology::scenarios::{calculate_fitness, Scenario};
use crate::biology::trainer_engine::{breed_genome_traced, init_rust_sandbox, step_trainer_sandbox_physics, SandboxOrigin, TrainerSandbox};
use crate::shared::rng::{derive, SimRng};
use crate::shared::types::AppConfig;
pub use crate::shared::types::{MutationChannelMode, TrialAggregation};
use rand::RngCore;

/// Stream index of the breeding RNGs (`derive(seed, [generation, BREED_STREAM, slot])`).
const BREED_STREAM: u64 = u64::MAX;
/// Stream index of the shared trial layouts (`derive(seed, [LAYOUT_STREAM, layout_id, trial])`).
const LAYOUT_STREAM: u64 = u64::MAX - 1;

/// A trial counts as immobile if the creature travelled less than this in total (px over the trial).
pub const IMMOBILE_DISTANCE_PX: f32 = 5.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainerParams {
    pub population: usize,
    pub elite_ratio: f32,
    /// Mutation operator calls per mutant / hall-of-fame child: uniform in `[mutation_ops_min, mutation_ops_max]`.
    /// The UI action field `mutationRate` sets `mutation_ops_max`; the 1/5 rule adapts it (TCK-174).
    pub mutation_ops_min: u32,
    pub mutation_ops_max: u32,
    /// Population-level 1/5 success rule on `mutation_ops_max` (`one_fifth_update`)
    pub one_fifth_rule: bool,
    pub inflow_rate: f32,
    pub hof_rate: f32,
    pub multi_trial: bool,
    /// Trials per generation when `multi_trial` is on
    pub trials: u32,
    /// Combination of the per-trial fitness when `multi_trial` is on
    pub trial_aggregation: TrialAggregation,
    pub lamarckian: bool,
    pub scenario: Scenario,
    /// Minimal criterion for generation 1 and random immigrants (RV-S09, `trainer_engine::random_genome`)
    pub minimal_criterion: bool,
    /// Body/brain mutation channels of mutated children (TCK-181, `trainer_engine::breed_genome_traced`)
    pub mutation_channels: MutationChannelMode,
}

impl Default for TrainerParams {
    /// Defaults of the trainer UI (trials, aggregation and mutation ops from config.json).
    fn default() -> Self {
        let config = AppConfig::global();
        Self {
            population: 16,
            elite_ratio: 0.25,
            mutation_ops_min: config.mutation_ops_min,
            mutation_ops_max: config.mutation_ops_max,
            one_fifth_rule: config.mutation_one_fifth_rule,
            inflow_rate: 0.15,
            hof_rate: 0.15,
            multi_trial: true,
            trials: config.trainer_trials,
            trial_aggregation: config.trainer_trial_aggregation,
            lamarckian: false,
            scenario: Scenario::Standard,
            minimal_criterion: config.trainer_minimal_criterion.enabled,
            mutation_channels: config.mutation_channels,
        }
    }
}

/// A scored genome (elite of a generation, parent or hall-of-fame entry).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenomeRecord {
    pub genome: String,
    pub name: String,
    pub fitness: f32,
    pub generation: u32,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
}

/// Outcome of one sandbox in one trial.
#[derive(Debug, Clone, Copy, PartialEq)]
struct TrialRecord {
    finish_tick: Option<u32>,
    distance_traveled: f32,
    /// Energy at trial end
    end_energy: f32,
    /// The creature ran out of energy during the trial (TCK-178)
    starved: bool,
}

/// Generation summary, computed at epoch end over the final fitness, before breeding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerationStats {
    pub generation: u32,
    /// Share of sandbox-trials with at least one catch
    pub success_rate: f32,
    /// Over the `finish_tick` of successful trials; `None` without any catch
    pub mean_time_to_first_catch: Option<f32>,
    pub median_time_to_first_catch: Option<f32>,
    pub best_fitness: f32,
    pub median_fitness: f32,
    pub mean_fitness: f32,
    /// p10, p25, p50, p75, p90 (linear interpolation)
    pub fitness_quantiles: [f32; 5],
    pub mean_genome_length: f32,
    /// Share of sandbox-trials below `IMMOBILE_DISTANCE_PX` travelled
    pub immobile_share: f32,
    /// Mean energy at trial end over the sandbox-trials (TCK-178)
    pub mean_end_energy: f32,
    /// Share of sandbox-trials in which the creature ran out of energy (TCK-178)
    pub starved_share: f32,
    /// Mean organ count of the generation's sandboxes (sensor equipment, TCK-178)
    pub mean_organ_count: f32,
    /// Share of the generation's sandboxes with at least one eye or nose (TCK-178)
    pub food_organ_share: f32,
    /// Shared layout this generation was evaluated on (TCK-174)
    pub layout_id: u64,
    /// Max mutation ops per child used to breed this generation (TCK-174; adapted by the 1/5 rule)
    pub mutation_ops_max: u32,
    /// Share of mutants with a higher fitness than their parent's unmodified elite slot of the same generation;
    /// `None` without such mutants (TCK-174, input of the 1/5 rule)
    pub child_success_share: Option<f32>,
    /// Mutant children of this generation per mutation channel (TCK-181, module doc)
    pub mutation_channels: MutationChannelStats,
    /// Channel of the last mutation in each elite's lineage, elites in rank order (an unmodified elite copy keeps its
    /// parent's); `None` for a lineage without any mutation (random immigrant or generation 1) or a restored parent
    pub elite_last_mutation: Vec<Option<MutationChannel>>,
}

/// Mutant children (origin `Mutant`) of one mutation channel in one generation (TCK-181).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChannelStats {
    pub mutants: u32,
    /// Share with a higher fitness than their parent (same reference as `child_success_share`); `None` without mutants
    pub beat_parent_share: Option<f32>,
    /// Share ranked among the elites of this generation (the parents of the next one); `None` without mutants
    pub elite_share: Option<f32>,
}

/// `ChannelStats` per channel of a child's mutations (union of its operator calls, `dna::classify_mutation`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MutationChannelStats {
    pub body: ChannelStats,
    pub brain: ChannelStats,
    pub both: ChannelStats,
    pub neutral: ChannelStats,
    /// Mutants with at least one call in an organ gene (counted as body; organ payloads also carry thalamus -> motor weights)
    pub organ_mutants: u32,
}

impl MutationChannelStats {
    /// From the mutant slots: (channel set, beat its parent, ranked among the elites).
    fn compute(mutants: &[(MutationTouch, bool, bool)]) -> Self {
        let of = |channel: MutationChannel| {
            let members: Vec<_> = mutants.iter().filter(|m| m.0.channel() == channel).collect();
            let share = |f: fn(&(MutationTouch, bool, bool)) -> bool| (!members.is_empty()).then(|| members.iter().filter(|m| f(m)).count() as f32 / members.len() as f32);
            ChannelStats { mutants: members.len() as u32, beat_parent_share: share(|m| m.1), elite_share: share(|m| m.2) }
        };
        Self {
            body: of(MutationChannel::Body),
            brain: of(MutationChannel::Brain),
            both: of(MutationChannel::Both),
            neutral: of(MutationChannel::Neutral),
            organ_mutants: mutants.iter().filter(|m| m.0.organ).count() as u32,
        }
    }
}

/// Breeding record of one slot (TCK-181): its origin, the channels of its own mutations (mutated origins) and the
/// channel of the last mutation in its lineage.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SlotLineage {
    origin: SandboxOrigin,
    own: Option<MutationTouch>,
    last: Option<MutationChannel>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrainerEvent {
    GenerationCompleted {
        generation: u32,
        best_fitness: f32,
        avg_fitness: f32,
        best_genome: String,
        elites: Vec<GenomeRecord>,
        stats: GenerationStats,
    },
}

pub struct TrainerSim {
    pub params: TrainerParams,
    pub generation: u32,
    /// Ticks of the current epoch (trial)
    pub epoch_ticks: u32,
    pub sandboxes: Vec<TrainerSandbox>,
    /// Recompute `current_fitness` every tick for the UI telemetry (default on). Off, it is only
    /// computed at epoch end, which gives the same epoch results.
    pub track_live_fitness: bool,
    trial_index: u32,
    /// Fitness per sandbox and finished trial of the current generation
    acc_fitness: Vec<Vec<f32>>,
    acc_eaten: Vec<u32>,
    trial_records: Vec<TrialRecord>,
    /// Parent index per breeding slot (elites: their own parent, mutants: their source parent, hof/random: none);
    /// aligned with `sandboxes` until they are ranked at generation end
    slot_parents: Vec<Option<usize>>,
    /// Breeding record per slot, aligned with `sandboxes` like `slot_parents` (TCK-181)
    slot_lineage: Vec<SlotLineage>,
    /// Channel of the last mutation in each parent's lineage, aligned with `parents`
    parent_last_mutation: Vec<Option<MutationChannel>>,
    /// Shared layout of the current generation (module doc)
    layout_id: u64,
    /// Elites of the previous generation, fitness descending
    parents: Vec<GenomeRecord>,
    /// Best genomes of the run, fitness descending, at most `hof_retention`
    hof: Vec<GenomeRecord>,
    hof_retention: usize,
    seed: u64,
}

/// Linear-interpolated quantile of an ascending slice (non-empty).
fn quantile(sorted: &[f32], q: f32) -> f32 {
    let pos = q * (sorted.len() - 1) as f32;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f32)
}

fn sort_desc_by_fitness(records: &mut [GenomeRecord]) {
    records.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));
}

/// Shared layout stream of `trial` (module doc).
fn layout_rng(seed: u64, layout_id: u64, trial: u32) -> SimRng {
    derive(seed, &[LAYOUT_STREAM, layout_id, trial as u64])
}

/// Combines the per-trial fitness of one sandbox: mean, or median (mean of the two middle values for an even
/// count); 0 without trials.
pub fn aggregate_trials(fitness: &[f32], aggregation: TrialAggregation) -> f32 {
    if fitness.is_empty() {
        return 0.0;
    }
    match aggregation {
        TrialAggregation::Mean => fitness.iter().sum::<f32>() / fitness.len() as f32,
        TrialAggregation::Median => {
            let mut sorted = fitness.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            quantile(&sorted, 0.5)
        }
    }
}

/// 1/5 success rule on the max ops per child `k` (TCK-174): `better` of `total` mutants beat their parent. Share
/// above 1/5 → `k + 1`, below 1/5 → `k - 1`, exactly 1/5 or no mutants → unchanged; the result is clamped to
/// `[lo, max(hi, lo)]`. Integer comparison (`5 · better` vs `total`), so the rule is exact and deterministic.
pub fn one_fifth_update(k: u32, better: usize, total: usize, lo: u32, hi: u32) -> u32 {
    let next = if total == 0 || 5 * better == total {
        k
    } else if 5 * better > total {
        k.saturating_add(1)
    } else {
        k.saturating_sub(1)
    };
    next.clamp(lo, hi.max(lo))
}

/// Layout advance rule (TCK-174): the layout is solved when every elite (the first `elite_count` entries of the
/// fitness ranking, at least one) caught food in every trial.
pub fn elites_solved_layout(ranked_solved: &[bool], elite_count: usize) -> bool {
    !ranked_solved.is_empty() && ranked_solved.iter().take(elite_count.max(1)).all(|&s| s)
}

impl GenerationStats {
    fn compute(generation: u32, trials: &[TrialRecord], sandboxes: &[TrainerSandbox]) -> Self {
        let trial_count = trials.len().max(1) as f32;
        let mut catch_ticks: Vec<f32> = trials.iter().filter_map(|t| t.finish_tick).map(|t| t as f32).collect();
        catch_ticks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let (mean_catch, median_catch) = if catch_ticks.is_empty() {
            (None, None)
        } else {
            (Some(catch_ticks.iter().sum::<f32>() / catch_ticks.len() as f32), Some(quantile(&catch_ticks, 0.5)))
        };

        let mut fitness: Vec<f32> = sandboxes.iter().map(|s| s.current_fitness).collect();
        fitness.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = fitness.len().max(1) as f32;
        let quantiles = [0.1, 0.25, 0.5, 0.75, 0.9].map(|q| if fitness.is_empty() { 0.0 } else { quantile(&fitness, q) });

        Self {
            generation,
            success_rate: catch_ticks.len() as f32 / trial_count,
            mean_time_to_first_catch: mean_catch,
            median_time_to_first_catch: median_catch,
            best_fitness: fitness.last().copied().unwrap_or(0.0),
            median_fitness: quantiles[2],
            mean_fitness: fitness.iter().sum::<f32>() / n,
            fitness_quantiles: quantiles,
            mean_genome_length: sandboxes.iter().map(|s| s.agent.genome.len() as f32).sum::<f32>() / n,
            immobile_share: trials.iter().filter(|t| t.distance_traveled < IMMOBILE_DISTANCE_PX).count() as f32 / trial_count,
            mean_end_energy: trials.iter().map(|t| t.end_energy).sum::<f32>() / trial_count,
            starved_share: trials.iter().filter(|t| t.starved).count() as f32 / trial_count,
            mean_organ_count: sandboxes.iter().map(|s| s.agent.phenotype.organelles.len() as f32).sum::<f32>() / n,
            food_organ_share: sandboxes
                .iter()
                .filter(|s| s.agent.phenotype.organelles.iter().any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose)))
                .count() as f32
                / n,
            layout_id: 0,
            mutation_ops_max: 0,
            child_success_share: None,
            mutation_channels: MutationChannelStats::default(),
            elite_last_mutation: Vec::new(),
        }
    }
}

impl TrainerSim {
    /// Generation 1: all sandboxes are structured random genomes (`breed_genome`, origin Random).
    pub fn new(params: TrainerParams, seed: u64, hof_retention: usize) -> Self {
        Self::restore(params, seed, 1, Vec::new(), Vec::new(), hof_retention)
    }

    /// Continues a run: breeds `generation` from `parents` (fitness descending) and `hof`.
    /// The shared layout restarts at layout 0 (it is not persisted).
    pub fn restore(mut params: TrainerParams, seed: u64, generation: u32, parents: Vec<GenomeRecord>, mut hof: Vec<GenomeRecord>, hof_retention: usize) -> Self {
        params.population = params.population.max(1);
        sort_desc_by_fitness(&mut hof);
        hof.truncate(hof_retention);
        let mut sim = Self {
            params,
            generation,
            epoch_ticks: 0,
            sandboxes: Vec::new(),
            track_live_fitness: true,
            trial_index: 0,
            acc_fitness: Vec::new(),
            acc_eaten: Vec::new(),
            trial_records: Vec::new(),
            slot_parents: Vec::new(),
            slot_lineage: Vec::new(),
            parent_last_mutation: vec![None; parents.len()],
            layout_id: 0,
            parents,
            hof,
            hof_retention,
            seed,
        };
        sim.breed(false);
        sim
    }

    /// Shared layout of the current generation.
    pub fn layout_id(&self) -> u64 {
        self.layout_id
    }

    /// One tick for all sandboxes incl. trial reset and breeding at epoch end.
    /// `parallel` steps the sandboxes (the tick uses no RNG) and breeds the next generation (each breeding slot with
    /// its own RNG streams) on the rayon pool; results never depend on it.
    pub fn step(&mut self, parallel: bool) -> Option<TrainerEvent> {
        self.epoch_ticks += 1;
        let epoch_duration = self.params.scenario.plugin().epoch_ticks();

        let live_fitness = self.track_live_fitness;
        let weights = &AppConfig::global().trainer_fitness;
        let step_one = |sb: &mut TrainerSandbox| {
            if live_fitness {
                sb.current_fitness = calculate_fitness(sb, epoch_duration, weights);
            }
            step_trainer_sandbox_physics(sb);
        };
        if parallel {
            self.sandboxes.par_iter_mut().for_each(step_one);
        } else {
            self.sandboxes.iter_mut().for_each(step_one);
        }

        if self.epoch_ticks >= epoch_duration {
            return self.end_epoch(epoch_duration, parallel);
        }
        None
    }

    /// Switches the room; on a change every sandbox restarts its trial in the new chamber
    /// (genomes, epoch tick counter and trial accumulators are kept, as in the UI before). Food positions and the
    /// respawn seed come from the current trial's shared layout stream, so they stay identical for all sandboxes;
    /// headings are kept.
    pub fn set_scenario(&mut self, scenario: Scenario) {
        if scenario == self.params.scenario {
            return;
        }
        self.params.scenario = scenario;
        let center = scenario.plugin().chamber_size() / 2.0;
        for sb in self.sandboxes.iter_mut() {
            let mut rng = layout_rng(self.seed, self.layout_id, self.trial_index);
            sb.scenario = scenario;
            sb.finished = false;
            sb.finish_tick = None;
            sb.agent.px = center;
            sb.agent.py = center;
            sb.agent.vx = 0.0;
            sb.agent.vy = 0.0;
            sb.agent.age = 0;
            sb.distance_traveled = 0.0;
            sb.epoch_ticks = 0;
            sb.wall_collisions = 0;
            sb.wall_collision_cooldown = 0;
            sb.consumed_count = 0;
            sb.accumulated_yield = 0.0;
            sb.coverage_mask = 0;
            sb.starved = false;
            scenario.plugin().initialize(sb, &mut rng);
            sb.respawn_seed = rng.next_u64();
        }
    }

    /// Replaces the creature of a sandbox (catalogue assignment). The agent is rebuilt through the
    /// factory (phenotype, antisense, neuron buffers match the genome); position, heading and motion
    /// are kept, the sandbox's trial counters restart. Returns the sandbox, `None` for an unknown id.
    pub fn assign_creature(&mut self, sandbox_id: u32, genome: &str, methylations: Option<&[f32]>, synapse_weights: Option<Vec<f32>>) -> Option<&TrainerSandbox> {
        let sb = self.sandboxes.iter_mut().find(|s| s.id == sandbox_id)?;
        let old = &sb.agent;
        let mut agent = spawn_creature(genome, SpawnOptions {
            id: old.id,
            px: old.px,
            py: old.py,
            heading_angle: old.heading_angle,
            generation: old.generation,
            methylations,
            synapse_weights,
        });
        agent.vx = old.vx;
        agent.vy = old.vy;
        agent.bend_angle = old.bend_angle;
        agent.omega_rot = old.omega_rot;
        sb.agent = agent;

        sb.finished = false;
        sb.finish_tick = None;
        sb.current_fitness = 0.0;
        sb.distance_traveled = 0.0;
        sb.wall_collisions = 0;
        sb.wall_collision_cooldown = 0;
        sb.consumed_spore_type = None;
        sb.epoch_ticks = 0;
        sb.accumulated_yield = 0.0;
        sb.consumed_count = 0;
        sb.starved = false;
        Some(&*sb)
    }

    fn end_epoch(&mut self, epoch_duration: u32, parallel: bool) -> Option<TrainerEvent> {
        let n = self.sandboxes.len();
        if self.acc_fitness.len() != n {
            self.acc_fitness = vec![Vec::new(); n];
        }
        if self.acc_eaten.len() != n {
            self.acc_eaten = vec![0; n];
        }
        let weights = &AppConfig::global().trainer_fitness;
        for (idx, sb) in self.sandboxes.iter_mut().enumerate() {
            sb.current_fitness = calculate_fitness(sb, epoch_duration, weights);
            self.trial_records.push(TrialRecord {
                finish_tick: sb.finish_tick,
                distance_traveled: sb.distance_traveled,
                end_energy: sb.agent.energy,
                starved: sb.starved,
            });
            self.acc_fitness[idx].push(sb.current_fitness);
            if sb.finished {
                self.acc_eaten[idx] += 1;
            }
        }

        let trials = if self.params.multi_trial { self.params.trials.max(1) } else { 1 };
        if self.trial_index + 1 < trials {
            self.trial_index += 1;
            self.epoch_ticks = 0;
            for sb in self.sandboxes.iter_mut() {
                sb.reset_trial(&mut layout_rng(self.seed, self.layout_id, self.trial_index));
            }
            return None;
        }

        // Caught food in every trial (layout advance rule)
        let solved: Vec<bool> = self.acc_eaten.iter().map(|&eaten| eaten >= trials).collect();
        if self.params.multi_trial {
            // Generation fitness = aggregate of the per-trial fitness (catch units, no gate)
            for (idx, sb) in self.sandboxes.iter_mut().enumerate() {
                sb.current_fitness = aggregate_trials(&self.acc_fitness[idx], self.params.trial_aggregation);
            }
        }
        self.trial_index = 0;
        self.acc_fitness.clear();
        self.acc_eaten.clear();

        Some(self.complete_generation(parallel, solved))
    }

    /// Per slot: `Some(beat its parent)` for mutants with a parent, else `None`. The reference of a parent is the
    /// fitness of its unmodified elite slot in this generation (same layout), else its recorded fitness.
    fn child_successes(&self) -> Vec<Option<bool>> {
        let slots = || self.sandboxes.iter().zip(&self.slot_parents);
        let parent_fitness = |p: usize| {
            slots()
                .find(|(sb, parent)| sb.origin_type == SandboxOrigin::Elite && **parent == Some(p))
                .map(|(sb, _)| sb.current_fitness)
                .unwrap_or(self.parents[p].fitness)
        };
        slots()
            .map(|(sb, parent)| if sb.origin_type == SandboxOrigin::Mutant { parent.map(|p| sb.current_fitness > parent_fitness(p)) } else { None })
            .collect()
    }

    fn complete_generation(&mut self, parallel: bool, solved: Vec<bool>) -> TrainerEvent {
        let n = self.sandboxes.len();
        let mut stats = GenerationStats::compute(self.generation, &self.trial_records, &self.sandboxes);
        self.trial_records.clear();
        let successes = self.child_successes();
        let (better, total) = successes.iter().flatten().fold((0, 0), |(better, total), &b| (better + b as usize, total + 1));
        stats.layout_id = self.layout_id;
        stats.mutation_ops_max = self.params.mutation_ops_max;
        stats.child_success_share = (total > 0).then(|| better as f32 / total as f32);

        // Rank by fitness (stable) together with the solved flags and the breeding slot
        let mut ranked: Vec<(TrainerSandbox, bool, usize)> =
            std::mem::take(&mut self.sandboxes).into_iter().zip(solved).enumerate().map(|(slot, (sb, solved))| (sb, solved, slot)).collect();
        ranked.sort_by(|a, b| b.0.current_fitness.partial_cmp(&a.0.current_fitness).unwrap_or(std::cmp::Ordering::Equal));
        let mut ranked_solved = Vec::with_capacity(n);
        let mut ranked_slots = Vec::with_capacity(n);
        for (sb, solved, slot) in ranked {
            self.sandboxes.push(sb);
            ranked_solved.push(solved);
            ranked_slots.push(slot);
        }
        self.slot_parents.clear();

        let best_fitness = self.sandboxes[0].current_fitness;
        let avg_fitness = self.sandboxes.iter().map(|s| s.current_fitness).sum::<f32>() / n as f32;
        let best_genome = self.sandboxes[0].agent.genome.clone();

        let elite_count = (((n as f32) * self.params.elite_ratio).round() as usize).clamp(1, n);

        // Mutation channels (TCK-181): mutant slots by channel, and the lineage channel of every elite
        let lineage = std::mem::take(&mut self.slot_lineage);
        let mut is_elite = vec![false; n];
        ranked_slots.iter().take(elite_count).for_each(|&slot| is_elite[slot] = true);
        let mutants: Vec<(MutationTouch, bool, bool)> = lineage
            .iter()
            .enumerate()
            .filter(|(_, l)| l.origin == SandboxOrigin::Mutant)
            .filter_map(|(slot, l)| l.own.map(|touch| (touch, successes.get(slot).copied().flatten().unwrap_or(false), is_elite[slot])))
            .collect();
        stats.mutation_channels = MutationChannelStats::compute(&mutants);
        let elite_last_mutation: Vec<Option<MutationChannel>> = ranked_slots.iter().take(elite_count).map(|&slot| lineage.get(slot).and_then(|l| l.last)).collect();
        stats.elite_last_mutation = elite_last_mutation.clone();
        let elites: Vec<GenomeRecord> = self
            .sandboxes
            .iter()
            .take(elite_count)
            .map(|sb| GenomeRecord {
                genome: sb.agent.genome.clone(),
                name: sb.agent.phenotype.latin_name.clone(),
                fitness: sb.current_fitness,
                generation: self.generation,
                methylations: sb.agent.phenotype.methylations.clone(),
                synapse_weights: sb.agent.synapse_weights.clone(),
            })
            .collect();

        if elites_solved_layout(&ranked_solved, elite_count) {
            self.layout_id += 1;
        }
        if self.params.one_fifth_rule {
            let config = AppConfig::global();
            let p = &mut self.params;
            p.mutation_ops_max = one_fifth_update(p.mutation_ops_max, better, total, p.mutation_ops_min.max(1), config.mutation_ops_limit);
        }

        let generation = self.generation;
        self.parents = elites.clone();
        self.parent_last_mutation = elite_last_mutation;
        self.hof.extend(elites.iter().cloned());
        sort_desc_by_fitness(&mut self.hof);
        self.hof.truncate(self.hof_retention);

        self.generation += 1;
        self.epoch_ticks = 0;
        self.breed(parallel);

        TrainerEvent::GenerationCompleted { generation, best_fitness, avg_fitness, best_genome, elites, stats }
    }

    /// Builds the sandboxes of `self.generation` from parents and hall of fame.
    /// Slot plan: elites (unmodified) | hof (mutated like mutants, TCK-174) | random immigrants | mutants of the parents.
    /// Without parents every slot is a structured random genome (generation 1).
    /// Every slot breeds with its own RNG stream and starts on the shared layout of trial 0, so `parallel` (rayon)
    /// never changes the result.
    fn breed(&mut self, parallel: bool) {
        let n = self.params.population;
        let p = &self.params;

        let elite_count = (((n as f32) * p.elite_ratio).round() as usize).clamp(1, n);
        let hof_count = ((n as f32) * p.hof_rate).round() as usize;
        let inflow_count = ((n as f32) * p.inflow_rate).round() as usize;

        // (source genome, origin, index of the source parent) per slot
        let plan: Vec<_> = (0..n).map(|idx| -> (&str, SandboxOrigin, Option<usize>) {
            if self.parents.is_empty() {
                (WILDTYPE_GENOME, SandboxOrigin::Random, None)
            } else if idx < elite_count {
                if idx < self.parents.len() {
                    (&self.parents[idx].genome, SandboxOrigin::Elite, Some(idx))
                } else {
                    (&self.parents[0].genome, SandboxOrigin::Mutant, Some(0))
                }
            } else if idx < elite_count + hof_count {
                let hof_idx = idx - elite_count;
                if hof_idx < self.hof.len() {
                    (&self.hof[hof_idx].genome, SandboxOrigin::Hof, None)
                } else {
                    (&self.parents[0].genome, SandboxOrigin::Mutant, Some(0))
                }
            } else if idx < elite_count + hof_count + inflow_count {
                (WILDTYPE_GENOME, SandboxOrigin::Random, None)
            } else {
                let source_idx = (idx - elite_count - hof_count - inflow_count) % self.parents.len();
                (&self.parents[source_idx].genome, SandboxOrigin::Mutant, Some(source_idx))
            }
        }).collect();

        let (seed, generation, parents, lamarckian, scenario) = (self.seed, self.generation as u64, &self.parents, p.lamarckian, p.scenario);
        let (ops_min, ops_max, layout_id, minimal_criterion) = (p.mutation_ops_min, p.mutation_ops_max, self.layout_id, p.minimal_criterion);
        let (channel_mode, parent_last_mutation) = (p.mutation_channels, &self.parent_last_mutation);
        let lamarck_chance = AppConfig::global().rules.lamarckian_assimilation_chance;
        // Lamarckian sources: each parent's synapses (keys and genome weights) for matching by key
        let parent_synapses: Vec<Vec<CTRNNSynapse>> = if lamarckian { parents.iter().map(|r| parse_genome(&r.genome, None, None).brain.synapses).collect() } else { Vec::new() };
        let build_slot = |(idx, &(source, origin, parent_idx)): (usize, &(&str, SandboxOrigin, Option<usize>))| {
            let mut breed_rng = derive(seed, &[generation, BREED_STREAM, idx as u64]);
            let (genome, own) = breed_genome_traced(source, origin, ops_min, ops_max, minimal_criterion, channel_mode, &mut breed_rng);
            let last = match own {
                Some(touch) => Some(touch.channel()),
                None if origin == SandboxOrigin::Elite => parent_idx.and_then(|i| parent_last_mutation.get(i).copied().flatten()),
                None => None,
            };
            let lineage = SlotLineage { origin, own, last };

            // Lamarckian inheritance: the source parent's learned methylations, and per plastic synapse with the same key
            // in this sandbox's (possibly mutated) genome, with probability `lamarckianAssimilationChance`, its learned
            // weight offset (TCK-152; drawn from the slot's breed stream)
            let inherited = if lamarckian { parent_idx } else { None };
            let methylations = inherited.map(|i| &parents[i].methylations[..]).filter(|m| !m.is_empty());
            let synapse_weights = inherited.filter(|&i| !parents[i].synapse_weights.is_empty()).map(|i| {
                let child = parse_genome(&genome, None, None).brain.synapses;
                inherit_learned_weights(&parent_synapses[i], &parents[i].synapse_weights, &child, lamarck_chance, &mut breed_rng)
            });

            // Shared layout of trial 0: heading, food and respawn seed identical for every slot
            let sandbox = init_rust_sandbox((idx + 1) as u32, &genome, generation as u32, origin, scenario, methylations, synapse_weights, &mut layout_rng(seed, layout_id, 0));
            (sandbox, lineage)
        };
        let (sandboxes, lineage): (Vec<TrainerSandbox>, Vec<SlotLineage>) = if parallel {
            plan.par_iter().enumerate().map(build_slot).unzip()
        } else {
            plan.iter().enumerate().map(build_slot).unzip()
        };

        self.slot_parents = plan.iter().map(|&(_, _, parent)| parent).collect();
        self.slot_lineage = lineage;
        self.sandboxes = sandboxes;
        self.trial_index = 0;
        self.acc_fitness = vec![Vec::new(); n];
        self.acc_eaten = vec![0; n];
        self.trial_records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::dna::parse_genome;

    fn small_params() -> TrainerParams {
        TrainerParams { population: 8, ..TrainerParams::default() }
    }

    /// Runs until `generations` generations completed; returns the stats and elite genomes per generation.
    fn run(sim: &mut TrainerSim, generations: usize) -> Vec<(GenerationStats, Vec<String>)> {
        let mut out = Vec::new();
        while out.len() < generations {
            if let Some(TrainerEvent::GenerationCompleted { stats, elites, .. }) = sim.step(false) {
                out.push((stats, elites.into_iter().map(|e| e.genome).collect()));
            }
        }
        out
    }

    fn genomes(sim: &TrainerSim) -> Vec<String> {
        sim.sandboxes.iter().map(|s| s.agent.genome.clone()).collect()
    }

    #[test]
    fn test_same_seed_same_generations_different_seed_differs() {
        let a = run(&mut TrainerSim::new(small_params(), 42, 200), 3);
        let b = run(&mut TrainerSim::new(small_params(), 42, 200), 3);
        assert_eq!(a, b);
        assert_eq!(a.iter().map(|(s, _)| s.generation).collect::<Vec<_>>(), vec![1, 2, 3]);

        let c = run(&mut TrainerSim::new(small_params(), 43, 200), 1);
        assert_ne!(a[0], c[0]);
    }

    #[test]
    fn test_restore_continues_with_given_parents() {
        let params = TrainerParams { multi_trial: false, ..small_params() };
        let mut original = TrainerSim::new(params.clone(), 9, 200);
        run(&mut original, 1);
        assert_eq!(original.generation, 2);

        let restored = TrainerSim::restore(params, 9, 2, original.parents.clone(), original.hof.clone(), 200);
        assert_eq!(restored.generation, 2);
        assert_eq!(genomes(&restored), genomes(&original));
        // Elite slots carry the parents unmodified
        assert_eq!(restored.sandboxes[0].origin_type, SandboxOrigin::Elite);
        assert_eq!(restored.sandboxes[0].agent.genome, original.parents[0].genome);
        assert!(restored.sandboxes.iter().all(|s| s.agent.generation == 2));
    }

    fn lamarck_parent() -> GenomeRecord {
        let len = WILDTYPE_GENOME.len();
        let synapses = parse_genome(WILDTYPE_GENOME, None, None).brain.synapses.len();
        GenomeRecord {
            genome: WILDTYPE_GENOME.to_string(),
            name: "parent".to_string(),
            fitness: 100.0,
            generation: 1,
            methylations: vec![0.3; len],
            synapse_weights: vec![0.25; synapses],
        }
    }

    #[test]
    fn test_lamarck_fallback_mutants_and_phenotype_matches_genome() {
        // hof_rate > elite_ratio with a single parent and empty hof: elite and hof slots fall back to
        // mutants of parent 0 (used to underflow the Lamarck source index)
        let params = TrainerParams { population: 8, elite_ratio: 0.25, hof_rate: 0.5, inflow_rate: 0.0, lamarckian: true, ..TrainerParams::default() };
        let parent = lamarck_parent();
        let sim = TrainerSim::restore(params, 5, 2, vec![parent.clone()], Vec::new(), 200);

        assert_eq!(sim.sandboxes[0].origin_type, SandboxOrigin::Elite);
        assert!(sim.sandboxes[1..].iter().all(|s| s.origin_type == SandboxOrigin::Mutant));
        assert!(sim.sandboxes.iter().any(|s| s.agent.genome != parent.genome));

        for sb in &sim.sandboxes {
            let expected = parse_genome(&sb.agent.genome, None, Some(&parent.methylations));
            assert_eq!(sb.agent.phenotype, expected, "sandbox {} phenotype does not match its genome", sb.id);
            assert_eq!(sb.agent.antisense, sb.agent.phenotype.antisense_strand);
            assert_eq!(sb.agent.neuron_states.len(), sb.agent.phenotype.brain.neurons.len());
        }
        // The unmutated elite inherits each plastic learned weight with probability lamarckianAssimilationChance;
        // the rest, and every fixed reflex, keep the genome weight (TCK-152)
        let elite = &sim.sandboxes[0].agent;
        let synapses = &elite.phenotype.brain.synapses;
        let inherited = synapses.iter().zip(&elite.synapse_weights).filter(|(s, w)| **w != s.weight).count();
        for ((s, &w), &learned) in synapses.iter().zip(&elite.synapse_weights).zip(&parent.synapse_weights) {
            assert!(w == s.weight || (s.plastic && w == learned), "synapse {:?}: {}", s, w);
        }
        assert!(inherited > 0 && inherited < synapses.iter().filter(|s| s.plastic).count(), "{} inherited", inherited);
    }

    /// Structured parent genome with learned weights = genome weight + a distinct offset per synapse.
    fn learned_parent(seed: u64) -> (GenomeRecord, Vec<CTRNNSynapse>, Vec<f32>) {
        let config = crate::shared::types::AppConfig::global();
        let genome = crate::biology::dna::generate_structured_genome(&config.genome_composition, &mut crate::shared::rng::seeded(seed));
        let synapses = parse_genome(&genome, None, None).brain.synapses;
        let offsets: Vec<f32> = (0..synapses.len()).map(|i| 0.01 * (i + 1) as f32).collect();
        let learned = synapses.iter().zip(&offsets).map(|(s, o)| s.weight + o).collect();
        let record = GenomeRecord { genome, name: "parent".to_string(), fitness: 100.0, generation: 1, methylations: Vec::new(), synapse_weights: learned };
        (record, synapses, offsets)
    }

    /// Learned offset per key that `weights` carry over the genome weights of `genome`.
    fn offsets_by_key(genome: &str, weights: &[f32]) -> std::collections::BTreeMap<u64, f32> {
        parse_genome(genome, None, None).brain.synapses.iter().zip(weights).map(|(s, w)| (s.key, w - s.weight)).collect()
    }

    #[test]
    fn test_lamarck_offsets_follow_synapse_keys_across_gene_duplication() {
        use crate::biology::dna::{mutate_genome_with_kind, MutationKind};
        let ops = &crate::shared::types::AppConfig::global().mutation_operators;
        let mut checked = 0;
        for seed in 0..40 {
            let (parent, synapses, offsets) = learned_parent(seed);
            let Some(child_genome) = mutate_genome_with_kind(&parent.genome, MutationKind::GeneDuplication, ops, &mut crate::shared::rng::seeded(seed)) else { continue };
            let child = parse_genome(&child_genome, None, None).brain.synapses;
            let inherited = inherit_learned_weights(&synapses, &parent.synapse_weights, &child, 1.0, &mut crate::shared::rng::seeded(seed));
            let child_offsets = offsets_by_key(&child_genome, &inherited);
            // The duplication shifts synapse indices, but every plastic parent offset lands on its own key; fixed reflexes
            // are never inherited
            for (s, o) in synapses.iter().zip(&offsets) {
                let c = child_offsets.get(&s.key).expect("parent synapse kept by the duplication");
                let expected = if s.plastic { *o } else { 0.0 };
                assert!((c - expected).abs() < 1e-5, "key {} offset {} -> {}", s.key, expected, c);
                checked += 1;
            }
            // The copy's new synapses start at their genome weight
            assert!(child_offsets.iter().filter(|(k, _)| !synapses.iter().any(|s| s.key == **k)).all(|(_, &o)| o == 0.0));
        }
        assert!(checked > 1000, "{} offsets checked", checked);
    }

    #[test]
    fn test_lamarck_breeding_matches_learned_offsets_by_key() {
        let params = TrainerParams { population: 8, elite_ratio: 0.25, hof_rate: 0.0, inflow_rate: 0.0, lamarckian: true, ..TrainerParams::default() };
        let (parent, synapses, offsets) = learned_parent(3);
        let sim = TrainerSim::restore(params, 5, 2, vec![parent.clone()], Vec::new(), 200);
        // Plastic parent offsets only; each homologous child synapse takes the full offset or none (TCK-152)
        let parent_offsets: std::collections::BTreeMap<u64, f32> = synapses.iter().zip(&offsets).filter(|(s, _)| s.plastic).map(|(s, o)| (s.key, *o)).collect();
        let (mut matched, mut taken) = (0, 0);
        for sb in &sim.sandboxes {
            for (key, o) in offsets_by_key(&sb.agent.genome, &sb.agent.synapse_weights) {
                let expected = parent_offsets.get(&key).copied().unwrap_or(0.0);
                assert!(o == 0.0 || (o - expected).abs() < 1e-5, "sandbox {} key {}: offset {} expected {} or 0", sb.id, key, o, expected);
                matched += parent_offsets.contains_key(&key) as usize;
                taken += (o != 0.0) as usize;
            }
        }
        let share = taken as f32 / matched as f32;
        let chance = crate::shared::types::AppConfig::global().rules.lamarckian_assimilation_chance;
        assert!(matched > 200, "{} homologous synapses", matched);
        assert!((share - chance).abs() < 0.08, "inherited share {} vs chance {}", share, chance);
    }

    #[test]
    fn test_phenotype_matches_genome_after_breeding() {
        let params = TrainerParams { multi_trial: false, lamarckian: true, ..small_params() };
        let mut sim = TrainerSim::new(params, 11, 200);
        run(&mut sim, 1);
        let parents = sim.parents.clone();

        for sb in &sim.sandboxes {
            // Lamarckian sources are the parents (elites and mutants); hof and random get no methylations
            let methylations = match sb.origin_type {
                SandboxOrigin::Elite | SandboxOrigin::Mutant => parents.iter().map(|p| p.methylations.clone()).find(|m| {
                    parse_genome(&sb.agent.genome, None, Some(m)) == sb.agent.phenotype
                }),
                _ => None,
            };
            let expected = parse_genome(&sb.agent.genome, None, methylations.as_deref());
            assert_eq!(sb.agent.phenotype, expected, "sandbox {} phenotype does not match its genome", sb.id);
        }
    }

    #[test]
    fn test_assign_creature_rebuilds_agent() {
        let mut sim = TrainerSim::new(TrainerParams { population: 2, ..TrainerParams::default() }, 3, 200);
        let (px, heading) = (sim.sandboxes[1].agent.px, sim.sandboxes[1].agent.heading_angle);
        let sb = sim.assign_creature(2, WILDTYPE_GENOME, None, None).unwrap();
        assert_eq!(sb.agent.phenotype, parse_genome(WILDTYPE_GENOME, None, None));
        assert_eq!(sb.agent.antisense, sb.agent.phenotype.antisense_strand);
        assert_eq!(sb.agent.neuron_states.len(), sb.agent.phenotype.brain.neurons.len());
        assert_eq!((sb.agent.px, sb.agent.heading_angle), (px, heading));
        assert!(sim.assign_creature(99, WILDTYPE_GENOME, None, None).is_none());
    }

    #[test]
    fn test_parallel_step_equals_sequential() {
        let params = TrainerParams { population: 12, ..TrainerParams::default() };
        let mut seq = TrainerSim::new(params.clone(), 77, 200);
        let mut par = TrainerSim::new(params, 77, 200);
        let (mut seq_events, mut par_events) = (Vec::new(), Vec::new());
        while seq_events.len() < 2 {
            seq_events.extend(seq.step(false));
            par_events.extend(par.step(true));
        }
        assert_eq!(seq_events, par_events);
        assert_eq!(genomes(&seq), genomes(&par));
        let state = |sim: &TrainerSim| sim.sandboxes.iter().map(|s| (s.agent.px.to_bits(), s.agent.py.to_bits(), s.current_fitness.to_bits())).collect::<Vec<_>>();
        assert_eq!(state(&seq), state(&par));
    }

    #[test]
    fn test_parallel_breeding_equals_sequential() {
        let params = TrainerParams { population: 20, multi_trial: false, lamarckian: true, ..TrainerParams::default() };
        let mut sim = TrainerSim::new(params, 21, 200);
        run(&mut sim, 1);
        let snapshot = |sim: &TrainerSim| serde_json::to_string(&sim.sandboxes).unwrap();
        let sequential = snapshot(&sim);
        assert!(sim.sandboxes.iter().any(|s| s.origin_type == SandboxOrigin::Mutant));

        sim.breed(true);
        assert_eq!(snapshot(&sim), sequential);
    }

    #[test]
    fn test_quantile_interpolates() {
        let sorted = [0.0, 10.0, 20.0, 30.0, 40.0];
        assert_eq!(quantile(&sorted, 0.5), 20.0);
        assert_eq!(quantile(&sorted, 0.1), 4.0);
        assert_eq!(quantile(&[7.0], 0.9), 7.0);
    }
    /// Start state of a sandbox's trial: heading, food positions, respawn seed (bit patterns).
    fn trial_start(sb: &TrainerSandbox) -> (u32, Vec<(u32, u32)>, u64) {
        (sb.agent.heading_angle.to_bits(), sb.foods.iter().map(|f| (f.x.to_bits(), f.y.to_bits())).collect(), sb.respawn_seed)
    }

    #[test]
    fn test_layout_identical_across_sandboxes_and_trials_differ() {
        for scenario in [Scenario::Standard, Scenario::Exploration] {
            let mut sim = TrainerSim::new(TrainerParams { population: 6, scenario, ..TrainerParams::default() }, 31, 200);
            let first = trial_start(&sim.sandboxes[0]);
            assert!(sim.sandboxes.iter().all(|sb| trial_start(sb) == first), "{:?} trial 0 differs between sandboxes", scenario);

            // Next trial: a new shared layout, again identical for all sandboxes
            let epoch = scenario.plugin().epoch_ticks();
            for _ in 0..epoch {
                assert!(sim.step(false).is_none());
            }
            let second = trial_start(&sim.sandboxes[0]);
            assert_ne!(second, first);
            assert!(sim.sandboxes.iter().all(|sb| trial_start(sb) == second && sb.agent.age == 0));
        }
    }

    #[test]
    fn test_layout_persists_across_generations_until_solved() {
        let params = TrainerParams { population: 6, trials: 2, ..TrainerParams::default() };
        let epoch = params.scenario.plugin().epoch_ticks();
        let mut sim = TrainerSim::new(params, 32, 200);
        let layout0 = trial_start(&sim.sandboxes[0]);
        // Runs one generation; at the end of every trial all sandboxes are marked as (not) having caught food
        let run_generation = |sim: &mut TrainerSim, caught: bool| loop {
            for _ in 0..epoch - 1 {
                sim.step(false);
            }
            sim.epoch_ticks = epoch;
            sim.sandboxes.iter_mut().for_each(|sb| sb.finished = caught);
            if let Some(TrainerEvent::GenerationCompleted { stats, .. }) = sim.end_epoch(epoch, false) {
                return stats;
            }
        };

        // Not solved: the next generation is evaluated on the same layout
        let stats = run_generation(&mut sim, false);
        assert_eq!((stats.layout_id, sim.layout_id()), (0, 0));
        assert_eq!(trial_start(&sim.sandboxes[0]), layout0);
        // Every elite caught food in every trial: the layout advances
        let stats = run_generation(&mut sim, true);
        assert_eq!((stats.layout_id, sim.layout_id()), (0, 1));
        let layout1 = trial_start(&sim.sandboxes[0]);
        assert_ne!(layout1, layout0);
        assert!(sim.sandboxes.iter().all(|sb| trial_start(sb) == layout1));
    }

    #[test]
    fn test_elites_solved_layout_rule() {
        assert!(elites_solved_layout(&[true, true, false, false], 2));
        assert!(!elites_solved_layout(&[true, false, true, true], 2));
        assert!(elites_solved_layout(&[true, false], 0), "at least one elite");
        assert!(!elites_solved_layout(&[], 1));
    }

    #[test]
    fn test_one_fifth_rule_direction_and_bounds() {
        assert_eq!(one_fifth_update(3, 3, 10, 1, 8), 4); // 30 % > 1/5
        assert_eq!(one_fifth_update(3, 1, 10, 1, 8), 2); // 10 % < 1/5
        assert_eq!(one_fifth_update(3, 2, 10, 1, 8), 3); // exactly 1/5
        assert_eq!(one_fifth_update(3, 0, 0, 1, 8), 3); // no mutants
        assert_eq!(one_fifth_update(8, 10, 10, 1, 8), 8);
        assert_eq!(one_fifth_update(1, 0, 10, 1, 8), 1);
        assert_eq!(one_fifth_update(12, 0, 10, 1, 8), 8);
        assert_eq!(one_fifth_update(0, 10, 10, 1, 8), 1);
    }

    #[test]
    fn test_one_fifth_rule_adapts_ops_within_bounds() {
        let limit = crate::shared::types::AppConfig::global().mutation_ops_limit;
        let params = TrainerParams { population: 10, one_fifth_rule: true, multi_trial: false, ..TrainerParams::default() };
        let mut sim = TrainerSim::new(params, 33, 200);
        let mut k_before = sim.params.mutation_ops_max;
        for _ in 0..4 {
            let (stats, _) = run(&mut sim, 1).remove(0);
            assert_eq!(stats.mutation_ops_max, k_before);
            let k = sim.params.mutation_ops_max;
            assert!((1..=limit).contains(&k));
            let expected = match stats.child_success_share {
                Some(share) if share > 0.2 => k_before + 1,
                Some(share) if share < 0.2 => k_before.saturating_sub(1),
                _ => k_before,
            };
            assert_eq!(k, expected.clamp(1, limit));
            k_before = k;
        }
        // Without the rule the ops bound stays fixed
        let mut fixed = TrainerSim::new(TrainerParams { population: 10, one_fifth_rule: false, multi_trial: false, ..TrainerParams::default() }, 33, 200);
        let k0 = fixed.params.mutation_ops_max;
        run(&mut fixed, 3);
        assert_eq!(fixed.params.mutation_ops_max, k0);
    }

    #[test]
    fn test_hof_slots_are_mutated() {
        let params = TrainerParams { population: 10, elite_ratio: 0.1, hof_rate: 0.5, inflow_rate: 0.0, ..TrainerParams::default() };
        let parent = lamarck_parent();
        let hof: Vec<GenomeRecord> = (0..5).map(|i| GenomeRecord { fitness: 50.0 - i as f32, ..parent.clone() }).collect();
        let sim = TrainerSim::restore(params, 34, 2, vec![parent.clone()], hof, 200);
        let hof_slots: Vec<_> = sim.sandboxes.iter().filter(|s| s.origin_type == SandboxOrigin::Hof).collect();
        assert_eq!(hof_slots.len(), 5);
        assert!(hof_slots.iter().filter(|s| s.agent.genome != parent.genome).count() >= 4, "hof slots must be mutated");
        assert_eq!(sim.sandboxes[0].agent.genome, parent.genome, "elite stays unmodified");
    }

    #[test]
    fn test_avg_fitness_divides_by_the_sandbox_count() {
        let mut sim = TrainerSim::new(TrainerParams { population: 6, multi_trial: false, ..TrainerParams::default() }, 35, 200);
        // A population setting that no longer matches the sandboxes (e.g. changed without a rebuild)
        sim.params.population = 60;
        loop {
            if let Some(TrainerEvent::GenerationCompleted { avg_fitness, stats, .. }) = sim.step(false) {
                assert!((avg_fitness - stats.mean_fitness).abs() <= 1e-3 * stats.mean_fitness.abs().max(1.0), "avg {} vs mean {}", avg_fitness, stats.mean_fitness);
                break;
            }
        }
    }

    #[test]
    fn test_trial_aggregation_mean_and_median() {
        assert_eq!(aggregate_trials(&[1.0, 2.0, 9.0], TrialAggregation::Mean), 4.0);
        assert_eq!(aggregate_trials(&[9.0, 1.0, 2.0], TrialAggregation::Median), 2.0);
        assert_eq!(aggregate_trials(&[4.0, 1.0, 2.0, 9.0], TrialAggregation::Median), 3.0);
        assert_eq!(aggregate_trials(&[], TrialAggregation::Median), 0.0);
    }

    #[test]
    fn test_median_aggregation_is_deterministic_and_parallel_safe() {
        let params = TrainerParams { population: 8, trials: 3, trial_aggregation: TrialAggregation::Median, ..TrainerParams::default() };
        let mut seq = TrainerSim::new(params.clone(), 36, 200);
        let mut par = TrainerSim::new(params, 36, 200);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        while a.is_empty() {
            a.extend(seq.step(false));
            b.extend(par.step(true));
        }
        assert_eq!(a, b);
    }

    #[test]
    fn test_live_telemetry_fitness_equals_epoch_end_fitness() {
        // UI path (fitness every tick) and runner path (epoch end only) use the same function: same events
        let params = TrainerParams { population: 8, trials: 2, ..TrainerParams::default() };
        let mut live = TrainerSim::new(params.clone(), 37, 200);
        let mut headless = TrainerSim::new(params, 37, 200);
        headless.track_live_fitness = false;
        let (mut a, mut b) = (Vec::new(), Vec::new());
        let weights = &AppConfig::global().trainer_fitness;
        let epoch = live.params.scenario.plugin().epoch_ticks();
        let mut expected_next: Option<Vec<f32>> = None;
        while a.is_empty() {
            a.extend(live.step(false));
            b.extend(headless.step(false));
            // The telemetry value of a tick is the shared fitness of the state the previous tick left
            if let Some(expected) = expected_next.take() {
                assert_eq!(live.sandboxes.iter().map(|sb| sb.current_fitness).collect::<Vec<_>>(), expected);
            }
            if a.is_empty() && live.epoch_ticks % 50 == 0 {
                expected_next = Some(live.sandboxes.iter().map(|sb| calculate_fitness(sb, epoch, weights)).collect());
            }
        }
        assert_eq!(a, b);
    }

    #[test]
    fn test_generation_fitness_is_the_trial_mean_without_gate() {
        let params = TrainerParams { population: 6, trials: 3, ..TrainerParams::default() };
        let epoch = params.scenario.plugin().epoch_ticks();
        let mut sim = TrainerSim::new(params, 38, 200);
        let weights = AppConfig::global().trainer_fitness.clone();
        let mut per_trial: Vec<Vec<f32>> = vec![Vec::new(); 6];
        loop {
            for _ in 0..epoch - 1 {
                assert!(sim.step(false).is_none());
            }
            for (i, sb) in sim.sandboxes.iter().enumerate() {
                // The last tick is stepped below; record the fitness the sim will compute at its end
                let mut probe = sb.clone();
                crate::biology::trainer_engine::step_trainer_sandbox_physics(&mut probe);
                per_trial[i].push(calculate_fitness(&probe, epoch, &weights));
            }
            if let Some(TrainerEvent::GenerationCompleted { stats, .. }) = sim.step(false) {
                let mut expected: Vec<f32> = per_trial.iter().map(|t| t.iter().sum::<f32>() / t.len() as f32).collect();
                expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
                assert!((stats.best_fitness - expected[5]).abs() < 1e-5, "best {} vs {}", stats.best_fitness, expected[5]);
                assert!((stats.mean_fitness - expected.iter().sum::<f32>() / 6.0).abs() < 1e-5);
                break;
            }
        }
    }

    #[test]
    fn test_minimal_criterion_population_is_deterministic_and_parallel_safe() {
        let params = TrainerParams { population: 10, minimal_criterion: true, multi_trial: false, ..TrainerParams::default() };
        let seq = TrainerSim::new(params.clone(), 39, 200);
        let mut par = TrainerSim::new(params.clone(), 39, 200);
        par.breed(true);
        assert_eq!(genomes(&seq), genomes(&par));
        let config = AppConfig::global();
        assert!(seq.sandboxes.iter().all(|sb| crate::biology::trainer_engine::meets_minimal_criterion(&sb.agent.genome, config)));
        // Off: other genomes from the same seed (the plain generator)
        let plain = TrainerSim::new(TrainerParams { minimal_criterion: false, ..params }, 39, 200);
        assert_ne!(genomes(&seq), genomes(&plain));
    }

    #[test]
    fn test_mutation_channel_stats_count_mutants_and_separate_is_parallel_safe() {
        for mode in [MutationChannelMode::Mixed, MutationChannelMode::Separate] {
            let params = TrainerParams { population: 12, multi_trial: false, mutation_channels: mode, ..TrainerParams::default() };
            let mut seq = TrainerSim::new(params.clone(), 55, 200);
            let mut par = TrainerSim::new(params, 55, 200);
            let (mut seq_events, mut par_events) = (Vec::new(), Vec::new());
            while seq_events.len() < 3 {
                seq_events.extend(seq.step(false));
                par_events.extend(par.step(true));
            }
            assert_eq!(seq_events, par_events, "{:?}", mode);
            for event in &seq_events {
                let TrainerEvent::GenerationCompleted { generation, elites, stats, .. } = event;
                let c = &stats.mutation_channels;
                let counted = c.body.mutants + c.brain.mutants + c.both.mutants + c.neutral.mutants;
                assert_eq!(stats.elite_last_mutation.len(), elites.len());
                if *generation == 1 {
                    assert_eq!(counted, 0, "generation 1 has no mutants");
                    assert!(stats.elite_last_mutation.iter().all(Option::is_none));
                } else {
                    assert!(counted > 0, "{:?} generation {}", mode, generation);
                }
                if mode == MutationChannelMode::Separate {
                    assert_eq!(c.both.mutants, 0, "separate: no child touches both channels");
                    assert!(stats.elite_last_mutation.iter().flatten().all(|&ch| ch != MutationChannel::Both));
                }
            }
        }
    }
}
