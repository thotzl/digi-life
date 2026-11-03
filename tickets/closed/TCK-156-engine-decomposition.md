---
id: TCK-156
title: Engine Decomposition (OceanSim / TrainerSim)
status: closed
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-A01, RV-A05, RV-P04
---

# TCK-156: Engine Decomposition (OceanSim / TrainerSim)

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R3 (after TCK-200; parallel with TCK-201). Trainer slices are done (`TrainerSim`, `trainer_ipc.rs`, factory). Remaining scope, from the 2026-11-02 review:
- `OceanSim` (IO-free, seeded `SimRng` streams, uses the shared world systems of TCK-200); `step()` returns Ocean events and species changes instead of writing SQL. Ocean behaviour changes by design through seeding and the shared system order (no Ocean reference exists yet; TCK-158 adds one).
- `Engine` with a `Mode` trait (`OceanMode`, `TrainerMode`: `on_enter`, `on_exit`, `handle`, `step`, `emit_telemetry`), `EventSink` (`TauriSink` with buffer/coalescing, `VecSink` for tests), store ports from TCK-159; `run_loop(rx, sink, store)` testable without Tauri.
- Remove duplication in `engine.rs`: founder spawn 6x (233-271, 514-555, 556-611, 649-688, 692-725, 1358-1416) -> `OceanSpawner::founder(Catalogue|Random|Genome)`; spores 3x + nutrient centres 2x -> `SporeField`; decomposition 2x (1013, 1231); INIT_STATE 2x (481, 773); `base_dna` literal (231); species INSERT 7x -> `SpeciesLedger` (pure cache + diff).
- Fixes: DB init `expect` panics the engine thread silently (97) -> error event; two connections both run `init_db` (main.rs:44, engine.rs:97); Ocean and trainer share `last_emit_time` (1421, 1542); warp cap 35 as a constant, clamped value echoed to the UI (425, 795, 829, 1511); irrefutable `let TrainerEvent::GenerationCompleted` (1514); full snapshot serialised on the frame thread every 10 s (1488); SET_MODE trainer rebuilds from DB on every navigation (805); nutrient centres not persisted; `WorldSpec` (19200x10800, RV-A05) from config, sent in INIT_STATE.
- Contracts `wiki/contracts/mode.md`, `event-sink.md`.
- Gate: golden master and trainer references unchanged.


> Pool ticket: requires a user decision before implementation. Findings (RV-A01, RV-A05) below.

## Description
`spawn_simulation_thread` is a ~2000-line closure; creature construction duplicated 6x; world size literals in Rust and TS.

## Proposed Scope
Structs with `step()` + event sink, `CreatureFactory`, constants from one source. Include nearest-food lookup via the spatial grid (RV-P04, deferred from TCK-139 because a ring search can change tie-breaking).

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-A01 · P1** — `src/server/engine.rs`: no struct/impl; the whole state lives as ~35 local variables in one closure (~2,000 lines). Creature construction duplicated 6× (seed, inject, clone, reset ×2, restock) with diverging start values (energy 0.6 vs 0.8, `antisense` empty vs set, `neuron_states` empty vs preallocated); spore spawn with nutrient centers 3×; `TRAINER_STATE_CHANGED` payload 5×; `base_dna` literal 3× (+ `brain.rs` test + `cli_trainer.rs`). Ocean and Trainer share `last_emit_time`. Not testable: needs a real `tauri::WebviewWindow`.
- Target:
  ```
  struct OceanSim   { creatures, food, world, rng, ids, config } impl { fn step(&mut self) -> Vec<SimEvent> }
  struct TrainerSim { sandboxes, hyperparams, generation, ... }   impl { fn step(&mut self) -> Vec<SimEvent> }
  struct Engine     { ocean, trainer, mode, repo: Box<dyn Repository> }
  fn run_loop(engine, rx: Receiver<Action>, sink: impl EventSink)   // Tauri window only as EventSink impl
  ```
  `step()` free of IO → headless-testable; one `CreatureFactory::spawn(genome, opts)`. Non-goal: ECS framework (ADR-001).
- **RV-A05 · P2** — `19200 × 10800` hardcoded in Rust (`engine.rs`, `trainer_engine.rs`) and TS (5 places); `/ 25` population display in `ocean.ts`; trainer epoch `300/1800` and chamber `1000/3500` literals in the engine. Target: world size via `INIT_STATE`; plugin provides `chamber_size()`, `epoch_ticks()`, `spawn_min_distance()`.
- **RV-P04 · P2** (deferred from TCK-139) — nearest-food search scans all spores twice and all creatures, O(C × F + C²); the grid returns IDs, then `position(id)` is O(F). Target: grid stores indices, ring search up to sensor range. Deferred because a ring search can change tie-breaking between equidistant spores.

## Verification (2026-11-03, v1.12.37)
- `src/server/engine.rs` (1,374 lines) split into `src/server/engine/`: `mod.rs` (`Engine` with `new`/`handle`/`frame`, `EngineInput { Action, TrainerStoreChanged }`, `run_loop`, `spawn_simulation_thread` - the only Tauri dependency), `mode.rs` (`Mode` trait `on_enter`/`on_exit`/`handle`/`step`/`emit_telemetry`, `ModeCtx { sink, store, config, now }`, `MAX_STEPS_PER_FRAME`, `clamp_warp`, `Interval`), `ocean_mode.rs`, `trainer_mode.rs` (thin wrapper over `trainer::sim::TrainerSim`), `sink.rs` (`EventSink`, `TauriSink`, `VecSink`, coalescing), `store.rs` (`EngineStore` over the four repositories, `open_store` with in-memory fallback, `SnapshotWriter` writer thread). Largest engine module 330 lines.
- IO-free seeded Ocean `src/sim/ocean/`: `OceanSim` (`fresh`, `restore`, `reset`, `release`, `tick`, `step`, `restock`, `snapshot`) on the shared `src/sim/` systems returning `OceanStep { events, species, born }`; `OceanSpawner::founder(Random|Wildtype|Genome)`, `SporeField`, pure `SpeciesLedger`, typed `OceanSnapshot` (reads older snapshots). `OceanConfig` moved to `src/config/ocean.rs` (+ `worldWidth`/`worldHeight`); `sim` no longer depends on `server`.
- Removed duplicates: founder spawn 6x, spores 3x + nutrient centres 2x, decomposition 2x, INIT_STATE 2x, `base_dna` literal, species writes 7x + cache re-read, inline bite target/peer snapshot/patchy position/prey-plant search.
- Fixes: DB open failure falls back to an in-memory store and is reported on every handshake (was a silent thread panic; `test_unopenable_store_falls_back_to_memory_and_reports_on_handshake`); one `init_db` (engine uses `open_db`); per-mode emit throttles; warp clamped when set and echoed (`SIM_STATE.speed`, `TRAINER_STATE_CHANGED.warpSpeed`; `test_warp_is_clamped_and_echoed_for_ocean_and_trainer`); exhaustive `match TrainerEvent`; snapshot serialised on a writer thread; trainer rebuilt on entry only when not loaded or after `TrainerStoreChanged` (catalogue -> training, clear history); nutrient centres persisted; world size from config in `INIT_STATE.world`.
- Ocean behaviour changes by design (no Ocean reference yet, TCK-158): seeded `SimRng` streams (`ocean_seed`, per-tick streams, reset with a clock nonce; seed/tick/release count in the snapshot so a restored Ocean continues identically), shared system order (respawn/decomposition applied after each creature's step), species writes once per frame, a snapshot with unparseable spores starts fresh. Mitosis children still only shown (RV-B01, TCK-143).
- Integration on master (after TCK-160 and TCK-201): compatibility shims `biology::scenarios` / `biology::trainer_sim` removed, `TrainerMode` imports `crate::trainer`; warp slider now 1x..10x plus "Max (35x)" (the former "50x"/"Unlimited" positions both ran at 35 and the engine now echoes 35; `frontend/shared/warp.ts`, test updated).
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 277 passed, 1 ignored (headless engine tests with `VecSink` + `MemoryDb`, `sim::ocean` tests); golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum, rehearsal references match; `npm test` 41 passed; build ok; doc links OK after the ticket move; `bindings/OceanUiEvent.ts` (`SIM_STATE.speed`) committed. UI checks OK: smoke (Ocean 25 -> 26 after inject, trainer generation, metrics, curriculum, room switch, pause, reset), selection/reset (tck194), dialogs; 0 console errors.
- Docs: `wiki/contracts/mode.md`, `wiki/contracts/event-sink.md`, `wiki/architecture/simulation-engine.md` rewritten; ocean, system-overview, repository-map, configuration, config-section, invariants 2 and 3 (now tested), creature-system, physics, trainer, ipc-protocol, ipc-message, add-ipc-message, persistence, metabolism-and-reproduction, genome-compiler, testing, analytics, master index; skill `server-simulation`; mode part of `add-room-or-mode.md`.
- Left open: RV-P04 (nearest-food lookup scans all foods per creature); open tickets TCK-143/146/148/202 still cite old `engine.rs` line numbers (code locations to be re-found when they start).
