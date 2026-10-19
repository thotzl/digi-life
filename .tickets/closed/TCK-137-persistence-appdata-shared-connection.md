---
id: TCK-137
title: Persistence in AppData, Shared Connection, Error Logging
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
revisit: RV-B03, RV-A04, RV-D05
---

# TCK-137: Persistence in AppData, Shared Connection, Error Logging

## Description
- The DB lives at the CWD-relative `target/pixel_life_local.db` (wiped by `cargo clean`, likely unwritable in installed releases).
- Every Tauri command re-opens the DB and re-runs schema creation/migration; no WAL.
- DB errors are silently discarded.

## Scope
- Resolve the DB path via Tauri `app_data_dir()` once at setup; one-time copy of a legacy `target/` DB if present.
- Run schema init once; commands use a shared, mutex-guarded connection; enable WAL.
- Log DB errors instead of discarding them (no other behaviour change).

## Acceptance
- `cargo test` green incl. tests for the path migration helper. App behaviour unchanged apart from the DB location.

## Verification
- Tests `database::tests::test_file_db_uses_wal_journal` and `database::tests::test_legacy_db_is_copied_once_and_never_overwrites` (`src/database.rs`) failed to compile before (no migration helper, no WAL).
- `src/database.rs`: `DB_PATH` constant replaced by `set_db_path`/`db_path()` (default = legacy `target/pixel_life_local.db` for the CLI trainer); `migrate_legacy_db` copies a legacy DB once and never overwrites; `init_db` enables WAL and a 5 s busy timeout; `DbState` shared connection; `LogDbError` helper.
- `src/main.rs`: resolves `app_data_dir()/pixel_life_local.db` at setup, runs the legacy migration, initializes the schema once and manages `DbState`.
- `src/server/api.rs`: 13 DB commands use the shared `State<DbState>` connection instead of re-opening and re-migrating per call; unreadable rows are logged instead of silently skipped.
- `src/server/engine.rs`: the simulation thread opens `db_path()`; all 20 discarded `conn.execute` results now log failures via `log_db_err`.
- `src/bin/cli_trainer.rs`: uses `db_path()` (still the legacy location).
- `cargo test`: 39 passed, 0 failed; `cargo build`: success (only the 3 pre-existing `cli_trainer` warnings).
- Runtime check of the app data location: done in the final `tauri dev` smoke test (see docs/revisit).
