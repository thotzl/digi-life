---
id: TCK-183
title: Trainer Persistence, Retention and DB Reset on Model Change
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173
---

# TCK-183: Trainer Persistence, Retention and DB Reset on Model Change

Part of epic TCK-171. Parts in phase P2 (runner, retention) and P3 (reset rule).

## Description
- The headless runner (TCK-173) with population 300 and many generations would write elites incl. synapse weights to `trainer_genomes` every generation (`src/server/engine.rs`, breeding step); the table is never pruned (RV-D03).
- Brain v2 (P3: TCK-179, TCK-182, TCK-144) changes what `parse_genome` builds from a genome; stored synapse weights (`trainer_genomes.synapse_weights`, catalogue, HoF) are indexed by synapse position and no longer match. Backward compatibility is not required (user decision 2026-10-21): the DB may be emptied.

## Scope
- P2: headless runner runs in memory by default; DB writes only on request (e.g. `--persist`).
- P2: retention for `trainer_genomes`: keep the best N per run (N from config), prune the rest.
- P3: rule "reset on model change": a schema/model version constant; if the stored version differs, the trainer tables, catalogue and HoF are recreated empty at start. No data migration.
- Wiki `wiki/architecture/persistence.md` describes the rule.

## Acceptance
- Runner without `--persist` leaves the DB untouched.
- After a model version bump the app starts with empty trainer/catalogue tables and no errors.
- `trainer_genomes` stays bounded over a long UI run.

## Progress P2 (v1.12.16)
- Runner in memory by default; `--persist` requires `--db <path>`; a run without `--persist` opens/creates no DB file (verified).
- Retention: `prune_trainer_genomes` keeps the best `trainerGenomeRetentionPerRun` (200) rows per run plus all active rows; called after every generation save (engine and runner); test on `:memory:`; in-memory HoF bounded to the same N.
- Remaining for P3: model-version reset rule.

## Verification (P3, v1.12.17)
- `MODEL_VERSION = 2` and a `meta` table in `src/database.rs`; at `init_db` a different or missing stored version (with model data present) drops and recreates `trainer_genomes`, `creature_catalogue`, `simulation_state`, `simulation_history`, `species_records` in one transaction, keeps `sessions`, logs once. Tests `test_fresh_db_stores_model_version_and_reset_drops_model_tables`, `test_file_db_model_version_reset_on_reopen`.
- Runner check: `--persist --db` on a version-1 DB logged the reset, restarted the trainer at generation 1 and wrote fresh rows; the Ocean starts fresh from the existing seed path.
- P2 parts (runner in memory, retention) see `## Progress P2`.
