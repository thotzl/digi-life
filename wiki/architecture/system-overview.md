---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, .agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-109-tauri-desktop-shell.md, tickets/closed/TCK-110-rust-simulation-core.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-124-tauri-exclusive-consolidation.md, tickets/closed/TCK-107-refactoring-dry-and-shared-modularization.md, tickets/closed/TCK-108-headless-backend-evolutionary-trainer-migration.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-156-engine-decomposition.md]
---

# System Overview

Pixel DNA Life is a single-user desktop app: **Tauri v2** shell, **Rust 2024** simulation core, **SQLite** persistence, **Vite + TypeScript + Preact Signals** webview UI. There is no network server, no WebSocket and no port.

## Runtime structure

```
Tauri main.rs
 ├─ setup: resolve app_data_dir DB path, migrate legacy DB, init schema once, manage DbState
 ├─ invoke_handler: 12 commands (src/server/api/), typed results + IpcError, shared mutex-guarded DB connection
 └─ spawn_simulation_thread(window, rx, config)    (src/server/engine/)
      ├─ own SQLite connection to the same file (open_db, no migration; MemoryDb fallback)
      ├─ rx: EngineInput (mpsc): typed ClientActions + trainer store notifications from the commands
      ├─ Engine: OceanMode (OceanSim) | TrainerMode (TrainerSim), mode switch by the router
      ├─ snapshot writer thread (own connection, Ocean snapshot every 10 s)
      └─ TauriSink: window.emit("simulation-state", UiEvent)  buffered, flushed once per frame
Webview (frontend/)
 ├─ core/ipc.ts call / send / setMode -> commands
 └─ core/ipc.ts onUiEvent("simulation-state") -> render loops (Canvas 2D) + signals HUD
```

## Layers

- **Shell** — `src/main.rs`: Tauri builder, command registration, DB setup, starts the simulation thread.
- **Engine** — `src/server/engine/`: `Engine` + `Mode` (`OceanMode`, `TrainerMode`) + `EventSink` + store ports, ~60 Hz loop; runs headless in tests (TCK-156), see [[wiki/architecture/simulation-engine]], [[wiki/contracts/mode]], [[wiki/contracts/event-sink]].
- **Headless runner** — `src/bin/trainer_runner.rs`: trainer runs and mechanics probes without UI, see [[wiki/simulation/analytics-and-balancing]].
- **API** — `src/server/api/`: Tauri commands; protocol types in `src/server/ipc.rs`, `ocean_ipc.rs`, `trainer_ipc.rs`; see [[wiki/architecture/ipc-protocol]] and [[wiki/contracts/ipc-message]].
- **Simulation kernel** — `src/sim/`: shared creature systems (TCK-200) and the IO-free, seeded Ocean `OceanSim` (`src/sim/ocean/`, TCK-156), see [[wiki/contracts/creature-system]], [[wiki/simulation/ocean]].
- **Domain** — `src/biology/` (genome compiler, creature factory, IO-free trainer `TrainerSim`, sandboxes, scenarios, runner report, probes) and `src/shared/` (brain, physics, spatial grid, world generation, IPC structs `dto`, geometry/time/JSON-column helpers, seeded RNG, deterministic math). See [[wiki/biology/genome-compiler]], [[wiki/biology/ctrnn-brain]], [[wiki/simulation/physics]].
- **Persistence** — `src/persistence/` (repository traits, the only module with SQL), see [[wiki/architecture/persistence]] and [[wiki/contracts/persistence-repo]].
- **Frontend** — `frontend/`, see [[wiki/frontend/structure]].
- **Config** — root `config.json`, one section per subsystem, compiled into the binary, loaded only by the entry points (`main.rs`, `trainer_runner.rs`) and passed explicitly; see [[wiki/architecture/configuration]] and [[wiki/contracts/config-section]].
- **Logging** — `log` facade, backend `src/logging.rs` (TCK-198): dev builds and the runner log to stderr; the release app (no console, `windows_subsystem = "windows"`) writes `<app data dir>/logs/broodbattle.log`, rotated at 1 MiB into `broodbattle.log.1`.

## Design principles

- **Rust is the single source of truth** for simulation, world generation and biology; the client only renders and sends actions, it never mutates authoritative positions (FIX-127, TCK-115).
- **`config.json` is the single source of truth** for tunables (TCK-138).
- **No DB work in the hot loop** (RV-P01, TCK-139).
- Structural changes are targeted, without framework switches ([[wiki/decisions/adr-001-non-goals]]).

## Origin

- The project started as a browser/Node.js prototype with WebSocket replication and isomorphic TS modules (TCK-107, TCK-108). The Rust core (TCK-110) and Tauri shell (TCK-109) replaced it; TCK-115 removed the Express server and duplicate TS parsers. Anything mentioning WebSocket, `src-tauri/` or JSON database files describes the retired design.

## Known limitations

- Mitosis children of the Ocean are shown but not simulated (RV-B01, TCK-143).

## Key Takeaways

- One process: Tauri shell + one simulation thread + webview; IPC via Tauri commands and one event.
- Rust owns all simulation state; the frontend is a renderer.
- The engine is decomposed into modes over IO-free simulations and is tested headless (TCK-156).

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, `src/main.rs`, TCK-107..110, TCK-115, TCK-156, TCK-173.
