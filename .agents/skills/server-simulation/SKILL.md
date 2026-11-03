---
name: server-simulation
description: Pointers for changing the BroodBattle Rust side - engine thread and modes, shared creature systems, Ocean, trainer, physics, Tauri IPC and SQLite persistence (src/server/, src/sim/, src/trainer/, src/persistence/).
---
# server-simulation: Engine, Simulation, IPC & Persistence Pointers

## I. Read first
- New mode, sink, room, evaluator, dynamic, IPC message, table or config key:
  `.agents/skills/plugin-authoring/SKILL.md` (one guide + its contract).
- Rules: `wiki/architecture/invariants.md` 2, 3, 4, 6, 8, 11, 12, 13, 16.
- Current state: `wiki/architecture/simulation-engine.md`, `wiki/architecture/ipc-protocol.md`,
  `wiki/architecture/persistence.md`, `wiki/simulation/physics.md`, `wiki/simulation/ocean.md`,
  `wiki/simulation/trainer.md`; module list `wiki/architecture/module-map.md`.

## II. Task rules
- Engine behaviour is tested headless (`server::engine::tests`, `VecSink` + `MemoryDb`); add a test there instead of
  launching the app.
- Commands use the shared `DbState` through `with_store` / `collect_rows` (`src/server/api/mod.rs`).
