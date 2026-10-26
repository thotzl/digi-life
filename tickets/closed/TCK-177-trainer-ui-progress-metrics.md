---
id: TCK-177
title: "Trainer UI: Success Rate, Median and Progress Curve"
status: closed
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

## Verification (P8, 2026-10-26)
- Typed trainer IPC (`src/server/trainer_ipc.rs`, ts-rs bindings `TrainerUiEvent`, `TrainerTelemetryTick`, `TrainerGenerationCompleted`, `TrainerGenerationStats`, `TrainerState`, `TrainerAction`, …): the telemetry tick, generation completed, state changed and reset completed events are typed instead of `json!`; trainer actions are a typed enum (`UPDATE_TRAINER_HYPERPARAMS`, `START_TRAINING`, `PAUSE_TRAINING`, `TRAINER_RESET`, `SET_TRAINER_MODE`, `SET_TRAINER_LEVEL`, `SELECT_TRAINER_SANDBOX`, `ASSIGN_SANDBOX_CREATURE`); malformed trainer actions are logged and ignored. The trainer view uses the generated types.
- One source for UI and report: `TRAINER_GENERATION_COMPLETED.stats` is built from the runner's `GenerationStats` (a test asserts equal JSON values).
- UI (`frontend/views/TrainerView.ts`, `frontend/trainer.ts`, `frontend/trainerMetrics.ts`, `frontend/render/progressChart.ts`): success rate (`#stat-success`), median fitness (`#stat-median-fit`), median time to first catch (`#stat-first-catch`), level with auto/manual/fixed (`#stat-level`), progress curve (`#canvas-progress`: success, best, median, level per generation), mode select with "Curriculum (auto)" and "Manual Level", level slider (`#slider-level`), pending-switch note (`#lbl-pending-switch`). Exact values in the `title` attributes.
- Render loop: the trainer draws in a `requestAnimationFrame` loop while the trainer view is active (`currentView` effect); the 25 Hz telemetry only stores snapshots and marks sandboxes dirty; the loop stops when the view is left.
- Checks: `cargo test` 171 passed, `npm run build`, `npx tsc --noEmit`, vitest 8 passed; UI smoke (`.agents/artifacts/uicheck/out-p8`): metrics shown without NaN (e.g. success 22.9 %, median 0.50, first catch 113 t), progress chart drawn, curriculum mode shows "auto", room switch, pause and reset work, RTT 2–3 ms.
