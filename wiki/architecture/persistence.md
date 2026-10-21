---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, README.md, docs/revisit/05-persistence.md, tickets/closed/TCK-137-persistence-appdata-shared-connection.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md]
---

# Persistence (SQLite)

## Location and connections

- File `pixel_life_local.db` in the Tauri app data dir (Windows: `%APPDATA%\com.pixeldna.life\`), resolved once in `main.rs` setup via `app_data_dir()` (`database::set_db_path` / `db_path()`) (TCK-137, RV-B03).
- A legacy `target/pixel_life_local.db` is copied once on first start and never overwritten (`migrate_legacy_db`).
- `init_db` runs once at setup: schema creation, WAL journal, 5 s busy timeout.
- Commands share one mutex-guarded connection (`DbState`); the engine thread holds its own connection to the same file.
- `src/bin/cli_trainer.rs` still uses the legacy path via the `db_path()` default.
- Write failures are logged via `LogDbError::log_db_err`; unreadable rows in API commands are logged (TCK-137, RV-D05).

## Schema (`src/database.rs`)

| Table | Content |
| :--- | :--- |
| `sessions` | only to satisfy a foreign key (`desktop-session-001` hardcoded) |
| `species_records` | one row per distinct genome: `latin_name` (UNIQUE), `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` |
| `simulation_history` | Ocean time series every 10 s: `generation`, `population_count`, `average_fitness`/`max_fitness` (actually avg/max energy); no reader in the app |
| `simulation_state` | Ocean snapshot, key `current_state`, JSON of creatures, food, counters; upsert every 10 s; loaded on startup |
| `trainer_genomes` | trainer population per `run_id`/`generation`: `fitness`, `active`, `genome`, `methylations`, `synapse_weights` (JSON) |
| `creature_catalogue` | user-saved specimens: `id`, `name`, `genome`, `source`, `fitness`, `carnivory`, `methylations`, `synapse_weights`, `created_at`; index on `genome` (TCK-128) |

- `RESET_EVOLUTION` deletes `species_records`, `simulation_history`, `simulation_state` (incl. `progenitor_genome`) and must clear `newly_spawned_creatures` ([[wiki/architecture/invariants]]).
- Reading while the app runs: open read-only (`sqlite3 -readonly`), see [[wiki/simulation/analytics-and-balancing]].

## Known limitations

- **Migrations** are `CREATE TABLE IF NOT EXISTS` plus a manual column check; `ALTER TABLE` errors are discarded; there is no schema version (RV-D02).
- **No retention:** `species_records`, `trainer_genomes` and `simulation_history` grow without bound (RV-D03).
- **Timestamps:** `species_records` times and `trainer_genomes.created_at` in seconds (variable named `now_millis`); `simulation_state.updated_at` ms (engine) and s (`apply_champion`); catalogue/sessions SQLite text (RV-D04).
- **Silent JSON fallbacks:** corrupt methylation/synapse JSON is replaced by defaults (`unwrap_or_default`) (RV-D05).
- **Dead state:** `progenitor_genome` is written by `apply_champion` and never read (RV-D06).

## Troubleshooting

- Schema mismatch or startup freeze: close the app, delete `pixel_life_local.db` (+ `-wal`/`-shm`) from the app data dir; the schema is recreated on start. Deletes all species, trainer runs and catalogue entries.

## Key Takeaways

- One DB file in AppData, WAL, schema init once, shared command connection plus an engine connection.
- No schema versioning and no retention.
- Ocean state is auto-saved every 10 s and restored on start.

Sources: `.agents/skills/server-simulation/SKILL.md`, `docs/revisit/05-persistence.md`, TCK-128, TCK-137, `README.md`.
