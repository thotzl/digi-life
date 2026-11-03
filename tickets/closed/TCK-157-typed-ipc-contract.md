---
id: TCK-157
title: Typed IPC Contract and ts-rs Bindings in Frontend
status: closed
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-A02, RV-A03, RV-F01, RV-A06
---

# TCK-157: Typed IPC Contract and ts-rs Bindings in Frontend

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R1 (after TCK-198; parallel with TCK-159). Trainer actions/events are typed (TCK-177). Remaining scope:
- `ClientAction = Engine(ClientReady, SetMode) | Ocean(OceanAction) | Trainer(TrainerAction)` with ts-rs; channel `Sender<ClientAction>`; `handle_client_action` deserialises once; catalogue -> Ocean sends `OceanAction::SpawnClone` typed (api.rs:329-340); unknown messages rejected with a logged error.
- `UiEvent` enum for every event (Ocean events are `json!` at engine.rs:307-1535; `DATABASE_CHANGED`, logs).
- Typed command results (`#[derive(Serialize, TS)]` structs instead of `serde_json::Value`, api.rs:25/62/89/116/209/215); `api.rs` split into `api/{species,catalogue,trainer}.rs` with a `collect_rows` helper (5 copied loops).
- Remove dead commands (no caller in `frontend/`): `get_trainer_population`, `get_trainer_hof`, `save_trainer_generation` (api.rs:143, deactivates genomes, drops weights), `apply_champion` (writes the unread `progenitor_genome`).
- Frontend `core/ipc.ts`: typed `send(ClientAction)`, `call<Command>()` with typed errors and a toast hook (replaces `safeInvoke`, which swallows errors, RV-F04), one `listen` dispatching typed events; `set_mode` as a command with a result (RV-F05); `TelemetryPayload` matches the backend.
- Contract `wiki/contracts/ipc-message.md`, guide `add-ipc-message.md`.


> Pool ticket: requires a user decision before implementation. Findings (RV-A02, RV-A03, RV-F01, RV-A06) below.

## Description
Actions/events are stringly-typed JSON; ts-rs bindings are generated but unused; mixed casing and timestamp units.

## Proposed Scope
`Action`/`SimEvent` enums exported via ts-rs; frontend imports `bindings/`.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-A02 · P1** — frontend → Rust: `invoke("handle_client_action", { action: JSON.stringify({type}) })`, double-serialized and unchecked; missing fields silently ignored (`as_u64()` → `None`). Rust → frontend: one event `simulation-state` with `json!` payloads, ~12 types, mixed casing (`load_learned_synapses`, `sandbox_id` vs `warpSpeed`, `runId`). Target: `#[derive(Deserialize, TS)] #[serde(tag = "type")] enum Action`, taken directly by the command; `#[derive(Serialize, TS)] enum SimEvent` on separate events (`ocean://telemetry`, `trainer://telemetry`, `db://changed`, `log`); both exported via ts-rs.
- **RV-A03 / RV-F01 · P1** — `bindings/` (17 files) generated but never imported; handwritten `frontend/shared/types.ts` (210 LOC) already deviates (`OrganelleLocus` `x?/y?` vs `SensoryPatch` `geneStartIndex/geneEndIndex`); ~41 `any`, mostly at the IPC boundary. Target: `types.ts` re-exports from `bindings/`, only client-side types stay local.
- **RV-A06 · P3** — `now_millis` is seconds at 9 sites (`as_secs`) and ms at 2; catalogue `created_at` is SQLite text. Align with RV-D04 (TCK-159).

## Note (epic TCK-171 P8, 2026-10-26)
Trainer subset done in TCK-177: typed trainer events and actions (`src/server/trainer_ipc.rs`, ts-rs bindings). Still open: Ocean events/actions, `SET_MODE` and the remaining untyped payloads.

## Verification (2026-11-03, v1.12.34)
- Types: `src/server/ipc.rs` `ClientAction = Engine(EngineAction) | Ocean(OceanAction) | Trainer(TrainerAction)` (`EngineAction`: `CLIENT_READY`, `SET_MODE`), `UiEvent = Engine(EngineEvent) | Ocean(OceanUiEvent) | Trainer(TrainerUiEvent)` (`EngineEvent`: `DATABASE_CHANGED`, `LOG_EVENT` with `LogKind`), `IpcError { kind, message }`; `ClientAction::from_value` parses once and rejects unknown types and malformed fields with a logged, user-visible error. `src/server/ocean_ipc.rs`: `OceanAction` (toggle, inject, `SPAWN_CLONE`, reset, select, speed) and `OceanUiEvent` (`INIT_STATE`, `SIM_STATE`, `TELEMETRY_TICK`, `FOOD_TICK`, `EAT_EVENT`, `BITE_EVENT`). Channel `Sender<ClientAction>`, typed event buffer; the catalogue spawn sends a typed `SpawnClone`. Untyped messages remaining: 0 (the only `json!` left is the DB snapshot).
- Commands in `src/server/api/{actions,species,trainer,catalogue}.rs` return `Result<DTO, IpcError>` (`with_store`, `collect_rows`, `send_to_engine` in `api/mod.rs`); `set_mode` is a command; `handle_client_action` rejects `SET_MODE`. Dead commands removed after grep of `frontend/`: `get_trainer_population`, `get_trainer_hof`, `save_trainer_generation`, `apply_champion` (+ `OceanSnapshotRepo::save_progenitor_genome`, RV-D06). No-op actions `SPAWN_FOOD` / `CLIENT_LOG` removed from the frontend (the engine never handled them; RV-B09 stays open in TCK-202).
- Frontend `frontend/core/ipc.ts`: typed `call(cmd, args)` (rejects with `IpcError` or transport/unavailable), `send(action)` (never rejects), `setMode(mode)` (router only, invariant 1), `onUiEvent` (one `simulation-state` listener), `onIpcError` (default toast, `core/toast.ts`); `api.ts`/`safeInvoke` removed (RV-F04), phenotype cache in `core/phenotypes.ts`; hand-written `TelemetryPayload` removed.
- Found during integration (pre-existing, exposed by the new event timing): the `TRAINER_RESET_COMPLETED` handler rebuilt the tile grid with `push = true` and sent the reset-time hyperparameters (warp 1) back to the engine; when the event arrived after the user moved the speed slider, the slider change was overwritten. Reproduced with the TCK-194 UI check (1 of 3 runs: generation stuck at 1 at "max" warp) and a temporary engine log (`reset, warp 9999, start, warp 1`); fixed by `rebuildSandboxGrid(false)` in the reset handler; afterwards 3/3 runs green.
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 249 passed, 1 ignored; golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum, rehearsal references match; `npm test` 19 passed (new `frontend/core/ipc.test.ts`); build ok; doc links OK (85 files); bindings committed (19 new IPC types), no drift. UI smoke OK (0 console errors); selection/reset check 3/3 OK.
- Docs: `wiki/contracts/ipc-message.md`, `wiki/guides/add-ipc-message.md`, `wiki/architecture/ipc-protocol.md` rewritten; invariants #1 (`set_mode` command), master index, system-overview, simulation-engine, persistence, persistence-repo contract, frontend/structure (RV-F04 resolved), repository-map, add-table guide, trainer; skills `client-rendering`, `server-simulation`.
- Behaviour notes: f32 values in events are serialized in their shortest form (0.9 instead of 0.8999999761581421); malformed actions now raise a toast instead of being ignored.
