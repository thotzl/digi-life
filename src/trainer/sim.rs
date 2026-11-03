//! IO-free trainer: population of sandboxes, trial loop and generation boundary. It orchestrates the room of each
//! trial (`room`), the evaluation (`Evaluator`, today `SoloTrials`), the breeding (`Breeder`) and the curriculum; the
//! caller (engine or headless runner) persists the returned events.
//!
//! Shared layouts (TCK-174): every sandbox of a generation faces the same task. Trial `t` of layout `layout_id` uses
//! one RNG stream `derive(seed, [streams::LAYOUT, layout_id, t])`, replayed for every sandbox, from which the room
//! draws its layout (start heading, food positions; the creature always starts in the chamber centre) and then the
//! sandbox its respawn seed. Shared therefore: start position, start heading, initial food positions and respawn seed.
//! The respawn after the k-th catch of a trial draws its candidate points from `derive(respawn_seed, [k])`; the
//! accepted point also depends on the creature's position at the catch (minimum respawn distance), so two candidates
//! get the same respawn when they make their k-th catch at the same position, not in general. Not shared: food drift
//! caused by the creature pushing it. When the layout advances (`layout_id + 1`) depends on `TrainerParams::layout_policy`
//! (config `trainerLayoutPolicy`): `Rotate` (default since P8) after every generation, so selection cannot reward
//! memorising a layout; `Hold` once every elite of a generation caught food in every trial (`elites_solved_layout`).
//!
//! Fitness (TCK-175): every trial is scored by the room (`Room::score`, the level rooms: `room::calculate_fitness` in
//! catch units), per tick for the UI telemetry and at trial end; the generation fitness of a sandbox is the aggregate
//! of its trial fitness (`Evaluator::aggregate`), without a gate or bonus per successful trial.
//!
//! Mutation channels (TCK-181): every mutated child (mutant, hof) records the gene channels its operator calls touched
//! (`genome::classify_mutation`: body / brain / both / neutral); elites carry the channel of the last mutation in their
//! lineage (`Breeder`). `GenerationStats::mutation_channels` covers the mutant slots (origin `Mutant`, the population
//! of the 1/5 rule); `elite_last_mutation` covers the elites. Mode `TrainerParams::mutation_channels`.
//!
//! Training mode (TCK-176, TCK-201): a fixed room of the registry (`TrainingMode::Room`, the UI modes `standard` and
//! `exploration`) or the curriculum. In the curriculum every trial runs in a `LevelRoom` at a difficulty level `d` in
//! [0, 1]; the level (`TrainerParams::level`) changes only at generation ends: the gate (`gate_level`) compares the
//! success rate of the generation's non-rehearsal trials with `upper` / `lower` and moves the level by `step`, clamped
//! to [0, 1]; a manual level (`set_level`) turns the gate off, `set_scenario(Curriculum)` turns it on again.
//! Rehearsal: `round(trials · rehearsalShare)` trials (at most all but one) run at `min(rehearsalLevel, d)`; which ones
//! is drawn per layout from the layout's stream (`rehearsal_trials`), so the choice is shared and fixed while the
//! layout is. Rehearsal trials count for fitness and the layout rule; the gate ignores them. Metabolic ramp: all
//! curriculum trials (rehearsal included) drain `metabolic_ramp(d)` times the configured trainer scale.
//! Room switches (`set_scenario`, `set_mode`, `set_level`) requested during a generation wait for its end; requested
//! at the start of a generation (no tick stepped yet) they apply at once by restarting trial 0 in the new room. Epoch
//! ticks, trial index and the accumulated fitness therefore always restart with the switch.

use std::sync::Arc;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::biology::creature_factory::{SpawnOptions, spawn_creature};
use crate::config::AppConfig;
use crate::shared::dto::CreatureAgent;
use crate::shared::rng::{derive, seed_from_str};

pub use super::breeder::{GenomeRecord, elites_solved_layout, one_fifth_update};
pub use super::curriculum::{CurriculumParams, LevelMove, gate_level, metabolic_ramp, rehearsal_trials, snap_level};
pub use super::evaluator::aggregate_trials;
pub use super::stats::{ChannelStats, GenerationStats, IMMOBILE_DISTANCE_PX, MutationChannelStats};
pub use crate::biology::trainer_config::{LayoutPolicy, MutationChannelMode, TrialAggregation};

use super::breeder::Breeder;
use super::evaluator::{Evaluator, SoloTrials};
use super::mode::{Scenario, TrainingMode};
use super::room::{LevelRoom, Room, registry};
use super::sandbox::{SandboxOrigin, TrainerSandbox};
use super::stats::TrialRecord;
use super::streams;

/// Room switch requested by `set_scenario` / `set_mode` / `set_level` (`level`: manual level, gate off).
#[derive(Debug, Clone, Copy, PartialEq)]
struct RoomSwitch {
    mode: TrainingMode,
    level: Option<f32>,
}

/// What the room of a trial is built from: a registry room, or a level room of the curriculum.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RoomSpec {
    Registered(&'static str),
    Level { level: f32, metabolic_scale: f32 },
}

impl RoomSpec {
    fn build(self, config: &AppConfig) -> Arc<dyn Room> {
        match self {
            RoomSpec::Registered(id) => {
                registry::build(id, config).unwrap_or_else(|| panic!("room '{}' is registered", id))
            }
            RoomSpec::Level { level, metabolic_scale } => {
                Arc::new(LevelRoom::at(level, config).with_metabolic_scale(metabolic_scale))
            }
        }
    }
}

/// Success rate and mean generation fitness of a population evaluated in one room without breeding
/// (`TrainerSim::evaluate_population`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopulationEval {
    pub level: f32,
    pub trials: u32,
    /// Share of sandbox-trials with at least one catch
    pub success_rate: f32,
    /// Mean over the sandboxes of their aggregated trial fitness
    pub mean_fitness: f32,
    /// The same over the unmodified elite slots (`None` without elites, e.g. generation 1)
    pub elite_success_rate: Option<f32>,
    pub elite_mean_fitness: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// UI training mode (`standard`, `exploration`, `curriculum`); the room of a fixed mode is its registry room
    pub scenario: Scenario,
    /// Fixed room by registry id for rooms without a UI mode (headless runner, `TrainerSim::set_mode`); overrides the
    /// room of a fixed `scenario`, ignored in the curriculum. `None`: the room of `scenario`.
    pub room: Option<&'static str>,
    /// Minimal criterion for generation 1 and random immigrants (RV-S09, `trainer::genomes::random_genome`)
    pub minimal_criterion: bool,
    /// Body/brain mutation channels of mutated children (TCK-181, `trainer::genomes::breed_genome_traced`)
    pub mutation_channels: MutationChannelMode,
    /// When the shared trial layout changes (config `trainerLayoutPolicy`)
    pub layout_policy: LayoutPolicy,
    /// Gate, rehearsal and metabolic ramp of the curriculum (TCK-176)
    pub curriculum: CurriculumParams,
    /// Difficulty level of the curriculum room; moved by the gate at generation ends (module doc)
    pub level: f32,
    /// Curriculum gate on (automatic level); off after a manual level (`TrainerSim::set_level`)
    pub level_auto: bool,
}

impl TrainerParams {
    /// Defaults of the trainer UI (population, trials, aggregation and mutation ops from config section `trainer`).
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            population: config.trainer.trainer_population.max(1),
            elite_ratio: 0.25,
            mutation_ops_min: config.trainer.mutation_ops_min,
            mutation_ops_max: config.trainer.mutation_ops_max,
            one_fifth_rule: config.trainer.mutation_one_fifth_rule,
            inflow_rate: 0.15,
            hof_rate: 0.15,
            multi_trial: true,
            trials: config.trainer.trainer_trials,
            trial_aggregation: config.trainer.trainer_trial_aggregation,
            lamarckian: false,
            scenario: Scenario::Standard,
            room: None,
            minimal_criterion: config.trainer.trainer_minimal_criterion.enabled,
            mutation_channels: config.trainer.mutation_channels,
            layout_policy: config.trainer.trainer_layout_policy,
            curriculum: CurriculumParams::from_config(&config.trainer.trainer_curriculum),
            level: snap_level(config.trainer.trainer_curriculum.start_level),
            level_auto: true,
        }
    }

    /// Training mode of `scenario` and `room`.
    pub fn mode(&self) -> TrainingMode {
        match (self.scenario.room_id(), self.room) {
            (None, _) => TrainingMode::Curriculum,
            (Some(_), Some(id)) => TrainingMode::Room(id),
            (Some(id), None) => TrainingMode::Room(id),
        }
    }

    /// Sets `scenario` and `room` to `mode` (`room` only for a room without a UI mode).
    pub fn set_mode(&mut self, mode: TrainingMode) {
        self.scenario = mode.scenario();
        self.room = match mode {
            TrainingMode::Room(id) if self.scenario.room_id() != Some(id) => Some(id),
            _ => None,
        };
    }
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

/// The creature a catalogue assignment put into a sandbox (`TrainerSim::assign_creature`).
#[derive(Debug, Clone, Copy)]
pub struct AssignedCreature<'a> {
    pub sandbox_id: u32,
    pub agent: &'a CreatureAgent,
}

pub struct TrainerSim {
    pub params: TrainerParams,
    pub generation: u32,
    /// Ticks of the current trial
    pub epoch_ticks: u32,
    pub sandboxes: Vec<TrainerSandbox>,
    /// Recompute `current_fitness` every tick for the UI telemetry (default on). Off, it is only
    /// computed at epoch end, which gives the same epoch results.
    pub track_live_fitness: bool,
    trial_index: u32,
    /// Fitness per sandbox and finished trial of the current generation
    acc_fitness: Vec<Vec<f32>>,
    /// Trials with a catch per sandbox in the current generation
    acc_eaten: Vec<u32>,
    trial_records: Vec<TrialRecord>,
    /// Shared layout of the current generation (module doc)
    layout_id: u64,
    breeder: Breeder,
    evaluator: Arc<dyn Evaluator>,
    seed: u64,
    /// Room of the current trial (all sandboxes run the same room) and what it was built from
    room: Arc<dyn Room>,
    room_spec: RoomSpec,
    /// Rehearsal trials of the current generation (curriculum only, sorted)
    rehearsal: Vec<u32>,
    /// Room switch requested during the current generation; applied at its end
    pending: Option<RoomSwitch>,
    /// Config of the run (sections `trainer`, `genome`, `brain`, `metabolism`, `physics` are read)
    config: AppConfig,
}

/// Seed of a UI trainer run: `seed_from_str("{config_seed}:{run_id}")`; a reset adds a nonce
/// (`"{config_seed}:{run_id}:{nonce}"`), so it starts a new run under the same run id instead of replaying it (TCK-194).
pub fn run_seed(config_seed: &str, run_id: &str, nonce: Option<u64>) -> u64 {
    match nonce {
        Some(nonce) => seed_from_str(&format!("{}:{}:{}", config_seed, run_id, nonce)),
        None => seed_from_str(&format!("{}:{}", config_seed, run_id)),
    }
}

/// Per-run state a restore continues with (TCK-191, table `trainer_run_state`): training mode, curriculum level and
/// gate, the shared layout and the `mutation_ops_max` adapted by the 1/5 rule. Taken right after a generation end
/// (`of`), i.e. the state the next generation runs with; a switch still pending at that moment is not part of it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrainerRunState {
    pub scenario: Scenario,
    pub level: f32,
    pub level_auto: bool,
    pub layout_id: u64,
    pub mutation_ops_max: u32,
    /// Population of the run (TCK-192); `None` for rows saved before it was stored (config default applies)
    pub population: Option<usize>,
    /// Run seed (TCK-194; a reset draws a new one, `run_seed`); `None` for rows saved before it was stored (the seed
    /// without nonce applies)
    pub seed: Option<u64>,
}

impl TrainerRunState {
    pub fn of(sim: &TrainerSim) -> Self {
        let p = &sim.params;
        Self {
            scenario: p.scenario,
            level: p.level,
            level_auto: p.level_auto,
            layout_id: sim.layout_id,
            mutation_ops_max: p.mutation_ops_max,
            population: Some(p.population),
            seed: Some(sim.seed),
        }
    }

    /// Writes the state into `params` (the layout goes to `TrainerSim::restore_at_layout`). `mutation_ops_max` is
    /// clamped to the current `[mutationOpsMin, mutationOpsLimit]` like the UI field.
    pub fn apply(&self, params: &mut TrainerParams, config: &AppConfig) {
        let lo = params.mutation_ops_min.max(1);
        params.scenario = self.scenario;
        params.room = None;
        params.level = snap_level(self.level);
        params.level_auto = self.level_auto;
        params.mutation_ops_max = self
            .mutation_ops_max
            .clamp(lo, config.trainer.mutation_ops_limit.max(lo));
        if let Some(population) = self.population {
            params.population = population.max(1);
        }
    }
}

impl TrainerSim {
    /// Generation 1: all sandboxes are structured random genomes (origin Random). `config` is copied into the sim.
    pub fn new(params: TrainerParams, seed: u64, hof_retention: usize, config: &AppConfig) -> Self {
        Self::restore(params, seed, 1, Vec::new(), Vec::new(), hof_retention, config)
    }

    /// Continues a run: breeds `generation` from `parents` (fitness descending) and `hof` on shared layout 0.
    pub fn restore(
        params: TrainerParams,
        seed: u64,
        generation: u32,
        parents: Vec<GenomeRecord>,
        hof: Vec<GenomeRecord>,
        hof_retention: usize,
        config: &AppConfig,
    ) -> Self {
        Self::restore_at_layout(params, seed, generation, 0, parents, hof, hof_retention, config)
    }

    /// `restore` on shared layout `layout_id` (the persisted run state, TCK-191, `TrainerRunState`).
    #[allow(clippy::too_many_arguments)]
    pub fn restore_at_layout(
        mut params: TrainerParams,
        seed: u64,
        generation: u32,
        layout_id: u64,
        parents: Vec<GenomeRecord>,
        hof: Vec<GenomeRecord>,
        hof_retention: usize,
        config: &AppConfig,
    ) -> Self {
        params.population = params.population.max(1);
        params.level = snap_level(params.level);
        let room_spec = RoomSpec::Registered("standard");
        let mut sim = Self {
            room: room_spec.build(config),
            room_spec,
            config: config.clone(),
            rehearsal: Vec::new(),
            pending: None,
            params,
            generation,
            epoch_ticks: 0,
            sandboxes: Vec::new(),
            track_live_fitness: true,
            trial_index: 0,
            acc_fitness: Vec::new(),
            acc_eaten: Vec::new(),
            trial_records: Vec::new(),
            layout_id,
            breeder: Breeder::new(parents, hof, hof_retention),
            evaluator: Arc::new(SoloTrials),
            seed,
        };
        sim.breed(false);
        sim
    }

    /// Seed of the run (all RNG streams derive from it).
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Shared layout of the current generation.
    pub fn layout_id(&self) -> u64 {
        self.layout_id
    }

    /// Config the sim runs with.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Elites of the previous generation (the parents of the current one), fitness descending.
    pub fn parents(&self) -> &[GenomeRecord] {
        self.breeder.parents()
    }

    /// Hall of fame of the run, fitness descending.
    pub fn hof(&self) -> &[GenomeRecord] {
        self.breeder.hof()
    }

    /// Room of the current trial.
    pub fn room(&self) -> &dyn Room {
        self.room.as_ref()
    }

    /// Evaluation scheme of the run (default `SoloTrials`).
    pub fn evaluator(&self) -> &dyn Evaluator {
        self.evaluator.as_ref()
    }

    /// Replaces the evaluation scheme (headless experiments); takes effect with the next score.
    pub fn set_evaluator(&mut self, evaluator: Arc<dyn Evaluator>) {
        self.evaluator = evaluator;
    }

    /// One tick for all sandboxes incl. trial reset and breeding at the end of the last trial.
    /// `parallel` steps the sandboxes (the tick uses no RNG) and breeds the next generation (each breeding slot with
    /// its own RNG streams) on the rayon pool; results never depend on it.
    pub fn step(&mut self, parallel: bool) -> Option<TrainerEvent> {
        self.epoch_ticks += 1;
        let live_fitness = self.track_live_fitness;
        let (config, evaluator) = (&self.config, self.evaluator.as_ref());
        let step_one = |sb: &mut TrainerSandbox| {
            if live_fitness {
                sb.current_fitness = evaluator.score(sb);
            }
            sb.step(config);
        };
        if parallel {
            self.sandboxes.par_iter_mut().for_each(step_one);
        } else {
            self.sandboxes.iter_mut().for_each(step_one);
        }

        if self.sandboxes.iter().all(TrainerSandbox::is_done) {
            return self.end_epoch(parallel);
        }
        None
    }

    /// Training mode of a requested switch that waits for the end of the current generation (as a UI mode).
    pub fn pending_scenario(&self) -> Option<Scenario> {
        self.pending.map(|s| s.mode.scenario())
    }

    /// Switches the UI training mode (fixed room or curriculum; `Curriculum` turns the gate on and keeps the level).
    /// Takes effect at the next generation (module doc).
    pub fn set_scenario(&mut self, scenario: Scenario) {
        self.set_mode(scenario.into());
    }

    /// Switches the training mode to any registry room or the curriculum (`set_scenario` for rooms without a UI mode).
    pub fn set_mode(&mut self, mode: TrainingMode) {
        self.request(RoomSwitch { mode, level: None });
    }

    /// Manual difficulty level: curriculum mode at `level` with the gate off. Takes effect at the next generation.
    pub fn set_level(&mut self, level: f32) {
        self.request(RoomSwitch {
            mode: TrainingMode::Curriculum,
            level: Some(snap_level(level)),
        });
    }

    fn request(&mut self, switch: RoomSwitch) {
        self.pending = Some(switch);
        // Nothing of this generation stepped yet: its start is the generation boundary
        if self.epoch_ticks == 0 && self.trial_index == 0 {
            self.apply_pending();
            self.restart_generation();
        }
    }

    /// Applies a pending switch to the params; returns whether there was one.
    fn apply_pending(&mut self) -> bool {
        let Some(switch) = self.pending.take() else {
            return false;
        };
        self.params.set_mode(switch.mode);
        match switch.level {
            Some(level) => {
                self.params.level = level;
                self.params.level_auto = false;
            }
            None if switch.mode == TrainingMode::Curriculum => self.params.level_auto = true,
            None => {}
        }
        true
    }

    /// Makes `spec` the room of the current trial (rebuilt only when it changed).
    fn use_room(&mut self, spec: RoomSpec) {
        if spec != self.room_spec {
            self.room = spec.build(&self.config);
            self.room_spec = spec;
        }
    }

    /// Restarts trial 0 of the current generation in the room the params now select (same shared layout stream as a
    /// freshly bred generation); nothing happens if the room did not change.
    fn restart_generation(&mut self) {
        self.plan_rehearsal();
        let spec = self.trial_room(0);
        if spec == self.room_spec {
            return;
        }
        self.use_room(spec);
        for sb in self.sandboxes.iter_mut() {
            sb.reset_trial_in(
                self.room.clone(),
                &mut streams::layout(self.seed, self.layout_id, 0),
                &self.config,
            );
        }
    }

    /// Trials per generation.
    fn trial_count(&self) -> u32 {
        if self.params.multi_trial {
            self.params.trials.max(1)
        } else {
            1
        }
    }

    /// Chooses the rehearsal trials of the current layout (curriculum only).
    fn plan_rehearsal(&mut self) {
        self.rehearsal = if self.params.mode() == TrainingMode::Curriculum {
            rehearsal_trials(
                self.seed,
                self.layout_id,
                self.trial_count(),
                self.params.curriculum.rehearsal_share,
            )
        } else {
            Vec::new()
        };
    }

    /// Room of `trial` of the current generation: the fixed room, or in curriculum mode the level room at the current
    /// level (rehearsal trials: `min(rehearsalLevel, level)`) with the metabolic ramp of the current level.
    fn trial_room(&self, trial: u32) -> RoomSpec {
        let p = &self.params;
        match p.mode() {
            TrainingMode::Room(id) => RoomSpec::Registered(id),
            TrainingMode::Curriculum => {
                let c = &p.curriculum;
                let level = if self.rehearsal.contains(&trial) {
                    c.rehearsal_level.min(p.level)
                } else {
                    p.level
                };
                RoomSpec::Level {
                    level,
                    metabolic_scale: metabolic_ramp(p.level, c.metabolic_ramp_start),
                }
            }
        }
    }

    /// Level of the generation's non-rehearsal trials: the curriculum level, or the fixed room's level.
    fn current_level(&self) -> f32 {
        match self.params.mode() {
            TrainingMode::Curriculum => self.params.level,
            TrainingMode::Room(_) => self.room.level(),
        }
    }

    /// Evaluates the current population in `room` without breeding or changing the sim (TCK-176 measurement): every
    /// sandbox (a clone, starting from its bred state) runs `trials` trials (`None`: the run's trial count) on the
    /// evaluation layouts `derive(seed, [streams::EVAL, trial])` through the run's evaluator, learned weights carried
    /// across its trials as in training; fitness is aggregated like the generation fitness. Deterministic; `parallel`
    /// does not change the result.
    pub fn evaluate_population(&self, room: &Arc<dyn Room>, trials: Option<u32>, parallel: bool) -> PopulationEval {
        let trials = trials.unwrap_or_else(|| self.trial_count()).max(1);
        let seed = self.seed;
        let layout = move |trial: u32| derive(seed, &[streams::EVAL, trial as u64]);
        let evals = self
            .evaluator
            .evaluate(room, &self.sandboxes, trials, &layout, &self.config, parallel);
        let aggregation = self.params.trial_aggregation;
        let results: Vec<(f32, u32, bool)> = evals
            .iter()
            .zip(&self.sandboxes)
            .map(|(e, sb)| {
                (
                    self.evaluator.aggregate(&e.trial_fitness, aggregation),
                    e.successes,
                    sb.origin_type == SandboxOrigin::Elite,
                )
            })
            .collect();
        let summary = |selected: Vec<&(f32, u32, bool)>| {
            (!selected.is_empty()).then(|| {
                let n = selected.len() as f32;
                (
                    selected.iter().map(|r| r.1).sum::<u32>() as f32 / (n * trials as f32),
                    selected.iter().map(|r| r.0).sum::<f32>() / n,
                )
            })
        };
        let all = summary(results.iter().collect()).unwrap_or((0.0, 0.0));
        let elites = summary(results.iter().filter(|r| r.2).collect());
        PopulationEval {
            level: room.level(),
            trials,
            success_rate: all.0,
            mean_fitness: all.1,
            elite_success_rate: elites.map(|e| e.0),
            elite_mean_fitness: elites.map(|e| e.1),
        }
    }

    /// Replaces the creature of a sandbox (catalogue assignment). The agent is rebuilt through the factory
    /// (phenotype, antisense, neuron buffers match the genome) and the sandbox starts the current trial afresh like a
    /// trial reset (`TrainerSandbox::reset_trial` on the trial's shared layout: start pose, start energy, food, trial
    /// record), while its trial clock stays that of the generation. The slot's trial fitness of this generation and its
    /// lineage are cleared (TCK-201), so the previous creature's trials do not count for it and the slot leaves the
    /// 1/5 rule and the mutation channel statistics. Returns the new creature, `None` for an unknown id.
    pub fn assign_creature(
        &mut self,
        sandbox_id: u32,
        genome: &str,
        methylations: Option<&[f32]>,
        synapse_weights: Option<Vec<f32>>,
    ) -> Option<AssignedCreature<'_>> {
        let slot = self.sandboxes.iter().position(|s| s.id == sandbox_id)?;
        let (tick, mut layout) = (
            self.epoch_ticks,
            streams::layout(self.seed, self.layout_id, self.trial_index),
        );
        let sb = &mut self.sandboxes[slot];
        let (id, generation) = (sb.agent().id, sb.agent().generation);
        *sb.agent_mut() = spawn_creature(
            genome,
            SpawnOptions {
                id,
                px: 0.0,
                py: 0.0,
                heading_angle: 0.0,
                generation,
                methylations,
                synapse_weights,
            },
            &self.config,
        );
        sb.reset_trial(&mut layout, &self.config);
        sb.chamber.tick = tick;

        if let Some(fitness) = self.acc_fitness.get_mut(slot) {
            fitness.clear();
        }
        if let Some(eaten) = self.acc_eaten.get_mut(slot) {
            *eaten = 0;
        }
        self.trial_records.retain(|r| r.slot != slot);
        self.breeder.forget_slot(slot);
        Some(AssignedCreature {
            sandbox_id,
            agent: self.sandboxes[slot].agent(),
        })
    }

    /// End of a trial: records every slot's trial score, then starts the next trial or completes the generation.
    fn end_epoch(&mut self, parallel: bool) -> Option<TrainerEvent> {
        let n = self.sandboxes.len();
        if self.acc_fitness.len() != n {
            self.acc_fitness = vec![Vec::new(); n];
        }
        if self.acc_eaten.len() != n {
            self.acc_eaten = vec![0; n];
        }
        let rehearsal = self.rehearsal.contains(&self.trial_index);
        for (slot, sb) in self.sandboxes.iter_mut().enumerate() {
            sb.current_fitness = self.evaluator.score(sb);
            let trial = sb.trial();
            self.trial_records.push(TrialRecord {
                slot,
                finish_tick: trial.finish_tick,
                distance_traveled: trial.distance_traveled,
                end_energy: sb.agent().energy,
                starved: trial.starved,
                rehearsal,
            });
            self.acc_fitness[slot].push(sb.current_fitness);
            if trial.finished {
                self.acc_eaten[slot] += 1;
            }
        }

        let trials = self.trial_count();
        if self.trial_index + 1 < trials {
            self.trial_index += 1;
            self.epoch_ticks = 0;
            self.use_room(self.trial_room(self.trial_index));
            for sb in self.sandboxes.iter_mut() {
                sb.reset_trial_in(
                    self.room.clone(),
                    &mut streams::layout(self.seed, self.layout_id, self.trial_index),
                    &self.config,
                );
            }
            return None;
        }

        // Caught food in every trial (layout advance rule)
        let solved: Vec<bool> = self.acc_eaten.iter().map(|&eaten| eaten >= trials).collect();
        if self.params.multi_trial {
            // Generation fitness = aggregate of the per-trial fitness (catch units, no gate)
            for (slot, sb) in self.sandboxes.iter_mut().enumerate() {
                sb.current_fitness = self
                    .evaluator
                    .aggregate(&self.acc_fitness[slot], self.params.trial_aggregation);
            }
        }
        self.trial_index = 0;
        self.acc_fitness.clear();
        self.acc_eaten.clear();

        Some(self.complete_generation(parallel, solved))
    }

    fn complete_generation(&mut self, parallel: bool, solved: Vec<bool>) -> TrainerEvent {
        let n = self.sandboxes.len();
        let mut stats = GenerationStats::compute(
            self.generation,
            &self.trial_records,
            &self.sandboxes,
            self.params.mode(),
        );
        self.trial_records.clear();
        stats.layout_id = self.layout_id;
        stats.mutation_ops_max = self.params.mutation_ops_max;
        stats.level = self.current_level();

        // Ranking, elites, parents and hall of fame, breeding statistics
        let selection = self
            .breeder
            .select(&mut self.sandboxes, solved, self.params.elite_ratio, self.generation);
        stats.child_success_share = selection.child_success_share();
        stats.mutation_channels = selection.mutation_channels.clone();
        stats.elite_last_mutation = selection.elite_last_mutation.clone();

        let best_fitness = self.sandboxes[0].current_fitness;
        let avg_fitness = self.sandboxes.iter().map(|s| s.current_fitness).sum::<f32>() / n as f32;
        let best_genome = self.sandboxes[0].agent().genome.clone();

        let advance = match self.params.layout_policy {
            LayoutPolicy::Hold => elites_solved_layout(&selection.ranked_solved, selection.elite_count),
            LayoutPolicy::Rotate => true,
        };
        if advance {
            self.layout_id += 1;
        }
        if self.params.one_fifth_rule {
            let limit = self.config.trainer.mutation_ops_limit;
            let p = &mut self.params;
            p.mutation_ops_max = selection.adapt_mutation_ops(p.mutation_ops_max, p.mutation_ops_min.max(1), limit);
        }
        // Curriculum gate on the non-rehearsal trials; a pending manual switch replaces it (module doc)
        if self.params.mode() == TrainingMode::Curriculum && self.params.level_auto && self.pending.is_none() {
            let (level, movement) = gate_level(self.params.level, stats.level_success_rate, &self.params.curriculum);
            self.params.level = level;
            stats.level_move = Some(movement);
        }
        self.apply_pending();

        let generation = self.generation;
        self.generation += 1;
        self.epoch_ticks = 0;
        self.breed(parallel);

        TrainerEvent::GenerationCompleted {
            generation,
            best_fitness,
            avg_fitness,
            best_genome,
            elites: selection.elites,
            stats,
        }
    }

    /// Builds the sandboxes of `self.generation` in the room of its trial 0 (curriculum: level and rehearsal choice
    /// of this generation) through the breeder.
    fn breed(&mut self, parallel: bool) {
        self.plan_rehearsal();
        self.use_room(self.trial_room(0));
        let n = self.params.population;
        self.sandboxes = self.breeder.breed(
            &self.params,
            self.seed,
            self.generation,
            self.layout_id,
            &self.room,
            &self.config,
            parallel,
        );
        self.trial_index = 0;
        self.acc_fitness = vec![Vec::new(); n];
        self.acc_eaten = vec![0; n];
        self.trial_records.clear();
    }
}

#[cfg(test)]
mod tests;
