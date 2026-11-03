---
sources: [src/server/ipc.rs, src/server/ocean_ipc.rs, src/server/trainer_ipc.rs, src/server/api/mod.rs, frontend/core/ipc.ts]
---

# Contract: IPC message

## Purpose

Every message between the webview and the Rust backend is a typed Rust value exported to `bindings/` by ts-rs
(TCK-157): client actions (`ClientAction`), backend events (`UiEvent`), command results (one `#[derive(Serialize,
TS)]` struct per result) and command errors (`IpcError`). The frontend talks to Tauri only through
`frontend/core/ipc.ts`. A feature that needs a new button, a new event or a new query (arena controls, creator
drafts, a new room's live view) adds one action variant, event variant or command here. Current message list:
[[wiki/architecture/ipc-protocol]].

## Signature

```rust
// src/server/ipc.rs
#[serde(untagged)] pub enum ClientAction { Engine(EngineAction), Ocean(OceanAction), Trainer(TrainerAction) }
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")] pub enum EngineAction { ClientReady, SetMode { mode: SimMode } }
impl ClientAction { pub fn from_value(value: &serde_json::Value) -> Result<Self, IpcError>; }
#[serde(untagged)] pub enum UiEvent { Engine(EngineEvent), Ocean(OceanUiEvent), Trainer(TrainerUiEvent) }
#[serde(tag = "type", ...)] pub enum EngineEvent { DatabaseChanged, LogEvent { message: String, log_type: LogKind } }
impl UiEvent { pub fn database_changed() -> Self; pub fn log(message: impl Into<String>, kind: LogKind) -> Self; }
impl From<EngineEvent | OceanUiEvent | TrainerUiEvent> for UiEvent;
pub struct IpcError { pub kind: IpcErrorKind, pub message: String }  // invalidAction | notFound | database | engineUnavailable
pub const UI_EVENT_NAME: &str = "simulation-state";

// src/server/ocean_ipc.rs:   OceanAction, SpawnClone, OceanUiEvent, OceanInitState, OceanRules, OceanTelemetryTick, SelectedBrain
// src/server/trainer_ipc.rs: TrainerAction, TrainerHyperparamsUpdate, AssignSandboxCreature, TrainerUiEvent, ...

// src/server/api/mod.rs (helpers of the command modules actions.rs, species.rs, trainer.rs, catalogue.rs)
pub type CommandResult<T> = Result<T, IpcError>;
fn with_store<T>(db: &DbState, command: &str, query: impl FnOnce(&Db) -> RepoResult<T>) -> CommandResult<T>;
fn collect_rows<R, T>(db: &DbState, command: &str, query: impl FnOnce(&Db) -> RepoResult<Vec<R>>, map: impl FnMut(R) -> T) -> CommandResult<Vec<T>>;
fn send_to_engine(tx: &Sender<EngineInput>, action: ClientAction) -> CommandResult<()>;
```

```ts
// frontend/core/ipc.ts
export type Action = Exclude<ClientAction, { type: "SET_MODE" }>;
export interface Commands { [command]: { args: ...; result: ... } }   // one entry per registered command
export function call<C extends Command>(command: C, ...args): Promise<Commands[C]["result"]>;  // rejects with CallError
export function send(action: Action): Promise<boolean>;                // true = accepted; never rejects
export function setMode(mode: SimMode): Promise<SimMode | null>;      // router only
export function onUiEvent(handler: (event: UiEvent) => void): () => void;
export function onIpcError(handler: (command: Command, error: CallError) => void): void;  // default: toast
```

## Lifecycle / call order

1. UI → `send(action)` → command `handle_client_action(action: Value)` → `ClientAction::from_value` (one typed parse)
   → `EngineInput::Action` on the `Sender<EngineInput>` → the engine thread drains the channel at the start of every frame (~60 Hz) and matches
   the variant. A rejected action never reaches the engine.
2. UI → `call(command, args)` → the command runs on the Tauri thread (async commands on the runtime), locks
   `DbState` through `with_store` / `collect_rows`, returns its DTO or `IpcError`.
3. Engine → `ctx.emit(event)` (modes, [[wiki/contracts/mode]]) or `sink.emit(event.into())` (`Engine::handle`) anywhere
   in the frame → buffered in the `EventSink` → `sink.flush()` sends all events in order at the end of the frame on
   `simulation-state` (`DATABASE_CHANGED` coalesced to one per frame, [[wiki/contracts/event-sink]]).
4. UI → `onUiEvent(handler)`: one Tauri listener for the app, every handler gets every event and switches on `type`.

## Invariants

- No untyped message ([[wiki/architecture/invariants]] 13). Check: `grep -rn "json!" src/server/engine` lists nothing (the Ocean snapshot is the typed `OceanSnapshot`, TCK-156); `grep -rln "@tauri-apps/api" frontend`
  lists only `core/ipc.ts`, the tests that mock it (`core/ipc.test.ts`, `views/views.dom.test.ts`) and `views/main-menu/index.ts` (app version via `@tauri-apps/api/app`, no command).
- Action and event `type` names are unique across the three groups (`ClientAction::from_value` picks the group by
  the tag; a known tag with bad fields is `invalidAction` "malformed client action 'X': ...", an unknown tag
  "unknown client action type 'X'").
- `SET_MODE` only through the `set_mode` command, called only by `navigateTo` (and the trainer's unload reset to
  Ocean); `handle_client_action` rejects it ([[wiki/architecture/invariants]] 1).
- Events go through the `EventSink` only (`ctx.emit`), never `window.emit`; nothing leaves before the frame's flush
  ([[wiki/architecture/invariants]] 6).
- Errors reach the user: `call` reports every failure once through the error hook (toast); callers catch only for
  control flow. Commands log store errors (`IpcError::database`), rejected actions are logged by
  `handle_client_action`.
- Field casing: camelCase for new payloads; `AssignSandboxCreature` and `SpawnClone` keep snake_case fields.
- Integers that leave JS-safe range (u64 ids) are sent as decimal strings; `i64` fields carry `#[ts(type = "number")]`.
- Bindings are regenerated by `cargo test` and committed ([[wiki/architecture/invariants]] 7).

## Registration point

- Action: a variant of `EngineAction`, `OceanAction` or `TrainerAction` + its match arm in the engine's action loop
  (`Engine::handle` or the mode's `handle`, `src/server/engine/`, [[wiki/contracts/mode]]).
- Event: a variant of `EngineEvent`, `OceanUiEvent` or `TrainerUiEvent` + its `type` branch in the consuming view.
- Command: a `#[tauri::command]` in the owning `src/server/api/<area>.rs`, one line in `generate_handler!`
  (`src/main.rs`) and one entry in `Commands` (`frontend/core/ipc.ts`).

## Config keys

None.

## Events / IPC

This contract is the IPC seam itself; the full list is in [[wiki/architecture/ipc-protocol]].

## Test recipe

`cargo test server::` runs the parse tests (`ipc::tests::test_client_actions_parse_into_their_group`,
`test_unknown_and_malformed_actions_are_rejected_with_the_type`), the wire-name tests of every event group
(`test_engine_events_and_errors_keep_their_wire_names`, `ocean_ipc::tests::test_ocean_events_keep_type_tag_and_ui_field_names`,
`trainer_ipc::tests::test_trainer_events_keep_type_tag_and_ui_field_names`) and the DTO tests of the commands
(`api::species::tests`, `api::catalogue::tests`). `npm test` runs `frontend/core/ipc.test.ts` (call, send, error hook).
A new message adds its JSON to the group's parse or wire-name test.

## Minimal example

A Pause-all action and its answer:

```rust
// src/server/ocean_ipc.rs, OceanAction
/// Freeze every creature in place (answered with SIM_STATE)
FreezeAll,

// src/server/engine/ocean_mode.rs, OceanMode::handle
OceanAction::FreezeAll => {
    self.running = false;
    ctx.emit(self.sim_state());
}

// src/server/ipc.rs tests
assert_eq!(parse(json!({ "type": "FREEZE_ALL" })), Ok(ClientAction::Ocean(OceanAction::FreezeAll)));
```

```ts
// frontend (after `cargo test` regenerated bindings/OceanAction.ts)
send({ type: "FREEZE_ALL" });
```
