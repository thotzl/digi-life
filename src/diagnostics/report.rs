//! Headless trainer run and its JSON report (used by `src/bin/trainer_runner.rs`).
//! Everything except `perf` is a pure function of the run config (same seed → identical report).

use std::time::Instant;

use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::config::AppConfig;
use crate::shared::rng::seed_from_str;
use crate::trainer::mode::TrainingMode;
use crate::trainer::room::{registry, Room};
use crate::trainer::sim::{snap_level, CurriculumParams, GenerationStats, LayoutPolicy, MutationChannelMode, PopulationEval, TrainerEvent, TrainerParams, TrainerSim, TrialAggregation};

/// Run settings: the trainer parameters of the run (`TrainerParams`, defaults from `TrainerParams::from_config`)
/// plus what only a headless run has.
#[derive(Debug, Clone, PartialEq)]
pub struct RunConfig {
    pub seed: String,
    pub params: TrainerParams,
    pub generations: u32,
    /// Room per generation span (switch test, TCK-176); empty = the mode of `params` for all generations. The spans'
    /// generation counts must add up to `generations`; the population and the generation counter carry over.
    pub schedule: Vec<ScheduleSpan>,
    /// Registry rooms in which the final population is evaluated without breeding (`final_eval`)
    pub eval_final: Vec<&'static str>,
    /// Trials per sandbox of the final evaluation (`None`: the run's trials)
    pub eval_trials: Option<u32>,
    pub parallel: bool,
}

/// `generations` generations in `scenario` (one span of `RunConfig::schedule`; a registry room id or "curriculum").
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScheduleSpan {
    pub scenario: TrainingMode,
    pub generations: u32,
}

impl RunConfig {
    /// Run settings with the trainer defaults (`TrainerParams::from_config`) in training mode `mode`; the curriculum
    /// starts at `curriculum.start_level`.
    pub fn with_defaults(seed: &str, mode: TrainingMode, population: usize, generations: u32, app: &AppConfig) -> Self {
        let mut params = TrainerParams { population, ..TrainerParams::from_config(app) };
        params.set_mode(mode);
        Self { seed: seed.to_string(), params, generations, schedule: Vec::new(), eval_final: Vec::new(), eval_trials: None, parallel: false }
    }

    /// Training mode of the first generation.
    pub fn first_mode(&self) -> TrainingMode {
        self.schedule.first().map_or(self.params.mode(), |span| span.scenario)
    }

    /// Params the sim starts with: the first schedule span's mode, the curriculum at its start level.
    pub fn start_params(&self) -> TrainerParams {
        let mut params = TrainerParams { level: snap_level(self.params.curriculum.start_level), ..self.params.clone() };
        params.set_mode(self.first_mode());
        params
    }
}

/// Report header: the run settings (same JSON as before TCK-201).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportMeta {
    pub seed: String,
    pub scenario: TrainingMode,
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
    /// Layout policy (hold / rotate)
    pub layout_policy: LayoutPolicy,
    /// Curriculum settings (TCK-176; used by curriculum generations only)
    pub curriculum: CurriculumParams,
    /// Room per generation span; absent for a single-room run
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schedule: Vec<ScheduleSpan>,
    /// Crate version that produced the report
    pub version: String,
}

impl ReportMeta {
    fn of(config: &RunConfig, params: &TrainerParams) -> Self {
        Self {
            seed: config.seed.clone(),
            scenario: config.first_mode(),
            population: params.population,
            generations: config.generations,
            trials: params.trials,
            trial_aggregation: params.trial_aggregation,
            multi_trial: params.multi_trial,
            elite_ratio: params.elite_ratio,
            mutation_ops_min: params.mutation_ops_min,
            mutation_ops_max: params.mutation_ops_max,
            one_fifth_rule: params.one_fifth_rule,
            minimal_criterion: params.minimal_criterion,
            mutation_channels: params.mutation_channels,
            layout_policy: params.layout_policy,
            curriculum: params.curriculum,
            schedule: config.schedule.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
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
    /// Final population evaluated per room without breeding (`--eval-final`); absent without it
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub final_eval: Vec<FinalEval>,
    pub perf: PerfStats,
}

/// Evaluation of the final population (after the last generation: its elites unmodified plus their offspring, as bred
/// for the next generation, not yet evaluated) in one room (`TrainerSim::evaluate_population`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalEval {
    /// Registry room id
    pub scenario: String,
    #[serde(flatten)]
    pub eval: PopulationEval,
}

/// Runs `config.generations` generations with the app config `app` and builds the report. `on_generation` receives
/// every completed generation (progress output, optional persistence by the caller).
pub fn run_trainer(config: &RunConfig, app: &AppConfig, mut on_generation: impl FnMut(&TrainerEvent, &TrainerSim)) -> TrainerReport {
    let params = config.start_params();
    let population = params.population.max(1);
    let meta = ReportMeta::of(config, &TrainerParams { population, ..params.clone() });
    let mut sim = TrainerSim::new(params, seed_from_str(&config.seed), app.trainer.trainer_genome_retention_per_run, app);
    // No telemetry: fitness is only needed at epoch end
    sim.track_live_fitness = false;

    let start = Instant::now();
    let mut ticks: u64 = 0;
    let mut generation_end_seconds = 0.0;
    let mut generations = Vec::with_capacity(config.generations as usize);
    // Generation counts after which the schedule switches to the next span's room
    let switches: Vec<(usize, TrainingMode)> = config
        .schedule
        .iter()
        .scan(0usize, |end, span| {
            *end += span.generations as usize;
            Some(*end)
        })
        .zip(config.schedule.iter().skip(1))
        .map(|(end, next)| (end, next.scenario))
        .collect();
    while generations.len() < config.generations as usize {
        ticks += 1;
        let tick_start = Instant::now();
        if let Some(event) = sim.step(config.parallel) {
            generation_end_seconds += tick_start.elapsed().as_secs_f64();
            // At a generation start the switch applies at once (the next generation runs in the new room); before the
            // callback, so it sees the state the next generation runs with (run state, TCK-191)
            if let Some(&(_, next)) = switches.iter().find(|(end, _)| *end == generations.len() + 1) {
                sim.set_mode(next);
            }
            on_generation(&event, &sim);
            let TrainerEvent::GenerationCompleted { stats, .. } = event;
            generations.push(stats);
        }
    }
    let wall_seconds = start.elapsed().as_secs_f64();
    let final_eval = config
        .eval_final
        .iter()
        .map(|&id| {
            let room: Arc<dyn Room> = registry::build(id, app).unwrap_or_else(|| panic!("room '{}' is registered", id));
            FinalEval { scenario: id.to_string(), eval: sim.evaluate_population(&room, config.eval_trials, config.parallel) }
        })
        .collect();
    let per_sandbox = if wall_seconds > 0.0 { ticks as f64 / wall_seconds } else { 0.0 };

    TrainerReport {
        meta,
        generations,
        final_eval,
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
        {
            let mut c = RunConfig { parallel, ..RunConfig::with_defaults(seed, TrainingMode::Room("standard"), 6, 2, crate::config::test_config()) };
            c.params.trials = 2;
            c
        }
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
        let a = run_trainer(&config("report-seed", false), crate::config::test_config(), |_, _| completed += 1);
        assert_eq!(completed, 2);
        assert_eq!(a.generations.len(), 2);
        assert_eq!(a.meta.version, env!("CARGO_PKG_VERSION"));

        let b = run_trainer(&config("report-seed", false), crate::config::test_config(), |_, _| {});
        let parallel = run_trainer(&config("report-seed", true), crate::config::test_config(), |_, _| {});
        assert_eq!(reproducible_json(&a), reproducible_json(&b));
        assert_eq!(reproducible_json(&a), reproducible_json(&parallel));

        let other = run_trainer(&config("other-seed", false), crate::config::test_config(), |_, _| {});
        assert_ne!(a.generations, other.generations);
        // Single-room runs carry no schedule or evaluation keys
        let json = reproducible_json(&a);
        assert!(!json.contains("\"schedule\"") && !json.contains("\"final_eval\""));
    }

    #[test]
    fn test_schedule_switches_rooms_and_keeps_the_population() {
        let schedule = vec![
            ScheduleSpan { scenario: TrainingMode::Room("standard"), generations: 2 },
            ScheduleSpan { scenario: TrainingMode::Room("exploration"), generations: 1 },
            ScheduleSpan { scenario: TrainingMode::Room("standard"), generations: 1 },
        ];
        let standard = TrainingMode::Room("standard");
        let one_trial = |generations: u32| {
            let mut c = RunConfig::with_defaults("schedule-seed", standard, 4, generations, crate::config::test_config());
            c.params.trials = 1;
            c
        };
        let scheduled = |parallel: bool| RunConfig { schedule: schedule.clone(), eval_final: vec!["standard", "exploration"], parallel, ..one_trial(4) };
        let report = run_trainer(&scheduled(false), crate::config::test_config(), |_, _| {});
        let rooms: Vec<(u32, &str, f32)> = report.generations.iter().map(|g| (g.generation, g.scenario.id(), g.level)).collect();
        assert_eq!(rooms, vec![(1, "standard", 0.0), (2, "standard", 0.0), (3, "exploration", 1.0), (4, "standard", 0.0)]);
        // The first span equals a plain run of the same room
        let plain = run_trainer(&one_trial(2), crate::config::test_config(), |_, _| {});
        assert_eq!(report.generations[..2], plain.generations[..]);
        // Final evaluation per room, deterministic and parallel-safe
        assert_eq!(report.final_eval.iter().map(|e| (e.scenario.as_str(), e.eval.level, e.eval.trials)).collect::<Vec<_>>(), vec![("standard", 0.0, 1), ("exploration", 1.0, 1)]);
        assert_eq!(reproducible_json(&report), reproducible_json(&run_trainer(&scheduled(true), crate::config::test_config(), |_, _| {})));
        assert_eq!(report.meta.schedule, schedule);
    }
}
