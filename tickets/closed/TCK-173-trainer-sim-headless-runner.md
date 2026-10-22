---
id: TCK-173
title: TrainerSim, Seeded RNG and Headless Trainer Runner
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
---

# TCK-173: TrainerSim, Seeded RNG and Headless Trainer Runner

Part of epic TCK-171. Trainer-only slice of TCK-156 (engine decomposition) and TCK-158 (seedable RNG, headless runner); the Ocean side stays untouched.

## Description
Training changes cannot be compared today: the trainer lives inside the ~2000-line engine closure (`src/server/engine.rs`), uses `thread_rng()` and runs only with the UI, capped at 35 steps per frame. Without a reproducible measurement every tuning step is guesswork.

## Scope
- Extract the trainer state and generation loop from the engine closure into `TrainerSim` with `step()` free of IO (events returned, DB writes via the caller). Behaviour-preserving; the UI path keeps working.
- One seeded RNG per `TrainerSim` (`ChaCha8Rng` recommended: identical sequences across `rand` versions and platforms), passed as a parameter instead of `thread_rng()` in `scenarios.rs` and `trainer_engine.rs`.
- Headless binary: `--seed --generations --scenario --chamber --population` → JSON report per generation: success rate, mean and median time to first catch, best / median / mean fitness, fitness distribution, mean genome length, share of immobile genomes.
- Baseline measurement: small and large chamber, current settings, several seeds; numbers into this ticket and `wiki/simulation/trainer.md`.

### Refactoring of the trainer path (behaviour-preserving)
- `StandardScenario::step_physics` and `ExplorationScenario::step_physics` are identical except for the food the creature sees (`src/biology/scenarios.rs:94-133` vs. `235-317`) → one shared step with a hook "visible foods".
- Spawn rejection loops copied 5× without iteration limit (`scenarios.rs:53-76,197-209`, `src/biology/trainer_engine.rs:215-225`, `src/server/engine.rs:1832-1845`) → one helper `spawn_at_min_distance(rng, bounds, from, min_dist, max_tries)` (distances themselves are unified in TCK-175).
- "Target distance by diet" copied 3× (`scenarios.rs:78-90,211-229`, `engine.rs:1847-1876`) → one helper.
- Trial reset in the engine duplicates `init_rust_sandbox` + `plugin.initialize` and misses fields (neuron states kept, heading/energy partly reset) (`engine.rs:1806-1830`) → `TrainerSandbox::reset_trial()`.
- `TRAINER_STATE_CHANGED` payload built 5× (`engine.rs:827-902,1035-1042`) → one function.
- Enums with ts-rs export instead of strings (trainer subset of TCK-155): `Scenario` (`"standard"/"exploration"`), `SandboxOrigin` (`"elite"/"hof"/"mutant"/"random"`), `DietClass` (carnivory thresholds 0.4 / 0.6 as named constants).
- Literals: chamber size / epoch length / scenario detection via `>= 2000.0` (`engine.rs:777,1753,1802`) → from the scenario plugin.
- Shared helpers go into existing modules (`src/shared/`); search for existing helpers before adding new ones.

### Runtime performance (behaviour-preserving, golden master unchanged)
- Hot-loop string matching → enums: organ type via emoji label `contains("👁️")` per organ per tick (`src/shared/brain.rs:56`), activation type as `String` match per neuron per tick (`brain.rs:98-104`) (TCK-155 subset).
- Reuse buffers instead of per-tick allocations: inputs (`brain.rs:162`), outputs (`brain.rs:140`), virtual foods in the large room (`src/biology/scenarios.rs:278`).
- Skip pruned synapses (weight 0) in the forward pass (`brain.rs:85-88`) via a compact active list, re-compacted periodically.
- Run sandboxes in parallel in the headless runner (e.g. `rayon`), one RNG stream per sandbox so results stay reproducible.
- Report metric: ticks per second (per sandbox and total) to track performance; brain topology changes are in TCK-182.

## Acceptance
- Same seed → bit-identical report.
- Equivalence: UI trainer behaves as before (existing tests green; golden master unchanged).
- Baseline report for small and large chamber recorded.

## Notes
- Warp cap 35 stays for the UI path (invariant); the headless runner has no cap.
- `src/bin/cli_trainer.rs` is outdated (RV-S11); replace or remove it here or in TCK-158.

## Verification (P2, v1.12.16)
- `src/biology/trainer_sim.rs`: IO-free `TrainerSim` (`new`, `restore`, `step(parallel)`, `set_scenario`, `assign_creature`), `TrainerParams`, `GenomeRecord`, `GenerationStats`, `TrainerEvent`; the engine keeps only active/running/warp cap 35/selected sandbox/run id/telemetry/DB.
- Seeded RNG: `src/shared/rng.rs` (ChaCha8); per-sandbox streams `derive(seed,[gen,trial,i])`, per-slot breed streams `derive(seed,[gen,BREED_STREAM,slot])`; no `thread_rng` in the trainer path. Cross-platform determinism via `src/shared/fmath.rs` (libm) in the per-tick path.
- Runner: `src/bin/trainer_runner.rs` + `src/biology/trainer_report.rs` (replaces `cli_trainer.rs`, RV-S11). Same seed -> identical report minus `perf`; parallel == sequential; `RAYON_NUM_THREADS=1/3` identical (tests `test_same_seed_identical_report_and_parallel_equals_sequential`, `test_parallel_step_equals_sequential`, `test_parallel_breeding_equals_sequential`).
- Refactoring: `Scenario`/`SandboxOrigin`/`DietClass` enums (ts-rs), plugin `chamber_size`/`epoch_ticks`, one shared `step_physics`, `spawn_at_min_distance` (try limit 1000, RV-S08 loops bounded), one `target_distance_by_diet`, `TrainerSandbox::reset_trial`, one state payload builder, no `>= 2000.0`/300/1800 literals in the engine, dead `world` field removed.
- Bugs fixed while porting: Lamarck source index underflow (fallback mutants), Lamarck mutants built from the unmutated parent genome, ASSIGN kept old antisense/neuron buffers. Quirks kept for TCK-174: `hof` origin is not mutated, avg fitness divides by N, generation number after restart, `set_scenario` keeps accumulators.
- Performance (behaviour-preserving, report bit-identical before/after): `BrainRuntime` (activation enum, organ/motion flags, active-synapse list), reused buffers, live fitness only for the UI; per-tick step +20-30 % (std pop 32: 718k -> 879k sandbox-ticks/s); parallel breeding: generation end at pop 300 2.79 s -> 0.67 s.
- Baseline (5 seeds, 30 gens, pop 16, mean of last 5 gens): success rate small 0.040 (gen 1 0.004), large 0.023 (gen 1 0.000); pop 300: small 0.165 (20 gens), large 0.054 (10 gens); time to first catch small ~190-216 ticks, large ~880-1210 ticks; immobile share gen 1 small 0.108; throughput standard pop 300 parallel ~206k ticks/s total, exploration pop 16 ~469k.
- `cargo test` green, 0 warnings, `npm run build` green, golden master unchanged; UI smoke (TCK-186) passed.
