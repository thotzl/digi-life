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
