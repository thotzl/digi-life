//! Headless trainer run and its JSON report (used by `src/bin/trainer_runner.rs`).
//! Everything except `perf` is a pure function of the run config (same seed → identical report).

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::biology::scenarios::Scenario;
use crate::biology::trainer_sim::{GenerationStats, MutationChannelMode, TrainerEvent, TrainerParams, TrainerSim, TrialAggregation};
use crate::shared::rng::seed_from_str;
use crate::shared::types::AppConfig;

/// Run settings; all other trainer parameters are the UI defaults (`TrainerParams::default`).
#[derive(Debug, Clone, PartialEq)]
pub struct RunConfig {
    pub seed: String,
    pub scenario: Scenario,
    pub population: usize,
    pub generations: u32,
    pub trials: u32,
    pub trial_aggregation: TrialAggregation,
    pub multi_trial: bool,
    pub one_fifth_rule: bool,
    pub elite_ratio: f32,
    /// Minimal criterion for generation 1 and random immigrants (RV-S09)
    pub minimal_criterion: bool,
    /// Body/brain mutation channels (TCK-181, experimental)
    pub mutation_channels: MutationChannelMode,
    pub parallel: bool,
}

impl RunConfig {
    /// Run settings with the trainer defaults (config.json / `TrainerParams::default`) for everything but the
    /// required run identity.
    pub fn with_defaults(seed: &str, scenario: Scenario, population: usize, generations: u32) -> Self {
        let defaults = TrainerParams::default();
        Self {
            seed: seed.to_string(),
            scenario,
            population,
            generations,
            trials: defaults.trials,
            trial_aggregation: defaults.trial_aggregation,
            multi_trial: defaults.multi_trial,
            one_fifth_rule: defaults.one_fifth_rule,
            elite_ratio: defaults.elite_ratio,
            minimal_criterion: defaults.minimal_criterion,
            mutation_channels: defaults.mutation_channels,
            parallel: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportMeta {
    pub seed: String,
    pub scenario: Scenario,
    pub population: usize,
    pub generations: u32,
    pub trials: u32,
    pub trial_aggregation: TrialAggregation,
    pub multi_trial: bool,
    pub elite_ratio: f32,
    pub mutation_ops_min: u32,
    /// Max mutation ops per child at the start of the run (the 1/5 rule may adapt it; see the generations)
    pub mutation_ops_max: u32,
    pub one_fifth_rule: bool,
    /// Minimal criterion for random genomes was on (RV-S09)
    pub minimal_criterion: bool,
    /// Body/brain mutation channel mode (TCK-181)
    pub mutation_channels: MutationChannelMode,
    /// Crate version that produced the report
    pub version: String,
}

/// Throughput of the run; excluded from reproducibility comparisons.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfStats {
    /// Sandbox ticks per second, summed over all sandboxes
    pub ticks_per_second_total: f64,
    pub ticks_per_second_per_sandbox: f64,
    pub wall_seconds: f64,
    /// Share of `wall_seconds` spent in ticks that ended a generation (fitness, stats, breeding)
    pub generation_end_seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrainerReport {
    pub meta: ReportMeta,
    pub generations: Vec<GenerationStats>,
    pub perf: PerfStats,
}

impl RunConfig {
    pub fn params(&self) -> TrainerParams {
        TrainerParams {
            population: self.population,
            multi_trial: self.multi_trial,
            trials: self.trials,
            trial_aggregation: self.trial_aggregation,
            one_fifth_rule: self.one_fifth_rule,
            elite_ratio: self.elite_ratio,
            minimal_criterion: self.minimal_criterion,
            mutation_channels: self.mutation_channels,
            scenario: self.scenario,
            ..TrainerParams::default()
        }
    }
}

/// Runs `config.generations` generations and builds the report. `on_generation` receives every
/// completed generation (progress output, optional persistence by the caller).
pub fn run_trainer(config: &RunConfig, mut on_generation: impl FnMut(&TrainerEvent)) -> TrainerReport {
    let params = config.params();
    let population = params.population.max(1);
    let (mutation_ops_min, mutation_ops_max) = (params.mutation_ops_min, params.mutation_ops_max);
    let mut sim = TrainerSim::new(params, seed_from_str(&config.seed), AppConfig::global().trainer_genome_retention_per_run);
    // No telemetry: fitness is only needed at epoch end
    sim.track_live_fitness = false;

    let start = Instant::now();
    let mut ticks: u64 = 0;
    let mut generation_end_seconds = 0.0;
    let mut generations = Vec::with_capacity(config.generations as usize);
    while generations.len() < config.generations as usize {
        ticks += 1;
        let tick_start = Instant::now();
        if let Some(event) = sim.step(config.parallel) {
            generation_end_seconds += tick_start.elapsed().as_secs_f64();
            on_generation(&event);
            let TrainerEvent::GenerationCompleted { stats, .. } = event;
            generations.push(stats);
        }
    }
    let wall_seconds = start.elapsed().as_secs_f64();
    let per_sandbox = if wall_seconds > 0.0 { ticks as f64 / wall_seconds } else { 0.0 };

    TrainerReport {
        meta: ReportMeta {
            seed: config.seed.clone(),
            scenario: config.scenario,
            population,
            generations: config.generations,
            trials: config.trials,
            trial_aggregation: config.trial_aggregation,
            multi_trial: config.multi_trial,
            elite_ratio: config.elite_ratio,
            mutation_ops_min,
            mutation_ops_max,
            one_fifth_rule: config.one_fifth_rule,
            minimal_criterion: config.minimal_criterion,
            mutation_channels: config.mutation_channels,
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        generations,
        perf: PerfStats {
            ticks_per_second_total: per_sandbox * population as f64,
            ticks_per_second_per_sandbox: per_sandbox,
            wall_seconds,
            generation_end_seconds,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(seed: &str, parallel: bool) -> RunConfig {
        RunConfig { trials: 2, parallel, ..RunConfig::with_defaults(seed, Scenario::Standard, 6, 2) }
    }

    /// Serialized report without `perf`.
    fn reproducible_json(report: &TrainerReport) -> String {
        let mut value = serde_json::to_value(report).unwrap();
        value.as_object_mut().unwrap().remove("perf");
        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn test_same_seed_identical_report_and_parallel_equals_sequential() {
        let mut completed = 0;
        let a = run_trainer(&config("report-seed", false), |_| completed += 1);
        assert_eq!(completed, 2);
        assert_eq!(a.generations.len(), 2);
        assert_eq!(a.meta.version, env!("CARGO_PKG_VERSION"));

        let b = run_trainer(&config("report-seed", false), |_| {});
        let parallel = run_trainer(&config("report-seed", true), |_| {});
        assert_eq!(reproducible_json(&a), reproducible_json(&b));
        assert_eq!(reproducible_json(&a), reproducible_json(&parallel));

        let other = run_trainer(&config("other-seed", false), |_| {});
        assert_ne!(a.generations, other.generations);
    }
}
