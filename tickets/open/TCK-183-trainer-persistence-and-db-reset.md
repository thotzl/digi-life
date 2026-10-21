---
id: TCK-183
title: Trainer Persistence, Retention and DB Reset on Model Change
status: open
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
