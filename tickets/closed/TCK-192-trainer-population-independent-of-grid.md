---
id: TCK-192
title: Trainer Population Independent of the Visible Sandbox Grid
status: closed
priority: high
assigned: agent
created_at: 2026-10-26
epic_phase: Training Track (wrap-up)
depends_on: TCK-171
---

# TCK-192: Trainer Population Independent of the Visible Sandbox Grid

Wrap-up of epic TCK-171 (open gap in TCK-171 `## Result`).

## Description
The app trains with as many creatures as sandbox tiles are shown (grid size N, default 16). At pop 16 the epic's success targets are not met on unseen layouts (small 0.383, large 0.096); at pop 300 they are (0.699 small direct, 0.336 large with curriculum). The training progress of the epic is therefore only visible in the headless runner.

## Scope
- Population size as its own trainer parameter (config default and UI control), independent of the number of displayed tiles; the grid shows the first N sandboxes (elites first), telemetry only for displayed sandboxes; selection, breeding and stats over the whole population.
- Choose the default population from measurement (success on unseen layouts vs. runtime) and check UI responsiveness (RTT, frame drops) at that size.

## Acceptance
- UI trains with the chosen population; UI smoke green with RTT in the usual range; metrics equal the headless report.

## Verification (2026-10-26)
- Config `trainerPopulation` (default 100, missing key → 100; runner `--population` overrides). `UPDATE_TRAINER_HYPERPARAMS`: `N` = visible sandboxes only (engine `trainer_display_count`, never touches the simulation), new `population` restores the population from the DB on change; `TrainerState` carries `population` and `N` (≤ population). `TrainerTelemetryTick::build` sends only the first N sandboxes (breeding order, stable within a generation), brain activations only for a displayed sandbox. Population stored with the run state (`trainer_run_state.population`, column added to v1.12.23 tables by `init_db`).
- UI: `#slider-population` (16 / 50 / 100 / 200 / 300, label `#lbl-population`), "Visible Sandboxes" slider (`#slider-grid-size`, max = min(population, 100)); population sent only on its own change.
- Default from measurement (unseen layouts, 60 gens): pop 100 (5 seeds) small direct 0.708, curriculum large 0.376 vs. pop 300 (10 seeds) 0.709 / 0.395 at ~⅓ of the compute (curriculum run 75 s vs. 116 s); pop 16: 0.454 / 0.227 (TCK-171 `## Wrap-up`).
- Tests: `test_telemetry_carries_only_the_displayed_sandboxes`, `test_display_count_does_not_change_the_simulation` (same seed, no telemetry / N 4 / N 10 → identical generation stats), `test_run_state_table_of_v1_12_23_gains_the_population_column`, run-state apply restores the population, IPC tests, vitest for the slider steps; CI trainer reference unchanged.
- UI smoke at pop 100 (`.agents/artifacts/uicheck/out-tck192`): RTT 13 ms at max warp, 2 ms paused; generation, metrics, curriculum switch, room switch, pause and reset pass; 16 tiles of a population of 100. Fixed on the way: stale texts "Enable Multi-Trial (3 Runs per Gen)" and the champion exporter's non-existent `rules.progenitorGenome` key (dead `apply_champion` path noted in TCK-123).
