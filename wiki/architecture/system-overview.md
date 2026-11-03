---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, .agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-109-tauri-desktop-shell.md, tickets/closed/TCK-110-rust-simulation-core.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-124-tauri-exclusive-consolidation.md, tickets/closed/TCK-107-refactoring-dry-and-shared-modularization.md, tickets/closed/TCK-108-headless-backend-evolutionary-trainer-migration.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-156-engine-decomposition.md]
---

# System Overview

BroodBattle (repository `digi-life`, formerly Pixel DNA Life) is a single-user desktop app: **Tauri v2** shell, **Rust 2024** simulation core, **SQLite** persistence, **Vite + TypeScript + Preact Signals** webview UI. There is no network server, no WebSocket and no port.

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

## Plugin host and registries

The engine is the plugin host; every extension point is a registry or an enum with one registration point
(TCK-196). Contracts: one per seam; registered plugins: [[wiki/architecture/module-map]].

```
Engine (src/server/engine/)  -- host: inputs -> active Mode -> EventSink, once per frame
 ├─ Mode registry: SimMode + Engine arms ............ OceanMode | TrainerMode          contract mode
 ├─ EventSink ....................................... TauriSink | VecSink             contract event-sink
 ├─ EngineStore = repository traits (persistence/) .. Db (SQLite) | MemoryDb           contract persistence-repo
 └─ IPC groups: ClientAction / UiEvent per mode, commands in api/ ...                  contract ipc-message
OceanSim (sim/ocean/)            TrainerSim (trainer/)
 └─ World + WorldSettings         ├─ Room registry ROOMS ............ LevelRoom (standard, exploration)   contract room
     └─ System list (sim/systems/) ├─ Evaluator ...................... SoloTrials                          contract evaluator
         Sense Think Metabolize    └─ Chamber -> same World + System list
         Death Move Eat  .........................................................     contract creature-system
Creature model (biology/): parse_genome -> organs via OrganKind::spec registry ....... contract organ-kind
Config: config.json -> AppConfig sections, passed down from the entry points ......... contract config-section
Webview: router + VIEWS registry (ViewModule) -> core/ipc.ts ......................... contract frontend-view
```

## Layers

- **Shell** — `src/main.rs`: Tauri builder, command registration, DB setup, starts the simulation thread.
- **Engine** — `src/server/engine/`: `Engine` + `Mode` (`OceanMode`, `TrainerMode`) + `EventSink` + store ports, ~60 Hz loop; runs headless in tests (TCK-156), see [[wiki/architecture/simulation-engine]], [[wiki/contracts/mode]], [[wiki/contracts/event-sink]].
- **API** — `src/server/api/`: Tauri commands; protocol types in `src/server/ipc.rs`, `ocean_ipc.rs`, `trainer_ipc.rs`; see [[wiki/architecture/ipc-protocol]] and [[wiki/contracts/ipc-message]].
- **Simulation kernel** — `src/sim/`: shared creature systems (TCK-200) and the IO-free, seeded Ocean `OceanSim` (`src/sim/ocean/`, TCK-156), see [[wiki/contracts/creature-system]], [[wiki/simulation/ocean]].
- **Trainer** — `src/trainer/`: IO-free `TrainerSim`, rooms, chambers, evaluator, breeder, curriculum (TCK-201), see [[wiki/simulation/trainer]], [[wiki/contracts/room]], [[wiki/contracts/evaluator]].
- **Creature model** — `src/biology/` (genome, phenotype, organs, brain compiler, creature factory, metabolism, diet, trainer config and store adapter) and `src/shared/` (brain execution, physics, spatial grid, world generation, simulation structs `dto`, RNG, deterministic math, helpers). See [[wiki/biology/genome-compiler]], [[wiki/contracts/organ-kind]], [[wiki/biology/ctrnn-brain]], [[wiki/simulation/physics]].
- **Headless runner** — `src/bin/trainer_runner.rs` + `src/diagnostics/` (feature `runner`): trainer runs, Ocean runs and mechanics probes without UI, see `wiki/simulation/analytics-and-balancing.md`.
- **Persistence** — `src/persistence/` (repository traits, the only module with SQL), see [[wiki/architecture/persistence]] and [[wiki/contracts/persistence-repo]].
- **Frontend** — `frontend/`, see [[wiki/frontend/structure]] and [[wiki/contracts/frontend-view]].
- **Config** — root `config.json`, one section per subsystem, compiled into the binary, loaded only by the entry points and passed explicitly; see [[wiki/architecture/configuration]] and [[wiki/contracts/config-section]].
- **Logging** — `log` facade, backend `src/logging.rs` (TCK-198): dev builds and the runner log to stderr; the release app (no console, `windows_subsystem = "windows"`) writes `<app data dir>/logs/broodbattle.log`, rotated at 1 MiB into `broodbattle.log.1`.

## Design principles

- Rust owns the simulation, the client renders ([[wiki/architecture/invariants]] 14); `config.json` holds every
  tunable (11); no DB work in the hot loop (6); simulations are IO-free and seeded (16).
- Structural changes are targeted, without framework switches ([[wiki/decisions/adr-001-non-goals]]).
- The retired browser/Node.js design (WebSocket, `src-tauri/`, JSON database files): [[wiki/history/replaced-models]].

## Known limitations


## Key Takeaways

- One process: Tauri shell + one simulation thread + webview; IPC via Tauri commands and one event.
- Rust owns all simulation state; the frontend is a renderer.
- The engine is the plugin host: modes, sinks and stores; simulations take rooms, evaluators, systems and organ kinds
  from registries, each with a contract (TCK-196).

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, `src/main.rs`, TCK-107..110, TCK-115, TCK-156, TCK-173.
