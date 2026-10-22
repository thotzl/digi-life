//! Headless trainer run and its JSON report (used by `src/bin/trainer_runner.rs`).
//! Everything except `perf` is a pure function of the run config (same seed → identical report).

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::biology::scenarios::Scenario;
use crate::biology::trainer_sim::{GenerationStats, TrainerEvent, TrainerParams, TrainerSim};
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
    pub multi_trial: bool,
    pub parallel: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportMeta {
    pub seed: String,
    pub scenario: Scenario,
    pub population: usize,
    pub generations: u32,
    pub trials: u32,
    pub multi_trial: bool,
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
            multi_trial: config.multi_trial,
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
        RunConfig { seed: seed.to_string(), scenario: Scenario::Standard, population: 6, generations: 2, trials: 2, multi_trial: true, parallel }
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
