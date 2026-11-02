---
id: TCK-156
title: Engine Decomposition (OceanSim / TrainerSim)
status: open
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
