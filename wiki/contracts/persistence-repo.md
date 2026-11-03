---
sources: [tickets/closed/TCK-159-db-migrations-retention.md, src/persistence/mod.rs, src/persistence/repos.rs, src/persistence/schema.rs]
---

# Contract: persistence repository

## Purpose

Every read and write of stored data goes through a repository trait of `src/persistence/`; no other module contains
SQL or imports `rusqlite` (TCK-159). A feature that stores something (a new room's results, an arena ladder, creator
drafts) adds methods to a trait or a new trait plus its table(s), its migration and both implementations. Two
implementations exist: `Db` (SQLite, the app and the runner) and `MemoryDb` (in memory, tests and headless use).
Current tables and retention: [[wiki/architecture/persistence]].

## Signature

```rust
// src/persistence/repos.rs — implemented by `Db` (species.rs, catalogue.rs, trainer.rs, ocean.rs) and `MemoryDb` (memory.rs)
pub trait SpeciesRepo {        // species_records
    fn insert_species(&self, species: &NewSpecies) -> RepoResult<()>;
    fn list_species(&self) -> RepoResult<Vec<SpeciesRecord>>;
    fn species_states(&self) -> RepoResult<Vec<SpeciesState>>;
    fn species_name(&self, id: &str) -> RepoResult<Option<String>>;
    fn mark_species_alive(&self, id: &str) -> RepoResult<()>;
    fn mark_species_extinct(&self, id: &str) -> RepoResult<()>;
    fn set_species_peak(&self, id: &str, peak: i32) -> RepoResult<()>;
    fn clear_species(&self) -> RepoResult<()>;
    fn prune_extinct_species(&self, keep: usize) -> RepoResult<usize>;
}
pub trait CatalogueRepo {      // creature_catalogue
    fn list_catalogue(&self) -> RepoResult<Vec<CatalogueEntry>>;
    fn catalogue_entry(&self, id: &str) -> RepoResult<Option<CatalogueEntry>>;
    fn save_catalogue_entry(&self, entry: &NewCatalogueEntry) -> RepoResult<()>;
    fn rename_catalogue_entry(&self, id: &str, name: &str) -> RepoResult<()>;
    fn delete_catalogue_entry(&self, id: &str) -> RepoResult<()>;
}
pub trait TrainerRepo {        // trainer_genomes, trainer_run_state
    fn save_trainer_generation(&self, run_id: &str, genomes: &[TrainerGenome], retention: Option<usize>) -> RepoResult<()>;
    fn assign_trainer_genome(&self, run_id: &str, slot: u32, genome: &TrainerGenome) -> RepoResult<()>;
    fn add_trainer_genome(&self, run_id: &str, genome: &TrainerGenome) -> RepoResult<()>;
    fn prune_trainer_genomes(&self, run_id: &str, retention: usize) -> RepoResult<()>;
    fn trainer_genomes(&self, run_id: &str, active_only: bool, limit: i64) -> RepoResult<Vec<TrainerGenome>>;
    fn ranked_trainer_genomes(&self) -> RepoResult<Vec<RankedGenome>>;
    fn trainer_runs(&self) -> RepoResult<Vec<TrainerRunSummary>>;
    fn save_trainer_run_state(&self, run_id: &str, state: &TrainerRunStateRow) -> RepoResult<()>;
    fn trainer_run_state(&self, run_id: &str) -> RepoResult<Option<TrainerRunStateRow>>;
    fn delete_trainer_run(&self, run_id: &str) -> RepoResult<()>;
}
pub trait OceanSnapshotRepo {  // simulation_state
    fn save_ocean_snapshot(&self, state_json: &str) -> RepoResult<()>;
    fn ocean_snapshot(&self) -> RepoResult<Option<String>>;
    fn clear_ocean_state(&self) -> RepoResult<()>;
}

// src/persistence/mod.rs
pub fn init_db<P: AsRef<Path>>(path: P) -> RepoResult<Db>;        // ":memory:" for tests
pub fn open_read_only<P: AsRef<Path>>(path: P) -> RepoResult<Db>; // no migration, writes fail
pub struct DbState(pub Mutex<Db>);                                 // Tauri managed state of the commands
pub trait LogDbError { fn log_db_err(self, context: &str); }       // for Result<T, impl Display>
pub struct MemoryDb;                                               // MemoryDb::new()
pub const MODEL_VERSION: u32;                                      // src/persistence/schema.rs
```

The DTOs (`src/persistence/dto.rs`) are plain structs without model types: `NewSpecies`, `SpeciesRecord`,
`SpeciesState`, `SpeciesStatus`, `NewCatalogueEntry`, `CatalogueEntry`, `TrainerGenome`, `RankedGenome`,
`TrainerRunSummary`, `TrainerRunStateRow`, `RepoError` / `RepoResult`. Model types are mapped by an adapter next to
the model: `biology::trainer_store` maps `GenomeRecord` / `TrainerRunState` / `TrainerSim` onto `TrainerRepo` and
logs failures (`save_elites`, `load_genomes`, `save_assigned`, `save_run_state`, `load_run_state`).

## Lifecycle / call order

1. `init_db(path)` once per connection: app setup (`main.rs`, `DbState`), the engine thread (its own connection to
   the same file), runner `run --persist --db`. Order inside: decide the model reset (stored `meta.model_version`
   vs `MODEL_VERSION`) → backup copy if a reset is due → schema migrations above `PRAGMA user_version` (one
   transaction) → drop the model-scoped tables if a reset is due and store `MODEL_VERSION` → `CREATE ... IF NOT
   EXISTS` of every latest table.
2. Commands lock `DbState` per call; the engine calls its store on actions and timers.
3. `open_read_only` (runner `probe --db`) skips step 1 entirely.

## Invariants

- SQL and `rusqlite` only in `src/persistence/` ([[wiki/architecture/invariants]] 12; check: `grep -rnE "rusqlite|SELECT |INSERT |DELETE FROM" src
  --include=*.rs` lists only that folder).
- No repository call per physics step ([[wiki/architecture/invariants]] 6): call on actions, generation ends or the
  10 s timer.
- `Db` and `MemoryDb` behave the same for every trait method; the contract suite in `persistence::tests::contract`
  runs both.
- Repositories set their own timestamps (`shared::time`); callers never pass a time.
- Generated row ids are unique per process (`unique_suffix`: Unix ms + counter); a second save within the same
  second never collides.
- Migrations are append-only: a released migration is never edited. A change to a table is a new migration plus the
  same change to the table's latest `TABLES` definition; `test_migrated_schema_equals_latest_tables` compares both.
- Rows of a model-scoped table are dropped on a `MODEL_VERSION` change, never migrated
  ([[wiki/architecture/invariants]] 9). Tables whose rows do not depend on the model set `model_scoped: false`.
- An existing user database must migrate without data loss in the tables the app reads; a migration that drops data
  says so in its name and in [[wiki/architecture/persistence]].
- Write failures of fire-and-forget calls are logged with `log_db_err`; commands return it as a logged `IpcError` (kind `database`).

## Registration point

- New table: a `TableDef` in the owning module's `TABLES` (`species.rs`, `catalogue.rs`, `trainer.rs`, `ocean.rs`, or a
  new module listed in `schema::tables()`), and a migration step appended as a new `Migration` in
  `schema::MIGRATIONS`.
- New query: a method on the owning trait, implemented in the module's `impl ... for Db` and in `memory.rs`.

## Config keys

`trainer.trainerGenomeRetentionPerRun` (`prune_trainer_genomes`, via `trainer_store::save_elites`),
`ocean.speciesRecordRetention` (`prune_extinct_species`, engine 10 s timer). See [[wiki/architecture/configuration]].

## Events / IPC

None emitted by the persistence module. The engine emits `DATABASE_CHANGED` after species writes; the Tauri commands
in `src/server/api/` map repository rows onto their typed DTOs through `with_store` / `collect_rows`, a repository
error becomes an `IpcError` of kind `database` ([[wiki/contracts/ipc-message]]).

## Test recipe

`cargo test persistence::` runs the migration tests (`test_v1_12_30_database_migrates_without_data_loss`,
`test_old_model_and_schema_version_together`, `test_migrated_schema_equals_latest_tables`), the model-reset and
backup tests and the contract suite (`contract::test_*_repo_contract`, SQLite and memory). A new table adds rows to
the v1.12.30 fixture only if it migrates existing data; a new trait method extends the contract test of its trait.
`cargo test biology::trainer_store` covers the trainer adapter on both stores.

## Minimal example

A per-run note stored by the trainer (new column on `trainer_run_state`, schema version 3):

```rust
// src/persistence/schema.rs, MIGRATIONS
Migration { version: 3, name: "trainer_run_state.note (TCK-xxx)", steps: &[trainer::v3_run_note] },

// src/persistence/trainer.rs
pub(super) fn v3_run_note(conn: &Connection) -> Result<()> {
    conn.execute("ALTER TABLE trainer_run_state ADD COLUMN note TEXT", [])?;
    Ok(())
}
// TABLES: append `note TEXT` to the CREATE TABLE of trainer_run_state (latest shape)

// src/persistence/repos.rs, TrainerRepo
fn set_run_note(&self, run_id: &str, note: &str) -> RepoResult<()>;

// src/persistence/trainer.rs, impl TrainerRepo for Db
fn set_run_note(&self, run_id: &str, note: &str) -> RepoResult<()> {
    self.conn().execute("UPDATE trainer_run_state SET note = ?1 WHERE run_id = ?2", params![note, run_id])?;
    Ok(())
}
// src/persistence/memory.rs: the same on the in-memory row, plus a contract test line
```
