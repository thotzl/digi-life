use tauri::{AppHandle, Manager, State};
use serde_json::json;
use std::sync::mpsc::Sender;
use rusqlite::params;
use crate::database::init_db;
use crate::biology::dna::parse_genome;

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
        eprintln!("[TAURI COMMAND] Error sending action to simulation thread: {}", e);
    }
    Ok(())
}

#[tauri::command]
pub fn get_registered_species() -> Result<serde_json::Value, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;
    
    let mut stmt = conn
        .prepare("SELECT id, latin_name, genome_string, parent_name, status, peak_population, birth_time, extinction_time, generation, carnivory FROM species_records")
        .map_err(|e| e.to_string())?;
        
    let rows = stmt.query_map([], |row| {
        let birth_time: i64 = row.get(6)?;
        let extinction_time: Option<i64> = row.get(7)?;
        Ok(json!({
            "id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
            "genome": row.get::<_, String>(2)?,
            "antisense": "",
            "parentSpeciesId": row.get::<_, Option<String>>(3)?,
            "status": row.get::<_, String>(4)?,
            "peakPopulation": row.get::<_, i32>(5)?,
            "birthTime": birth_time,
            "extinctionTime": extinction_time,
            "generation": row.get::<_, i32>(8)?,
            "carnivory": row.get::<_, f64>(9)?
        }))
    }).map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in rows {
        if let Ok(item) = row {
            list.push(item);
        }
    }
    
    Ok(json!(list))
}

#[tauri::command]
pub fn get_trainer_runs() -> Result<serde_json::Value, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT run_id, MAX(generation) as max_gen, MAX(fitness) as max_fit FROM trainer_genomes GROUP BY run_id")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(json!({
            "run_id": row.get::<_, String>(0)?,
            "max_gen": row.get::<_, i32>(1)?,
            "max_fit": row.get::<_, f64>(2)?
        }))
    }).map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in rows {
        if let Ok(item) = row {
            list.push(item);
        }
    }

    Ok(json!(list))
}

#[tauri::command]
pub fn get_trainer_population(run_id: String, limit: i32) -> Result<serde_json::Value, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT name, genome, fitness FROM trainer_genomes WHERE run_id = ?1 AND active = 1 ORDER BY fitness DESC LIMIT ?2")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![run_id, limit], |row| {
        Ok(json!({
            "name": row.get::<_, String>(0)?,
            "genome": row.get::<_, String>(1)?,
            "fitness": row.get::<_, f64>(2)?
        }))
    }).map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in rows {
        if let Ok(item) = row {
            list.push(item);
        }
    }

    Ok(json!(list))
}

#[tauri::command]
pub fn get_trainer_hof(run_id: String, limit: i32) -> Result<serde_json::Value, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT name, genome, fitness FROM trainer_genomes WHERE run_id = ?1 ORDER BY fitness DESC LIMIT ?2")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![run_id, limit], |row| {
        Ok(json!({
            "name": row.get::<_, String>(0)?,
            "genome": row.get::<_, String>(1)?,
            "fitness": row.get::<_, f64>(2)?
        }))
    }).map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in rows {
        if let Ok(item) = row {
            list.push(item);
        }
    }

    Ok(json!(list))
}

#[tauri::command]
pub fn save_trainer_generation(run_id: String, generation: i32, population: Vec<TrainerGenomeInput>) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let mut conn = init_db(db_path).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE trainer_genomes SET active = 0 WHERE run_id = ?1",
        params![run_id],
    ).map_err(|e| e.to_string())?;

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    for (idx, gen_input) in population.iter().enumerate() {
        let unique_id = format!("{}-{}-{}-{}", run_id, generation, idx, now_millis);
        tx.execute(
            "INSERT INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
            params![
                unique_id,
                run_id,
                generation,
                gen_input.name,
                gen_input.genome,
                gen_input.fitness,
                now_millis
            ],
        ).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub fn clear_trainer_history(run_id: String) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM trainer_genomes WHERE run_id = ?1",
        params![run_id],
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn apply_champion(genome: String) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    conn.execute(
        "INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES ('progenitor_genome', ?1, ?2)",
        params![genome, now_millis],
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn get_fossil_phenotype(genome: String) -> Result<serde_json::Value, String> {
    let pheno = parse_genome(&genome, None, None);
    serde_json::to_value(&pheno).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_catalogue_creatures() -> Result<serde_json::Value, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, genome, source, fitness, carnivory, methylations, synapse_weights, created_at FROM creature_catalogue ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let genome: String = row.get(2)?;
        let source: String = row.get(3)?;
        let fitness: f64 = row.get(4)?;
        let carnivory: f64 = row.get(5)?;
        let methylations_str: String = row.get(6)?;
        let synapse_weights_str: String = row.get(7)?;
        let created_at: String = row.get(8)?;

        let methylations: serde_json::Value = serde_json::from_str(&methylations_str).unwrap_or(json!([]));
        let synapse_weights: serde_json::Value = serde_json::from_str(&synapse_weights_str).unwrap_or(json!([]));

        Ok(json!({
            "id": id,
            "name": name,
            "genome": genome,
            "source": source,
            "fitness": fitness,
            "carnivory": carnivory,
            "methylations": methylations,
            "synapseWeights": synapse_weights,
            "createdAt": created_at
        }))
    }).map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in rows {
        if let Ok(item) = row {
            list.push(item);
        }
    }

    Ok(json!(list))
}

#[tauri::command]
pub fn save_to_catalogue(
    id: String,
    name: String,
    genome: String,
    source: String,
    fitness: f64,
    methylations: Vec<f32>,
    synapse_weights: Vec<f32>,
) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let pheno = parse_genome(&genome, None, None);
    let carnivory = pheno.carnivory as f64;

    let methylations_json = serde_json::to_string(&methylations).map_err(|e| e.to_string())?;
    let synapse_weights_json = serde_json::to_string(&synapse_weights).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO creature_catalogue (id, name, genome, source, fitness, carnivory, methylations, synapse_weights, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
        params![id, name, genome, source, fitness, carnivory, methylations_json, synapse_weights_json],
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn delete_from_catalogue(id: String) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM creature_catalogue WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn rename_catalogue_creature(id: String, new_name: String) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    conn.execute("UPDATE creature_catalogue SET name = ?1 WHERE id = ?2", params![new_name, id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn spawn_catalogue_creature_to_ocean(
    id: String,
    load_learned_synapses: bool,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT genome, methylations, synapse_weights FROM creature_catalogue WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    }).map_err(|e| e.to_string())?;

    if let Some(Ok((genome, methylations_str, synapse_weights_str))) = rows.next() {
        let methylations: serde_json::Value = serde_json::from_str(&methylations_str).unwrap_or(json!([]));
        let synapse_weights: serde_json::Value = serde_json::from_str(&synapse_weights_str).unwrap_or(json!([]));

        let action = json!({
            "type": "SPAWN_CLONE",
            "genome": genome,
            "methylations": methylations,
            "synapse_weights": synapse_weights,
            "load_learned_synapses": load_learned_synapses
        });

        let tx: State<Sender<String>> = app_handle.state();
        if let Err(e) = tx.send(action.to_string()) {
            return Err(format!("Failed to send spawn action to simulation thread: {}", e));
        }
        Ok(true)
    } else {
        Err("Creature not found in catalogue".to_string())
    }
}

#[tauri::command]
pub fn add_catalogue_creature_to_training(
    creature_id: String,
    run_id: String,
    generation: i32,
) -> Result<bool, String> {
    let db_path = crate::database::DB_PATH;
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT name, genome, fitness, methylations, synapse_weights FROM creature_catalogue WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![creature_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?
        ))
    }).map_err(|e| e.to_string())?;

    if let Some(Ok((name, genome, fitness, methylations, synapse_weights))) = rows.next() {
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let unique_id = format!("{}-{}-added-{}", run_id, generation, now_millis);

        conn.execute(
            "INSERT INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at, methylations, synapse_weights)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)",
            params![
                unique_id,
                run_id,
                generation,
                name,
                genome,
                fitness,
                (now_millis / 1000) as i64,
                methylations,
                synapse_weights
            ],
        ).map_err(|e| e.to_string())?;

        Ok(true)
    } else {
        Err("Creature not found in catalogue".to_string())
    }
}
