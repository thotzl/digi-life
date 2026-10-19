---
id: TCK-136
title: Trainer Sandbox Epoch Tick Counter
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
revisit: RV-B05
---

# TCK-136: Trainer Sandbox Epoch Tick Counter

## Description
`TrainerSandbox.epoch_ticks` is reset but never incremented. `finish_tick` is therefore always `Some(0)` and the scenario speed bonus is a constant for every successful candidate.

## Scope
- Reproduction test: `finish_tick` reflects the tick at which food was consumed.
- Increment `epoch_ticks` once per `step_trainer_sandbox_physics`.

## Acceptance
- Test green. Fitness values of new runs are not directly comparable with older HoF entries.

## Verification
- Reproduction test `biology::trainer_engine::tests::test_finish_tick_records_the_tick_of_first_consumption` (`src/biology/trainer_engine.rs`) failed before the fix (`left: Some(0), right: Some(6)`).
- Fix: `step_trainer_sandbox_physics` increments `sb.epoch_ticks` at the start of every tick.
- Guard: speed bonus in `src/biology/scenarios.rs` (standard + exploration) uses `saturating_sub`, so a finish tick beyond the epoch length can never underflow now that the counter is live.
- `cargo test`: 36 passed, 0 failed.
- Effect: the speed bonus now rewards earlier consumption as the fitness functions intended; fitness values of new runs differ from older HoF entries.
