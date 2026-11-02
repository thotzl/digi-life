---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, README.md, tickets/closed/TCK-137-persistence-appdata-shared-connection.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/open/TCK-159-db-migrations-retention.md, src/persistence/mod.rs]
---

# Persistence (SQLite)

## Location and connections

- File `pixel_life_local.db` in the Tauri app data dir, resolved once in `main.rs` setup via `app_data_dir()` (`persistence::set_db_path` / `db_path()`) (TCK-137, RV-B03). The folder follows the identifier (TCK-195): installed app `%APPDATA%\de.pixlich.broodbattle\` (alpha identifier), dev app `%APPDATA%\de.pixlich.broodbattle.dev\` (`tauri.dev.conf.json`); macOS `~/Library/Application Support/<identifier>/`, Linux `~/.local/share/<identifier>/`. Until v1.12.26 both used `%APPDATA%\com.pixeldna.life\`; that DB was copied once by hand into the dev folder, the old folder is left in place.
- A legacy `target/pixel_life_local.db` is copied once on first start and never overwritten (`migrate_legacy_db`).
- `init_db` runs per connection: WAL journal, 5 s busy timeout, foreign keys, then schema migrations and the model-version check (below).
- Commands share one mutex-guarded store (`DbState(Mutex<Db>)`); the engine thread holds its own `Db` on the same file.
- The headless runner `src/bin/trainer_runner.rs` runs in memory and opens no DB by default; `run --persist --db <path>` writes elites and run state to the given file (never the app DB implicitly); `probe --db <path>` reads HoF genomes through `open_read_only` (no migration, no reset) (TCK-173, TCK-183).
- Fire-and-forget writes log failures via `LogDbError::log_db_err`; commands return the error text; unreadable rows of list queries are skipped with a warning naming the table (TCK-137, RV-D05).

## Module and repositories (TCK-159)

SQL exists only in `src/persistence/`; everything else calls repository traits on plain DTOs. Contract and signatures: [[wiki/contracts/persistence-repo]]; adding a table: [[wiki/guides/add-table]].

| File | Content |
| :--- | :--- |
| `mod.rs` | DB location, `Db` (SQLite store), `init_db`, `open_read_only`, `DbState`, `LogDbError`, `unique_suffix` |
| `schema.rs` | `MIGRATIONS` (`PRAGMA user_version`), `MODEL_VERSION` reset with backup, `tables()` registry |
| `repos.rs` | `SpeciesRepo`, `CatalogueRepo`, `TrainerRepo`, `OceanSnapshotRepo` |
| `dto.rs` | DTOs and `RepoError` |
| `species.rs`, `catalogue.rs`, `trainer.rs`, `ocean.rs` | latest tables (`TABLES`), migration steps, `impl ... for Db` |
| `memory.rs` | `MemoryDb`: in-memory implementation of all four traits (contract suite runs both) |

Callers: `server::api` (Tauri commands; payloads unchanged), `server::engine` (species ledger, catalogue restock, Ocean snapshot, trainer), `biology::trainer_store` (adapter `GenomeRecord` / `TrainerRunState` ↔ `TrainerRepo`, used by the engine and the runner), `biology::probes::load_hof_genomes` (`ranked_trainer_genomes`, read-only). The catalogue query formerly duplicated in engine and API is `CatalogueRepo::list_catalogue` / `catalogue_entry`.

## Schema

| Table | Repository | Content |
| :--- | :--- | :--- |
| `meta` | (schema) | key/value; `model_version` = `MODEL_VERSION` the stored model rows were made with (TCK-183) |
| `species_records` | `SpeciesRepo` | one row per distinct genome: `latin_name` (UNIQUE), `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` (s); bounded (below) |
| `simulation_state` | `OceanSnapshotRepo` | Ocean snapshot, key `current_state`, JSON of creatures, food, counters (owned by the engine); upsert every 10 s; loaded on startup. Key `progenitor_genome` written by `apply_champion`, never read (RV-D06) |
| `trainer_genomes` | `TrainerRepo` | trainer elites per `run_id`/`generation`: `fitness`, `active` (current parents), `genome`, `methylations`, `synapse_weights` (JSON); bounded per run (below) |
| `trainer_run_state` | `TrainerRepo` | one row per `run_id`: training mode (`scenario`, UI mode string), curriculum `level`, `level_auto`, `layout_id` (u64 as decimal text), `mutation_ops_max` (adapted by the 1/5 rule), `updated_at` (s), `population` (TCK-192; nullable), `seed` (TCK-194; run seed, u64 as decimal text, nullable; missing = seed without reset nonce) (TCK-191) |
| `creature_catalogue` | `CatalogueRepo` | user-saved specimens: `id`, `name`, `genome`, `source`, `fitness`, `carnivory`, `methylations`, `synapse_weights`, `created_at` |

All tables except `meta` are model-scoped.

- **Schema migrations** (TCK-159): `PRAGMA user_version` holds the schema version, `schema::MIGRATIONS` the ordered list (current version 2). `init_db` applies every migration above the stored version in one transaction. Version 1 is the baseline of v1.12.30 (tables `CREATE IF NOT EXISTS` plus the columns older databases lack: `trainer_genomes.methylations`/`synapse_weights`, `trainer_run_state.population`/`seed`; errors propagate instead of being discarded). Version 2 drops `simulation_history` (Ocean energy time series every 10 s, no reader, unbounded, RV-D03) and `sessions` (only existed for its foreign key, RV-D06), with their rows; the engine no longer writes either. A database of v1.12.30 or older (user_version 0) runs both; a new database too. A user_version newer than the build is logged and left alone. Each module keeps its latest table definitions in `TABLES`; `test_migrated_schema_equals_latest_tables` keeps them equal to the migration chain. Test `test_v1_12_30_database_migrates_without_data_loss` builds the v1.12.30 schema (also with the pre-column shapes) with rows and checks that every row the app reads survives.
- `RESET_EVOLUTION` clears `species_records` and `simulation_state` (incl. `progenitor_genome`) and must clear `newly_spawned_creatures` ([[wiki/architecture/invariants]]).
- **Trainer genome IO** (`biology::trainer_store`, the IO side of `TrainerSim`): `save_elites` → `save_trainer_generation` marks the run's rows inactive, inserts the generation's elites as active and prunes the run, in one transaction; `load_genomes(run, active_only, limit)` returns the active parents or the hall of fame, best first (TCK-173). Generated row ids are `<run>-<gen>-<idx>-<unix ms>-<counter>` (`unique_suffix`); the former `<run>-<gen>-<idx>-<secs>` ids collided when a generation was saved twice within one second and lost its elites (TCK-159, test `test_trainer_genome_ids_never_collide_within_one_second`). UI sandbox assignments are stored as `<run>-<gen>-assigned-<slot>` (replacing that slot), catalogue additions as `<run>-<gen>-added-<unique>`.
- **Trainer run state** (TCK-191): `save_run_state` writes the run's `TrainerRunState` right after `save_elites` (engine after every generation, runner `--persist`); `load_run_state` is applied in the engine's `restore_trainer` at app start and after a run switch (`TrainerSim::restore_at_layout` continues at the saved layout; no row = current params, layout 0); re-entering the trainer on the same run keeps the in-memory state. A switch still pending at save time is not stored. `delete_trainer_run` (trainer reset, `clear_trainer_history`) removes the run's genomes and its state row in one transaction; the trainer reset then saves the new run's state with its fresh seed at once (TCK-194). Tests in `biology::trainer_store` run on both stores.
- **Retention**: `trainer_genomes` keeps the best `trainerGenomeRetentionPerRun` (200) rows per run by fitness plus all active rows, after every saved generation (engine and runner, TCK-183); the in-memory hall of fame of `TrainerSim` is bounded to the same N. `species_records` keeps every alive species and the `ocean.speciesRecordRetention` (2000) most recently extinct ones, pruned on the engine's 10 s snapshot timer (TCK-159). The number of trainer runs is not bounded automatically: runs are created by name in the UI and deleted there (`clear_trainer_history`). `simulation_history` no longer exists.
- **Reset on model change** (TCK-183): `MODEL_VERSION` (`src/persistence/schema.rs`, currently 6; 2 = Brain v2; 3 = Oja learning with fixed reflexes (TCK-152): stored phenotypes lack the `plastic` flag and weights learned under the old rule, incl. drifted reflexes, are invalid; 4 = Energy v2 (TCK-151, TCK-178): stored phenotypes still carry `basalMetabolicRate`, and trainer fitness and weights were selected and learned without an energy budget; 5 = fitness in catch units (TCK-175): stored trainer and catalogue fitness of the old formula, ~10²–10³, would outrank every new value in the hall of fame, retention and run list, which all sort by raw fitness; 6 = thalamus sum nodes with tanh (TCK-190): stored weights were selected and learned with a constant 0.5 drive from every silent channel) is bumped by every change that makes stored genomes' phenotypes or synapse weights invalid. `init_db` compares it with `meta.model_version` before the schema migrations; a different stored version, or none in a database that already has model tables, drops the model-scoped tables after the migrations (one transaction), recreates them empty in the latest shape and logs `model version <old> -> <new>: model tables reset` once. `meta` is kept; rows are not migrated. Before anything is changed the whole file is copied with `VACUUM INTO` (consistent with WAL) to `<db>.model-v<old>.bak.db` next to it (`backup_before_model_reset`, TCK-195), i.e. the file as the old build left it: one backup per old version, an existing one is never overwritten, in-memory DBs are skipped, a failed backup is logged and the reset still runs. Applies to every `init_db` (app setup, engine thread, runner `--persist`); `probe --db` opens read-only and never resets. Tests `test_fresh_db_stores_model_and_schema_version`, `test_file_db_model_version_reset_on_reopen`, `test_model_reset_backs_up_the_old_data_first`, `test_old_model_and_schema_version_together`.
- **Ocean snapshot restore**: a failed read, invalid JSON or unreadable creatures/spores in `current_state` are logged and the Ocean starts fresh (formerly dropped silently, TCK-159).
- Stored `synapse_weights` are in the synapse order of the phenotype that produced them; consumers rebuild the phenotype from the genome and match by synapse key or fall back to the genome weights on a length mismatch ([[wiki/biology/metabolism-and-reproduction]]).
- Vector columns (methylations, synapse weights) are `Vec<f32>` in the DTOs, written and read through `shared::json_col` (a failed encode/decode is logged, TCK-198); command payloads use `json_col::f32_values`, which keeps the stored spelling (`0.9`, not the f64 widening `0.8999999761581421`).
- Reading while the app runs: open read-only (`sqlite3 -readonly`), see [[wiki/simulation/analytics-and-balancing]].

## Known limitations

- **Timestamps:** `species_records` times and `trainer_genomes.created_at` in seconds; `simulation_state.updated_at` ms (snapshot) and s (`progenitor_genome`); `creature_catalogue.created_at` SQLite text (RV-D04). All come from `shared::time` (`now_secs`, `now_ms`: the name states the unit, TCK-198) and are set inside the repositories.
- **Silent JSON fallbacks:** corrupt methylation/synapse JSON is logged and replaced by an empty vector (RV-D05).
- **Dead state:** `progenitor_genome` is written by `apply_champion` and never read (RV-D06, removed with the typed IPC of TCK-157).
- **Ocean snapshot** is a JSON blob owned by the engine (no typed DTO yet).

## Troubleshooting

- Schema mismatch or startup freeze: close the app, delete `pixel_life_local.db` (+ `-wal`/`-shm`) from the app data dir; the schema is recreated on start. Deletes all species, trainer runs and catalogue entries.

## Key Takeaways

- One DB file in AppData, WAL; SQL only in `src/persistence/`, behind four repository traits with a SQLite and an in-memory implementation.
- Schema versions via `user_version` and an append-only migration list; model-version reset (with backup) for model-scoped rows.
- Retention: best 200 genomes per trainer run, 2000 extinct species; the unread `simulation_history` was removed.
- Ocean state is auto-saved every 10 s and restored on start; restore failures are logged.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-128, TCK-137, TCK-159, TCK-173, TCK-175, TCK-179, TCK-183, `README.md`.
