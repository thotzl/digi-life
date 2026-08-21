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
    let db_path = "pixel_life_local.db";
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
    let db_path = "pixel_life_local.db";
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
    let db_path = "pixel_life_local.db";
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
    let db_path = "pixel_life_local.db";
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
    let db_path = "pixel_life_local.db";
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
    let db_path = "pixel_life_local.db";
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM trainer_genomes WHERE run_id = ?1",
        params![run_id],
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn apply_champion(genome: String) -> Result<bool, String> {
    let db_path = "pixel_life_local.db";
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
