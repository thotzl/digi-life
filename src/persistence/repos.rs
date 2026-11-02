//! Repository traits (TCK-159): the only way code outside `persistence` reads or writes stored data. Implemented by
//! the SQLite store `Db` and the in-memory store `MemoryDb` (tests, headless use). Contract:
//! `wiki/contracts/persistence-repo.md`.

use super::dto::*;

/// `species_records`: the Ocean's species ledger (one row per distinct genome).
pub trait SpeciesRepo {
    /// Adds an alive species with peak 1 and `birth_time` = now (s); ignored if the genome or the latin name is
    /// already stored.
    fn insert_species(&self, species: &NewSpecies) -> RepoResult<()>;
    /// All species in insertion order; unreadable rows are skipped with a warning.
    fn list_species(&self) -> RepoResult<Vec<SpeciesRecord>>;
    /// Id, status and peak of every species in insertion order (the engine's ledger cache).
    fn species_states(&self) -> RepoResult<Vec<SpeciesState>>;
    fn species_name(&self, id: &str) -> RepoResult<Option<String>>;
    fn mark_species_alive(&self, id: &str) -> RepoResult<()>;
    /// Status extinct, `extinction_time` = now (s).
    fn mark_species_extinct(&self, id: &str) -> RepoResult<()>;
    fn set_species_peak(&self, id: &str, peak: i32) -> RepoResult<()>;
    fn clear_species(&self) -> RepoResult<()>;
    /// Retention: keeps every alive species and the `keep` most recently extinct ones; returns the deleted count.
    fn prune_extinct_species(&self, keep: usize) -> RepoResult<usize>;
}

/// `creature_catalogue`: specimens the user saved.
pub trait CatalogueRepo {
    /// All entries, newest first; unreadable rows are skipped with a warning.
    fn list_catalogue(&self) -> RepoResult<Vec<CatalogueEntry>>;
    fn catalogue_entry(&self, id: &str) -> RepoResult<Option<CatalogueEntry>>;
    /// Inserts or replaces the entry with this id; `created_at` = now.
    fn save_catalogue_entry(&self, entry: &NewCatalogueEntry) -> RepoResult<()>;
    fn rename_catalogue_entry(&self, id: &str, name: &str) -> RepoResult<()>;
    fn delete_catalogue_entry(&self, id: &str) -> RepoResult<()>;
}

/// `trainer_genomes` and `trainer_run_state`: elites, hall of fame and run state per run.
pub trait TrainerRepo {
    /// One generation in one transaction: the run's rows become inactive, `genomes` are inserted as active parents
    /// (each under a fresh unique id), then the run is pruned to `retention` (if given).
    fn save_trainer_generation(&self, run_id: &str, genomes: &[TrainerGenome], retention: Option<usize>) -> RepoResult<()>;
    /// A creature placed into a sandbox by the UI: active row `<run>-<gen>-assigned-<slot>`, replacing the
    /// previous assignment of that slot in that generation.
    fn assign_trainer_genome(&self, run_id: &str, slot: u32, genome: &TrainerGenome) -> RepoResult<()>;
    /// An extra active parent (catalogue → training) under a fresh unique id.
    fn add_trainer_genome(&self, run_id: &str, genome: &TrainerGenome) -> RepoResult<()>;
    /// Retention (TCK-183): keeps the best `retention` rows of the run by fitness plus all active rows.
    fn prune_trainer_genomes(&self, run_id: &str, retention: usize) -> RepoResult<()>;
    /// Best first: the active parents (`active_only`) or the hall of fame (all rows); a negative `limit` is no limit.
    fn trainer_genomes(&self, run_id: &str, active_only: bool, limit: i64) -> RepoResult<Vec<TrainerGenome>>;
    /// Every stored genome of every run, best first (ties by id); fails on the first unreadable row.
    fn ranked_trainer_genomes(&self) -> RepoResult<Vec<RankedGenome>>;
    /// One summary per run, ordered by run id.
    fn trainer_runs(&self) -> RepoResult<Vec<TrainerRunSummary>>;
    /// Inserts or replaces the run's state, `updated_at` = now (s).
    fn save_trainer_run_state(&self, run_id: &str, state: &TrainerRunStateRow) -> RepoResult<()>;
    fn trainer_run_state(&self, run_id: &str) -> RepoResult<Option<TrainerRunStateRow>>;
    /// Deletes the run's genomes and run state (one transaction).
    fn delete_trainer_run(&self, run_id: &str) -> RepoResult<()>;
}

/// `simulation_state`: the Ocean snapshot (JSON owned by the engine). The legacy key `progenitor_genome` (written by
/// the removed `apply_champion` command, never read; RV-D06, TCK-157) may still exist in old databases.
pub trait OceanSnapshotRepo {
    /// Replaces the snapshot (`current_state`), `updated_at` = now (ms).
    fn save_ocean_snapshot(&self, state_json: &str) -> RepoResult<()>;
    fn ocean_snapshot(&self) -> RepoResult<Option<String>>;
    /// Deletes every key, the snapshot and a legacy progenitor key (`RESET_EVOLUTION`).
    fn clear_ocean_state(&self) -> RepoResult<()>;
}
