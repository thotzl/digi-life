---
name: server-simulation
description: Rules for changing the Rust simulation engine thread, physics, Tauri IPC (commands and the simulation-state event) and SQLite persistence of Pixel DNA Life.
---
# server-simulation: Engine, IPC & Persistence Rules

## I. Knowledge (read first)
- Engine: `wiki/architecture/simulation-engine.md`
- IPC: `wiki/architecture/ipc-protocol.md`
- Persistence: `wiki/architecture/persistence.md`
- Physics: `wiki/simulation/physics.md`; Ocean: `wiki/simulation/ocean.md`; Trainer: `wiki/simulation/trainer.md`
- Invariants: `wiki/architecture/invariants.md`

## II. Rules
- Keep the warp cap of 35 steps per frame for Ocean and Trainer.
- No SQL, grid allocation or agent clone per physics step; emit through the event buffer (`emit_state`), never directly from the step loop.
- `RESET_EVOLUTION` must clear `newly_spawned_creatures`.
- Thrust is always scaled through `step_creature_kinematics`; never apply raw activations as force.
- IPC is typed (`ClientAction`, `UiEvent`, command DTOs, `IpcError`; no `json!` payloads, no `serde_json::Value` results). New actions, events or commands: guide `wiki/guides/add-ipc-message.md`, contract `wiki/contracts/ipc-message.md`. Unknown or malformed actions are rejected at `handle_client_action` with a logged error; commands use the shared `DbState` store via `with_store` / `collect_rows`.
- No SQL outside `src/persistence/`: call the repository traits (contract `wiki/contracts/persistence-repo.md`, guide `wiki/guides/add-table.md`). Log fire-and-forget DB errors via `log_db_err`; never discard a repository result.
- Performance changes must be behaviour-preserving and keep iteration order (golden master and equivalence tests green).
