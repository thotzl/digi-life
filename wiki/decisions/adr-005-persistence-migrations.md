---
sources: [tickets/closed/TCK-159-db-migrations-retention.md, tickets/closed/TCK-195-windows-release-build.md, tickets/closed/TCK-202-r5-model-corrections.md, src/persistence/schema.rs, src/persistence/repos.rs, wiki/contracts/persistence-repo.md]
---

# ADR-005: Persistence Module with Versioned Migrations, Repositories and Model-Reset Backup

- Status: accepted
- Date: 2026-11-03
- Tickets: TCK-159, TCK-195, TCK-202

## Context

SQL was spread over the engine (24 sites), the Tauri commands (14) and a probe (1). The schema evolved by `CREATE TABLE IF NOT EXISTS` plus hand-written column checks with inconsistent error handling; the persistence code imported the trainer domain. Since TCK-183 a changed `MODEL_VERSION` drops the model tables, which silently deletes trained creatures. With an installable alpha (TCK-195), users keep save data across updates, so schema changes and model changes must be safe.

## Decision

1. **One persistence module.** `src/persistence/` is the only place with SQL and `rusqlite` (invariant 12). It exposes repository traits on plain DTOs (`SpeciesRepo`, `CatalogueRepo`, `TrainerRepo`, `OceanSnapshotRepo`) with a SQLite implementation (`Db`) and an in-memory one (`MemoryDb`) for tests; it does not import domain types.
2. **Versioned schema migrations.** `PRAGMA user_version` with an ordered, frozen list of migrations applied in one transaction; migration 1 is the baseline of v1.12.30. Each module declares its tables in their latest shape; a test checks that the migration chain yields exactly those tables, and another migrates a filled v1.12.30 database without data loss.
3. **Schema version and model version are separate.** `user_version` versions the table schema (migrated); `MODEL_VERSION` versions the simulation model (stored phenotypes and weights become invalid, model tables are reset, no migration).
4. **Backup before a model reset.** Before dropping model tables the whole database is copied with `VACUUM INTO` to `<db>.model-v<old>.bak.db` (consistent with WAL), at most one backup per old version, never overwritten; a failed backup is logged and does not block the start.
5. **Retention.** Unread tables were dropped (`simulation_history`, `sessions`); extinct species are capped (`ocean.speciesRecordRetention`); trainer runs stay user-managed.

## Consequences

- A new table is one module entry, a migration and a repository method (`wiki/guides/add-table.md`); engine and commands are testable against `MemoryDb`.
- Updating the installed app migrates the schema in place; a model change resets trained creatures but leaves a readable backup (verified on the real dev database when model version 7 arrived).
- Old databases are read once and migrated forward; a database from a newer build is left untouched and logged.
