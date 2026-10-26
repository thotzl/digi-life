---
id: TCK-191
title: Persist Trainer Run State (Level, Mode, Layout, Mutation Ops)
status: closed
priority: medium
assigned: agent
created_at: 2026-10-26
epic_phase: Training Track (wrap-up)
depends_on: TCK-176
---

# TCK-191: Persist Trainer Run State (Level, Mode, Layout, Mutation Ops)

Wrap-up of epic TCK-171 (known limits of TCK-174 and TCK-176).

## Description
After an app restart, re-entering the trainer or switching runs, a run restarts at layout 0, the curriculum level resets to `trainerCurriculum.startLevel`, and the training mode and the mutation strength adapted by the 1/5 rule are lost. Only the elites are stored (`trainer_genomes`).

## Scope
- Table `trainer_run_state` per run (mode, level, auto/manual, layout id, max mutation ops), saved after each generation next to the elites, loaded on restore, cleared with the run and on model resets.

## Acceptance
- Round-trip and restore tests; fixed rooms unchanged (CI reference); restart check in the UI smoke.

## Verification (2026-10-26)
- Table `trainer_run_state` (`run_id` PK, `scenario` as UI mode string, `level`, `level_auto`, `layout_id` as decimal text, `mutation_ops_max`, `updated_at`), created in `init_db`, part of `MODEL_TABLES` (`src/database.rs`). `save_trainer_run_state` after `save_trainer_elites` (engine after each generation, runner `--persist`), `load_trainer_run_state` applied in the engine's `restore_trainer` at app start and after a run switch (`TrainerSim::restore_at_layout`, `TrainerRunState::apply` clamps the ops and snaps the level), `delete_trainer_run` on trainer reset and `clear_trainer_history`. A switch still pending at save time is not stored. Re-entering the trainer on the same run keeps the in-memory state (no layout reset any more).
- Tests: `test_restore_with_run_state_continues_layout_level_mode_and_mutation_ops`, `test_run_state_apply_clamps_mutation_ops_and_snaps_level`, `test_trainer_run_state_round_trip_and_missing_row` (incl. u64 above `i64::MAX`), `test_delete_trainer_run_clears_genomes_and_run_state_of_that_run_only`, model-reset tests extended; CI trainer reference unchanged.
- App restart check (`.agents/artifacts/uicheck/run-smoke.sh` with `SCRIPT2`, scripts `restart1.cjs` / `restart2.cjs`): curriculum mode trained to generation 5 → stored row `('default_run', 'curriculum', 0.0, 1, '4', 4)` → after the restart the trainer comes back in curriculum mode, level auto. Known: the generation counter restarts at the newest stored parent generation (here 4), so one generation number repeats.
