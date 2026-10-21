---
sources: [.agents/skills/server-simulation/SKILL.md, docs/revisit/02-architecture.md, docs/revisit/03-performance.md, .tickets/closed/TCK-139-ocean-hot-loop-performance.md, .tickets/closed/TCK-129-architecture-refactoring.md]
---

# Simulation Engine

`src/server/engine.rs` — `spawn_simulation_thread(window, rx)` is started from `main.rs` setup. The whole engine is one thread closure holding Ocean and Trainer state as local variables (~2,000 lines; refactor target RV-A01, TCK-156).

## Frame and timing

- Frame ~60 Hz (16.67 ms sleep budget).
- **Warp cap: 35 steps per frame** for both Ocean (`ocean_warp_factor`) and Trainer (`trainer_warp_speed`), ≈ 2,100 steps/s at full warp. Unbounded steps block the action channel and the UI (Pause stops responding). See [[architecture/invariants]].
- State emission throttled to 25 Hz (`emit_interval` 40 ms), food at 2 Hz.
- Ocean and Trainer share `last_emit_time`; a mode switch shifts the emission phase (RV-A01).

## Modes

- One engine, two modes: `SET_MODE` `ocean` | `trainer`.
- The mode is set only by `navigateTo` in `frontend/core/router.ts` ([[architecture/invariants]]).
- Ocean: [[simulation/ocean]]. Trainer: [[simulation/trainer]].

## Events

- `emit_state` only buffers; `flush_events()` sends the buffer once per frame in original order and coalesces repeated `DATABASE_CHANGED` (TCK-139, RV-P05).
- One Tauri event channel `simulation-state` for everything, see [[architecture/ipc-protocol]].

## Hot-loop rules (current state)

- No SQL per step except on actual changes. `species_records` is mirrored in `species_cache`; it is re-read only after species inserts/resets (`species_cache_dirty`), per-species counts come from one `HashMap` pass (RV-P01).
- Spatial grid allocated once (240 × 135 cells of 80 px) and `clear()`ed per step; buckets store vector indices (RV-P02).
- Spore pushes use `SporePusher` kinematic snapshots, prey/bite lookups use `PeerSnapshot` (id, species, position, velocity) — no agent clones (RV-P03).
- Catalogue loaded once per restock pass (RV-P07).
- Details and remaining items: [[architecture/performance]].

## Known structural issues (pool)

- Creature construction duplicated 6× with diverging start values (energy 0.6 vs 0.8, `antisense` empty vs set, `neuron_states` empty vs preallocated); spore spawn with nutrient centers 3×; `TRAINER_STATE_CHANGED` payload 5×; `base_dna` literal 3× (RV-A01, TCK-156).
- Not unit-testable: needs a real `tauri::WebviewWindow` (RV-A01). Extract logic into pure functions instead ([[quality/testing]]).
- World size `19200 × 10800` hardcoded in Rust (`engine.rs`, `trainer_engine.rs`) and TS; trainer epoch length `300/1800` and chamber size `1000/3500` are literals in the engine instead of the scenario plugin (RV-A05, TCK-156).
- `rand::thread_rng()` at ~20 sites — not reproducible; `config.seed` unused (RV-T04, TCK-158).

## Key Takeaways

- One thread, ~60 Hz, max 35 steps per frame, events flushed once per frame.
- The hot loop is free of per-step SQL, grid allocations and agent clones since TCK-139.
- Decomposition into `OceanSim`/`TrainerSim` and a seedable RNG are pending pool tickets (TCK-156, TCK-158).

Sources: `.agents/skills/server-simulation/SKILL.md`, `docs/revisit/02-architecture.md`, `docs/revisit/03-performance.md`, TCK-129, TCK-139.
