//! Species ledger and phenotype commands.

use serde::Serialize;
use tauri::State;
use ts_rs::TS;

use super::{collect_rows, CommandResult};
use crate::biology::phenotype::{parse_genome, CreaturePhenotype};
use crate::config::AppConfig;
use crate::persistence::{DbState, SpeciesRecord, SpeciesRepo, SpeciesStatus};

/// Alive or extinct (`species_records.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SpeciesLifeStatus {
    Alive,
    Extinct,
}

impl From<SpeciesStatus> for SpeciesLifeStatus {
    fn from(status: SpeciesStatus) -> Self {
        match status {
            SpeciesStatus::Alive => SpeciesLifeStatus::Alive,
            SpeciesStatus::Extinct => SpeciesLifeStatus::Extinct,
        }
    }
}

/// One species of the roster (`get_registered_species`). Times in Unix seconds.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SpeciesEntry {
    /// The genome (species id)
    pub id: String,
    /// Latin name
    pub name: String,
    pub genome: String,
    /// Always empty (kept field)
    pub antisense: String,
    /// Latin name of the parent species, if any
    pub parent_species_id: Option<String>,
    pub status: SpeciesLifeStatus,
    pub peak_population: i32,
    #[ts(type = "number")]
    pub birth_time: i64,
    #[ts(type = "number | null")]
    pub extinction_time: Option<i64>,
    pub generation: i32,
    pub carnivory: f64,
}

impl From<SpeciesRecord> for SpeciesEntry {
    fn from(s: SpeciesRecord) -> Self {
        Self {
            id: s.id,
            name: s.latin_name,
            genome: s.genome,
            antisense: String::new(),
            parent_species_id: s.parent_name,
            status: s.status.into(),
            peak_population: s.peak_population,
            birth_time: s.birth_time,
            extinction_time: s.extinction_time,
            generation: s.generation,
            carnivory: s.carnivory,
        }
    }
}

#[tauri::command]
pub fn get_registered_species(db: State<'_, DbState>) -> CommandResult<Vec<SpeciesEntry>> {
    collect_rows(&db, "get_registered_species", |store| store.list_species(), SpeciesEntry::from)
}

/// Phenotype of any genome (runs `parse_genome`).
// async: runs on the async runtime instead of the main thread, so a burst of phenotype requests
// (new genomes every trainer generation) cannot stall other commands and event delivery
#[tauri::command]
pub async fn get_fossil_phenotype(genome: String, config: State<'_, AppConfig>) -> CommandResult<CreaturePhenotype> {
    Ok(parse_genome(&genome, None, None, &config.brain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_species_entry_keeps_the_roster_field_names() {
        let entry = SpeciesEntry::from(SpeciesRecord {
            id: "G".to_string(),
            latin_name: "Testus".to_string(),
            genome: "G".to_string(),
            parent_name: None,
            status: SpeciesStatus::Extinct,
            peak_population: 3,
            birth_time: 10,
            extinction_time: Some(20),
            generation: 2,
            carnivory: 0.5,
        });
        assert_eq!(
            serde_json::to_value(entry).unwrap(),
            json!({
                "id": "G", "name": "Testus", "genome": "G", "antisense": "", "parentSpeciesId": null, "status": "extinct",
                "peakPopulation": 3, "birthTime": 10, "extinctionTime": 20, "generation": 2, "carnivory": 0.5
            })
        );
    }
}
