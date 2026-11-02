---
id: TCK-157
title: Typed IPC Contract and ts-rs Bindings in Frontend
status: open
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
