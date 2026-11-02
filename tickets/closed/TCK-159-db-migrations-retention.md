---
id: TCK-159
title: DB Migration Versioning, Retention and Timestamps
status: closed
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-D02, RV-D03, RV-D04, RV-D06
---

# TCK-159: DB Migration Versioning, Retention and Timestamps

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R1 (after TCK-198; parallel with TCK-157). Scope from the 2026-11-02 review:
- `PRAGMA user_version` with an ordered migration list applied in one transaction; modules contribute their migrations and declare model-scoped tables (today `CREATE TABLE IF NOT EXISTS` + two hand-written column checks with inconsistent error handling, database.rs:174-179 vs 204-213).
- Repository traits on plain DTOs (`SpeciesRepo`, `CatalogueRepo`, `TrainerRepo`, `OceanSnapshotRepo`), rusqlite and in-memory implementations; `database.rs` no longer imports `TrainerSim` (database.rs:3, 332).
- SQL only in the repository module (today engine.rs 24 sites, api.rs 14, probes.rs 1; catalogue query duplicated at engine.rs:632/1340, api.rs:317/356).
- Retention: runs, `species_records`, `simulation_history` (written every 10 s, never read by the app) bounded or removed; `trainer_genomes` id `run-gen-idx-secs` collides within one second (database.rs:297); snapshot restore errors logged instead of dropped (engine.rs:200-215); the `sessions` row only exists for a foreign key (engine.rs:193).
- RV-D06 (`apply_champion`, `progenitor_genome`) closed by TCK-157.
- Contract `wiki/contracts/persistence-repo.md`, guide `add-table.md`.


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

## Verification (2026-11-02, v1.12.31)
- Module map: `src/database.rs` removed; `src/persistence/` is the only place with SQL/`rusqlite`: `mod.rs` (paths, `Db`, `init_db`, `open_read_only`, `DbState`, generic `LogDbError`), `schema.rs` (`MIGRATIONS`, `MODEL_VERSION` 6, model reset with backup, table registry), `repos.rs` (`SpeciesRepo`, `CatalogueRepo`, `TrainerRepo`, `OceanSnapshotRepo`), `dto.rs` (plain DTOs, `RepoError`; no `biology` import), per-group `species.rs` / `catalogue.rs` / `trainer.rs` / `ocean.rs`, `memory.rs` (`MemoryDb`), `tests.rs` (migration, model reset, contract suite over both stores). `src/biology/trainer_store.rs` maps `GenomeRecord` / `TrainerRunState` onto `TrainerRepo`.
- Removed SQL sites: engine.rs 24, api.rs 14, probes.rs 1; remaining outside `src/persistence/`: 0 (grep). Command names, signatures and payloads unchanged.
- Migrations: `PRAGMA user_version` with ordered steps in one transaction; order reset decision -> backup -> migrations -> model-table drop -> latest `CREATE ... IF NOT EXISTS`. Migration 1 = frozen v1.12.30 baseline (former column checks, errors propagate), migration 2 drops `simulation_history` and `sessions` (no reader). Tests `test_v1_12_30_database_migrates_without_data_loss`, `test_old_model_and_schema_version_together`, `test_migrated_schema_equals_latest_tables`.
- Retention: `species_records` keeps all alive species plus the 2000 most recently extinct (`ocean.speciesRecordRetention`, pruned on the 10 s snapshot timer); trainer runs stay user-managed (200 rows per run).
- Bug, reproduction first: trainer genome ids `run-gen-idx-secs` collided within one second; `test_repro_trainer_genome_id_collision_within_one_second` failed on the old code with `left: 1, right: 2`; ids now end in Unix ms plus a process counter (`test_trainer_genome_ids_never_collide_within_one_second`); generation save and run delete are transactions. Snapshot restore errors are logged and the Ocean starts fresh.
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 197 passed, 1 ignored; golden master unchanged; standard, curriculum, rehearsal references match; `npm test` 13 passed; build ok; doc links OK (83 files); no bindings drift. Runner `run --persist --db` on a new file migrates 1 and 2; `probe heredity --db` reads read-only.
- UI smoke (dev, real dev DB copied and restored): migrations 1 and 2 applied, 15 Ocean creatures and trainer run at generation 155 restored, generation completed, reset to 1, 0 console errors.
- Docs: `wiki/contracts/persistence-repo.md`, `wiki/guides/add-table.md`, `wiki/architecture/persistence.md` rewritten; invariants #9, system-overview, repository-map, testing, configuration, ocean, trainer, analytics, master index; skills `server-simulation` (no SQL outside `src/persistence/`), `testing-strategies` (`MemoryDb`).
- Left open: timestamp units (RV-D04) now set in one place by the repositories; Ocean snapshot still a JSON blob (typed DTO in TCK-156).
