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

## Acceptance
- Same seed → bit-identical report.
- Equivalence: UI trainer behaves as before (existing tests green; golden master unchanged).
- Baseline report for small and large chamber recorded.

## Notes
- Warp cap 35 stays for the UI path (invariant); the headless runner has no cap.
- `src/bin/cli_trainer.rs` is outdated (RV-S11); replace or remove it here or in TCK-158.
