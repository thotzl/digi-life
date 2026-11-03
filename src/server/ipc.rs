//! Typed IPC protocol (TCK-157): every client action (`handle_client_action`, `set_mode`), every backend event (one
//! Tauri event `simulation-state`) and the error of every command. All types are tagged by `type` (actions, events)
//! and exported to `bindings/` by ts-rs (`cargo test`); the frontend IPC client `frontend/core/ipc.ts` uses only
//! these. Contract: `wiki/contracts/ipc-message.md`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use ts_rs::TS;

use crate::server::ocean_ipc::{OceanAction, OceanUiEvent};
use crate::server::trainer_ipc::{TrainerAction, TrainerUiEvent};

/// Name of the one Tauri event that carries every `UiEvent`.
pub const UI_EVENT_NAME: &str = "simulation-state";

/// Backend mode: which simulation the engine thread steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SimMode {
    Ocean,
    Trainer,
}

/// Actions that belong to neither simulation.
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum EngineAction {
    /// Handshake of a (re)loaded UI; answered with `INIT_STATE`, `DATABASE_CHANGED` and a log line. Never changes the
    /// mode (invariant 1).
    ClientReady,
    /// Mode switch. Only through the `set_mode` command (router, invariant 1); `handle_client_action` rejects it.
    SetMode { mode: SimMode },
}

/// Every action the UI can send, tagged by `type` (the three groups use disjoint type names).
#[derive(Debug, Clone, PartialEq, Deserialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum ClientAction {
    Engine(EngineAction),
    Ocean(OceanAction),
    Trainer(TrainerAction),
}

/// Outcome of parsing one action group: `None` if `action_type` is not one of the group's tags (serde's unknown
/// variant error for the tag itself; an unknown value of a nested enum field is a malformed action of this group).
fn parse_group<T: DeserializeOwned>(
    value: &serde_json::Value,
    action_type: &str,
) -> Option<Result<T, serde_json::Error>> {
    match T::deserialize(value) {
        Ok(action) => Some(Ok(action)),
        Err(e) if e.to_string().starts_with(&format!("unknown variant `{}`", action_type)) => None,
        Err(e) => Some(Err(e)),
    }
}

impl ClientAction {
    /// Parses the action JSON of `handle_client_action` (one typed deserialisation). An unknown `type` or a known type
    /// with missing or mistyped fields is an `InvalidAction` error naming the type.
    pub fn from_value(value: &serde_json::Value) -> Result<Self, IpcError> {
        let Some(action_type) = value.get("type").and_then(|t| t.as_str()) else {
            return Err(IpcError::invalid_action("client action without a string 'type' field"));
        };
        let malformed = |e: serde_json::Error| {
            IpcError::invalid_action(format!("malformed client action '{}': {}", action_type, e))
        };
        if let Some(parsed) = parse_group::<EngineAction>(value, action_type) {
            return parsed.map(ClientAction::Engine).map_err(malformed);
        }
        if let Some(parsed) = parse_group::<OceanAction>(value, action_type) {
            return parsed.map(ClientAction::Ocean).map_err(malformed);
        }
        if let Some(parsed) = parse_group::<TrainerAction>(value, action_type) {
            return parsed.map(ClientAction::Trainer).map_err(malformed);
        }
        Err(IpcError::invalid_action(format!(
            "unknown client action type '{}'",
            action_type
        )))
    }
}

/// Kind of a log line (`LOG_EVENT`); the Ocean terminal colours by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum LogKind {
    System,
    Mutation,
    Repair,
}

/// Events of neither simulation.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum EngineEvent {
    /// Stored species, catalogue or trainer rows changed; the UI refetches its lists. Coalesced to one per frame.
    DatabaseChanged,
    /// A line for the Ocean log terminal.
    LogEvent {
        message: String,
        #[serde(rename = "logType")]
        log_type: LogKind,
    },
}

/// Every event the backend sends to the UI (payload of `simulation-state`), tagged by `type`.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(untagged)]
#[ts(export)]
pub enum UiEvent {
    Engine(EngineEvent),
    Ocean(OceanUiEvent),
    Trainer(TrainerUiEvent),
}

impl UiEvent {
    pub fn database_changed() -> Self {
        UiEvent::Engine(EngineEvent::DatabaseChanged)
    }

    pub fn log(message: impl Into<String>, log_type: LogKind) -> Self {
        UiEvent::Engine(EngineEvent::LogEvent {
            message: message.into(),
            log_type,
        })
    }

    pub fn is_database_changed(&self) -> bool {
        matches!(self, UiEvent::Engine(EngineEvent::DatabaseChanged))
    }
}

impl From<EngineEvent> for UiEvent {
    fn from(event: EngineEvent) -> Self {
        UiEvent::Engine(event)
    }
}

impl From<OceanUiEvent> for UiEvent {
    fn from(event: OceanUiEvent) -> Self {
        UiEvent::Ocean(event)
    }
}

impl From<TrainerUiEvent> for UiEvent {
    fn from(event: TrainerUiEvent) -> Self {
        UiEvent::Trainer(event)
    }
}

/// What went wrong in a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IpcErrorKind {
    /// Unknown action type, missing or mistyped fields, or an action sent through the wrong command
    InvalidAction,
    /// A row the command needs does not exist
    NotFound,
    /// The store failed (lock or SQLite error)
    Database,
    /// The engine thread no longer receives actions
    EngineUnavailable,
}

/// Error of every Tauri command (the rejection value of `invoke`); the frontend shows `message` in a toast.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct IpcError {
    pub kind: IpcErrorKind,
    pub message: String,
}

impl IpcError {
    pub fn new(kind: IpcErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn invalid_action(message: impl Into<String>) -> Self {
        Self::new(IpcErrorKind::InvalidAction, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(IpcErrorKind::NotFound, message)
    }

    /// A store error of `command`, logged once here.
    pub fn database(command: &str, error: impl Display) -> Self {
        log::error!("command '{}': database error: {}", command, error);
        Self::new(IpcErrorKind::Database, error.to_string())
    }

    pub fn engine_unavailable(error: impl Display) -> Self {
        log::error!("sending an action to the engine thread failed: {}", error);
        Self::new(
            IpcErrorKind::EngineUnavailable,
            format!("simulation thread unavailable: {}", error),
        )
    }
}

impl Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ocean_ipc::SpawnClone;
    use serde_json::json;

    fn parse(v: serde_json::Value) -> Result<ClientAction, IpcError> {
        ClientAction::from_value(&v)
    }

    #[test]
    fn test_client_actions_parse_into_their_group() {
        assert_eq!(
            parse(json!({ "type": "CLIENT_READY" })),
            Ok(ClientAction::Engine(EngineAction::ClientReady))
        );
        assert_eq!(
            parse(json!({ "type": "SET_MODE", "mode": "trainer" })),
            Ok(ClientAction::Engine(EngineAction::SetMode { mode: SimMode::Trainer }))
        );
        assert_eq!(
            parse(json!({ "type": "TOGGLE_SIMULATION", "running": false })),
            Ok(ClientAction::Ocean(OceanAction::ToggleSimulation { running: false }))
        );
        assert_eq!(
            parse(json!({ "type": "INJECT_URZELLE" })),
            Ok(ClientAction::Ocean(OceanAction::InjectUrzelle))
        );
        assert_eq!(
            parse(json!({ "type": "RESET_EVOLUTION" })),
            Ok(ClientAction::Ocean(OceanAction::ResetEvolution))
        );
        assert_eq!(
            parse(json!({ "type": "SELECT_AGENT", "id": 7 })),
            Ok(ClientAction::Ocean(OceanAction::SelectAgent { id: Some(7) }))
        );
        assert_eq!(
            parse(json!({ "type": "SELECT_AGENT" })),
            Ok(ClientAction::Ocean(OceanAction::SelectAgent { id: None }))
        );
        assert_eq!(
            parse(json!({ "type": "SET_SIMULATION_SPEED", "speed": 9999 })),
            Ok(ClientAction::Ocean(OceanAction::SetSimulationSpeed { speed: 9999 }))
        );
        assert_eq!(
            parse(
                json!({ "type": "SPAWN_CLONE", "genome": "AB", "methylations": [0.5], "synapse_weights": null, "load_learned_synapses": true })
            ),
            Ok(ClientAction::Ocean(OceanAction::SpawnClone(SpawnClone {
                genome: "AB".to_string(),
                methylations: Some(vec![0.5]),
                synapse_weights: None,
                load_learned_synapses: Some(true),
            })))
        );
        assert_eq!(
            parse(json!({ "type": "START_TRAINING" })),
            Ok(ClientAction::Trainer(TrainerAction::StartTraining))
        );
        assert!(matches!(
            parse(json!({ "type": "ASSIGN_SANDBOX_CREATURE", "sandbox_id": 2, "genome": "AB" })),
            Ok(ClientAction::Trainer(TrainerAction::AssignSandboxCreature(_)))
        ));
    }

    #[test]
    fn test_unknown_and_malformed_actions_are_rejected_with_the_type() {
        let unknown = parse(json!({ "type": "SPAWN_FOOD", "x": 1.0, "y": 2.0 })).unwrap_err();
        assert_eq!(unknown.kind, IpcErrorKind::InvalidAction);
        assert_eq!(unknown.message, "unknown client action type 'SPAWN_FOOD'");
        // Known type, missing field: the group's error, not "unknown"
        let missing = parse(json!({ "type": "TOGGLE_SIMULATION" })).unwrap_err();
        assert!(
            missing
                .message
                .starts_with("malformed client action 'TOGGLE_SIMULATION'")
                && missing.message.contains("running"),
            "{}",
            missing.message
        );
        let mistyped = parse(json!({ "type": "SET_TRAINER_LEVEL", "level": "high" })).unwrap_err();
        assert!(
            mistyped
                .message
                .starts_with("malformed client action 'SET_TRAINER_LEVEL'"),
            "{}",
            mistyped.message
        );
        let bad_mode = parse(json!({ "type": "SET_MODE", "mode": "arena" })).unwrap_err();
        assert!(
            bad_mode.message.starts_with("malformed client action 'SET_MODE'"),
            "{}",
            bad_mode.message
        );
        assert_eq!(
            parse(json!({ "running": true })).unwrap_err().kind,
            IpcErrorKind::InvalidAction
        );
        assert_eq!(
            parse(json!("CLIENT_READY")).unwrap_err().kind,
            IpcErrorKind::InvalidAction
        );
    }

    #[test]
    fn test_engine_events_and_errors_keep_their_wire_names() {
        assert_eq!(
            serde_json::to_value(UiEvent::database_changed()).unwrap(),
            json!({ "type": "DATABASE_CHANGED" })
        );
        assert_eq!(
            serde_json::to_value(UiEvent::log("hello", LogKind::Repair)).unwrap(),
            json!({ "type": "LOG_EVENT", "message": "hello", "logType": "repair" })
        );
        assert!(UiEvent::database_changed().is_database_changed());
        assert!(!UiEvent::log("x", LogKind::System).is_database_changed());
        assert_eq!(
            serde_json::to_value(IpcError::not_found("gone")).unwrap(),
            json!({ "kind": "notFound", "message": "gone" })
        );
        assert_eq!(serde_json::to_value(SimMode::Trainer).unwrap(), json!("trainer"));
    }
}
