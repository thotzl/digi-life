---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, README.md, docs/revisit/05-persistence.md, tickets/closed/TCK-137-persistence-appdata-shared-connection.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md]
---

# Persistence (SQLite)

## Location and connections

- File `pixel_life_local.db` in the Tauri app data dir (Windows: `%APPDATA%\com.pixeldna.life\`), resolved once in `main.rs` setup via `app_data_dir()` (`database::set_db_path` / `db_path()`) (TCK-137, RV-B03).
- A legacy `target/pixel_life_local.db` is copied once on first start and never overwritten (`migrate_legacy_db`).
- `init_db` runs once at setup: model-version check (below), schema creation, WAL journal, 5 s busy timeout.
- Commands share one mutex-guarded connection (`DbState`); the engine thread holds its own connection to the same file.
- The headless runner `src/bin/trainer_runner.rs` runs in memory and opens no DB by default; `run --persist --db <path>` writes elites to the given file (never the app DB implicitly); `probe --db <path>` reads HoF genomes read-only (TCK-173, TCK-183).
- Write failures are logged via `LogDbError::log_db_err`; unreadable rows in API commands are logged (TCK-137, RV-D05).

## Schema (`src/database.rs`)

| Table | Content |
| :--- | :--- |
| `meta` | key/value; `model_version` = `MODEL_VERSION` the stored model rows were made with (TCK-183) |
| `sessions` | only to satisfy a foreign key (`desktop-session-001` hardcoded) |
| `species_records` | one row per distinct genome: `latin_name` (UNIQUE), `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` |
| `simulation_history` | Ocean time series every 10 s: `generation`, `population_count`, `average_fitness`/`max_fitness` (actually avg/max energy); no reader in the app |
| `simulation_state` | Ocean snapshot, key `current_state`, JSON of creatures, food, counters; upsert every 10 s; loaded on startup |
| `trainer_genomes` | trainer elites per `run_id`/`generation`: `fitness`, `active` (current parents), `genome`, `methylations`, `synapse_weights` (JSON); bounded per run (below) |
| `creature_catalogue` | user-saved specimens: `id`, `name`, `genome`, `source`, `fitness`, `carnivory`, `methylations`, `synapse_weights`, `created_at`; index on `genome` (TCK-128) |

- `RESET_EVOLUTION` deletes `species_records`, `simulation_history`, `simulation_state` (incl. `progenitor_genome`) and must clear `newly_spawned_creatures` ([[wiki/architecture/invariants]]).
- **Trainer genome IO** (`database.rs`, the IO side of `TrainerSim`): `save_trainer_elites` marks the run's rows inactive, inserts the generation's elites as active and calls `prune_trainer_genomes`; `load_trainer_genomes(run, active_only, limit)` returns the active parents or the hall of fame, best first (TCK-173).
- **Retention** (TCK-183): `prune_trainer_genomes` keeps the best `trainerGenomeRetentionPerRun` (200) rows per run by fitness plus all active rows and deletes the rest of that run; it runs after every saved generation (engine and runner). The in-memory hall of fame of `TrainerSim` is bounded to the same N. Test `test_trainer_retention_keeps_best_rows_and_active_parents`.
- **Reset on model change** (TCK-183): `MODEL_VERSION` (`src/database.rs`, currently 5; 2 = Brain v2; 3 = Oja learning with fixed reflexes (TCK-152): stored phenotypes lack the `plastic` flag and weights learned under the old rule, incl. drifted reflexes, are invalid; 4 = Energy v2 (TCK-151, TCK-178): stored phenotypes still carry `basalMetabolicRate`, and trainer fitness and weights were selected and learned without an energy budget; 5 = fitness in catch units (TCK-175): stored trainer and catalogue fitness of the old formula, ~10²–10³, would outrank every new value in the hall of fame, retention and run list, which all sort by raw fitness) is bumped by every change that makes stored genomes' phenotypes or synapse weights invalid. `init_db` compares it with `meta.model_version`; a different stored version, or none in a database that already has model tables, drops `trainer_genomes`, `creature_catalogue`, `simulation_state`, `simulation_history` and `species_records` (one transaction), recreates them empty and logs `[DB] Model version <old> -> <new>: model tables reset` once. `sessions` and `meta` are kept; there is no migration. Applies to every `init_db` (app setup, engine thread, runner `--persist`); `probe --db` opens read-only and never resets. Tests `test_fresh_db_stores_model_version_and_reset_drops_model_tables` (`:memory:`) and `test_file_db_model_version_reset_on_reopen`.
- Stored `synapse_weights` are in the synapse order of the phenotype that produced them; consumers rebuild the phenotype from the genome and match by synapse key or fall back to the genome weights on a length mismatch ([[wiki/biology/metabolism-and-reproduction]]).
- Reading while the app runs: open read-only (`sqlite3 -readonly`), see [[wiki/simulation/analytics-and-balancing]].

## Known limitations

- **Migrations** are `CREATE TABLE IF NOT EXISTS` plus a manual column check; `ALTER TABLE` errors are discarded (RV-D02). `MODEL_VERSION` versions the simulation model (reset, not migration), not the table schema.
- **No retention for Ocean tables:** `species_records` and `simulation_history` grow without bound (RV-D03); `trainer_genomes` is bounded per run (TCK-183), but the number of runs is not.
- **Timestamps:** `species_records` times and `trainer_genomes.created_at` in seconds (variable named `now_millis`); `simulation_state.updated_at` ms (engine) and s (`apply_champion`); catalogue/sessions SQLite text (RV-D04).
- **Silent JSON fallbacks:** corrupt methylation/synapse JSON is replaced by defaults (`unwrap_or_default`) (RV-D05).
- **Dead state:** `progenitor_genome` is written by `apply_champion` and never read (RV-D06).

## Troubleshooting

- Schema mismatch or startup freeze: close the app, delete `pixel_life_local.db` (+ `-wal`/`-shm`) from the app data dir; the schema is recreated on start. Deletes all species, trainer runs and catalogue entries.

## Key Takeaways

- One DB file in AppData, WAL, schema init once, shared command connection plus an engine connection.
- Model-version reset instead of migrations; retention only for `trainer_genomes` (best 200 per run).
- Ocean state is auto-saved every 10 s and restored on start.

Sources: `.agents/skills/server-simulation/SKILL.md`, `docs/revisit/05-persistence.md`, TCK-128, TCK-137, TCK-173, TCK-175, TCK-179, TCK-183, `README.md`.
