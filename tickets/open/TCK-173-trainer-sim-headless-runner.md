---
id: TCK-173
title: TrainerSim, Seeded RNG and Headless Trainer Runner
status: open
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

## Acceptance
- Same seed → bit-identical report.
- Equivalence: UI trainer behaves as before (existing tests green; golden master unchanged).
- Baseline report for small and large chamber recorded.

## Notes
- Warp cap 35 stays for the UI path (invariant); the headless runner has no cap.
- `src/bin/cli_trainer.rs` is outdated (RV-S11); replace or remove it here or in TCK-158.
