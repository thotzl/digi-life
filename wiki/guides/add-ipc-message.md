---
sources: [wiki/contracts/ipc-message.md, frontend/core/ipc.ts]
---

# Guide: add an IPC message

Read first: [[wiki/contracts/ipc-message]] and [[wiki/architecture/invariants]].

## Steps

Action (UI → engine):

1. Add a variant to `EngineAction` (`src/server/ipc.rs`), `OceanAction` (`src/server/ocean_ipc.rs`) or
   `TrainerAction` (`src/server/trainer_ipc.rs`). Pick a `type` name not used by another group. Optional fields:
   `#[serde(default)] #[ts(optional)] Option<T>`.
2. Handle it in the action loop of `src/server/engine.rs` (one match arm; no SQL per physics step).
3. Send it from the UI with `send({ type: "...", ... })` (`frontend/core/ipc.ts`); never `invoke` directly.

Event (engine → UI):

1. Add a variant to `EngineEvent`, `OceanUiEvent` or `TrainerUiEvent`; a payload with several fields gets its own
   `#[derive(Serialize, TS)] #[serde(rename_all = "camelCase")]` struct.
2. Emit it with `emit_state(MyEvent { .. }.into())`; never call `window.emit` yourself.
3. Handle its `type` in the view's `onUiEvent` handler.

Command (request / response):

1. Add a `#[tauri::command]` to the owning `src/server/api/<area>.rs` (or a new area module declared in
   `src/server/api/mod.rs`) returning `CommandResult<T>`; `T` is a `#[derive(Serialize, TS)] #[ts(export)]` DTO, never
   `serde_json::Value`. Use `with_store` / `collect_rows` for the store, `IpcError::not_found` for missing rows,
   `send_to_engine` to hand work to the engine thread.
2. Register it in `generate_handler!` (`src/main.rs`).
3. Add its `args` and `result` to `Commands` in `frontend/core/ipc.ts`; call it with `call("name", args)`.
4. `i64` fields: `#[ts(type = "number")]`; u64 ids: send as a decimal string.

All:

1. Tests: the JSON of a new action in `ipc::tests::test_client_actions_parse_into_their_group` (or the group's parse
   test), the JSON of a new event in its group's wire-name test, a DTO serialization test for a new command result.
2. Run `cargo test` and commit the regenerated `bindings/`.
3. Docs: add the message to the lists of the IPC protocol article (`wiki/architecture/ipc-protocol.md`).

## Done when

- `cargo test server::` and `npm test` pass; the new message's test is among them.
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green; `git diff -- bindings/`
  contains only the intended types.
- Golden master and the trainer references are unchanged unless the message changes the model on purpose.
