//! Engine commands: client actions and the mode switch.

use std::sync::mpsc::Sender;
use tauri::State;

use super::{send_to_engine, CommandResult};
use crate::server::engine::EngineInput;
use crate::server::ipc::{ClientAction, EngineAction, IpcError, SimMode};

/// Parses one client action (`ClientAction`, tag `type`) and queues it for the engine thread. Rejects an unknown or
/// malformed action, and `SET_MODE` (mode switches go through `set_mode`), with a logged `invalidAction` error.
#[tauri::command]
pub fn handle_client_action(action: serde_json::Value, tx: State<'_, Sender<EngineInput>>) -> CommandResult<()> {
    let action = ClientAction::from_value(&action).and_then(|action| match action {
        ClientAction::Engine(EngineAction::SetMode { .. }) => Err(IpcError::invalid_action("SET_MODE is sent with the set_mode command")),
        action => Ok(action),
    });
    match action {
        Ok(action) => send_to_engine(&tx, action),
        Err(e) => {
            log::error!("rejected client action: {}", e.message);
            Err(e)
        }
    }
}

/// Switches the backend mode (only the router calls it, invariant 1). Queued like an action, so it takes effect
/// after the actions sent before it; resolves with the requested mode once queued.
#[tauri::command]
pub fn set_mode(mode: SimMode, tx: State<'_, Sender<EngineInput>>) -> CommandResult<SimMode> {
    send_to_engine(&tx, ClientAction::Engine(EngineAction::SetMode { mode }))?;
    Ok(mode)
}
