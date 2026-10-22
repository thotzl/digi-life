---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, .agents/skills/server-simulation/SKILL.md, docs/revisit/02-architecture.md, tickets/closed/TCK-109-tauri-desktop-shell.md, tickets/closed/TCK-110-rust-simulation-core.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-124-tauri-exclusive-consolidation.md, tickets/closed/TCK-107-refactoring-dry-and-shared-modularization.md, tickets/closed/TCK-108-headless-backend-evolutionary-trainer-migration.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md]
---

# System Overview

Pixel DNA Life is a single-user desktop app: **Tauri v2** shell, **Rust 2024** simulation core, **SQLite** persistence, **Vite + TypeScript + Preact Signals** webview UI. There is no network server, no WebSocket and no port.

## Runtime structure

```
Tauri main.rs
 ├─ setup: resolve app_data_dir DB path, migrate legacy DB, init schema once, manage DbState
 ├─ invoke_handler: 15 commands (src/server/api.rs), shared mutex-guarded DB connection
 └─ spawn_simulation_thread(window, rx)            (src/server/engine.rs)
      ├─ own SQLite connection to the same file
      ├─ rx: client actions (mpsc) from handle_client_action
      ├─ Ocean step | Trainer step  (one engine, mode switch)
      └─ window.emit("simulation-state", ...)      buffered, flushed once per frame
Webview (frontend/)
 ├─ invoke(...)  -> commands
 └─ listen("simulation-state") -> render loops (Canvas 2D) + signals HUD
```

## Layers

- **Shell** — `src/main.rs`: Tauri builder, command registration, DB setup, starts the simulation thread.
- **Engine** — `src/server/engine.rs`: Ocean and Trainer loop (~60 Hz), see [[wiki/architecture/simulation-engine]].
- **Headless runner** — `src/bin/trainer_runner.rs`: trainer runs and mechanics probes without UI, see [[wiki/simulation/analytics-and-balancing]].
- **API** — `src/server/api.rs`: Tauri commands, see [[wiki/architecture/ipc-protocol]].
- **Domain** — `src/biology/` (genome compiler, creature factory, IO-free trainer `TrainerSim`, sandboxes, scenarios, runner report, probes) and `src/shared/` (brain, physics, spatial grid, world generation, types/config, seeded RNG, deterministic math). See [[wiki/biology/genome-compiler]], [[wiki/biology/ctrnn-brain]], [[wiki/simulation/physics]].
- **Persistence** — `src/database.rs`, see [[wiki/architecture/persistence]].
- **Frontend** — `frontend/`, see [[wiki/frontend/structure]].
- **Config** — root `config.json`, compiled into the binary, see [[wiki/architecture/configuration]].

## Design principles

- **Rust is the single source of truth** for simulation, world generation and biology; the client only renders and sends actions, it never mutates authoritative positions (FIX-127, TCK-115).
- **`config.json` is the single source of truth** for tunables (TCK-138).
- **No DB work in the hot loop** (RV-P01, TCK-139).
- Structural changes are targeted, without framework switches ([[wiki/decisions/adr-001-non-goals]]).

## Origin

- The project started as a browser/Node.js prototype with WebSocket replication and isomorphic TS modules (TCK-107, TCK-108). The Rust core (TCK-110) and Tauri shell (TCK-109) replaced it; TCK-115 removed the Express server and duplicate TS parsers. Anything mentioning WebSocket, `src-tauri/` or JSON database files describes the retired design.

## Known limitations

- The engine is one ~1,500-line thread closure with the Ocean state as local variables; it is not unit-testable because it needs a real `tauri::WebviewWindow` (RV-A01). The trainer logic is outside it in `TrainerSim` and is unit-tested (TCK-173).

## Key Takeaways

- One process: Tauri shell + one simulation thread + webview; IPC via Tauri commands and one event.
- Rust owns all simulation state; the frontend is a renderer.
- The engine is a single closure (RV-A01).

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/02-architecture.md`, `src/main.rs`, TCK-107..110, TCK-115, TCK-173.
