use tauri::{AppHandle, Manager, State};
use serde_json::json;
use std::sync::mpsc::Sender;
use crate::persistence::{CatalogueRepo, DbState, NewCatalogueEntry, OceanSnapshotRepo, SpeciesRepo, TrainerGenome, TrainerRepo};
use crate::biology::phenotype::parse_genome;
use crate::config::AppConfig;
use crate::shared::json_col;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainerGenomeInput {
    pub name: String,
    pub genome: String,
    pub fitness: f64,
}

#[tauri::command]
pub fn handle_client_action(action: String, app_handle: AppHandle) -> Result<(), String> {
    let tx: State<Sender<String>> = app_handle.state();
    if let Err(e) = tx.send(action) {
        log::error!("sending an action to the engine thread failed: {}", e);
    }
    Ok(())
}

#[tauri::command]
pub fn get_registered_species(db: State<'_, DbState>) -> Result<serde_json::Value, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let list: Vec<serde_json::Value> = store
        .list_species()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "name": s.latin_name,
                "genome": s.genome,
                "antisense": "",
                "parentSpeciesId": s.parent_name,
                "status": s.status.as_str(),
                "peakPopulation": s.peak_population,
                "birthTime": s.birth_time,
                "extinctionTime": s.extinction_time,
                "generation": s.generation,
                "carnivory": s.carnivory
            })
        })
        .collect();
    Ok(json!(list))
}

#[tauri::command]
pub fn get_trainer_runs(db: State<'_, DbState>) -> Result<serde_json::Value, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let list: Vec<serde_json::Value> = store
        .trainer_runs()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|run| json!({ "run_id": run.run_id, "max_gen": run.max_generation, "max_fit": run.max_fitness }))
        .collect();
    Ok(json!(list))
}

/// Name, genome and fitness of the best `limit` genomes of a run (active parents or hall of fame).
fn trainer_genome_list(db: &State<'_, DbState>, run_id: &str, active_only: bool, limit: i32) -> Result<serde_json::Value, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let list: Vec<serde_json::Value> = store
        .trainer_genomes(run_id, active_only, limit as i64)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|g| json!({ "name": g.name, "genome": g.genome, "fitness": g.fitness }))
        .collect();
    Ok(json!(list))
}

#[tauri::command]
pub fn get_trainer_population(db: State<'_, DbState>, run_id: String, limit: i32) -> Result<serde_json::Value, String> {
    trainer_genome_list(&db, &run_id, true, limit)
}

#[tauri::command]
pub fn get_trainer_hof(db: State<'_, DbState>, run_id: String, limit: i32) -> Result<serde_json::Value, String> {
    trainer_genome_list(&db, &run_id, false, limit)
}

#[tauri::command]
pub fn save_trainer_generation(db: State<'_, DbState>, run_id: String, generation: i32, population: Vec<TrainerGenomeInput>) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let rows: Vec<TrainerGenome> = population
        .into_iter()
        .map(|g| TrainerGenome { name: g.name, genome: g.genome, fitness: g.fitness, generation: generation as i64, methylations: Vec::new(), synapse_weights: Vec::new() })
        .collect();
    store.save_trainer_generation(&run_id, &rows, None).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn clear_trainer_history(db: State<'_, DbState>, run_id: String) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    // Genomes and run state (TCK-191)
    store.delete_trainer_run(&run_id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn apply_champion(db: State<'_, DbState>, genome: String) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    store.save_progenitor_genome(&genome).map_err(|e| e.to_string())?;
    Ok(true)
}

// async: runs on the async runtime instead of the main thread, so a burst of phenotype requests
// (new genomes every trainer generation) cannot stall other commands and event delivery
#[tauri::command]
pub async fn get_fossil_phenotype(genome: String, config: State<'_, AppConfig>) -> Result<serde_json::Value, String> {
    let pheno = parse_genome(&genome, None, None, &config.brain);
    serde_json::to_value(&pheno).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_catalogue_creatures(db: State<'_, DbState>) -> Result<serde_json::Value, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let list: Vec<serde_json::Value> = store
        .list_catalogue()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|c| {
            json!({
                "id": c.id,
                "name": c.name,
                "genome": c.genome,
                "source": c.source,
                "fitness": c.fitness,
                "carnivory": c.carnivory,
                "methylations": json_col::f32_values(&c.methylations),
                "synapseWeights": json_col::f32_values(&c.synapse_weights),
                "createdAt": c.created_at
            })
        })
        .collect();
    Ok(json!(list))
}

#[tauri::command]
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
) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let carnivory = parse_genome(&genome, None, None, &config.brain).carnivory as f64;
    store
        .save_catalogue_entry(&NewCatalogueEntry { id, name, genome, source, fitness, carnivory, methylations, synapse_weights })
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn delete_from_catalogue(db: State<'_, DbState>, id: String) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    store.delete_catalogue_entry(&id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn rename_catalogue_creature(db: State<'_, DbState>, id: String, new_name: String) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    store.rename_catalogue_entry(&id, &new_name).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn spawn_catalogue_creature_to_ocean(
    db: State<'_, DbState>,
    id: String,
    load_learned_synapses: bool,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let Some(entry) = store.catalogue_entry(&id).map_err(|e| e.to_string())? else {
        return Err("Creature not found in catalogue".to_string());
    };
    let action = json!({
        "type": "SPAWN_CLONE",
        "genome": entry.genome,
        "methylations": json_col::f32_values(&entry.methylations),
        "synapse_weights": json_col::f32_values(&entry.synapse_weights),
        "load_learned_synapses": load_learned_synapses
    });

    let tx: State<Sender<String>> = app_handle.state();
    if let Err(e) = tx.send(action.to_string()) {
        return Err(format!("Failed to send spawn action to simulation thread: {}", e));
    }
    Ok(true)
}

#[tauri::command]
pub fn add_catalogue_creature_to_training(
    db: State<'_, DbState>,
    creature_id: String,
    run_id: String,
    generation: i32,
) -> Result<bool, String> {
    let store = db.0.lock().map_err(|e| e.to_string())?;
    let Some(entry) = store.catalogue_entry(&creature_id).map_err(|e| e.to_string())? else {
        return Err("Creature not found in catalogue".to_string());
    };
    let row = TrainerGenome {
        name: entry.name,
        genome: entry.genome,
        fitness: entry.fitness,
        generation: generation as i64,
        methylations: entry.methylations,
        synapse_weights: entry.synapse_weights,
    };
    store.add_trainer_genome(&run_id, &row).map_err(|e| e.to_string())?;
    Ok(true)
}
