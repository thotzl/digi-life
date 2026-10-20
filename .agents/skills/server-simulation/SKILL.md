---
name: server-simulation
description: Rules and guidelines for the Rust simulation engine thread, physics, Tauri IPC (commands and the simulation-state event), and SQLite persistence of Pixel DNA Life.
---
# server-simulation: Rust Engine, Physics & Tauri IPC

## I. Simulation Thread (`src/server/engine.rs`)
- `spawn_simulation_thread(window, rx)` is started from `main.rs` setup. The whole engine is one thread closure holding Ocean and Trainer state as local variables (refactor target RV-A01).
- **Frame:** ~60 Hz (`16.67 ms` sleep budget). State emission throttled to 25 Hz (`emit_interval` 40 ms), food at 2 Hz.
- **Events:** `emit_state` only buffers; `flush_events()` sends the buffer once per frame in order and coalesces repeated `DATABASE_CHANGED`.
- **Hot loop:** no SQL per step except on actual changes. The spatial grid is reused (`clear()`), buckets store vector indices; spore pushes use `SporePusher`, prey/bite lookups use `PeerSnapshot` (no agent clones). `species_records` is mirrored in `species_cache` and re-read only after species inserts/resets (`species_cache_dirty`).
- **Warp cap:** steps per frame are clamped to **35** for both Ocean (`ocean_warp_factor`) and Trainer (`trainer_warp_speed`). Keep this cap; unbounded steps block the action channel and the UI (Pause stops responding).
- **Mode:** one engine, two modes (`SET_MODE` `ocean` | `trainer`). The mode is set only by `navigateTo` in `frontend/core/router.ts`.
- **Physics:** `shared/physics.rs` (`step_creature_kinematics`, `apply_creature_physics`, `step_food_spore_physics`); thrust is scaled by `thrustBaseMultiplier` from config. Neighbor queries via `shared/spatial_grid.rs`. World generation in `shared/map_generator.rs` (world size 19200 x 10800, hardcoded in Rust and TS, RV-A05).
- **Trainer:** sandboxes in `biology/trainer_engine.rs`; scenarios implement `TrainingScenarioPlugin` (`biology/scenarios.rs`: `standard`, `exploration`).

## II. IPC Protocol
- **Frontend -> backend:** `invoke("handle_client_action", { action: <JSON string> })` is forwarded over an mpsc channel to the engine and matched on `type` (`CLIENT_READY`, `TOGGLE_SIMULATION`, `INJECT_URZELLE`, `SPAWN_CLONE`, `RESET_EVOLUTION`, `SELECT_AGENT`, `SET_SIMULATION_SPEED`, `SET_MODE`, `SET_TRAINER_MODE`, `START_TRAINING`, `PAUSE_TRAINING`, `TRAINER_RESET`, `SELECT_TRAINER_SANDBOX`, `ASSIGN_SANDBOX_CREATURE`, `UPDATE_TRAINER_HYPERPARAMS`, ...). Unknown types are logged to stderr; missing or mistyped fields are still silently ignored (RV-A02).
- Other commands in `server/api.rs` (species, trainer runs/HoF, catalogue CRUD, `get_fossil_phenotype`, `apply_champion`, ...) share one SQLite connection (`DbState`, mutex-guarded) created once in `main.rs` setup; the engine thread holds its own connection to the same file.
- **Backend -> frontend:** single event `"simulation-state"`; both `ocean.ts` and `trainer.ts` listen and dispatch on payload `type`. Field casing is mixed.
- The client is a renderer: it never mutates authoritative positions; it only sends actions.

## III. Persistence (`src/database.rs`)
- SQLite file `pixel_life_local.db` in the Tauri app data dir (`database::db_path()`, set at startup; legacy `target/pixel_life_local.db` copied once). `init_db` creates the schema (`CREATE TABLE IF NOT EXISTS` + ad-hoc column migration, RV-D02), enables WAL and a 5 s busy timeout.
- Write failures are logged via `LogDbError::log_db_err` instead of discarded.
- Ocean auto-save every 10 s: row `simulation_state.key = 'current_state'` (creatures, food, counters) and a `simulation_history` row (population, generation, avg/max energy stored in `average_fitness`/`max_fitness`). Loaded on startup.
- Species ledger: `species_records`. Trainer lineage: `trainer_genomes` (by `run_id`). Catalogue: `creature_catalogue`.
- `RESET_EVOLUTION` deletes `species_records`, `simulation_history`, `simulation_state` and must clear `newly_spawned_creatures`.
