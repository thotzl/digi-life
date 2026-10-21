---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, README.md, docs/revisit/05-persistence.md, .tickets/closed/TCK-137-persistence-appdata-shared-connection.md, .tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md]
---

# Persistence (SQLite)

## Location and connections

- File `pixel_life_local.db` in the Tauri app data dir (Windows: `%APPDATA%\com.pixeldna.life\`), resolved once in `main.rs` setup via `app_data_dir()` (`database::set_db_path` / `db_path()`) (TCK-137, RV-B03).
- A legacy `target/pixel_life_local.db` is copied once on first start and never overwritten (`migrate_legacy_db`).
- `init_db` runs once at setup: schema creation, WAL journal, 5 s busy timeout.
- Commands share one mutex-guarded connection (`DbState`); the engine thread holds its own connection to the same file.
- `src/bin/cli_trainer.rs` still uses the legacy path via `db_path()` default.
- Write failures are logged via `LogDbError::log_db_err`; unreadable rows in API commands are logged (TCK-137).

## Schema (`src/database.rs`)

| Table | Content |
| :--- | :--- |
| `sessions` | only to satisfy a foreign key (`desktop-session-001` hardcoded) |
| `species_records` | one row per distinct genome: `latin_name` (UNIQUE), `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` |
| `simulation_history` | Ocean time series every 10 s: `generation`, `population_count`, `average_fitness`/`max_fitness` (actually avg/max energy) |
| `simulation_state` | Ocean snapshot, key `current_state`, JSON of creatures, food, counters; upsert every 10 s; loaded on startup |
| `trainer_genomes` | trainer population per `run_id`/`generation`: `fitness`, `active`, `genome`, `methylations`, `synapse_weights` (JSON) |
| `creature_catalogue` | user-saved specimens: `id`, `name`, `genome`, `source`, `fitness`, `carnivory`, `methylations`, `synapse_weights`, `created_at`; index on `genome` (TCK-128) |

- `RESET_EVOLUTION` deletes `species_records`, `simulation_history`, `simulation_state` (incl. `progenitor_genome`) and must clear `newly_spawned_creatures` ([[architecture/invariants]]).
- Reading while the app runs: open read-only (`sqlite3 -readonly`), see [[simulation/analytics-and-balancing]].

## Open findings (pool TCK-159)

- **RV-D02** — migrations are ad hoc (`CREATE TABLE IF NOT EXISTS` + manual `PRAGMA table_info` for two columns, `ALTER TABLE` errors discarded); no schema version. Target: `PRAGMA user_version` + numbered migrations in one transaction.
- **RV-D03** — unbounded growth: `species_records` (one row per genome, explodes once offspring live, RV-B01), `trainer_genomes` (elites per generation with JSON weights, never pruned), `simulation_history` (no reader). Target: retention (HoF top-N per run), use or drop history.
- **RV-D04** — timestamp units mixed: `species_records` times and `trainer_genomes.created_at` in seconds (variable named `now_millis`), `simulation_state.updated_at` ms (engine) and s (`apply_champion`), catalogue/sessions SQLite text. Target: Unix ms INTEGER, one helper.
- **RV-D05 (rest)** — `serde_json::from_str(...).unwrap_or_default()` for methylations/synapses silently loses learned weights on corrupt data.
- **RV-D06** — dead state: `progenitor_genome` written by `apply_champion`, never read; `sessions` only for the FK.

## Troubleshooting

- Schema mismatch or startup freeze: close the app, delete `pixel_life_local.db` (+ `-wal`/`-shm`) from the app data dir; the schema is recreated on start. Deletes all species, trainer runs and catalogue entries.

## Key Takeaways

- One DB file in AppData, WAL, schema init once, shared command connection plus an engine connection.
- No migration versioning and no retention yet (TCK-159).
- Ocean state is auto-saved every 10 s and restored on start.

Sources: `.agents/skills/server-simulation/SKILL.md`, `docs/revisit/05-persistence.md`, TCK-128, TCK-137, `README.md`.
