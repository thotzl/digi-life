---
id: TCK-154
title: Trainer Scenario Parameters into Plugins
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S08, RV-S09
---

# TCK-154: Trainer Scenario Parameters into Plugins

> Pool ticket: requires a user decision before implementation. Findings (RV-S08, RV-S09) below.

## Description
Exploration spawn distances differ (650/1200/550); re-trial logic duplicates `initialize`; rejection sampling has no iteration limit; Gen-1 ignores `base_dna`.

## Proposed Scope
Move chamber size, epoch length and spawn distances into the scenario plugin; decide the intended distances.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S08 · P2 · static** — exploration spawn distances: init `min_dist = 650` (`scenarios.rs`), re-trial `1200` (`engine.rs`), respawn after eating `550` (`trainer_engine.rs`). Trial 1 is systematically easier than trials 2/3. Rejection-sampling loops (`while !valid_spawn`) have no iteration limit → endless loop for smaller chambers/larger distances. Target: values in the scenario plugin (`spawn_min_distance()`); re-trial calls `plugin.initialize` instead of the duplicated logic in `engine.rs`.
- **RV-S09 · P2** — `rebuild_sandbox_grid` passes `base_dna`, but `init_rust_sandbox` replaces it with `generate_random_genome(256)` for `origin == "random"`. Random genomes rarely contain `NEU`/`SY`/`EYE` → generation 1 mostly starts blind with the fallback brain. CHANGELOG says "pristine wildtypes are born blind" → probably intended; then remove the `base_dna` parameter.
