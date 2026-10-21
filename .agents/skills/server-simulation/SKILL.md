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
- New actions: handle them in the engine match; unknown types must stay logged. New commands: register in `main.rs` and use the shared `DbState` connection.
- Log DB errors via `log_db_err`; never discard `conn.execute` results.
- Performance changes must be behaviour-preserving and keep iteration order (golden master and equivalence tests green).
