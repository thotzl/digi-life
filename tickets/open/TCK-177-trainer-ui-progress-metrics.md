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
- Typed trainer events and actions (trainer subset of TCK-157): `#[derive(Serialize, TS)]` event types for trainer telemetry, generation completed and state changed instead of `json!` payloads; trainer actions as a typed enum; the trainer view imports the generated `bindings/` types instead of handwritten ones in `frontend/shared/types.ts`.

### Trainer rendering loop (user, 2026-10-21)
- Like the Ocean (`frontend/ocean.ts` `drawBetaSimulationFrame`): the trainer draws in a `requestAnimationFrame` loop at display rate from the latest received `TRAINER_TELEMETRY_TICK` snapshot, instead of drawing inside the telemetry handler (`frontend/trainer.ts` ~566-575). The handler only stores the snapshot. Backend emission stays at 25 Hz; jumps between keyframes are accepted (decoupled backend/frontend). The loop stops when the trainer view is not active.

## Acceptance
- Metrics in the UI match the headless report for the same seed.
- `npm run build` green; checked manually in the trainer view.
