// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Emitter, Manager, State};
use std::thread;
use std::time::{Duration, Instant};
use serde_json::json;
use std::sync::mpsc::{channel, Sender, Receiver};
use rusqlite::params;
use rand::Rng;

use src_tauri::shared::types::{CreatureAgent, FoodSpore, TelemetryCreature};
use src_tauri::shared::spatial_grid::SpatialGrid;
use src_tauri::shared::physics::apply_creature_physics;
use src_tauri::shared::brain::execute_brain;
use src_tauri::biology::dna::{parse_genome, mutate_genome, generate_random_genome};
use src_tauri::biology::trainer_engine::{
    init_rust_sandbox, step_trainer_sandbox_physics, calculate_sandbox_fitness,
    TrainerSandbox, TrainerTelemetrySandbox,
};
use src_tauri::database::init_db;

#[tauri::command]
fn handle_client_action(action: String, app_handle: AppHandle) -> Result<(), String> {
    let tx: State<Sender<String>> = app_handle.state();
    if let Err(e) = tx.send(action) {
        eprintln!("[TAURI COMMAND] Error sending action to simulation thread: {}", e);
    }
    Ok(())
}

#[tauri::command]
fn get_registered_species() -> Result<serde_json::Value, String> {
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
fn get_trainer_runs() -> Result<serde_json::Value, String> {
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
fn get_trainer_population(run_id: String, limit: i32) -> Result<serde_json::Value, String> {
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
fn get_trainer_hof(run_id: String, limit: i32) -> Result<serde_json::Value, String> {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrainerGenomeInput {
    pub name: String,
    pub genome: String,
    pub fitness: f64,
}

#[tauri::command]
fn save_trainer_generation(run_id: String, generation: i32, population: Vec<TrainerGenomeInput>) -> Result<bool, String> {
    let db_path = "pixel_life_local.db";
    let mut conn = init_db(db_path).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Mark previous active genomes for this run as inactive
    tx.execute(
        "UPDATE trainer_genomes SET active = 0 WHERE run_id = ?1",
        params![run_id],
    ).map_err(|e| e.to_string())?;

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Insert new generation genomes
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
fn clear_trainer_history(run_id: String) -> Result<bool, String> {
    let db_path = "pixel_life_local.db";
    let conn = init_db(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM trainer_genomes WHERE run_id = ?1",
        params![run_id],
    ).map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
fn apply_champion(genome: String) -> Result<bool, String> {
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

fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();

    tauri::Builder::default()
        .manage(tx)
        .invoke_handler(tauri::generate_handler![
            handle_client_action,
            get_registered_species,
            get_trainer_runs,
            get_trainer_population,
            get_trainer_hof,
            save_trainer_generation,
            clear_trainer_history,
            apply_champion
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").expect("Failed to locate main window");

            // Start parallel headless simulation and replication thread
            thread::spawn(move || {
                let db_path = "pixel_life_local.db";
                let conn = init_db(db_path).expect("Failed to initialize local DB");

                let emit_state = |payload: serde_json::Value| {
                    if let Err(e) = window.app_handle().emit("simulation-state", payload) {
                        eprintln!("[SIMULATION] Error emitting 'simulation-state' event: {}", e);
                    }
                };

                let logical_width = 19200.0;
                let logical_height = 10800.0;
                
                let app_config = src_tauri::shared::types::AppConfig::load();
                
                let mut is_running = false;
                let mut creatures: Vec<CreatureAgent> = Vec::new();
                let mut food_pellets: Vec<FoodSpore> = Vec::new();
                let mut nutrient_centers: Vec<(f32, f32)> = Vec::new();
                {
                    let mut rng_seed = rand::thread_rng();
                    for _ in 0..12 {
                        let cx = rng_seed.gen_range(1000.0..logical_width - 1000.0);
                        let cy = rng_seed.gen_range(1000.0..logical_height - 1000.0);
                        nutrient_centers.push((cx, cy));
                    }
                }
                let mut newly_spawned_creatures: Vec<CreatureAgent> = Vec::new();
                let mut selected_agent_id: Option<u32> = None;
                let mut next_creature_id = 1;
                let mut next_spore_id = 1000;
                let mut highest_generation = 1;

                let mut is_trainer_active = false;
                let mut trainer_is_running = false;
                let mut trainer_warp_speed = 1;
                #[allow(non_snake_case)]
                let mut trainer_N = 16;
                let mut trainer_elite_ratio = 0.25;
                let mut trainer_mutation_rate = 0.15;
                let mut trainer_inflow_rate = 0.10;
                let mut trainer_hof_rate = 0.10;
                let mut trainer_multi_trial = false;
                let mut trainer_is_headless = false;

                let mut trainer_sandboxes: Vec<TrainerSandbox> = Vec::new();
                let mut trainer_generation = 1;
                let mut trainer_epoch_ticks = 0;
                let mut trainer_run_id = "default_run".to_string();
                let mut trainer_selected_sandbox_id: Option<u32> = Some(1);

                let rebuild_sandbox_grid = |n: usize, run_id: &str, generation_val: u32, mut_rate: f32, db_conn: &rusqlite::Connection, elite_ratio: f32, inflow_rate: f32, hof_rate: f32| -> (Vec<TrainerSandbox>, u32) {
                    let mut sandboxes = Vec::with_capacity(n);
                    let mut restored_generation = generation_val;
                    
                    // Try to load active parent genomes (elites) from SQLite
                    let mut parent_genomes = Vec::new();
                    if let Ok(mut stmt) = db_conn.prepare("SELECT genome, generation FROM trainer_genomes WHERE run_id = ?1 AND active = 1 ORDER BY fitness DESC LIMIT ?2") {
                        if let Ok(rows) = stmt.query_map(params![run_id, n as i32], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))) {
                            for row in rows {
                                if let Ok((genome, gen_val)) = row {
                                    parent_genomes.push(genome);
                                    restored_generation = restored_generation.max(gen_val as u32);
                                }
                            }
                        }
                    }

                    // Try to load Hall of Fame genomes from SQLite
                    let mut hof_genomes = Vec::new();
                    if let Ok(mut stmt) = db_conn.prepare("SELECT genome FROM trainer_genomes WHERE run_id = ?1 ORDER BY fitness DESC LIMIT ?2") {
                        if let Ok(rows) = stmt.query_map(params![run_id, n as i32], |row| row.get::<_, String>(0)) {
                            for row in rows {
                                if let Ok(genome) = row {
                                    hof_genomes.push(genome);
                                }
                            }
                        }
                    }

                    let base_dna = "HJKLABCDPQRS1234EFGHTRUSTANDBENDPROGENITORALIFEWELLFORMEDMEMBRANEFOURIERSEGMENTSHARMONICSWAVEPHASEPULSESTIFFNESS";

                    let elite_count = ((n as f32) * elite_ratio).round() as usize;
                    let elite_count = elite_count.clamp(1, n);

                    let hof_count = ((n as f32) * hof_rate).round() as usize;
                    let inflow_count = ((n as f32) * inflow_rate).round() as usize;

                    for idx in 0..n {
                        let id = (idx + 1) as u32;

                        if parent_genomes.is_empty() {
                            // Gen 1: Greenfield start
                            let sb = init_rust_sandbox(
                                id,
                                base_dna,
                                restored_generation,
                                mut_rate,
                                "random", // ALL are fresh random wildtypes on Gen 1!
                                1000.0,
                                1000.0,
                            );
                            sandboxes.push(sb);
                        } else {
                            // Gen 2+: Slider-controlled composition matching TS nextGenPlans 1:1!
                            let (parent_genome, origin, actual_mut_rate) = if idx < elite_count {
                                if idx < parent_genomes.len() {
                                    (parent_genomes[idx].clone(), "elite", 0.0) // elites are unmodified clones!
                                } else {
                                    (parent_genomes[0].clone(), "mutant", mut_rate)
                                }
                            } else if idx < elite_count + hof_count {
                                let hof_idx = idx - elite_count;
                                if hof_idx < hof_genomes.len() {
                                    (hof_genomes[hof_idx].clone(), "hof", mut_rate)
                                } else {
                                    (parent_genomes[0].clone(), "mutant", mut_rate)
                                }
                            } else if idx < elite_count + hof_count + inflow_count {
                                (base_dna.to_string(), "random", 0.0) // random immigrants are pure new wildtypes
                            } else {
                                let source_idx = (idx - elite_count - hof_count - inflow_count) % parent_genomes.len();
                                (parent_genomes[source_idx].clone(), "mutant", mut_rate)
                            };

                            let sb = init_rust_sandbox(
                                id,
                                &parent_genome,
                                restored_generation,
                                actual_mut_rate,
                                origin,
                                1000.0,
                                1000.0,
                            );
                            sandboxes.push(sb);
                        }
                    }
                    (sandboxes, restored_generation)
                };

                // Ensure the default desktop session exists to satisfy foreign key constraints
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO sessions (id, name, config_json) VALUES (?1, ?2, ?3)",
                    params!["desktop-session-001", "Desktop Evolution Session", "{}"],
                );

                let mut rng = rand::thread_rng();

                // Try to load saved state from the database (matches TS original's persistent session restore!)
                let mut loaded_state_success = false;
                if let Ok(mut stmt) = conn.prepare("SELECT state_json FROM simulation_state WHERE key = 'current_state'") {
                    if let Ok(mut rows) = stmt.query([]) {
                        if let Ok(Some(row)) = rows.next() {
                            if let Ok(state_str) = row.get::<_, String>(0) {
                                if let Ok(state_val) = serde_json::from_str::<serde_json::Value>(&state_str) {
                                    if let Some(c_arr) = state_val["creatures"].as_array() {
                                        if let Some(f_arr) = state_val["foodPellets"].as_array() {
                                            if let Ok(deser_creatures) = serde_json::from_value::<Vec<CreatureAgent>>(serde_json::Value::Array(c_arr.clone())) {
                                                if let Ok(deser_food) = serde_json::from_value::<Vec<FoodSpore>>(serde_json::Value::Array(f_arr.clone())) {
                                                    if !deser_creatures.is_empty() {
                                                        creatures = deser_creatures;
                                                        food_pellets = deser_food;
                                                        highest_generation = state_val["highestGeneration"].as_u64().unwrap_or(1) as u32;
                                                        next_creature_id = state_val["nextCreatureId"].as_u64().unwrap_or(1) as u32;
                                                        next_spore_id = state_val["nextSporeId"].as_u64().unwrap_or(1000) as u32;
                                                        loaded_state_success = true;
                                                        println!("[SIMULATION] Successfully restored persistent session from database! creatures={}, spores={}", creatures.len(), food_pellets.len());
                                                    } else {
                                                        println!("[SIMULATION] Restored session has empty creatures. Falling back to fresh organic wildtype seeding!");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !loaded_state_success {
                    // Clean the database schema on start to avoid fossil conflicts
                    let _ = conn.execute("DELETE FROM species_records", []);

                    // Inject 25 unique biological wildtype cells with randomized genomes (matches TS original!)
                    for _ in 0..25 {
                        let px = rng.gen_range(500.0..logical_width - 500.0);
                        let py = rng.gen_range(500.0..logical_height - 500.0);
                        let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                        let random_dna = generate_random_genome(256);
                        let random_pheno = parse_genome(&random_dna, None, None);

                        creatures.push(CreatureAgent {
                            id: next_creature_id,
                            species_id: random_dna.clone(),
                            px,
                            py,
                            vx: rng.gen_range(-0.4..0.4),
                            vy: rng.gen_range(-0.4..0.4),
                            heading_angle,
                            bend_angle: 0.0,
                            omega_rot: 0.0,
                            energy: 150.0,
                            adrenaline: 1.0,
                            age: 0,
                            generation: 1,
                            has_eaten: true,
                            genome: random_dna.clone(),
                            antisense: String::new(),
                            phenotype: random_pheno.clone(),
                            neuron_states: Vec::new(),
                            neuron_activations: Vec::new(),
                        });

                        // Seed each unique species into local SQLite database
                        let now_millis = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        let _ = conn.execute(
                            "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                            params![&random_dna, &random_pheno.latin_name, &random_dna, None::<String>, now_millis, random_pheno.carnivory],
                        );

                        next_creature_id += 1;
                    }

                    // Inject Initial Spores with patchy density distribution (nutrient centers!)
                    for _ in 0..app_config.food_spore_count {
                        let (x, y) = {
                            let roll = rng.gen_range(0.0..1.0);
                            if !nutrient_centers.is_empty() && roll < 0.75 {
                                // 75% Chance: Spawn close to one of our nutrient centers (high density algae forest!)
                                let center_idx = rng.gen_range(0..nutrient_centers.len());
                                let (cx, cy) = nutrient_centers[center_idx];
                                let radius = rng.gen_range(50.0..600.0);
                                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                let px = (cx + radius * angle.cos()).clamp(100.0, logical_width - 100.0);
                                let py = (cy + radius * angle.sin()).clamp(100.0, logical_height - 100.0);
                                (px, py)
                            } else {
                                // 25% Chance: Loose scattered spores in open ocean (low density!)
                                let px = rng.gen_range(100.0..logical_width - 100.0);
                                let py = rng.gen_range(100.0..logical_height - 100.0);
                                (px, py)
                            }
                        };

                        food_pellets.push(FoodSpore {
                            id: next_spore_id,
                            x,
                            y,
                            amount: 15.0,
                            vx: rng.gen_range(-0.15..0.15),
                            vy: rng.gen_range(-0.15..0.15),
                        });
                        next_spore_id += 1;
                    }
                }

                // Instantly notify frontend to fetch the newly seeded species roster!
                emit_state(json!({ "type": "DATABASE_CHANGED" }));

                let mut last_emit_time = Instant::now();
                let mut last_food_emit_time = Instant::now();
                let mut last_history_log_time = Instant::now();
                let emit_interval = Duration::from_millis(40); // 25Hz UI Stream
                let food_emit_interval = Duration::from_millis(500); // 2Hz Food Stream
                let sim_tick_interval = Duration::from_millis(16); // ~60Hz Physics

                loop {
                    let start_tick = Instant::now();

                    // Process UI Actions
                    while let Ok(action_str) = rx.try_recv() {
                        if let Ok(action) = serde_json::from_str::<serde_json::Value>(&action_str) {
                            if let Some(act_type) = action["type"].as_str() {
                                match act_type {
                                    "CLIENT_READY" => {
                                        is_trainer_active = false; // Safely force switch back to Ocean mode!
                                        println!("[SIMULATION] Client Handshake successful! Synchronizing {} creatures and {} spores...", creatures.len(), food_pellets.len());
                                        let init_json = json!({
                                            "type": "INIT_STATE",
                                            "highestGeneration": highest_generation,
                                            "running": is_running,
                                            "seed": "ocean-tauri-seed-77",
                                            "rules": {
                                                "sporeEnergy": 15.0
                                            },
                                            "foodPellets": food_pellets,
                                            "creatures": creatures
                                        });
                                        emit_state(init_json);
                                        emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                        emit_state(json!({
                                            "type": "LOG_EVENT",
                                            "message": "Handshake successful. Ecosystem synchronized.",
                                            "logType": "system"
                                        }));
                                    }
                                    "TOGGLE_SIMULATION" => {
                                        if let Some(running) = action["running"].as_bool() {
                                            is_running = running;
                                            emit_state(json!({ "type": "SIM_STATE", "running": is_running }));
                                        }
                                    }
                                    "SPAWN_FOOD" => {
                                        if let (Some(x), Some(y)) = (action["x"].as_f64(), action["y"].as_f64()) {
                                            food_pellets.push(FoodSpore {
                                                id: next_spore_id,
                                                x: x as f32,
                                                y: y as f32,
                                                amount: 15.0,
                                                vx: 0.0,
                                                vy: 0.0,
                                            });
                                            next_spore_id += 1;
                                        }
                                    }
                                    "INJECT_URZELLE" => {
                                        let mut rng = rand::thread_rng();
                                        let px = rng.gen_range(500.0..logical_width - 500.0);
                                        let py = rng.gen_range(500.0..logical_height - 500.0);
                                        let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                                        let random_dna = generate_random_genome(256);
                                        let random_pheno = parse_genome(&random_dna, None, None);

                                        let new_agent = CreatureAgent {
                                            id: next_creature_id,
                                            species_id: random_dna.clone(),
                                            px,
                                            py,
                                            vx: rng.gen_range(-0.4..0.4),
                                            vy: rng.gen_range(-0.4..0.4),
                                            heading_angle,
                                            bend_angle: 0.0,
                                            omega_rot: 0.0,
                                            energy: random_pheno.stomach_capacity * 0.80,
                                            adrenaline: 1.0,
                                            age: 0,
                                            generation: 1,
                                            has_eaten: false,
                                            genome: random_dna.clone(),
                                            antisense: String::new(),
                                            phenotype: random_pheno.clone(),
                                            neuron_states: Vec::new(),
                                            neuron_activations: Vec::new(),
                                        };

                                        // Seed unique species in local SQLite database
                                        let now_millis = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs() as i64;
                                        let _ = conn.execute(
                                            "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                            params![&random_dna, &random_pheno.latin_name, &random_dna, None::<String>, now_millis, random_pheno.carnivory],
                                        );

                                        emit_state(json!({ "type": "DATABASE_CHANGED" }));

                                        creatures.push(new_agent.clone());
                                        newly_spawned_creatures.push(new_agent.clone());
                                        emit_state(json!({ "type": "LOG_EVENT", "message": format!("Injected brand new biological Wildtype Specimen #{} ({}).", next_creature_id, random_pheno.latin_name), "logType": "system" }));
                                        next_creature_id += 1;
                                    }
                                    "RESET_EVOLUTION" => {
                                        creatures.clear();
                                        food_pellets.clear();
                                        highest_generation = 1;
                                        next_creature_id = 1;
                                        next_spore_id = 1000;
                                        selected_agent_id = None;

                                        let mut rng = rand::thread_rng();

                                        // Wipe local species ledger DB table
                                        let _ = conn.execute("DELETE FROM species_records", []);

                                        // Inject 25 unique biological wildtype cells with randomized genomes (matches TS original!)
                                        for _ in 0..25 {
                                            let px = rng.gen_range(500.0..logical_width - 500.0);
                                            let py = rng.gen_range(500.0..logical_height - 500.0);
                                            let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                                            let random_dna = generate_random_genome(256);
                                            let random_pheno = parse_genome(&random_dna, None, None);

                                            creatures.push(CreatureAgent {
                                                id: next_creature_id,
                                                species_id: random_dna.clone(),
                                                px,
                                                py,
                                                vx: rng.gen_range(-0.4..0.4),
                                                vy: rng.gen_range(-0.4..0.4),
                                                heading_angle,
                                                bend_angle: 0.0,
                                                omega_rot: 0.0,
                                                energy: 150.0,
                                                adrenaline: 1.0,
                                                age: 0,
                                                generation: 1,
                                                has_eaten: true,
                                                genome: random_dna.clone(),
                                                antisense: String::new(),
                                                phenotype: random_pheno.clone(),
                                                neuron_states: Vec::new(),
                                                neuron_activations: Vec::new(),
                                            });

                                            // Seed each unique species into local SQLite database
                                            let now_millis = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs() as i64;
                                            let _ = conn.execute(
                                                "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                                params![&random_dna, &random_pheno.latin_name, &random_dna, None::<String>, now_millis, random_pheno.carnivory],
                                            );

                                            next_creature_id += 1;
                                        }

                                        // Regenerate 12 random nutrient centers for the reset biosphere
                                        nutrient_centers.clear();
                                        for _ in 0..12 {
                                            let cx = rng.gen_range(1000.0..logical_width - 1000.0);
                                            let cy = rng.gen_range(1000.0..logical_height - 1000.0);
                                            nutrient_centers.push((cx, cy));
                                        }

                                        // Inject Initial Spores with patchy density distribution (nutrient centers!)
                                        for _ in 0..app_config.food_spore_count {
                                            let (x, y) = {
                                                let roll = rng.gen_range(0.0..1.0);
                                                if !nutrient_centers.is_empty() && roll < 0.75 {
                                                    // 75% Chance: Spawn close to one of our nutrient centers (high density algae forest!)
                                                    let center_idx = rng.gen_range(0..nutrient_centers.len());
                                                    let (cx, cy) = nutrient_centers[center_idx];
                                                    let radius = rng.gen_range(50.0..600.0);
                                                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                                    let px = (cx + radius * angle.cos()).clamp(100.0, logical_width - 100.0);
                                                    let py = (cy + radius * angle.sin()).clamp(100.0, logical_height - 100.0);
                                                    (px, py)
                                                } else {
                                                    // 25% Chance: Loose scattered spores in open ocean (low density!)
                                                    let px = rng.gen_range(100.0..logical_width - 100.0);
                                                    let py = rng.gen_range(100.0..logical_height - 100.0);
                                                    (px, py)
                                                }
                                            };

                                            food_pellets.push(FoodSpore {
                                                id: next_spore_id,
                                                x,
                                                y,
                                                amount: 15.0,
                                                vx: rng.gen_range(-0.15..0.15),
                                                vy: rng.gen_range(-0.15..0.15),
                                            });
                                            next_spore_id += 1;
                                        }

                                        emit_state(json!({ "type": "DATABASE_CHANGED" }));

                                        // Emit full fresh INIT_STATE so client successfully overwrites its local lists
                                        let init_json = json!({
                                            "type": "INIT_STATE",
                                            "highestGeneration": highest_generation,
                                            "running": is_running,
                                            "seed": "ocean-tauri-seed-77",
                                            "rules": {
                                                "sporeEnergy": 15.0
                                            },
                                            "foodPellets": food_pellets,
                                            "creatures": creatures
                                        });
                                        emit_state(init_json);

                                        emit_state(json!({ "type": "LOG_EVENT", "message": "Evolution reset. Biosphere repopulated with 15 unique wildtype cells.", "logType": "system" }));
                                    }
                                    "SELECT_AGENT" => {
                                        if let Some(agent_id) = action["id"].as_u64() {
                                            selected_agent_id = Some(agent_id as u32);
                                        } else {
                                            selected_agent_id = None;
                                        }
                                    }
                                    "CLIENT_ERROR" => {
                                        if let Some(err_str) = action["error"].as_str() {
                                            eprintln!("[CLIENT ERROR DESKTOP] Received webview exception: {}", err_str);
                                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                                .create(true)
                                                .append(true)
                                                .open("client_debug.log")
                                            {
                                                use std::io::Write;
                                                let _ = writeln!(file, "CLIENT EXCEPTION: {}", err_str);
                                            }
                                        }
                                    }
                                    "CLIENT_LOG" => {
                                        if let Some(msg) = action["message"].as_str() {
                                            println!("[CLIENT LOG] {}", msg);
                                        }
                                    }
                                    "SET_MODE" => {
                                        if let Some(mode) = action["mode"].as_str() {
                                            if mode == "trainer" {
                                                is_trainer_active = true;
                                                // Initialize sandboxes grid in Rust when entering trainer
                                                let (sbs, restored_gen) = rebuild_sandbox_grid(trainer_N, &trainer_run_id, trainer_generation, trainer_mutation_rate, &conn, trainer_elite_ratio, trainer_inflow_rate, trainer_hof_rate);
                                                trainer_sandboxes = sbs;
                                                trainer_generation = restored_gen;
                                                println!("[TRAINER] Rebuilt sandbox grid with {} chambers for run '{}', restored Gen {}", trainer_N, trainer_run_id, trainer_generation);
                                                
                                                // Broadcast full hyperparams truth back to the UI!
                                                emit_state(json!({
                                                    "type": "TRAINER_STATE_CHANGED",
                                                    "isRunning": trainer_is_running,
                                                    "N": trainer_N,
                                                    "warpSpeed": trainer_warp_speed,
                                                    "eliteRatio": trainer_elite_ratio,
                                                    "mutationRate": trainer_mutation_rate,
                                                    "inflowRate": trainer_inflow_rate,
                                                    "hofRate": trainer_hof_rate,
                                                    "multiTrial": trainer_multi_trial,
                                                    "isHeadless": trainer_is_headless,
                                                    "runId": trainer_run_id
                                                }));
                                            } else {
                                                is_trainer_active = false;
                                                println!("[TRAINER] Leaving trainer mode. Resuming Ocean substrate.");
                                            }
                                        }
                                    }
                                    "START_TRAINING" => {
                                        trainer_is_running = true;
                                        emit_state(json!({
                                            "type": "TRAINER_STATE_CHANGED",
                                            "isRunning": true,
                                            "N": trainer_N,
                                            "warpSpeed": trainer_warp_speed,
                                            "eliteRatio": trainer_elite_ratio,
                                            "mutationRate": trainer_mutation_rate,
                                            "inflowRate": trainer_inflow_rate,
                                            "hofRate": trainer_hof_rate,
                                            "multiTrial": trainer_multi_trial,
                                            "isHeadless": trainer_is_headless,
                                            "runId": trainer_run_id
                                        }));
                                    }
                                    "PAUSE_TRAINING" => {
                                        trainer_is_running = false;
                                        emit_state(json!({
                                            "type": "TRAINER_STATE_CHANGED",
                                            "isRunning": false,
                                            "N": trainer_N,
                                            "warpSpeed": trainer_warp_speed,
                                            "eliteRatio": trainer_elite_ratio,
                                            "mutationRate": trainer_mutation_rate,
                                            "inflowRate": trainer_inflow_rate,
                                            "hofRate": trainer_hof_rate,
                                            "multiTrial": trainer_multi_trial,
                                            "isHeadless": trainer_is_headless,
                                            "runId": trainer_run_id
                                        }));
                                    }
                                    "TRAINER_RESET" => {
                                        trainer_is_running = false;
                                        trainer_generation = 1;
                                        trainer_epoch_ticks = 0;
                                        
                                        // Delete all saved training genomes for the current run_id in SQLite (hard reset!)
                                        let _ = conn.execute("DELETE FROM trainer_genomes WHERE run_id = ?1", params![trainer_run_id]);

                                        let (sbs, _) = rebuild_sandbox_grid(trainer_N, &trainer_run_id, 1, trainer_mutation_rate, &conn, trainer_elite_ratio, trainer_inflow_rate, trainer_hof_rate);
                                        trainer_sandboxes = sbs;
                                        emit_state(json!({
                                            "type": "TRAINER_RESET_COMPLETED",
                                            "generation": 1,
                                            "isRunning": false,
                                            "N": trainer_N,
                                            "warpSpeed": trainer_warp_speed,
                                            "eliteRatio": trainer_elite_ratio,
                                            "mutationRate": trainer_mutation_rate,
                                            "inflowRate": trainer_inflow_rate,
                                            "hofRate": trainer_hof_rate,
                                            "multiTrial": trainer_multi_trial,
                                            "isHeadless": trainer_is_headless,
                                            "runId": trainer_run_id
                                        }));
                                    }
                                    "SELECT_TRAINER_SANDBOX" => {
                                        if let Some(sb_id) = action["id"].as_u64() {
                                            trainer_selected_sandbox_id = Some(sb_id as u32);
                                        }
                                    }
                                    "UPDATE_TRAINER_HYPERPARAMS" => {
                                        let mut grid_changed = false;

                                        if let Some(n) = action["N"].as_u64() {
                                            let prev_n = trainer_N;
                                            trainer_N = n as usize;
                                            if trainer_N != prev_n {
                                                grid_changed = true;
                                            }
                                        }
                                        if let Some(ws) = action["warpSpeed"].as_u64() {
                                            trainer_warp_speed = ws as u32;
                                        }
                                        if let Some(er) = action["eliteRatio"].as_f64() {
                                            trainer_elite_ratio = er as f32;
                                        }
                                        if let Some(mr) = action["mutationRate"].as_f64() {
                                            trainer_mutation_rate = mr as f32;
                                        }
                                        if let Some(ir) = action["inflowRate"].as_f64() {
                                            trainer_inflow_rate = ir as f32;
                                        }
                                        if let Some(hr) = action["hofRate"].as_f64() {
                                            trainer_hof_rate = hr as f32;
                                        }
                                        if let Some(mt) = action["multiTrial"].as_bool() {
                                            trainer_multi_trial = mt;
                                        }
                                        if let Some(hl) = action["isHeadless"].as_bool() {
                                            trainer_is_headless = hl;
                                        }
                                        if let Some(r_id) = action["runId"].as_str() {
                                            let prev_run = trainer_run_id.clone();
                                            trainer_run_id = r_id.to_string();
                                            if trainer_run_id != prev_run {
                                                trainer_generation = 1;
                                                trainer_epoch_ticks = 0;
                                                grid_changed = true;
                                            }
                                        }

                                        if grid_changed {
                                            let (sbs, restored_gen) = rebuild_sandbox_grid(trainer_N, &trainer_run_id, trainer_generation, trainer_mutation_rate, &conn, trainer_elite_ratio, trainer_inflow_rate, trainer_hof_rate);
                                            trainer_sandboxes = sbs;
                                            trainer_generation = restored_gen;
                                        }

                                        // Broadcast the validated hyperparameters back to the UI (Self-Healing UI!)
                                        emit_state(json!({
                                            "type": "TRAINER_STATE_CHANGED",
                                            "isRunning": trainer_is_running,
                                            "N": trainer_N,
                                            "warpSpeed": trainer_warp_speed,
                                            "eliteRatio": trainer_elite_ratio,
                                            "mutationRate": trainer_mutation_rate,
                                            "inflowRate": trainer_inflow_rate,
                                            "hofRate": trainer_hof_rate,
                                            "multiTrial": trainer_multi_trial,
                                            "isHeadless": trainer_is_headless,
                                            "runId": trainer_run_id
                                        }));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    if !is_trainer_active {
                        if is_running {
                            // 1. Move and drift food spores
                        for pellet in &mut food_pellets {
                            pellet.vx *= 0.95;
                            pellet.vy *= 0.95;
                            pellet.x += pellet.vx;
                            pellet.y += pellet.vy;

                            if pellet.x < 8.0 { pellet.x = 8.0; pellet.vx *= -0.5; }
                            else if pellet.x > logical_width - 8.0 { pellet.x = logical_width - 8.0; pellet.vx *= -0.5; }

                            if pellet.y < 8.0 { pellet.y = 8.0; pellet.vy *= -0.5; }
                            else if pellet.y > logical_height - 8.0 { pellet.y = logical_height - 8.0; pellet.vy *= -0.5; }
                        }

                        // Populate Spatial Grid for collisions
                        let mut grid = SpatialGrid::new(logical_width, logical_height, 80.0);
                        for pellet in &food_pellets {
                            grid.insert_food(pellet.id, pellet.x, pellet.y);
                        }
                        for agent in &creatures {
                            grid.insert_creature(agent.id, agent.px, agent.py);
                        }

                        let alive_clones = creatures.clone();
                        let mut damage_map = std::collections::HashMap::<u32, f32>::new();

                        let mut next_creatures = Vec::new();

                        // 2. Physics & Biology Loop over all creatures
                        for mut agent in creatures.drain(..) {
                            agent.age += 1;
                            
                            // Passive metabolism + Adrenaline surcharge (aligned with config rules!)
                            let metabolic_surcharge = 1.0 + (agent.adrenaline - 1.0) * app_config.rules.adrenaline_metabolic_surcharge_scale;
                            let bmr_decay = agent.phenotype.basal_metabolic_rate * app_config.rules.bmr_base_scale * app_config.basal_metabolic_rate_multiplier * metabolic_surcharge;
                            agent.energy -= bmr_decay;

                            // Photosynthesis for green creatures in light zone
                            let hue = agent.phenotype.primary_color.h;
                            let is_green_prey = (agent.phenotype.carnivory < app_config.rules.biting_carnivory_threshold) && (hue >= 75.0 && hue <= 175.0);
                            let in_light_zone = agent.py < logical_height * 0.35;
                            if is_green_prey && in_light_zone {
                                agent.energy = (agent.energy + app_config.rules.photosynthesis_energy_gain).min(agent.phenotype.stomach_capacity);
                            }

                            // Spatial Temperature Stratification and Thermal Limits stress penalty
                            let local_temp = 38.0 - (agent.py / logical_height) * 26.0;
                            let temp_min = agent.phenotype.thermal_tolerance_min;
                            let temp_max = agent.phenotype.thermal_tolerance_max;
                            let thermal_stress = if local_temp < temp_min { temp_min - local_temp } else if local_temp > temp_max { local_temp - temp_max } else { 0.0 };
                            if thermal_stress > 0.1 {
                                agent.energy -= thermal_stress * app_config.rules.thermal_stress_penalty_scale;
                            }

                            if agent.energy <= 0.0 || agent.age >= app_config.rules.creature_max_age_ticks {
                                // Decompose corpse into nutrient spores (elastic spore relocation, no array deletion!)
                                let l_dead = agent.phenotype.spinal_harmonics.base_length;
                                let r_dead = agent.phenotype.spinal_harmonics.mean_radius;
                                let num_pellets = (((l_dead * r_dead) / app_config.rules.decomposition_size_ratio).floor() as i32)
                                    .clamp(app_config.rules.decomposition_spore_min, app_config.rules.decomposition_spore_max);

                                let mut rng = rand::thread_rng();
                                for _ in 0..num_pellets {
                                    if !food_pellets.is_empty() {
                                        let p_idx = rng.gen_range(0..food_pellets.len());
                                        food_pellets[p_idx].x = agent.px + rng.gen_range(-16.0..16.0);
                                        food_pellets[p_idx].y = agent.py + rng.gen_range(-16.0..16.0);
                                        food_pellets[p_idx].vx = rng.gen_range(-0.2..0.2);
                                        food_pellets[p_idx].vy = rng.gen_range(-0.2..0.2);
                                    }
                                }

                                emit_state(json!({
                                    "type": "LOG_EVENT",
                                    "message": format!("✝️ Corpse Decomposition! Species #{} {} died. {} nutrient spores released.", agent.id, if agent.energy <= 0.0 { "starved" } else { "of old age" }, num_pellets),
                                    "logType": "mutation"
                                }));
                                continue;
                            }

                            // CTRNN Sensors & Brain (Dynamically construct inputs matching true physical sensors)
                            let k = agent.phenotype.organelles.len();
                            let mut inputs = vec![0.0; k + 1];
                            for i in 0..k {
                                inputs[i] = 0.5; // default moderate stimulus
                            }
                            inputs[k] = 0.5; // clock / hunger
                            
                            let outputs = execute_brain(&agent.phenotype.brain, &inputs, &mut agent.neuron_states, &mut agent.neuron_activations);
                            let out_thrust = outputs[0];
                            let out_left = outputs[1];

                            let mean_radius = agent.phenotype.spinal_harmonics.mean_radius;
                            let base_length = agent.phenotype.spinal_harmonics.base_length;
                            let mass = mean_radius.powf(1.5) * (base_length / 25.0);
                            let drag_forward = mean_radius * 0.015;
                            
                            // Upscale motor thrust force using biological phenotype properties (pulse speed, segments, and limbs)
                            let pulse = agent.phenotype.pulse_speed;
                            let wave_phase = agent.phenotype.wave_phase;
                            let mut thrust_mag = agent.phenotype.stiffness * (pulse * 1000.0 * pulse * 1000.0) * 6.0;
                            let eta_swim = ((base_length / (mean_radius * 3.5)) * wave_phase.sin().max(0.01) * agent.phenotype.stiffness).clamp(0.1, 3.2);
                            thrust_mag *= eta_swim;

                            let limbs_count = agent.phenotype.organelles.iter().filter(|o| o.expression_style >= 0.72).count() as f32;
                            thrust_mag *= 1.0 + limbs_count * 0.12;
                            thrust_mag *= 1.0 + agent.phenotype.spinal_harmonics.parapodia_amp * 1.0;

                            let net_thrust = out_thrust * thrust_mag;

                            // Apply Native Rust Physics
                            apply_creature_physics(&mut agent, net_thrust, out_left, mass, drag_forward, 0.0, 0.0, logical_width, logical_height);

                            // Grazing (Eat Food)
                            let eat_radius = mean_radius * app_config.rules.grazing_radius_multiplier + app_config.rules.grazing_radius_offset;
                            let nearby_foods = grid.get_nearby_food(agent.px, agent.py, eat_radius);
                            for f_id in nearby_foods {
                                if let Some(idx) = food_pellets.iter().position(|x| x.id == f_id) {
                                    let f = &food_pellets[idx];
                                    let dx = f.x - agent.px;
                                    let dy = f.y - agent.py;
                                    if (dx*dx + dy*dy).sqrt() <= eat_radius {
                                        let herbivore_efficiency = 1.0 - agent.phenotype.carnivory;
                                        if herbivore_efficiency > 0.05 {
                                            let energy_gain = 15.0 * herbivore_efficiency * app_config.rules.grazing_efficiency_herbivore_scale;
                                            agent.energy = (agent.energy + energy_gain).min(agent.phenotype.stomach_capacity);
                                            agent.has_eaten = true;

                                            // Relocate / biological-respawn the spore (no array deletion!) with patchy density distribution!
                                            let mut rng = rand::thread_rng();
                                            let (new_x, new_y) = {
                                                let roll = rng.gen_range(0.0..1.0);
                                                if !nutrient_centers.is_empty() && roll < 0.75 {
                                                    let center_idx = rng.gen_range(0..nutrient_centers.len());
                                                    let (cx, cy) = nutrient_centers[center_idx];
                                                    let radius = rng.gen_range(50.0..600.0);
                                                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                                    let px = (cx + radius * angle.cos()).clamp(100.0, logical_width - 100.0);
                                                    let py = (cy + radius * angle.sin()).clamp(100.0, logical_height - 100.0);
                                                    (px, py)
                                                } else {
                                                    let px = rng.gen_range(100.0..logical_width - 100.0);
                                                    let py = rng.gen_range(100.0..logical_height - 100.0);
                                                    (px, py)
                                                }
                                            };
                                            let new_vx = rng.gen_range(-0.15..0.15);
                                            let new_vy = rng.gen_range(-0.15..0.15);

                                            let mutable_f = &mut food_pellets[idx];
                                            mutable_f.x = new_x;
                                            mutable_f.y = new_y;
                                            mutable_f.vx = new_vx;
                                            mutable_f.vy = new_vy;

                                            // Broadcast Eat event for client-side algae sparks
                                            emit_state(json!({
                                                "type": "EAT_EVENT",
                                                "agentId": agent.id,
                                                "x": new_x,
                                                "y": new_y
                                            }));
                                        }
                                    }
                                }
                            }

                            // Fight Biting (Predation)
                            if agent.phenotype.carnivory >= app_config.rules.biting_carnivory_threshold {
                                let bite_range = mean_radius * app_config.rules.biting_radius_multiplier * 0.5 + app_config.rules.biting_radius_offset;
                                let nearby_peers = grid.get_nearby_creatures(agent.px, agent.py, bite_range);
                                for v_id in nearby_peers {
                                    if v_id == agent.id { continue; }
                                    if let Some(victim) = alive_clones.iter().find(|c| c.id == v_id) {
                                        if victim.species_id != agent.species_id {
                                            let dx = victim.px - agent.px;
                                            let dy = victim.py - agent.py;
                                            let dist = (dx*dx + dy*dy).sqrt();
                                            if dist <= bite_range {
                                                // Record damage (configured in config.json!)
                                                *damage_map.entry(victim.id).or_insert(0.0) += app_config.rules.biting_energy_damage;

                                                // Attacker gain energy
                                                let carnivore_efficiency = agent.phenotype.carnivory;
                                                let energy_gain = app_config.rules.biting_base_energy_gain * carnivore_efficiency * app_config.rules.biting_efficiency_carnivore_scale;
                                                agent.energy = (agent.energy + energy_gain).min(agent.phenotype.stomach_capacity);
                                                agent.has_eaten = true;

                                                // Broadcast bite shockwave ring
                                                emit_state(json!({
                                                    "type": "BITE_EVENT",
                                                    "attackerId": agent.id,
                                                    "victimId": victim.id,
                                                    "x": victim.px,
                                                    "y": victim.py
                                                }));

                                                emit_state(json!({
                                                    "type": "LOG_EVENT",
                                                    "message": format!("⚡ [BITE ATTACK] {} #{} bites #{}! (+{:.0}nJ / -{:.0}nJ damage)", agent.phenotype.latin_name.chars().take(16).collect::<String>(), agent.id, victim.id, energy_gain.round(), app_config.rules.biting_energy_damage),
                                                    "logType": "mutation"
                                                }));
                                            }
                                        }
                                    }
                                }
                            }

                            // Asexual Reproduction (Mitosis)
                            let can_reproduce = agent.age >= agent.phenotype.mature_age && agent.has_eaten;
                            let threshold = agent.phenotype.stomach_capacity * agent.phenotype.repro_threshold;
                            
                            if can_reproduce && agent.energy >= threshold {
                                agent.energy *= app_config.rules.reproduction_split_loss_ratio;
                                agent.has_eaten = false;
                                
                                let recoil = agent.phenotype.split_loss * app_config.rules.reproduction_recoil_velocity_scale;
                                agent.vx += recoil * (agent.heading_angle + std::f32::consts::PI).cos();
                                agent.vy += recoil * (agent.heading_angle + std::f32::consts::PI).sin();

                                let mut child = agent.clone();
                                child.id = next_creature_id;
                                next_creature_id += 1;
                                child.vx += recoil * agent.heading_angle.cos();
                                child.vy += recoil * agent.heading_angle.sin();
                                child.age = 0;
                                child.generation += 1;
                                child.has_eaten = false;
                                
                                if child.generation > highest_generation {
                                    highest_generation = child.generation;
                                }

                                // Mutate
                                if let Some((mutated_dna, _, _, _)) = mutate_genome(&child.genome) {
                                    child.genome = mutated_dna.clone();
                                    child.phenotype = parse_genome(&mutated_dna, None, None);
                                    child.neuron_states.clear();
                                    child.neuron_activations.clear();
                                    
                                    // Save new mutated species record to local DB
                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64;
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
                                        params![
                                            &mutated_dna, 
                                            &child.phenotype.latin_name, 
                                            &mutated_dna, 
                                            Some(agent.genome.clone()),
                                            now_millis,
                                            child.generation as i32,
                                            child.phenotype.carnivory
                                        ],
                                    );
                                    emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                }

                                // Push to newly_spawned_creatures for synchronous, race-free spawning in TELEMETRY_TICK!
                                newly_spawned_creatures.push(child.clone());

                                next_creatures.push(child);
                            }

                            next_creatures.push(agent);
                        }

                        creatures = next_creatures;

                        // Apply recorded bite damages
                        for agent in &mut creatures {
                            if let Some(&damage) = damage_map.get(&agent.id) {
                                agent.energy = (agent.energy - damage).max(0.0);
                            }
                        }

                        // Maintain population (restocking founder cells up to config, matches TS index.ts!)
                        let target_population = app_config.target_population;
                        while creatures.len() < target_population {
                            let mut rng = rand::thread_rng();
                            
                            // Query all top training champions from the database
                            let mut training_champs = Vec::new();
                            if let Ok(mut stmt) = conn.prepare(
                                "SELECT t1.genome, t1.generation FROM trainer_genomes t1
                                 INNER JOIN (
                                     SELECT run_id, MAX(fitness) as max_fit
                                     FROM trainer_genomes
                                     GROUP BY run_id
                                 ) t2 ON t1.run_id = t2.run_id AND t1.fitness = t2.max_fit"
                            ) {
                                if let Ok(rows) = stmt.query_map([], |row| {
                                    Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
                                }) {
                                    for row in rows {
                                        if let Ok((genome, gen_val)) = row {
                                            training_champs.push((genome, gen_val));
                                        }
                                    }
                                }
                            }

                            // Query successful alive wild species in DB to clone
                            let mut cached_alive_species = Vec::new();
                            if let Ok(mut stmt) = conn.prepare("SELECT genome_string, generation FROM species_records WHERE status = 'alive'") {
                                if let Ok(rows) = stmt.query_map([], |row| {
                                    Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?))
                                }) {
                                    for row in rows {
                                        if let Ok((genome, generation_val)) = row {
                                            cached_alive_species.push((genome, generation_val));
                                        }
                                    }
                                }
                            }

                            let (g, generation_val, source_tag) = {
                                let roll = rng.gen_range(0.0..1.0);
                                if !training_champs.is_empty() && roll < 0.40 {
                                    // 40% Chance: Spawn one of our trained champions from the database!
                                    let idx = rng.gen_range(0..training_champs.len());
                                    (training_champs[idx].0.clone(), training_champs[idx].1 as u32, "🏆 Evolved Champion")
                                } else if !cached_alive_species.is_empty() && roll < 0.80 {
                                    // 40% Chance: Clone an already successful species living in the ocean
                                    let idx = rng.gen_range(0..cached_alive_species.len());
                                    (cached_alive_species[idx].0.clone(), cached_alive_species[idx].1 as u32, "🧬 Wild Clone")
                                } else {
                                    // 20% Chance: Wild random founder mutation
                                    (generate_random_genome(256), 1, "🌱 Random Founder")
                                }
                            };

                            let random_pheno = parse_genome(&g, None, None);
                            let px = rng.gen_range(500.0..logical_width - 500.0);
                            let py = rng.gen_range(500.0..logical_height - 500.0);
                            let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                            let new_restocked = CreatureAgent {
                                id: next_creature_id,
                                species_id: g.clone(),
                                px,
                                py,
                                vx: rng.gen_range(-0.4..0.4),
                                vy: rng.gen_range(-0.4..0.4),
                                heading_angle,
                                bend_angle: 0.0,
                                omega_rot: 0.0,
                                energy: random_pheno.stomach_capacity * app_config.rules.restock_initial_stomach_ratio,
                                adrenaline: 1.0,
                                age: 0,
                                generation: generation_val,
                                has_eaten: false,
                                genome: g.clone(),
                                antisense: String::new(),
                                phenotype: random_pheno.clone(),
                                neuron_states: Vec::new(),
                                neuron_activations: Vec::new(),
                            };

                            creatures.push(new_restocked.clone());
                            newly_spawned_creatures.push(new_restocked.clone());

                            // Seed each unique species into local SQLite database if not exists
                            let now_millis = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64;
                            let _ = conn.execute(
                                "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
                                params![&g, &random_pheno.latin_name, &g, None::<String>, now_millis, generation_val, random_pheno.carnivory],
                            );

                            emit_state(json!({ "type": "DATABASE_CHANGED" }));
                            emit_state(json!({
                                "type": "LOG_EVENT",
                                "message": format!("Restocked population: Spawned {} Gen {} ({}).", source_tag, generation_val, random_pheno.latin_name),
                                "logType": "system"
                            }));

                            next_creature_id += 1;
                        }

                        // Extinction and Peak Population Tracking (matches TS index.ts!)
                        let mut db_changed = false;
                        let active_ids: std::collections::HashSet<String> = creatures
                            .iter()
                            .map(|c| c.species_id.clone())
                            .collect();

                        // Query all species records currently in DB to track status changes (defensive match, no unwraps!)
                        if let Ok(mut stmt) = conn.prepare("SELECT id, status, peak_population FROM species_records") {
                            let rows = stmt.query_map([], |row| {
                                Ok((
                                    row.get::<_, String>(0)?,
                                    row.get::<_, String>(1)?,
                                    row.get::<_, i32>(2)?,
                                ))
                            });
                            if let Ok(rows) = rows {
                                let mut updates = Vec::new();
                                for row in rows {
                                    if let Ok((id, status, peak_pop)) = row {
                                        if status == "alive" && !active_ids.contains(&id) {
                                            // Species went extinct!
                                            let now_millis = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs() as i64;
                                            updates.push(json!({
                                                "id": id,
                                                "action": "extinct",
                                                "extinction_time": now_millis
                                            }));
                                            db_changed = true;
                                        } else if active_ids.contains(&id) {
                                            let current_count = creatures.iter().filter(|c| c.species_id == id).count() as i32;
                                            if current_count > peak_pop {
                                                updates.push(json!({
                                                    "id": id,
                                                    "action": "peak",
                                                    "peak_population": current_count
                                                }));
                                                db_changed = true;
                                            }
                                        }
                                    }
                                }

                                // Apply database updates
                                for update in updates {
                                    let id = update["id"].as_str().unwrap();
                                    if update["action"] == "extinct" {
                                        let extinct_time = update["extinction_time"].as_i64().unwrap();
                                        let _ = conn.execute(
                                            "UPDATE species_records SET status = 'extinct', extinction_time = ?1 WHERE id = ?2",
                                            params![extinct_time, id],
                                        );
                                        
                                        // Fetch name for logging
                                        if let Ok(mut n_stmt) = conn.prepare("SELECT latin_name FROM species_records WHERE id = ?1") {
                                            if let Ok(name) = n_stmt.query_row(params![id], |r| r.get::<_, String>(0)) {
                                                emit_state(json!({
                                                    "type": "LOG_EVENT",
                                                    "message": format!("✝️ Extinction Event! Species '{}' has gone completely extinct. The only relics remain in the silent fossil archive.", name),
                                                    "logType": "mutation"
                                                }));
                                            }
                                        }
                                    } else if update["action"] == "peak" {
                                        let peak_pop = update["peak_population"].as_i64().unwrap();
                                        let _ = conn.execute(
                                            "UPDATE species_records SET peak_population = ?1 WHERE id = ?2",
                                            params![peak_pop, id],
                                        );
                                    }
                                }
                            }
                        }

                        if db_changed {
                            emit_state(json!({ "type": "DATABASE_CHANGED" }));
                        }

                        // Broadcast State to UI (Throttled 25Hz, lightweight creatures only!)
                        if last_emit_time.elapsed() >= emit_interval {
                            let telemetry_creatures: Vec<TelemetryCreature> = creatures
                                .iter()
                                .map(|c| TelemetryCreature::from(c))
                                .collect();

                            // Compute selected brain live activations if an agent is selected
                            let selected_brain_json = if let Some(sel_id) = selected_agent_id {
                                if let Some(sel_agent) = creatures.iter().find(|c| c.id == sel_id) {
                                    json!({
                                        "id": sel_id,
                                        "activations": sel_agent.neuron_activations
                                    })
                                } else {
                                    json!(null)
                                }
                            } else {
                                json!(null)
                            };

                            let telemetry_json = json!({
                                "type": "TELEMETRY_TICK",
                                "highestGeneration": highest_generation,
                                "creatures": telemetry_creatures,
                                "newCreatures": newly_spawned_creatures,
                                "selectedBrain": selected_brain_json
                            });
                            emit_state(telemetry_json);
                            newly_spawned_creatures.clear(); // Clear newborns after sending
                            last_emit_time = Instant::now();
                        }

                        // Broadcast Food Spores to UI (Throttled 2Hz, slow drifting food!)
                        if last_food_emit_time.elapsed() >= food_emit_interval {
                            emit_state(json!({
                                "type": "FOOD_TICK",
                                "foodPellets": food_pellets
                            }));
                            last_food_emit_time = Instant::now();
                        }

                        // Log Simulation History to SQLite DB every 10 seconds (continuous time-series)
                        if last_history_log_time.elapsed() >= Duration::from_secs(10) {
                            if !creatures.is_empty() {
                                let pop_count = creatures.len() as i32;
                                let avg_energy: f32 = creatures.iter().map(|c| c.energy).sum::<f32>() / pop_count as f32;
                                let max_energy: f32 = creatures.iter().map(|c| c.energy).fold(0.0, |a, b| a.max(b));

                                let _ = conn.execute(
                                    "INSERT INTO simulation_history (session_id, generation, average_fitness, max_fitness, population_count) VALUES (?1, ?2, ?3, ?4, ?5)",
                                    params![
                                        "desktop-session-001",
                                        highest_generation as i32,
                                        avg_energy,
                                        max_energy,
                                        pop_count
                                    ],
                                );

                                // Save current simulation state JSON to the database (session persistence!)
                                let state_val = json!({
                                    "creatures": creatures,
                                    "foodPellets": food_pellets,
                                    "highestGeneration": highest_generation,
                                    "nextCreatureId": next_creature_id,
                                    "nextSporeId": next_spore_id
                                });
                                if let Ok(state_str) = serde_json::to_string(&state_val) {
                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as i64;
                                    let _ = conn.execute(
                                        "INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES ('current_state', ?1, ?2)",
                                        params![state_str, now_millis],
                                    );
                                }
                            }
                            last_history_log_time = Instant::now();
                        }
                    }
                    } else {
                        if trainer_is_running {
                            // Run trainer_warp_speed physics steps
                            for _step in 0..trainer_warp_speed {
                                if !trainer_is_running {
                                    break;
                                }

                                trainer_epoch_ticks += 1;

                                // Step all sandbox physical loops in Rust!
                                for sb in &mut trainer_sandboxes {
                                    step_trainer_sandbox_physics(sb, 1000.0, 1000.0);
                                }

                                // If we completed the 300-tick epoch:
                                if trainer_epoch_ticks >= 300 {
                                    // 1. Calculate fitness for all sandboxes
                                    for sb in &mut trainer_sandboxes {
                                        let is_carnivore = sb.agent.phenotype.carnivory >= 0.35;
                                        let target_idx = if is_carnivore { 1 } else { 0 };
                                        let cur_dist = ((sb.foods[target_idx].x - sb.agent.px).powi(2) + (sb.foods[target_idx].y - sb.agent.py).powi(2)).sqrt();

                                        sb.current_fitness = calculate_sandbox_fitness(
                                            sb.finished,
                                            sb.finish_tick,
                                            300,
                                            sb.start_distance,
                                            sb.distance_traveled,
                                            sb.wall_collisions,
                                            cur_dist,
                                            sb.agent.px,
                                            sb.agent.py,
                                        );
                                    }

                                    // Sort sandboxes by fitness (highest first)
                                    trainer_sandboxes.sort_by(|a, b| b.current_fitness.partial_cmp(&a.current_fitness).unwrap_or(std::cmp::Ordering::Equal));

                                    let best_fit = trainer_sandboxes[0].current_fitness;
                                    let avg_fit: f32 = trainer_sandboxes.iter().map(|sb| sb.current_fitness).sum::<f32>() / (trainer_N as f32);

                                    println!("[TRAINER RUN] Gen {} completed! Best Fit: {:.1}, Avg Fit: {:.1}", trainer_generation, best_fit, avg_fit);

                                    // 2. Save Elite Champions to local SQLite
                                    let elite_count = (((trainer_N as f32) * trainer_elite_ratio).round() as usize).clamp(1, trainer_N);
                                    
                                    // Mark previous active genomes as inactive
                                    let _ = conn.execute("UPDATE trainer_genomes SET active = 0 WHERE run_id = ?1", params![trainer_run_id]);

                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64;

                                    // Save the elites to SQLite
                                    for (idx, sb) in trainer_sandboxes.iter().take(elite_count).enumerate() {
                                        let unique_id = format!("{}-{}-{}-{}", trainer_run_id, trainer_generation, idx, now_millis);
                                        let _ = conn.execute(
                                            "INSERT INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at)
                                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
                                            params![
                                                unique_id,
                                                trainer_run_id,
                                                trainer_generation as i32,
                                                sb.agent.phenotype.latin_name,
                                                sb.agent.genome,
                                                sb.current_fitness as f64,
                                                now_millis
                                            ],
                                        );
                                    }

                                    // 3. Broadcast GENERATION_COMPLETED to UI
                                    emit_state(json!({
                                        "type": "TRAINER_GENERATION_COMPLETED",
                                        "generation": trainer_generation,
                                        "bestFitness": best_fit,
                                        "avgFitness": avg_fit,
                                        "bestGenome": trainer_sandboxes[0].agent.genome
                                    }));

                                    // Increment generation and reset epoch ticks
                                    trainer_generation += 1;
                                    trainer_epoch_ticks = 0;

                                    // Rebuild sandbox grid in Rust!
                                    let (sbs, _) = rebuild_sandbox_grid(trainer_N, &trainer_run_id, trainer_generation, trainer_mutation_rate, &conn, trainer_elite_ratio, trainer_inflow_rate, trainer_hof_rate);
                                    trainer_sandboxes = sbs;

                                    // If headless is OFF, let the UI know it has been rebuilt
                                    emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                    
                                    // Automatically restart training loop for next gen
                                    trainer_is_running = true;
                                }
                            }
                        }

                        // Broadcast Trainer Telemetry Tick to UI (Lightweight 25Hz!)
                        if last_emit_time.elapsed() >= emit_interval {
                            let telemetry_sandboxes: Vec<TrainerTelemetrySandbox> = trainer_sandboxes
                                .iter()
                                .map(|sb| {
                                    TrainerTelemetrySandbox {
                                        id: sb.id,
                                        px: sb.agent.px,
                                        py: sb.agent.py,
                                        vx: sb.agent.vx,
                                        vy: sb.agent.vy,
                                        heading_angle: sb.agent.heading_angle,
                                        omega_rot: sb.agent.omega_rot,
                                        finished: sb.finished,
                                        current_fitness: sb.current_fitness,
                                        origin_type: sb.origin_type.clone(),
                                        consumed_spore_type: sb.consumed_spore_type.clone(),
                                        foods: sb.foods.clone(),
                                        latin_name: sb.agent.phenotype.latin_name.clone(),
                                        primary_color_h: sb.agent.phenotype.primary_color.h,
                                        primary_color_s: sb.agent.phenotype.primary_color.s,
                                        primary_color_l: sb.agent.phenotype.primary_color.l,
                                        energy: sb.agent.energy,
                                        age: sb.agent.age,
                                        generation: sb.agent.generation,
                                        genome: sb.agent.genome.clone(),
                                    }
                                })
                                .collect();

                            // Compute selected sandbox brain live activations if focused
                            let selected_brain_json = if let Some(sel_id) = trainer_selected_sandbox_id {
                                if let Some(sel_sb) = trainer_sandboxes.iter().find(|sb| sb.id == sel_id) {
                                    json!({
                                        "id": sel_id,
                                        "activations": sel_sb.agent.neuron_activations,
                                        "states": sel_sb.agent.neuron_states
                                    })
                                } else {
                                    json!(null)
                                }
                            } else {
                                json!(null)
                            };

                            let time_str = format!("{:.1}s", ((300.0 - trainer_epoch_ticks as f32) / 60.0).max(0.0));

                            emit_state(json!({
                                "type": "TRAINER_TELEMETRY_TICK",
                                "generation": trainer_generation,
                                "sandboxes": telemetry_sandboxes,
                                "selectedBrain": selected_brain_json,
                                "timeStr": time_str,
                                "isRunning": trainer_is_running,
                            }));

                            last_emit_time = Instant::now();
                        }
                    }

                    // Physics loop pacing (60Hz)
                    let elapsed = start_tick.elapsed();
                    if elapsed < sim_tick_interval {
                        thread::sleep(sim_tick_interval - elapsed);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
