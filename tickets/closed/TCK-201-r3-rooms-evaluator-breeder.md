---
id: TCK-201
title: "R3 Rooms as Plugins, Evaluator and Breeder, Multi-Entity Chambers"
status: closed
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

## Verification (2026-11-03, v1.12.36)
- New `src/trainer/` replaces `biology/{scenarios,trainer_engine,trainer_sim}.rs`: `sim.rs` (`TrainerSim` orchestrates only; `TrainerParams`, `TrainerEvent`, `TrainerRunState`, `run_seed`, `PopulationEval`), `room/` (object-safe `Room` trait: `id`, `chamber_size`, `episode_ticks`, `entities`, `world_settings`, `obstacles`, `layout`, `is_done`, `score`, `level`, `telemetry`; registry `ROOMS` with `standard`, `exploration`; `LevelRoom` + `calculate_fitness`), `chamber.rs` (`Chamber` with N entities on `sim::step_creature`), `sandbox.rs` (`TrainerSandbox { room: Arc<dyn Room>, chamber }`), `evaluator.rs` (`Evaluator`, `SoloTrials` - one trial loop for `end_epoch`, `evaluate_population` and the probes; `set_evaluator`), `breeder.rs` (`Breeder` struct: parents, hall of fame, slot plan, selection, lineage, 1/5 input), `curriculum.rs`, `genomes.rs`, `mode.rs` (`Scenario` UI mode, `TrainingMode`), `stats.rs`, `streams.rs` (all named RNG streams incl. reserved `PAIRING`, `BIRTHS`). `src/diagnostics/{probes,report}.rs` only for tests and the `runner` feature; `src/shared/stats.rs` (`quantile`, `Distribution`, `pearson`).
- A new room touches its module under `src/trainer/room/` and one `ROOMS` entry; runner `--scenario/--schedule/--eval-final <id>` and `TrainerSim::set_mode(TrainingMode::Room(id))` then work without enum, runner or binding changes (a UI mode still needs a `Scenario` variant).
- `assign_creature` fix, reproduction first: `test_assign_creature_starts_the_slot_fresh` failed on the old code with `left (Some(100.0), 50.0, 0.24811779, 34628173824, false, 1, Some(0), false) != right (None, 3.4028235e38, 0.0, 0, true, 0, None, true)` (stale approach/rotation/coverage, tick out of sync, previous trial fitness, parent and lineage kept); the assigned creature now restarts the current trial on the shared layout and the slot's trial records and lineage are cleared.
- Legacy probes removed: the letter-soup genome set (heredity group, speed/equipment/steering/topology/metabolism set, 6 wall cells), heredity kind `legacy` with `legacy_mutate` and `legacy_op_shares`; `heredity.mixed_op_shares` changes (pooled the removed group). All other probe outputs identical (seed `tck201`, `--samples 60`, hebbian 6); hebbian byte-identical.
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 259 passed, 1 ignored; golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum, rehearsal references match; `npm test` 41 passed; build ok; doc links OK (89 files); bindings: doc comments only. UI smoke OK (0 console errors).
- Docs: `wiki/contracts/room.md`, `wiki/contracts/evaluator.md`, `wiki/guides/add-room-or-mode.md` (mode part filled by TCK-156), `wiki/history/trainer-measurements-p2-p8.md`; `wiki/simulation/trainer.md` reduced to the current model; analytics, repository-map, testing, creature-system, add-dynamic, config-section, configuration, simulation-engine, glossary, master index, skill `simulation-analytics`.
- Temporary compatibility shims `biology::scenarios` / `biology::trainer_sim` in `lib.rs` for the engine; removed with TCK-156.
