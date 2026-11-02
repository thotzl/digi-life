//! Config section `trainer` (`TrainerConfig`): defaults of the trainer run (`TrainerParams::from_config`), fitness
//! weights, spawn distances, curriculum and the minimal criterion. Read by `trainer_sim`, `trainer_engine` and
//! `scenarios`.

use serde::{Deserialize, Serialize};

/// Section `trainer` of `config.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerConfig {
    /// Trainer population (sandboxes per generation) of the app, `TrainerParams::from_config().population`;
    /// independent of the displayed sandbox tiles (TCK-192). The runner's `--population` overrides it. Missing in older
    /// config overrides: 100.
    #[serde(default = "default_trainer_population")]
    pub trainer_population: usize,
    /// Trainer genomes kept per run in the DB and in the in-memory hall of fame (best by fitness)
    pub trainer_genome_retention_per_run: usize,
    /// Trials per trainer generation with multi-trial on (TCK-174)
    pub trainer_trials: u32,
    /// Per-trial fitness aggregation of the trainer: "mean" or "median" (TCK-174)
    pub trainer_trial_aggregation: TrialAggregation,
    /// Mutation operator calls per trainer child (mutant and hall-of-fame slots), uniform in [min, max] (TCK-174)
    pub mutation_ops_min: u32,
    pub mutation_ops_max: u32,
    /// Population-level 1/5 success rule on the trainer's max ops per child (TCK-174; on in config.json since P7)
    pub mutation_one_fifth_rule: bool,
    /// Upper bound of the max ops per child (1/5 rule and UI slider); the lower bound is max(1, mutationOpsMin)
    pub mutation_ops_limit: u32,
    /// Body/brain mutation channels of trainer children: "mixed" (default) or "separate" (TCK-181, experimental)
    pub mutation_channels: MutationChannelMode,
    /// When the shared trial layout changes: "hold" (until all elites solve it) or "rotate" (every generation)
    pub trainer_layout_policy: LayoutPolicy,
    /// Trainer fitness weights (TCK-175)
    pub trainer_fitness: TrainerFitnessConfig,
    /// Minimum food distance per trainer room for trial starts and respawns after eating (TCK-154 / RV-S08); the same
    /// value for the first trial, every re-trial and every respawn.
    ///
    /// Choice (sensor range = organ scale · 550 px; food-sensing organs of structured genomes p10/p50/p90 = 256 / 575 /
    /// 926 px, max 990; equipment probe, 2000 genomes):
    /// - `standard` 200 px (1000 px chamber, centre start, farthest point ≈ 670 px): food is never at the spawn, and
    ///   for 89 % of structured genomes some food is in sensor range at spawn (homing task).
    /// - `exploration` 500 px (3500 px chamber): just below the median sensor range, so a median organ reaches food at
    ///   the minimum distance; food is in range at spawn for 35 % of structured genomes (in the view cone for 13 %), the
    ///   rest must search. For comparison 650 px (old first trial): 23 %, 800 px: 10 %, 1200 px (old re-trials): 0 %.
    pub trainer_spawn_min_distance: RoomPair<f32>,
    /// Difficulty level, gate, rehearsal and metabolic ramp of the training curriculum (TCK-176)
    pub trainer_curriculum: TrainerCurriculumConfig,
    /// Minimal criterion for random trainer genomes (RV-S09; on in config.json since P7)
    pub trainer_minimal_criterion: TrainerMinimalCriterion,
}

fn default_trainer_population() -> usize {
    100
}

/// Weights of the trainer fitness in catch units (`scenarios::calculate_fitness`, TCK-175). Both stay below 1 so a
/// catch with full diet fit always outweighs the bonuses of a trial without one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerFitnessConfig {
    /// Bonus of a first catch at tick 0, falling linearly to 0 at the end of the trial (time to first catch)
    pub first_catch_bonus: f32,
    /// Bonus for full relative approach towards the nearest diet target that came into food-sensing range (shaping)
    pub approach_bonus: f32,
}

/// One value per fixed trainer room: `standard` = difficulty level 0, `exploration` = level 1 (TCK-176).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomPair<T> {
    pub standard: T,
    pub exploration: T,
}

/// Automatic training curriculum (TCK-176): one difficulty level `d` in [0, 1] whose room interpolates between the
/// small room (`standard`, d = 0) and the large room (`exploration`, d = 1); see `scenarios::LevelScenario`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerCurriculumConfig {
    /// Gate: success rate of the non-rehearsal trials above this raises the level by `step`
    pub upper: f32,
    /// Gate: success rate below this lowers the level by `step`
    pub lower: f32,
    pub step: f32,
    /// Level of a new run in curriculum mode
    pub start_level: f32,
    /// Share of the trials per generation run at the rehearsal level (rounded to whole trials, at least one trial
    /// stays at the current level); 0 = off
    pub rehearsal_share: f32,
    /// Level of the rehearsal trials (capped at the current level)
    pub rehearsal_level: f32,
    /// Metabolic pressure ramp in curriculum mode: drain scale `rampStart + (1 − rampStart) · d` times
    /// `metabolism.trainerMetabolicScale`; 1 = no ramp
    pub metabolic_ramp_start: f32,
    /// Chamber edge length (px) of the fixed rooms; geometric in d between them
    pub chamber_size: RoomPair<f32>,
    /// Ticks per trial of the fixed rooms; geometric in d between them
    pub epoch_ticks: RoomPair<u32>,
    /// Plants and meatballs per trial of the fixed rooms; linear in d, rounded
    pub food_per_type: RoomPair<u32>,
}

/// Minimal criterion for random trainer genomes (generation 1 and immigrants, RV-S09): at least one food-sensing
/// organ (eye or nose) and able to move (`trainer_engine::meets_minimal_criterion`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainerMinimalCriterion {
    pub enabled: bool,
    /// Redraws after the first candidate; when none passes, the last candidate is kept (bounded, deterministic)
    pub max_redraws: u32,
    /// Ticks of the mobility check (fixed standard-room trial)
    pub mobility_ticks: u32,
    /// Path length (px) the creature must cover within `mobility_ticks`
    pub mobility_min_distance: f32,
}

/// How the trainer combines the per-trial fitness of a sandbox into its generation fitness (TCK-174).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrialAggregation {
    #[default]
    Mean,
    Median,
}

/// Mutation channels of the trainer's mutated children (TCK-181): `Mixed` lets every operator call hit any gene;
/// `Separate` gives each child one channel (body or brain genes, drawn per child from its breeding stream) and
/// restricts all its operator calls to genes of that channel (`genome::mutate_genome_in_channel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MutationChannelMode {
    #[default]
    Mixed,
    Separate,
}

/// When the shared trial layout of the trainer changes (TCK-174, revisited in P8). `Hold` keeps a layout until every
/// elite solves it; `Rotate` uses a new layout every generation, so selection cannot reward memorising one layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutPolicy {
    Hold,
    #[default]
    Rotate,
}
