//! Typed trainer IPC (TCK-177, TCK-157): the trainer events on `simulation-state` (`UiEvent::Trainer`) and the
//! trainer actions of `handle_client_action` (`ClientAction::Trainer`). Both are tagged by `type`; the TypeScript
//! types are generated into `bindings/` by ts-rs (`cargo test`). Contract: `wiki/contracts/ipc-message.md`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::shared::dto::FoodSpore;
use crate::trainer::mode::Scenario;
use crate::trainer::sandbox::{SandboxOrigin, TrainerSandbox};
use crate::trainer::sim::{GenerationStats, LevelMove, TrainerSim};

/// Trainer events sent to the UI (payload of the `simulation-state` event, tag `type`).
#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum TrainerUiEvent {
    TrainerTelemetryTick(TrainerTelemetryTick),
    TrainerGenerationCompleted(TrainerGenerationCompleted),
    TrainerStateChanged(TrainerState),
    TrainerResetCompleted(TrainerState),
}

/// Slot indices of the displayed sandboxes (TCK-194): a sample of `n` slots with every origin group (elite, hall of
/// fame, random, mutant) in the share it has in the population, so the tiles look like a population of `n` (pop 100,
/// 16 tiles: 4 / 2 / 2 / 8). Each present group gets `round(n · group / population)` slots, at least one; extra
/// slots go to the group with the most spare slots, missing ones come off the group with the most slots (the last of
/// equals). Takes the first slots of each group (elites: best first) and returns them in slot order. All slots when
/// `n` covers the population.
pub fn display_slots(origins: &[SandboxOrigin], n: usize) -> Vec<usize> {
    let len = origins.len();
    if n >= len {
        return (0..len).collect();
    }
    let groups: Vec<Vec<usize>> = [SandboxOrigin::Elite, SandboxOrigin::Hof, SandboxOrigin::Random, SandboxOrigin::Mutant]
        .iter()
        .map(|origin| (0..len).filter(|&i| origins[i] == *origin).collect())
        .collect();
    let mut quota: Vec<usize> = groups
        .iter()
        .map(|g| if g.is_empty() { 0 } else { ((n * g.len()) as f32 / len as f32).round().max(1.0) as usize })
        .collect();
    while quota.iter().sum::<usize>() < n {
        let g = (0..groups.len()).max_by_key(|&g| groups[g].len() - quota[g]).unwrap_or(0);
        quota[g] += 1;
    }
    while quota.iter().sum::<usize>() > n {
        let g = (0..groups.len()).max_by_key(|&g| quota[g]).unwrap_or(0);
        quota[g] -= 1;
    }
    let mut slots: Vec<usize> = groups.iter().zip(&quota).flat_map(|(g, &q)| g.iter().take(q).copied()).collect();
    slots.sort_unstable();
    slots
}

/// One displayed sandbox in the live frame (`TrainerTelemetryTick`).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TrainerTelemetrySandbox {
    pub id: u32,
    #[serde(rename = "chamberSize")]
    #[ts(rename = "chamberSize")]
    pub chamber_size: f32,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub heading_angle: f32,
    pub omega_rot: f32,
    pub finished: bool,
    pub current_fitness: f32,
    pub origin_type: SandboxOrigin,
    pub consumed_spore_type: Option<String>,
    pub foods: Vec<FoodSpore>,
    pub latin_name: String,
    pub primary_color_h: f32,
    pub primary_color_s: f32,
    pub primary_color_l: f32,
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    pub genome: String,
    /// Training mode of the trainer (fixed room or curriculum)
    pub scenario: Scenario,
    /// Difficulty level of this sandbox's current trial room (TCK-176; 0 = small room, 1 = large room)
    pub level: f32,
    pub cumulative_rotation: f32,
    pub coverage_mask: u64,
}

/// Live frame of the displayed sandboxes (25 Hz): a sample of `N` covering every origin group (TCK-192, TCK-194).
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainerTelemetryTick {
    pub generation: u32,
    pub is_running: bool,
    /// Remaining time of the current trial, e.g. "4.2s"
    pub time_str: String,
    pub sandboxes: Vec<TrainerTelemetrySandbox>,
    pub selected_brain: Option<TrainerSelectedBrain>,
}

impl TrainerTelemetryTick {
    /// Frame of `display_count` sandboxes chosen by `display_slots` (TCK-192, TCK-194). Within a generation the
    /// sandboxes keep their breeding slot order (ids 1..=population: elites, hall of fame, random immigrants,
    /// mutants), so the displayed ids are stable for the whole generation. The payload size depends on
    /// `display_count`, not on the population. The brain view gets activations only for a displayed sandbox.
    /// Reads the sim only: the display count never changes the simulation.
    pub fn build(trainer: &TrainerSim, display_count: usize, selected_sandbox_id: Option<u32>, is_running: bool) -> Self {
        let origins: Vec<SandboxOrigin> = trainer.sandboxes.iter().map(|sb| sb.origin_type).collect();
        let displayed: Vec<&TrainerSandbox> = display_slots(&origins, display_count).into_iter().map(|i| &trainer.sandboxes[i]).collect();
        let scenario = trainer.params.scenario;
        let sandboxes = displayed
            .iter()
            .map(|sb| {
                let (agent, trial) = (sb.agent(), sb.trial());
                TrainerTelemetrySandbox {
                    id: sb.id,
                    chamber_size: sb.chamber.size,
                    px: agent.px,
                    py: agent.py,
                    vx: agent.vx,
                    vy: agent.vy,
                    heading_angle: agent.heading_angle,
                    omega_rot: agent.omega_rot,
                    finished: trial.finished,
                    current_fitness: sb.current_fitness,
                    origin_type: sb.origin_type,
                    consumed_spore_type: trial.consumed_spore_type.clone(),
                    foods: sb.chamber.foods.clone(),
                    latin_name: agent.phenotype.latin_name.clone(),
                    primary_color_h: agent.phenotype.primary_color.h,
                    primary_color_s: agent.phenotype.primary_color.s,
                    primary_color_l: agent.phenotype.primary_color.l,
                    energy: agent.energy,
                    age: agent.age,
                    generation: agent.generation,
                    genome: agent.genome.clone(),
                    scenario,
                    level: sb.room.telemetry(&sb.chamber).level,
                    cumulative_rotation: trial.cumulative_rotation,
                    coverage_mask: trial.coverage_mask,
                }
            })
            .collect();
        let time_str = format!("{:.1}s", ((trainer.room().episode_ticks() as f32 - trainer.epoch_ticks as f32) / 60.0).max(0.0));
        let selected_brain = selected_sandbox_id.and_then(|sel_id| {
            displayed.iter().find(|s| s.id == sel_id).map(|sb| TrainerSelectedBrain { id: sel_id, activations: sb.agent().neuron_activations.clone() })
        });
        Self { generation: trainer.generation, is_running, time_str, sandboxes, selected_brain }
    }
}

/// Live neuron activations of the selected sandbox (brain view).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct TrainerSelectedBrain {
    pub id: u32,
    pub activations: Vec<f32>,
}

/// End of a generation: champion, the generation's report stats and the trainer state for the next generation.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainerGenerationCompleted {
    pub generation: u32,
    pub best_fitness: f32,
    pub avg_fitness: f32,
    pub best_genome: String,
    /// Max mutation ops per child for the next generation (adapted by the 1/5 rule, TCK-174)
    pub mutation_ops_max: u32,
    /// Training mode and difficulty level of the next generation (TCK-176)
    pub training_mode: Scenario,
    pub next_level: f32,
    /// Per-generation stats of the headless report (`GenerationStats`), one source for UI and report
    pub stats: TrainerGenerationStats,
}

/// UI subset of `GenerationStats` (same values as the report's `generations[]` entry).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainerGenerationStats {
    pub generation: u32,
    /// Share of sandbox-trials with at least one catch (all trials, rehearsal included)
    pub success_rate: f32,
    /// Success rate of the non-rehearsal trials (curriculum gate input)
    pub level_success_rate: f32,
    pub rehearsal_success_rate: Option<f32>,
    pub rehearsal_trials: u32,
    pub best_fitness: f32,
    pub median_fitness: f32,
    pub mean_fitness: f32,
    /// Ticks (60 per simulated second) to the first catch over the successful trials; `null` without any catch
    pub mean_time_to_first_catch: Option<f32>,
    pub median_time_to_first_catch: Option<f32>,
    pub immobile_share: f32,
    pub starved_share: f32,
    pub scenario: Scenario,
    /// Difficulty level of the generation's non-rehearsal trials (0 = small room, 1 = large room)
    pub level: f32,
    /// Gate decision after this generation; `null` when the gate is off
    pub level_move: Option<LevelMove>,
    /// Max mutation ops per child used to breed this generation
    pub mutation_ops_max: u32,
    /// Shared layout id as a decimal string (u64 does not fit a JS number)
    pub layout_id: String,
}

impl From<&GenerationStats> for TrainerGenerationStats {
    fn from(s: &GenerationStats) -> Self {
        Self {
            generation: s.generation,
            success_rate: s.success_rate,
            level_success_rate: s.level_success_rate,
            rehearsal_success_rate: s.rehearsal_success_rate,
            rehearsal_trials: s.rehearsal_trials,
            best_fitness: s.best_fitness,
            median_fitness: s.median_fitness,
            mean_fitness: s.mean_fitness,
            mean_time_to_first_catch: s.mean_time_to_first_catch,
            median_time_to_first_catch: s.median_time_to_first_catch,
            immobile_share: s.immobile_share,
            starved_share: s.starved_share,
            scenario: s.scenario.scenario(),
            level: s.level,
            level_move: s.level_move,
            mutation_ops_max: s.mutation_ops_max,
            layout_id: s.layout_id.to_string(),
        }
    }
}

/// Trainer settings as the UI shows them (TRAINER_STATE_CHANGED / TRAINER_RESET_COMPLETED).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainerState {
    pub is_running: bool,
    /// Sandboxes per generation; selection, breeding and stats run over all of them (TCK-192)
    pub population: usize,
    /// Displayed sandboxes (grid tiles, telemetry), at most `population`
    #[serde(rename = "N")]
    pub display_count: usize,
    pub warp_speed: usize,
    pub elite_ratio: f32,
    /// Kept field name: since TCK-174 the max mutation operator calls per child (`mutation_ops_max`), slider range
    /// [max(1, mutationOpsMin), mutationOpsLimit]
    pub mutation_rate: u32,
    pub mutation_ops_min: u32,
    pub mutation_ops_limit: u32,
    pub inflow_rate: f32,
    pub hof_rate: f32,
    pub multi_trial: bool,
    pub is_headless: bool,
    pub run_id: String,
    pub lamarckian: bool,
    /// Curriculum (TCK-176): training mode, level of the current room, gate on/off, and a switch waiting for the end
    /// of the generation
    pub training_mode: Scenario,
    pub level: f32,
    pub level_auto: bool,
    pub pending_mode: Option<Scenario>,
    /// Current generation (1 after a reset)
    pub generation: u32,
}

/// Trainer actions from the UI (`handle_client_action`, tag `type`; one group of `ClientAction`).
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum TrainerAction {
    UpdateTrainerHyperparams(TrainerHyperparamsUpdate),
    StartTraining,
    PauseTraining,
    TrainerReset,
    /// Training mode switch; takes effect at the next generation (TCK-176)
    SetTrainerMode { mode: Scenario },
    /// Manual difficulty level in [0, 1] (turns the curriculum gate off); takes effect at the next generation
    SetTrainerLevel { level: f32 },
    SelectTrainerSandbox {
        #[serde(default)]
        #[ts(optional)]
        id: Option<u32>,
    },
    AssignSandboxCreature(AssignSandboxCreature),
}

/// Changed trainer hyperparameters; absent fields stay unchanged.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainerHyperparamsUpdate {
    /// Displayed sandboxes (grid tiles, telemetry); never changes the simulation (TCK-192)
    #[serde(rename = "N", default)]
    #[ts(optional)]
    pub display_count: Option<usize>,
    /// Population size; a change restores the population from the DB
    #[serde(default)]
    #[ts(optional)]
    pub population: Option<usize>,
    #[serde(default)]
    #[ts(optional)]
    pub warp_speed: Option<usize>,
    #[serde(default)]
    #[ts(optional)]
    pub elite_ratio: Option<f32>,
    /// Max mutation ops per child (TCK-174; field name kept)
    #[serde(default)]
    #[ts(optional)]
    pub mutation_rate: Option<f64>,
    #[serde(default)]
    #[ts(optional)]
    pub inflow_rate: Option<f32>,
    #[serde(default)]
    #[ts(optional)]
    pub hof_rate: Option<f32>,
    #[serde(default)]
    #[ts(optional)]
    pub multi_trial: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub is_headless: Option<bool>,
    #[serde(default)]
    #[ts(optional)]
    pub run_id: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub lamarckian: Option<bool>,
}

/// Puts a catalogue creature into a sandbox (field names as sent by the diagnostics panel).
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[ts(export)]
pub struct AssignSandboxCreature {
    pub sandbox_id: u32,
    pub genome: String,
    #[serde(default)]
    #[ts(optional)]
    pub methylations: Option<Vec<f32>>,
    #[serde(default)]
    #[ts(optional)]
    pub synapse_weights: Option<Vec<f32>>,
    #[serde(default)]
    #[ts(optional)]
    pub load_learned_synapses: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(v: serde_json::Value) -> Option<TrainerAction> {
        serde_json::from_value(v).ok()
    }

    #[test]
    fn test_trainer_actions_parse_from_ui_json() {
        assert_eq!(parse(json!({ "type": "START_TRAINING" })), Some(TrainerAction::StartTraining));
        assert_eq!(parse(json!({ "type": "PAUSE_TRAINING" })), Some(TrainerAction::PauseTraining));
        assert_eq!(parse(json!({ "type": "TRAINER_RESET" })), Some(TrainerAction::TrainerReset));
        assert_eq!(parse(json!({ "type": "SET_TRAINER_MODE", "mode": "curriculum" })), Some(TrainerAction::SetTrainerMode { mode: Scenario::Curriculum }));
        assert_eq!(parse(json!({ "type": "SET_TRAINER_LEVEL", "level": 0.4 })), Some(TrainerAction::SetTrainerLevel { level: 0.4 }));
        assert_eq!(parse(json!({ "type": "SELECT_TRAINER_SANDBOX", "id": 3 })), Some(TrainerAction::SelectTrainerSandbox { id: Some(3) }));
        assert_eq!(parse(json!({ "type": "SELECT_TRAINER_SANDBOX", "id": null })), Some(TrainerAction::SelectTrainerSandbox { id: None }));
        let update = parse(json!({ "type": "UPDATE_TRAINER_HYPERPARAMS", "N": 16, "population": 300, "warpSpeed": 2, "eliteRatio": 0.25, "runId": "r1", "lamarckian": true }));
        assert_eq!(
            update,
            Some(TrainerAction::UpdateTrainerHyperparams(TrainerHyperparamsUpdate {
                display_count: Some(16),
                population: Some(300),
                warp_speed: Some(2),
                elite_ratio: Some(0.25),
                run_id: Some("r1".to_string()),
                lamarckian: Some(true),
                ..Default::default()
            }))
        );
        let assign = parse(json!({ "type": "ASSIGN_SANDBOX_CREATURE", "sandbox_id": 2, "genome": "AB", "synapse_weights": null, "load_learned_synapses": false }));
        assert!(matches!(assign, Some(TrainerAction::AssignSandboxCreature(AssignSandboxCreature { sandbox_id: 2, methylations: None, synapse_weights: None, .. }))));
    }

    #[test]
    fn test_non_trainer_actions_are_not_trainer_actions() {
        assert_eq!(parse(json!({ "type": "SET_MODE", "mode": "trainer" })), None);
        assert_eq!(parse(json!({ "type": "TOGGLE_SIMULATION", "running": false })), None);
        assert_eq!(parse(json!({ "type": "SOMETHING_NEW" })), None);
    }

    /// The UI stats are the report's `GenerationStats` values (same JSON numbers), so UI and headless report agree.
    #[test]
    fn test_generation_stats_match_report_serialization() {
        use crate::trainer::sim::{TrainerEvent, TrainerParams, TrainerSim};
        let mut sim = TrainerSim::new(TrainerParams { population: 4, ..TrainerParams::from_config(crate::config::test_config()) }, 7, 4, crate::config::test_config());
        let stats = loop {
            if let Some(TrainerEvent::GenerationCompleted { stats, .. }) = sim.step(false) {
                break stats;
            }
        };
        let report = serde_json::to_value(&stats).unwrap();
        let ui = serde_json::to_value(TrainerGenerationStats::from(&stats)).unwrap();
        for (camel, snake) in [
            ("generation", "generation"),
            ("successRate", "success_rate"),
            ("levelSuccessRate", "level_success_rate"),
            ("rehearsalSuccessRate", "rehearsal_success_rate"),
            ("bestFitness", "best_fitness"),
            ("medianFitness", "median_fitness"),
            ("meanFitness", "mean_fitness"),
            ("meanTimeToFirstCatch", "mean_time_to_first_catch"),
            ("medianTimeToFirstCatch", "median_time_to_first_catch"),
            ("level", "level"),
            ("levelMove", "level_move"),
            ("mutationOpsMax", "mutation_ops_max"),
        ] {
            assert_eq!(ui[camel], report[snake], "{}", camel);
        }
        assert_eq!(ui["layoutId"], report["layout_id"].to_string());
    }

    #[test]
    fn test_trainer_events_keep_type_tag_and_ui_field_names() {
        let state = TrainerState {
            is_running: true,
            population: 100,
            display_count: 16,
            warp_speed: 1,
            elite_ratio: 0.25,
            mutation_rate: 3,
            mutation_ops_min: 1,
            mutation_ops_limit: 8,
            inflow_rate: 0.15,
            hof_rate: 0.15,
            multi_trial: true,
            is_headless: false,
            run_id: "default_run".to_string(),
            lamarckian: false,
            training_mode: Scenario::Curriculum,
            level: 0.3,
            level_auto: true,
            pending_mode: None,
            generation: 4,
        };
        let v = serde_json::to_value(TrainerUiEvent::TrainerStateChanged(state.clone())).unwrap();
        assert_eq!(v["type"], "TRAINER_STATE_CHANGED");
        assert_eq!(v["N"], 16);
        assert_eq!(v["population"], 100);
        assert_eq!(v["isRunning"], true);
        assert_eq!(v["mutationRate"], 3);
        assert_eq!(v["trainingMode"], "curriculum");
        assert_eq!(v["levelAuto"], true);
        assert!(v["pendingMode"].is_null());
        let reset = serde_json::to_value(TrainerUiEvent::TrainerResetCompleted(state)).unwrap();
        assert_eq!(reset["type"], "TRAINER_RESET_COMPLETED");

        let tick = serde_json::to_value(TrainerUiEvent::TrainerTelemetryTick(TrainerTelemetryTick {
            generation: 2,
            is_running: false,
            time_str: "5.0s".to_string(),
            sandboxes: vec![],
            selected_brain: Some(TrainerSelectedBrain { id: 1, activations: vec![0.5] }),
        }))
        .unwrap();
        assert_eq!(tick["type"], "TRAINER_TELEMETRY_TICK");
        assert_eq!(tick["timeStr"], "5.0s");
        assert_eq!(tick["selectedBrain"]["id"], 1);
    }

    /// TCK-192, TCK-194: the telemetry carries only the `N` sampled sandboxes (slots 3 elite | 2 hof | 2 random |
    /// 5 mutants at pop 12, sample 1 / 1 / 1 / 2) and the brain view only a displayed sandbox.
    #[test]
    fn test_telemetry_carries_only_the_displayed_sandboxes() {
        use crate::trainer::sandbox::SandboxOrigin;
        use crate::trainer::sim::{TrainerEvent, TrainerParams};
        let mut sim = TrainerSim::new(TrainerParams { population: 12, multi_trial: false, ..TrainerParams::from_config(crate::config::test_config()) }, 11, 200, crate::config::test_config());
        while !matches!(sim.step(false), Some(TrainerEvent::GenerationCompleted { .. })) {}
        let tick = TrainerTelemetryTick::build(&sim, 5, Some(4), true);
        assert_eq!(tick.sandboxes.iter().map(|s| s.id).collect::<Vec<_>>(), vec![1, 4, 6, 8, 9]);
        assert_eq!(
            tick.sandboxes.iter().map(|s| s.origin_type).collect::<Vec<_>>(),
            vec![SandboxOrigin::Elite, SandboxOrigin::Hof, SandboxOrigin::Random, SandboxOrigin::Mutant, SandboxOrigin::Mutant]
        );
        assert_eq!(tick.selected_brain.as_ref().map(|b| b.id), Some(4));
        assert!(TrainerTelemetryTick::build(&sim, 5, Some(3), true).selected_brain.is_none(), "sandbox 3 is not displayed");
        assert_eq!(TrainerTelemetryTick::build(&sim, 100, None, true).sandboxes.len(), 12, "display count above the population");
    }

    /// TCK-194: edge cases of the display sample.
    #[test]
    fn test_display_slots_edge_cases() {
        use crate::trainer::sandbox::SandboxOrigin::{Elite, Hof, Mutant, Random};
        // Generation 1: one group only
        assert_eq!(display_slots(&[Random; 10], 4), vec![0, 1, 2, 3]);
        // Sample covers the population
        assert_eq!(display_slots(&[Elite, Hof, Mutant], 5), vec![0, 1, 2]);
        // Fewer tiles than groups: exactly n slots, first slots first
        let origins: Vec<_> = [Elite; 25].into_iter().chain([Hof; 15]).chain([Random; 15]).chain([Mutant; 45]).collect();
        assert_eq!(display_slots(&origins, 2), vec![0, 25]);
        // Every n gives exactly n distinct slots
        for n in 1..=100 {
            let slots = display_slots(&origins, n);
            assert_eq!(slots.len(), n);
            assert!(slots.windows(2).all(|w| w[0] < w[1]));
        }
    }

    /// TCK-194: at pop 100 the 16 displayed tiles sample every origin group like the breeding shares (4 / 2 / 2 / 8),
    /// not only the leading elite slots.
    #[test]
    fn test_telemetry_samples_every_origin_group() {
        use crate::biology::genome::WILDTYPE_GENOME;
        use crate::trainer::sandbox::SandboxOrigin;
        use crate::trainer::sim::{GenomeRecord, TrainerParams};
        let record = |fitness: f32| GenomeRecord { genome: WILDTYPE_GENOME.to_string(), name: "p".to_string(), fitness, generation: 1, methylations: Vec::new(), synapse_weights: Vec::new() };
        let parents: Vec<GenomeRecord> = (0..25).map(|i| record(25.0 - i as f32)).collect();
        let hof: Vec<GenomeRecord> = (0..15).map(|i| record(15.0 - i as f32)).collect();
        let sim = TrainerSim::restore(TrainerParams { population: 100, ..TrainerParams::from_config(crate::config::test_config()) }, 5, 2, parents, hof, 200, crate::config::test_config());
        let tick = TrainerTelemetryTick::build(&sim, 16, None, true);
        let count = |origin: SandboxOrigin| tick.sandboxes.iter().filter(|s| s.origin_type == origin).count();
        assert_eq!(
            [count(SandboxOrigin::Elite), count(SandboxOrigin::Hof), count(SandboxOrigin::Random), count(SandboxOrigin::Mutant)],
            [4, 2, 2, 8]
        );
    }

    /// TCK-192: the display count is a UI concern; building the telemetry with different counts every tick leaves
    /// the generation stats of the same seed unchanged.
    #[test]
    fn test_display_count_does_not_change_the_simulation() {
        use crate::trainer::sim::{TrainerEvent, TrainerParams};
        let run = |display_count: Option<usize>| {
            let mut sim = TrainerSim::new(TrainerParams { population: 10, trials: 2, ..TrainerParams::from_config(crate::config::test_config()) }, 21, 200, crate::config::test_config());
            let mut stats = Vec::new();
            while stats.len() < 2 {
                if let Some(n) = display_count {
                    assert!(TrainerTelemetryTick::build(&sim, n, Some(1), true).sandboxes.len() <= n);
                }
                if let Some(TrainerEvent::GenerationCompleted { stats: s, .. }) = sim.step(true) {
                    stats.push(s);
                }
            }
            stats
        };
        let reference = run(None);
        assert_eq!(run(Some(4)), reference);
        assert_eq!(run(Some(10)), reference);
    }
}
