---
id: TCK-177
title: "Trainer UI: Success Rate, Median and Progress Curve"
status: open
priority: medium
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173
---

# TCK-177: Trainer UI: Success Rate, Median and Progress Curve

Part of epic TCK-171.

## Description
The trainer UI only receives best and average fitness per generation (`TRAINER_GENERATION_COMPLETED`, `src/server/engine.rs`). Average fitness hides whether the population actually eats; progress over generations is not visible.

## Scope
- Per generation: success rate, median fitness, time to first catch (same metrics as the TCK-173 report), plus the difficulty level once TCK-176 exists.
- Curve over generations in the trainer view.

## Acceptance
- Metrics in the UI match the headless report for the same seed.
- `npm run build` green; checked manually in the trainer view.
