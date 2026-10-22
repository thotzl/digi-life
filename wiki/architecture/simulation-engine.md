---
sources: [.agents/skills/server-simulation/SKILL.md, docs/revisit/02-architecture.md, docs/revisit/03-performance.md, tickets/closed/TCK-139-ocean-hot-loop-performance.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-152-hebbian-plasticity-model.md]
---

# Simulation Engine

`src/server/engine.rs` — `spawn_simulation_thread(window, rx)` is started from `main.rs` setup. The engine is one thread closure holding the Ocean state as local variables (~1,500 lines, RV-A01). The trainer lives in `TrainerSim` (`src/biology/trainer_sim.rs`, TCK-173); the closure keeps only its UI state (active/running, warp, selected sandbox, run id), telemetry and DB writes ([[wiki/simulation/trainer]]).

## Frame and timing

- Frame ~60 Hz (16.67 ms sleep budget).
- **Warp cap: 35 steps per frame** for both Ocean (`ocean_warp_factor`) and Trainer (`trainer_warp_speed`), ≈ 2,100 steps/s at full warp (TCK-129). Unbounded steps block the action channel and the UI (Pause stops responding). See [[wiki/architecture/invariants]].
- State emission throttled to 25 Hz (`emit_interval` 40 ms), food at 2 Hz.
- Ocean and Trainer share `last_emit_time`; a mode switch shifts the emission phase.

## Modes

- One engine, two modes: `SET_MODE` `ocean` | `trainer`.
- The mode is set only by `navigateTo` in `frontend/core/router.ts` ([[wiki/architecture/invariants]]). `CLIENT_READY` only answers with `INIT_STATE` and no longer switches the trainer off; forcing Ocean there raced a fast navigation to the trainer after startup (TCK-186 smoke check).
- Ocean: [[wiki/simulation/ocean]]. Trainer: [[wiki/simulation/trainer]].

## Events

- `emit_state` only buffers; `flush_events()` sends the buffer once per frame in original order and coalesces repeated `DATABASE_CHANGED` (TCK-139, RV-P05).
- One Tauri event channel `simulation-state` for everything, see [[wiki/architecture/ipc-protocol]].

## Creature spawning and RNG

- All Ocean spawn sites (seed, inject, clone, reset, restock) and the trainer build agents through `CreatureFactory` (`src/biology/creature_factory.rs`: `spawn_creature`, `spawn_from_phenotype`, `start_energy`) with the same start energy, `antisense` and neuron buffers (TCK-185). Ocean mitosis children (clones and mutants) are built by `spawn_from_phenotype` too, with fresh neuron states and the parent's post-split energy (TCK-152, [[wiki/biology/metabolism-and-reproduction]]).
- Trainer: seeded `SimRng` (`ChaCha8Rng`, `src/shared/rng.rs`), run seed `seed_from_str("{config.seed}:{run_id}")`, sandboxes stepped in parallel with `TrainerSim::step(true)`; reproducible per run ([[wiki/simulation/trainer]]).
- Ocean: `rand::thread_rng()` at 11 sites in `engine.rs`; Ocean runs are not reproducible (RV-T04). `config.seed` seeds the Ocean world generation and the trainer run seed (TCK-149, TCK-173).

## Hot-loop rules

- No SQL per step except on actual changes. `species_records` is mirrored in `species_cache`; it is re-read only after species inserts/resets (`species_cache_dirty`), per-species counts come from one `HashMap` pass (RV-P01).
- Spatial grid allocated once (240 × 135 cells of 80 px) and `clear()`ed per step; buckets store vector indices (RV-P02).
- Spore pushes use `SporePusher` kinematic snapshots, prey/bite lookups use `PeerSnapshot` (id, species, position, velocity) — no agent clones (RV-P03).
- Catalogue loaded once per restock pass (RV-P07).
- Details: [[wiki/architecture/performance]].

## Known limitations

- Spore spawn with nutrient centers 3×; `base_dna` literal 1× (fresh-start seeding) in `engine.rs` besides `dna::WILDTYPE_GENOME`; the `RESET_EVOLUTION` copy was replaced by structured random founders (TCK-184) (RV-A01). Creature construction and the trainer state payload exist once (TCK-173, TCK-185).
- Not unit-testable (needs a Tauri window); logic is tested through extracted pure functions ([[wiki/quality/testing]]).
- World size `19200 × 10800` hardcoded in Rust and TS (RV-A05); trainer chamber size and epoch length come from the scenario plugin (TCK-173).
- Ocean RNG is `thread_rng` (RV-T04, above).

## Key Takeaways

- One thread, ~60 Hz, max 35 steps per frame, events flushed once per frame.
- The hot loop is free of per-step SQL, grid allocations and agent clones (TCK-139).
- Trainer state, RNG and breeding are in the IO-free `TrainerSim`; one `CreatureFactory` for all spawns.
- Ocean engine structure, world-size constants and Ocean RNG are known limitations (RV-A01, RV-A05, RV-T04).

Sources: `.agents/skills/server-simulation/SKILL.md`, `docs/revisit/02-architecture.md`, `docs/revisit/03-performance.md`, TCK-129, TCK-139, TCK-149, TCK-173, TCK-185, TCK-184, TCK-186.
