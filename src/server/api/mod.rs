//! Tauri commands (registered in `src/main.rs`), one module per area. Every command returns typed results
//! (`#[derive(Serialize, TS)]` structs exported to `bindings/`) and `IpcError` (TCK-157). Contract:
//! `wiki/contracts/ipc-message.md`; guide: `wiki/guides/add-ipc-message.md`.

pub mod actions;
pub mod catalogue;
pub mod species;
pub mod trainer;

use std::sync::mpsc::Sender;

use crate::persistence::{Db, DbState, RepoResult};
use crate::server::ipc::{ClientAction, IpcError};

pub type CommandResult<T> = Result<T, IpcError>;

/// Locks the shared store and runs `query`; a lock or repository error becomes a logged `IpcError` of kind `database`.
fn with_store<T>(db: &DbState, command: &str, query: impl FnOnce(&Db) -> RepoResult<T>) -> CommandResult<T> {
    let store = db.0.lock().map_err(|e| IpcError::database(command, e))?;
    query(&store).map_err(|e| IpcError::database(command, e))
}

/// The rows of a repository list query, each mapped to its command DTO.
fn collect_rows<R, T>(db: &DbState, command: &str, query: impl FnOnce(&Db) -> RepoResult<Vec<R>>, map: impl FnMut(R) -> T) -> CommandResult<Vec<T>> {
    with_store(db, command, query).map(|rows| rows.into_iter().map(map).collect())
}

/// Queues `action` for the engine thread (processed in order at the start of its next frame).
fn send_to_engine(tx: &Sender<ClientAction>, action: ClientAction) -> CommandResult<()> {
    tx.send(action).map_err(IpcError::engine_unavailable)
}
