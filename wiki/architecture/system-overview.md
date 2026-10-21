---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, .agents/skills/server-simulation/SKILL.md, docs/revisit/02-architecture.md]
---

# System Overview

Pixel DNA Life is a single-user desktop app: **Tauri v2** shell, **Rust 2024** simulation core, **SQLite** persistence, **Vite + TypeScript + Preact Signals** webview UI. There is no network server, no WebSocket and no port (the WebSocket/Node.js design was retired, see [[architecture/history]]).

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
- **Engine** — `src/server/engine.rs`: Ocean and Trainer loop (~60 Hz), see [[architecture/simulation-engine]].
- **API** — `src/server/api.rs`: Tauri commands, see [[architecture/ipc-protocol]].
- **Domain** — `src/biology/` (genome compiler, trainer sandboxes, scenarios) and `src/shared/` (brain, physics, spatial grid, world generation, types/config). See [[biology/genome-compiler]], [[biology/ctrnn-brain]], [[simulation/physics]].
- **Persistence** — `src/database.rs`, see [[architecture/persistence]].
- **Frontend** — `frontend/`, see [[frontend/structure]].
- **Config** — root `config.json`, compiled into the binary, see [[architecture/configuration]].

## Design principles

- **Rust is the single source of truth** for simulation, world generation and biology; the client only renders and sends actions, it never mutates authoritative positions (FIX-127, TCK-115).
- **`config.json` is the single source of truth** for tunables (TCK-138).
- **No DB work in the hot loop** (RV-P01, TCK-139).

## Target architecture (pool, not started)

From RV-A01 (TCK-156): split the engine closure into testable structs.

```
struct OceanSim   { creatures, food, world, rng, ids, config } impl { fn step(&mut self) -> Vec<SimEvent> }
struct TrainerSim { sandboxes, hyperparams, generation, ... }   impl { fn step(&mut self) -> Vec<SimEvent> }
struct Engine     { ocean, trainer, mode, repo: Box<dyn Repository> }
fn run_loop(engine, rx: Receiver<Action>, sink: impl EventSink)   // Tauri window only as EventSink impl
```

- `step()` free of IO: headless-testable, basis for a headless runner (TCK-158).
- One `CreatureFactory::spawn(genome, opts)` replaces 6 duplicated constructions.
- Non-goal: an ECS framework; `Vec<Creature>` suffices at 25–45 agents ([[roadmap]]).

## Key Takeaways

- One process: Tauri shell + one simulation thread + webview; IPC via Tauri commands and one event.
- Rust owns all simulation state; the frontend is a renderer.
- The engine is still one large closure; decomposition is pool ticket TCK-156.

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/02-architecture.md`, `src/main.rs`.
