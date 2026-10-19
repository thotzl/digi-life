---
id: TCK-136
title: Trainer Sandbox Epoch Tick Counter
status: open
assigned: Claude Code
created_at: 2026-10-19
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
