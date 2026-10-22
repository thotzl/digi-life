//! IO-free trainer: population of sandboxes, epoch/trial loop and breeding.
//! The caller (engine or headless runner) persists the returned events.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biology::creature_factory::{inherit_learned_weights, spawn_creature, SpawnOptions};
use crate::biology::dna::{parse_genome, CTRNNSynapse, WILDTYPE_GENOME};
use crate::biology::scenarios::Scenario;
use crate::biology::trainer_engine::{breed_genome, init_rust_sandbox, step_trainer_sandbox_physics, SandboxOrigin, TrainerSandbox};
use crate::shared::rng::{derive, SimRng};

/// Stream index of the breeding RNGs (`derive(seed, [generation, BREED_STREAM, slot])`).
const BREED_STREAM: u64 = u64::MAX;

/// A trial counts as immobile if the creature travelled less than this in total (px over the trial).
pub const IMMOBILE_DISTANCE_PX: f32 = 5.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainerParams {
    pub population: usize,
    pub elite_ratio: f32,
    pub mutation_rate: f32,
    pub inflow_rate: f32,
    pub hof_rate: f32,
    pub multi_trial: bool,
    /// Trials per generation when `multi_trial` is on
    pub trials: u32,
    pub lamarckian: bool,
    pub scenario: Scenario,
}

impl Default for TrainerParams {
    /// Defaults of the trainer UI.
    fn default() -> Self {
        Self {
            population: 16,
            elite_ratio: 0.15,
            mutation_rate: 0.06,
            inflow_rate: 0.15,
            hof_rate: 0.15,
            multi_trial: true,
            trials: 3,
            lamarckian: false,
            scenario: Scenario::Standard,
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
    acc_fitness: Vec<f32>,
    acc_eaten: Vec<u32>,
    trial_records: Vec<TrialRecord>,
    /// One stream per sandbox: `derive(seed, [generation, trial, sandbox_index])`
    rngs: Vec<SimRng>,
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
        }
    }
}

impl TrainerSim {
    /// Generation 1: all sandboxes are structured random genomes (`breed_genome`, origin Random).
    pub fn new(params: TrainerParams, seed: u64, hof_retention: usize) -> Self {
        Self::restore(params, seed, 1, Vec::new(), Vec::new(), hof_retention)
    }

    /// Continues a run: breeds `generation` from `parents` (fitness descending) and `hof`.
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
            rngs: Vec::new(),
            parents,
            hof,
            hof_retention,
            seed,
        };
        sim.breed(false);
        sim
    }

    /// One tick for all sandboxes incl. trial reset and breeding at epoch end.
    /// `parallel` steps the sandboxes and breeds the next generation (each sandbox and breeding slot
    /// with its own RNG streams) on the rayon pool;
    /// results never depend on it.
    pub fn step(&mut self, parallel: bool) -> Option<TrainerEvent> {
        self.epoch_ticks += 1;
        let epoch_duration = self.params.scenario.plugin().epoch_ticks();

        let live_fitness = self.track_live_fitness;
        let step_one = |(sb, rng): (&mut TrainerSandbox, &mut SimRng)| {
            if live_fitness {
                sb.current_fitness = sb.scenario.plugin().calculate_fitness(sb, epoch_duration);
            }
            step_trainer_sandbox_physics(sb, rng);
        };
        if parallel {
            self.sandboxes.par_iter_mut().zip(self.rngs.par_iter_mut()).for_each(step_one);
        } else {
            self.sandboxes.iter_mut().zip(self.rngs.iter_mut()).for_each(step_one);
        }

        if self.epoch_ticks >= epoch_duration {
            return self.end_epoch(epoch_duration, parallel);
        }
        None
    }

    /// Switches the room; on a change every sandbox restarts its trial in the new chamber
    /// (genomes, epoch tick counter and trial accumulators are kept, as in the UI before).
    pub fn set_scenario(&mut self, scenario: Scenario) {
        if scenario == self.params.scenario {
            return;
        }
        self.params.scenario = scenario;
        let center = scenario.plugin().chamber_size() / 2.0;
        for (sb, rng) in self.sandboxes.iter_mut().zip(self.rngs.iter_mut()) {
            sb.scenario = scenario;
            sb.finished = false;
            sb.finish_tick = None;
            sb.agent.px = center;
            sb.agent.py = center;
            sb.agent.vx = 0.0;
            sb.agent.vy = 0.0;
            sb.distance_traveled = 0.0;
            sb.epoch_ticks = 0;
            sb.wall_collisions = 0;
            sb.wall_collision_cooldown = 0;
            sb.consumed_count = 0;
            sb.accumulated_yield = 0.0;
            sb.coverage_mask = 0;
            scenario.plugin().initialize(sb, rng);
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
        Some(&*sb)
    }

    fn end_epoch(&mut self, epoch_duration: u32, parallel: bool) -> Option<TrainerEvent> {
        for sb in &mut self.sandboxes {
            sb.current_fitness = sb.scenario.plugin().calculate_fitness(sb, epoch_duration);
            self.trial_records.push(TrialRecord { finish_tick: sb.finish_tick, distance_traveled: sb.distance_traveled });
        }

        if self.params.multi_trial {
            let n = self.sandboxes.len();
            if self.acc_fitness.len() != n {
                self.acc_fitness = vec![0.0; n];
            }
            if self.acc_eaten.len() != n {
                self.acc_eaten = vec![0; n];
            }
            for (idx, sb) in self.sandboxes.iter().enumerate() {
                self.acc_fitness[idx] += sb.current_fitness;
                if sb.finished {
                    self.acc_eaten[idx] += 1;
                }
            }

            let trials = self.params.trials.max(1);
            if self.trial_index + 1 < trials {
                self.trial_index += 1;
                self.epoch_ticks = 0;
                for (idx, sb) in self.sandboxes.iter_mut().enumerate() {
                    self.rngs[idx] = derive(self.seed, &[self.generation as u64, self.trial_index as u64, idx as u64]);
                    sb.reset_trial(&mut self.rngs[idx]);
                }
                return None;
            }

            // Average the fitness across the trials and apply the viability gate
            for (idx, sb) in self.sandboxes.iter_mut().enumerate() {
                let avg_fit = self.acc_fitness[idx] / trials as f32;
                let eaten = self.acc_eaten[idx];
                sb.current_fitness = if eaten == 0 {
                    // Starvation filter: 90% penalty if the candidate did not eat in any trial
                    avg_fit * 0.1
                } else {
                    // Success: full average fitness plus 150 per successful trial
                    avg_fit + eaten as f32 * 150.0
                };
            }
            self.trial_index = 0;
            self.acc_fitness.clear();
            self.acc_eaten.clear();
        }

        Some(self.complete_generation(parallel))
    }

    fn complete_generation(&mut self, parallel: bool) -> TrainerEvent {
        let n = self.params.population;
        let stats = GenerationStats::compute(self.generation, &self.trial_records, &self.sandboxes);
        self.trial_records.clear();

        self.sandboxes.sort_by(|a, b| b.current_fitness.partial_cmp(&a.current_fitness).unwrap_or(std::cmp::Ordering::Equal));
        let best_fitness = self.sandboxes[0].current_fitness;
        let avg_fitness = self.sandboxes.iter().map(|s| s.current_fitness).sum::<f32>() / n as f32;
        let best_genome = self.sandboxes[0].agent.genome.clone();

        let elite_count = (((n as f32) * self.params.elite_ratio).round() as usize).clamp(1, n);
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

        let generation = self.generation;
        self.parents = elites.clone();
        self.hof.extend(elites.iter().cloned());
        sort_desc_by_fitness(&mut self.hof);
        self.hof.truncate(self.hof_retention);

        self.generation += 1;
        self.epoch_ticks = 0;
        self.breed(parallel);

        TrainerEvent::GenerationCompleted { generation, best_fitness, avg_fitness, best_genome, elites, stats }
    }

    /// Builds the sandboxes of `self.generation` from parents and hall of fame.
    /// Slot plan: elites (unmodified) | hof (copied, no mutation) | random immigrants | mutants of the parents.
    /// Without parents every slot is a structured random genome (generation 1).
    /// Every slot breeds and initializes with its own RNG streams, so `parallel` (rayon) never changes the result.
    fn breed(&mut self, parallel: bool) {
        let n = self.params.population;
        let p = &self.params;

        let elite_count = (((n as f32) * p.elite_ratio).round() as usize).clamp(1, n);
        let hof_count = ((n as f32) * p.hof_rate).round() as usize;
        let inflow_count = ((n as f32) * p.inflow_rate).round() as usize;

        // (source genome, origin, mutation rate, index of the Lamarckian source parent) per slot
        let plan: Vec<_> = (0..n).map(|idx| -> (&str, SandboxOrigin, f32, Option<usize>) {
            if self.parents.is_empty() {
                (WILDTYPE_GENOME, SandboxOrigin::Random, p.mutation_rate, None)
            } else if idx < elite_count {
                if idx < self.parents.len() {
                    (&self.parents[idx].genome, SandboxOrigin::Elite, 0.0, Some(idx))
                } else {
                    (&self.parents[0].genome, SandboxOrigin::Mutant, p.mutation_rate, Some(0))
                }
            } else if idx < elite_count + hof_count {
                let hof_idx = idx - elite_count;
                if hof_idx < self.hof.len() {
                    (&self.hof[hof_idx].genome, SandboxOrigin::Hof, p.mutation_rate, None)
                } else {
                    (&self.parents[0].genome, SandboxOrigin::Mutant, p.mutation_rate, Some(0))
                }
            } else if idx < elite_count + hof_count + inflow_count {
                (WILDTYPE_GENOME, SandboxOrigin::Random, 0.0, None)
            } else {
                let source_idx = (idx - elite_count - hof_count - inflow_count) % self.parents.len();
                (&self.parents[source_idx].genome, SandboxOrigin::Mutant, p.mutation_rate, Some(source_idx))
            }
        }).collect();

        let (seed, generation, parents, lamarckian, scenario) = (self.seed, self.generation as u64, &self.parents, p.lamarckian, p.scenario);
        let lamarck_chance = crate::shared::types::AppConfig::global().rules.lamarckian_assimilation_chance;
        // Lamarckian sources: each parent's synapses (keys and genome weights) for matching by key
        let parent_synapses: Vec<Vec<CTRNNSynapse>> = if lamarckian { parents.iter().map(|r| parse_genome(&r.genome, None, None).brain.synapses).collect() } else { Vec::new() };
        let build_slot = |(idx, &(source, origin, mut_rate, parent_idx)): (usize, &(&str, SandboxOrigin, f32, Option<usize>))| {
            let mut breed_rng = derive(seed, &[generation, BREED_STREAM, idx as u64]);
            let genome = breed_genome(source, origin, mut_rate, &mut breed_rng);

            // Lamarckian inheritance: the source parent's learned methylations, and per plastic synapse with the same key
            // in this sandbox's (possibly mutated) genome, with probability `lamarckianAssimilationChance`, its learned
            // weight offset (TCK-152; drawn from the slot's breed stream)
            let inherited = if lamarckian { parent_idx } else { None };
            let methylations = inherited.map(|i| &parents[i].methylations[..]).filter(|m| !m.is_empty());
            let synapse_weights = inherited.filter(|&i| !parents[i].synapse_weights.is_empty()).map(|i| {
                let child = parse_genome(&genome, None, None).brain.synapses;
                inherit_learned_weights(&parent_synapses[i], &parents[i].synapse_weights, &child, lamarck_chance, &mut breed_rng)
            });

            let mut rng = derive(seed, &[generation, 0, idx as u64]);
            let sb = init_rust_sandbox((idx + 1) as u32, &genome, generation as u32, origin, scenario, methylations, synapse_weights, &mut rng);
            (sb, rng)
        };
        let (sandboxes, rngs): (Vec<TrainerSandbox>, Vec<SimRng>) = if parallel {
            plan.par_iter().enumerate().map(build_slot).unzip()
        } else {
            plan.iter().enumerate().map(build_slot).unzip()
        };

        self.sandboxes = sandboxes;
        self.rngs = rngs;
        self.trial_index = 0;
        self.acc_fitness = vec![0.0; n];
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
        let params = TrainerParams { population: 8, elite_ratio: 0.25, hof_rate: 0.5, inflow_rate: 0.0, mutation_rate: 0.1, lamarckian: true, ..TrainerParams::default() };
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
        let params = TrainerParams { population: 8, elite_ratio: 0.25, hof_rate: 0.0, inflow_rate: 0.0, mutation_rate: 0.02, lamarckian: true, ..TrainerParams::default() };
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
        let params = TrainerParams { multi_trial: false, lamarckian: true, mutation_rate: 0.1, ..small_params() };
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
}
