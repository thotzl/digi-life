//! Tauri commands (registered in `src/main.rs`), one module per area. Every command returns typed results
//! (`#[derive(Serialize, TS)]` structs exported to `bindings/`) and `IpcError` (TCK-157). Contract:
//! `wiki/contracts/ipc-message.md`; guide: `wiki/guides/add-ipc-message.md`.

pub mod actions;
pub mod catalogue;
pub mod species;
pub mod trainer;

use std::sync::mpsc::Sender;

use crate::persistence::{Db, DbState, RepoResult};
use crate::server::engine::EngineInput;
use crate::server::ipc::{ClientAction, IpcError};

pub type CommandResult<T> = Result<T, IpcError>;

/// Locks the shared store and runs `query`; a lock or repository error becomes a logged `IpcError` of kind `database`.
fn with_store<T>(db: &DbState, command: &str, query: impl FnOnce(&Db) -> RepoResult<T>) -> CommandResult<T> {
    let store = db.0.lock().map_err(|e| IpcError::database(command, e))?;
    query(&store).map_err(|e| IpcError::database(command, e))
}

/// The rows of a repository list query, each mapped to its command DTO.
fn collect_rows<R, T>(
    db: &DbState,
    command: &str,
    query: impl FnOnce(&Db) -> RepoResult<Vec<R>>,
    map: impl FnMut(R) -> T,
) -> CommandResult<Vec<T>> {
    with_store(db, command, query).map(|rows| rows.into_iter().map(map).collect())
}

/// Queues `action` for the engine thread (processed in order at the start of its next frame).
fn send_to_engine(tx: &Sender<EngineInput>, action: ClientAction) -> CommandResult<()> {
    tx.send(EngineInput::Action(action))
        .map_err(IpcError::engine_unavailable)
}

/// Tells the engine that a command changed the stored trainer genomes of `run_id` (TCK-156: the engine rebuilds the
/// trainer from the store only then). The store write already succeeded, so a stopped engine is only logged.
fn notify_trainer_store_changed(tx: &Sender<EngineInput>, run_id: &str) {
    if let Err(e) = tx.send(EngineInput::TrainerStoreChanged {
        run_id: run_id.to_string(),
    }) {
        log::error!("notifying the engine thread of a trainer store change failed: {}", e);
    }
}
