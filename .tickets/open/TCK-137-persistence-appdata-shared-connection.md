---
id: TCK-137
title: Persistence in AppData, Shared Connection, Error Logging
status: open
assigned: Claude Code
created_at: 2026-10-19
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
