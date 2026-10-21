---
id: TCK-159
title: DB Migration Versioning, Retention and Timestamps
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-D02, RV-D03, RV-D04, RV-D06
---

# TCK-159: DB Migration Versioning, Retention and Timestamps

> Pool ticket: requires a user decision before implementation. Findings (RV-D02, RV-D03, RV-D04, RV-D06) below.

## Description
Ad-hoc migrations without `user_version`; unbounded growth of trainer_genomes/species_records; mixed timestamp units; dead `progenitor_genome`/`sessions` state.

## Proposed Scope
Versioned migrations, retention policy, unified unix-ms timestamps.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-D02 · P1** — migrations are `CREATE TABLE IF NOT EXISTS` plus a manual `PRAGMA table_info` check for two columns; `ALTER TABLE` errors discarded (`let _ =`); no schema version. Target: `PRAGMA user_version` + numbered migrations, once at start in one transaction (skill `core-database-safety`).
- **RV-D03 · P1** — unbounded growth: `species_records` one row per genome (thousands per hour once RV-B01 is fixed); `trainer_genomes` `elite_count` rows per generation incl. JSON weights, never pruned; `simulation_history` every 10 s, no reader; `simulation_state` full JSON every 10 s in the engine thread. Target: retention (HoF top-N per run), use `simulation_history` (charts) or drop it.
- **RV-D04 · P2** — timestamps: `species_records.birth_time/extinction_time` and `trainer_genomes.created_at` seconds; `simulation_state.updated_at` ms (engine) and s (`apply_champion`); `creature_catalogue.created_at`, `sessions.created_at` SQLite text. Target: Unix ms INTEGER, one helper.
- **RV-D05 (rest) · P2** — `serde_json::from_str(...).unwrap_or_default()` for methylations/synapses silently loses learned weights on corrupt data.
- **RV-D06 · P3** — `apply_champion` writes `progenitor_genome`, never read; `sessions` exists only for a FK (`desktop-session-001` hardcoded); `RESET_EVOLUTION` deletes all of `simulation_state` incl. `progenitor_genome`.
