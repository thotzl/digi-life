---
id: TCK-201
title: "R3 Rooms as Plugins, Evaluator and Breeder, Multi-Entity Chambers"
status: open
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R3
depends_on: TCK-200
---

# TCK-201: Rooms as Plugins, Evaluator and Breeder

A new room (arena, reproduction room, obstacle course) = one `Room` implementation + one registry entry; a new evaluation scheme (tournament) = one `Evaluator`.

## Findings (2026-11-02 review)
- `TrainingScenarioPlugin` (`scenarios.rs:130`) covers chamber size, epoch ticks, spawn distance, metabolic scale, `initialize(&mut TrainerSandbox)`, `visible_foods`. It is not a real plugin: `sb.room` is the concrete `LevelScenario` (`trainer_engine.rs:48`), `Scenario` a closed enum (`scenarios.rs:15`); a new room edits the enum, `Scenario::plugin`/`fixed_level`, `TrainerSim::trial_room` 730, runner `parse_room` 251, bindings, UI.
- A room cannot express: >1 creature per chamber (`TrainerSandbox.agent` single), obstacles/currents (trainer passes `&[]`), moving prey, own eat/bite/energy rules, own fitness/termination (global `calculate_fitness`), births during a trial, opponent pairing, room telemetry.
- Duplicated evaluation loops: `TrainerSim::evaluate_population` 754, `probes::evaluate_genome` 1309. `TrainerSim` mixes orchestration, breeding (`breed` 988), 1/5 rule, HoF, lineage stats. `RunConfig`/`ReportMeta` mirror `TrainerParams` field by field; `parse_room` duplicates `Scenario::from_mode`.
- Bugs (fix here, behaviour change only where stated in TCK-202): `assign_creature` 791 does not reset `sight_distance`, `approach_min_distance`, `cumulative_rotation`, `coverage_mask`, food; sets `epoch_ticks = 0` mid-trial; keeps `acc_fitness` and stale `slot_lineage`/`slot_parents` (skews `child_success_share` and the 1/5 rule).
- `probes.rs`, `trainer_report.rs` and rusqlite helpers are compiled into the game library; `legacy_mutate` 418 and the letter-soup populations are historical only.

## Scope
1. `Room` trait (object-safe): `id`, `layout(&mut World, &mut SimRng)`, `episode_ticks`, `is_done(&World)`, `score(&World, EntityId) -> f32`, `telemetry(&World)`; registry keyed by string id; `LevelRoom` = today's `LevelScenario` values. `Scenario` keeps only the UI mode; runner and bindings read room ids from the registry.
2. Sandboxes hold a `World` with N entities (arena-ready); today's rooms use N = 1.
3. `Evaluator` trait with `SoloTrials` (today's trainer) used by `end_epoch`, `evaluate_population` and the probes.
4. `Breeder` / `SelectionStrategy`: slot plan, 1/5 rule, HoF, lineage stats move out of `TrainerSim`, which only orchestrates.
5. `RunConfig` derives from `TrainerParams` (no mirror); `parse_room` via the registry.
6. `assign_creature` resets the sandbox like `reset_trial` and clears accumulators/lineage of that slot.
7. Diagnostics (`probes.rs`, report helpers) behind the `runner` feature or a `diagnostics` module; `pearson`/`Distribution` to `shared/stats.rs`; remove `legacy_mutate` and letter-soup populations.
8. New RNG streams (pairing, births) get named constants.

## Gate
Trainer references and golden master unchanged (the assign fix only affects manual assignment, not the references).

## Acceptance
- Adding a room touches only its module + registry entry (checked in TCK-204).
- Contract `wiki/contracts/room.md`, `wiki/contracts/evaluator.md`, guide `wiki/guides/add-room-or-mode.md`; `wiki/simulation/trainer.md` reduced to the current model.
