//! Plain rows the repositories read and write (TCK-159). No model types: the trainer adapter
//! (`biology::trainer_store`) maps `GenomeRecord` / `TrainerRunState` onto these.

use std::fmt;

/// Error of a repository call (SQLite error text; the in-memory store never fails).
#[derive(Debug, Clone, PartialEq)]
pub struct RepoError(pub String);

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RepoError {}

impl From<rusqlite::Error> for RepoError {
    fn from(e: rusqlite::Error) -> Self {
        RepoError(e.to_string())
    }
}

pub type RepoResult<T> = Result<T, RepoError>;

/// Status column of `species_records`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeciesStatus {
    Alive,
    Extinct,
}

impl SpeciesStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SpeciesStatus::Alive => "alive",
            SpeciesStatus::Extinct => "extinct",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "alive" => Some(SpeciesStatus::Alive),
            "extinct" => Some(SpeciesStatus::Extinct),
            _ => None,
        }
    }
}

/// A species seen for the first time; the genome is its id. Stored alive with peak 1 and `birth_time` = now (s).
#[derive(Debug, Clone, PartialEq)]
pub struct NewSpecies {
    pub genome: String,
    pub latin_name: String,
    pub parent_name: Option<String>,
    pub generation: i64,
    pub carnivory: f64,
}

/// One row of `species_records`. Times in Unix seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesRecord {
    pub id: String,
    pub latin_name: String,
    pub genome: String,
    pub parent_name: Option<String>,
    pub status: SpeciesStatus,
    pub peak_population: i32,
    pub birth_time: i64,
    pub extinction_time: Option<i64>,
    pub generation: i32,
    pub carnivory: f64,
}

/// The fields the engine's species ledger tracks per tick pass.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeciesState {
    pub id: String,
    pub status: SpeciesStatus,
    pub peak_population: i32,
}

/// A saved specimen of `creature_catalogue` as written by the UI.
#[derive(Debug, Clone, PartialEq)]
pub struct NewCatalogueEntry {
    pub id: String,
    pub name: String,
    pub genome: String,
    pub source: String,
    pub fitness: f64,
    pub carnivory: f64,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
}

/// One row of `creature_catalogue`; `created_at` is SQLite text (`YYYY-MM-DD HH:MM:SS`, UTC), set on save.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogueEntry {
    pub id: String,
    pub name: String,
    pub genome: String,
    pub source: String,
    pub fitness: f64,
    pub carnivory: f64,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
    pub created_at: String,
}

/// One trainer genome of a run (`trainer_genomes` without id, run and timestamps).
#[derive(Debug, Clone, PartialEq)]
pub struct TrainerGenome {
    pub name: String,
    pub genome: String,
    pub fitness: f64,
    pub generation: i64,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
}

/// A trainer genome of any run, for the hall-of-fame probes.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedGenome {
    pub genome: String,
    pub fitness: f64,
    pub run_id: String,
    pub generation: i64,
}

/// One line of the run list: newest generation and best fitness of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct TrainerRunSummary {
    pub run_id: String,
    pub max_generation: i64,
    pub max_fitness: f64,
}

/// Row of `trainer_run_state` (TCK-191); `scenario` is the UI mode string (`Scenario::mode`).
#[derive(Debug, Clone, PartialEq)]
pub struct TrainerRunStateRow {
    pub scenario: String,
    pub level: f32,
    pub level_auto: bool,
    pub layout_id: u64,
    pub mutation_ops_max: u32,
    /// `None` for rows of v1.12.23 (TCK-192) or a stored value <= 0
    pub population: Option<usize>,
    /// `None` for rows before TCK-194 or an unparsable value
    pub seed: Option<u64>,
}
