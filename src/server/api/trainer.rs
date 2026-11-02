//! Trainer run commands (the trainer itself is driven by `TrainerAction`s).

use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::{collect_rows, with_store, CommandResult};
use crate::persistence::{DbState, TrainerRepo, TrainerRunSummary};

/// One stored trainer run (`get_trainer_runs`).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrainerRunInfo {
    pub run_id: String,
    #[ts(type = "number")]
    pub max_generation: i64,
    pub max_fitness: f64,
}

impl From<TrainerRunSummary> for TrainerRunInfo {
    fn from(run: TrainerRunSummary) -> Self {
        Self { run_id: run.run_id, max_generation: run.max_generation, max_fitness: run.max_fitness }
    }
}

#[tauri::command]
pub fn get_trainer_runs(db: State<'_, DbState>) -> CommandResult<Vec<TrainerRunInfo>> {
    collect_rows(&db, "get_trainer_runs", |store| store.trainer_runs(), TrainerRunInfo::from)
}

/// Deletes a run's genomes and run state (TCK-191).
#[tauri::command]
pub fn clear_trainer_history(db: State<'_, DbState>, run_id: String) -> CommandResult<()> {
    with_store(&db, "clear_trainer_history", |store| store.delete_trainer_run(&run_id))
}
