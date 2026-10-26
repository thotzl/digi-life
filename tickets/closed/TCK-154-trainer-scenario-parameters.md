---
id: TCK-154
title: Trainer Scenario Parameters into Plugins
status: closed
priority: high
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S08, RV-S09
epic_phase: Training Track
---

# TCK-154: Trainer Scenario Parameters into Plugins

> Decision 2026-10-21: implemented as part of TCK-175 (training track, epic TCK-171). The RV-S09 question (blind generation 1) is decided there. Findings (RV-S08, RV-S09) below.

## Description
Exploration spawn distances differ (650/1200/550); re-trial logic duplicates `initialize`; rejection sampling has no iteration limit; Gen-1 ignores `base_dna`.

## Proposed Scope
Move chamber size, epoch length and spawn distances into the scenario plugin; decide the intended distances.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S08 · P2 · static** — exploration spawn distances: init `min_dist = 650` (`scenarios.rs`), re-trial `1200` (`engine.rs`), respawn after eating `550` (`trainer_engine.rs`). Trial 1 is systematically easier than trials 2/3. Rejection-sampling loops (`while !valid_spawn`) have no iteration limit → endless loop for smaller chambers/larger distances. Target: values in the scenario plugin (`spawn_min_distance()`); re-trial calls `plugin.initialize` instead of the duplicated logic in `engine.rs`.
- **RV-S09 · P2** — `rebuild_sandbox_grid` passes `base_dna`, but `init_rust_sandbox` replaces it with `generate_random_genome(256)` for `origin == "random"`. Random genomes rarely contain `NEU`/`SY`/`EYE` → generation 1 mostly starts blind with the fallback brain. CHANGELOG says "pristine wildtypes are born blind" → probably intended; then remove the `base_dna` parameter.

## Verification (P7, 2026-10-26)
- RV-S08: one `spawn_min_distance()` per scenario plugin, used for trial 0, re-trials and respawns (`src/biology/scenarios.rs`); values in config `trainerSpawnMinDistance` (`standard` 200, `exploration` 500; reasoning in the doc comment of `TrainerSpawnMinDistance`, `src/shared/types.rs`). Re-trials call `plugin.initialize` (`TrainerSandbox::reset_trial`), the duplicated engine logic is gone. Rejection sampling is bounded (`SPAWN_MAX_TRIES` 1000, returns the farthest candidate; test for an unreachable distance).
- Large-room distance from a sweep over 2000 structured genomes (food-sensing range p10/p50/p90/max = 256/575/926/990 px): food within sensor range at spawn 46 % (300 px), 35 % (500 px, chosen; 13 % also in the view cone), 23 % (650 px, old trial 0), 10 % (800 px), 0 % (1200 px, old trials 1+). Small room 200 px: 89 %.
- RV-S09: decided in TCK-175 (minimal criterion on); the trainer has no `base_dna` parameter any more.
- Tests: spawn distance identical for trial 0, re-trials and respawns in both rooms.
