//! Creature catalogue commands.

use serde::Serialize;
use std::sync::mpsc::Sender;
use tauri::State;
use ts_rs::TS;

use super::{collect_rows, send_to_engine, with_store, CommandResult};
use crate::biology::phenotype::parse_genome;
use crate::config::AppConfig;
use crate::persistence::{CatalogueEntry, CatalogueRepo, DbState, NewCatalogueEntry, TrainerGenome, TrainerRepo};
use crate::server::ipc::{ClientAction, IpcError};
use crate::server::ocean_ipc::{OceanAction, SpawnClone};

/// One saved specimen (`get_catalogue_creatures`).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CatalogueCreature {
    pub id: String,
    pub name: String,
    pub genome: String,
    /// Where it was saved ("Ocean Substrate", "Evolutionary Trainer")
    pub source: String,
    pub fitness: f64,
    pub carnivory: f64,
    pub methylations: Vec<f32>,
    pub synapse_weights: Vec<f32>,
    /// SQLite text `YYYY-MM-DD HH:MM:SS` (UTC)
    pub created_at: String,
}

impl From<CatalogueEntry> for CatalogueCreature {
    fn from(c: CatalogueEntry) -> Self {
        Self {
            id: c.id,
            name: c.name,
            genome: c.genome,
            source: c.source,
            fitness: c.fitness,
            carnivory: c.carnivory,
            methylations: c.methylations,
            synapse_weights: c.synapse_weights,
            created_at: c.created_at,
        }
    }
}

fn catalogue_entry(db: &DbState, command: &str, id: &str) -> CommandResult<CatalogueEntry> {
    with_store(db, command, |store| store.catalogue_entry(id))?.ok_or_else(|| IpcError::not_found(format!("creature '{}' not found in the catalogue", id)))
}

#[tauri::command]
pub fn get_catalogue_creatures(db: State<'_, DbState>) -> CommandResult<Vec<CatalogueCreature>> {
    collect_rows(&db, "get_catalogue_creatures", |store| store.list_catalogue(), CatalogueCreature::from)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn save_to_catalogue(
    db: State<'_, DbState>,
    config: State<'_, AppConfig>,
    id: String,
    name: String,
    genome: String,
    source: String,
    fitness: f64,
    methylations: Vec<f32>,
    synapse_weights: Vec<f32>,
) -> CommandResult<()> {
    let carnivory = parse_genome(&genome, None, None, &config.brain).carnivory as f64;
    let entry = NewCatalogueEntry { id, name, genome, source, fitness, carnivory, methylations, synapse_weights };
    with_store(&db, "save_to_catalogue", |store| store.save_catalogue_entry(&entry))
}

#[tauri::command]
pub fn delete_from_catalogue(db: State<'_, DbState>, id: String) -> CommandResult<()> {
    with_store(&db, "delete_from_catalogue", |store| store.delete_catalogue_entry(&id))
}

#[tauri::command]
pub fn rename_catalogue_creature(db: State<'_, DbState>, id: String, new_name: String) -> CommandResult<()> {
    with_store(&db, "rename_catalogue_creature", |store| store.rename_catalogue_entry(&id, &new_name))
}

/// Releases a clone of a catalogue creature into the Ocean (a typed `SPAWN_CLONE` for the engine thread).
#[tauri::command]
pub fn spawn_catalogue_creature_to_ocean(
    db: State<'_, DbState>,
    tx: State<'_, Sender<ClientAction>>,
    id: String,
    load_learned_synapses: bool,
) -> CommandResult<()> {
    let entry = catalogue_entry(&db, "spawn_catalogue_creature_to_ocean", &id)?;
    send_to_engine(&tx, ClientAction::Ocean(OceanAction::SpawnClone(spawn_clone(entry, load_learned_synapses))))
}

fn spawn_clone(entry: CatalogueEntry, load_learned_synapses: bool) -> SpawnClone {
    SpawnClone {
        genome: entry.genome,
        methylations: Some(entry.methylations),
        synapse_weights: Some(entry.synapse_weights),
        load_learned_synapses: Some(load_learned_synapses),
    }
}

/// Adds a catalogue creature to a trainer run as an active genome of `generation`.
#[tauri::command]
pub fn add_catalogue_creature_to_training(db: State<'_, DbState>, creature_id: String, run_id: String, generation: i32) -> CommandResult<()> {
    let command = "add_catalogue_creature_to_training";
    let entry = catalogue_entry(&db, command, &creature_id)?;
    let row = TrainerGenome {
        name: entry.name,
        genome: entry.genome,
        fitness: entry.fitness,
        generation: generation as i64,
        methylations: entry.methylations,
        synapse_weights: entry.synapse_weights,
    };
    with_store(&db, command, |store| store.add_trainer_genome(&run_id, &row))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> CatalogueEntry {
        CatalogueEntry {
            id: "c1".to_string(),
            name: "Testus".to_string(),
            genome: "AB".to_string(),
            source: "Ocean Substrate".to_string(),
            fitness: 1.5,
            carnivory: 0.25,
            methylations: vec![0.9, 0.0],
            synapse_weights: vec![-2.0],
            created_at: "2026-11-02 10:00:00".to_string(),
        }
    }

    /// Same payload the catalogue view read before TCK-157 (camelCase, f32 values in their stored spelling).
    #[test]
    fn test_catalogue_creature_keeps_field_names_and_f32_spelling() {
        let json = serde_json::to_string(&CatalogueCreature::from(entry())).unwrap();
        assert_eq!(
            json,
            r#"{"id":"c1","name":"Testus","genome":"AB","source":"Ocean Substrate","fitness":1.5,"carnivory":0.25,"methylations":[0.9,0.0],"synapseWeights":[-2.0],"createdAt":"2026-11-02 10:00:00"}"#
        );
    }

    /// The typed SPAWN_CLONE carries the stored vectors unchanged (before TCK-157 they went through JSON text).
    #[test]
    fn test_spawn_clone_carries_the_stored_vectors() {
        let clone = spawn_clone(entry(), true);
        assert_eq!(clone.genome, "AB");
        assert_eq!(clone.methylations, Some(vec![0.9, 0.0]));
        assert_eq!(clone.synapse_weights, Some(vec![-2.0]));
        assert_eq!(clone.load_learned_synapses, Some(true));
    }
}
