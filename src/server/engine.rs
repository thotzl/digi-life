use tauri::Emitter;
use std::thread;
use std::time::{Duration, Instant};
use serde_json::json;
use std::sync::mpsc::Receiver;
use rusqlite::params;
use rand::Rng;

use crate::shared::types::{CreatureAgent, FoodSpore, TelemetryCreature};
use crate::shared::spatial_grid::SpatialGrid;
use crate::shared::map_generator::generate_world;
use crate::biology::dna::{parse_genome, mutate_genome, generate_structured_genome};
use crate::biology::creature_factory::{inherit_learned_weights, spawn_from_phenotype, SpawnOptions};
use crate::biology::scenarios::Scenario;
use crate::biology::trainer_engine::TrainerTelemetrySandbox;
use crate::biology::trainer_sim::{TrainerEvent, TrainerParams, TrainerSim};
use crate::database::{init_db, load_trainer_genomes, save_trainer_elites, LogDbError};
use crate::shared::rng::seed_from_str;

/// Per-step snapshot of the fields other creatures read (prey search, bite targets).
struct PeerSnapshot {
    id: u32,
    species_id: String,
    px: f32,
    py: f32,
    vx: f32,
    vy: f32,
}

impl From<&CreatureAgent> for PeerSnapshot {
    fn from(c: &CreatureAgent) -> Self {
        Self { id: c.id, species_id: c.species_id.clone(), px: c.px, py: c.py, vx: c.vx, vy: c.vy }
    }
}

/// TRAINER_STATE_CHANGED / TRAINER_RESET_COMPLETED payload: the trainer settings as the UI shows them.
fn trainer_state_payload(event_type: &str, is_running: bool, params: &TrainerParams, warp_speed: usize, is_headless: bool, run_id: &str) -> serde_json::Value {
    json!({
        "type": event_type,
        "isRunning": is_running,
        "N": params.population,
        "warpSpeed": warp_speed,
        "eliteRatio": params.elite_ratio,
        "mutationRate": params.mutation_rate,
        "inflowRate": params.inflow_rate,
        "hofRate": params.hof_rate,
        "multiTrial": params.multi_trial,
        "isHeadless": is_headless,
        "runId": run_id,
        "lamarckian": params.lamarckian
    })
}

pub fn spawn_simulation_thread(window: tauri::WebviewWindow, rx: Receiver<String>) {
    thread::spawn(move || {
        let db_path = crate::database::db_path();
        let conn = init_db(&db_path).expect("Failed to initialize local DB");

        // Events are buffered and flushed once per frame (in order); repeated DATABASE_CHANGED
        // notifications within one frame are coalesced since the UI only refetches the roster.
        let pending_events = std::cell::RefCell::new(Vec::<serde_json::Value>::new());
        let emit_state = |payload: serde_json::Value| {
            pending_events.borrow_mut().push(payload);
        };
        let flush_events = || {
            let mut database_changed_sent = false;
            for payload in pending_events.borrow_mut().drain(..) {
                if payload["type"] == "DATABASE_CHANGED" {
                    if database_changed_sent {
                        continue;
                    }
                    database_changed_sent = true;
                }
                if let Err(e) = window.emit("simulation-state", payload) {
                    eprintln!("[SIMULATION] Error emitting 'simulation-state' event: {}", e);
                }
            }
        };

        let logical_width = 19200.0;
        let logical_height = 10800.0;
        let app_config = crate::shared::types::AppConfig::global();
        let ocean_world = generate_world(&app_config.seed, logical_width, logical_height);
        let mut grid = SpatialGrid::new(logical_width, logical_height, 80.0);

        // Mirror of species_records (id, status, peak) in SELECT order; reloaded only after inserts/resets
        let mut species_cache: Vec<(String, String, i32)> = Vec::new();
        let mut species_cache_dirty = true;

        let mut is_running = false;
        let mut ocean_warp_factor = 1;
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

        // Trainer: TrainerSim owns population, epochs and breeding; the engine keeps the UI state,
        // DB persistence and telemetry
        let mut is_trainer_active = false;
        let mut trainer_is_running = false;
        let mut trainer_warp_speed: usize = 1;
        let mut trainer_is_headless = false; // only echoed back to the UI
        let mut trainer_run_id = "default_run".to_string();
        let mut trainer_selected_sandbox_id: Option<u32> = Some(1);
        let trainer_retention = app_config.trainer_genome_retention_per_run;
        let trainer_seed = |run_id: &str| seed_from_str(&format!("{}:{}", app_config.seed, run_id));

        // Continues a run from the DB: active parents + hall of fame; the generation counter
        // continues from the newest parent
        let restore_trainer = |params: TrainerParams, run_id: &str, generation: u32, db_conn: &rusqlite::Connection| -> TrainerSim {
            let parents = load_trainer_genomes(db_conn, run_id, true, params.population);
            let hof = load_trainer_genomes(db_conn, run_id, false, trainer_retention);
            let generation = parents.iter().map(|p| p.generation).fold(generation, u32::max);
            TrainerSim::restore(params, trainer_seed(run_id), generation, parents, hof, trainer_retention)
        };
        let mut trainer = TrainerSim::new(TrainerParams::default(), trainer_seed(&trainer_run_id), trainer_retention);

        // Ensure the default desktop session exists to satisfy foreign key constraints
        conn.execute(
            "INSERT OR IGNORE INTO sessions (id, name, config_json) VALUES (?1, ?2, ?3)",
            params!["desktop-session-001", "Tauri Desktop Laboratory Studio", "{}"],
        ).log_db_err("INSERT OR IGNORE INTO sessions");

        // Try restoring previous simulation state on startup (session persistence!)
        let mut loaded_state_success = false;
        if let Ok(mut state_stmt) = conn.prepare("SELECT state_json FROM simulation_state WHERE key = 'current_state'") {
            if let Ok(state_str) = state_stmt.query_row([], |row| row.get::<_, String>(0)) {
                if let Ok(state_val) = serde_json::from_str::<serde_json::Value>(&state_str) {
                    if let Some(restored_creatures) = state_val["creatures"].as_array() {
                        if let Ok(creatures_list) = serde_json::from_value::<Vec<CreatureAgent>>(serde_json::Value::Array(restored_creatures.clone())) {
                            creatures = creatures_list;
                            loaded_state_success = true;
                            println!("[SIMULATION] Restored {} creatures successfully from database!", creatures.len());
                        }
                    }
                    if let Some(restored_spores) = state_val["foodPellets"].as_array() {
                        if let Ok(spores_list) = serde_json::from_value::<Vec<FoodSpore>>(serde_json::Value::Array(restored_spores.clone())) {
                            food_pellets = spores_list;
                        }
                    }
                    if let Some(gen_val) = state_val["highestGeneration"].as_u64() {
                        highest_generation = gen_val as u32;
                    }
                    if let Some(c_id) = state_val["nextCreatureId"].as_u64() {
                        next_creature_id = c_id as u32;
                    }
                    if let Some(s_id) = state_val["nextSporeId"].as_u64() {
                        next_spore_id = s_id as u32;
                    }
                }
            }
        }

        // Seeding initial 15 wildtype progenitor cells if starting fresh
        if !loaded_state_success {
            let mut rng = rand::thread_rng();
            let base_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";

            for _ in 0..15 {
                let mut mutated_dna = base_dna.to_string();
                if rng.gen_range(0.0..1.0) < 0.60 {
                    if let Some(mutation) = mutate_genome(&base_dna, &app_config.mutation_operators, &mut rng) {
                        mutated_dna = mutation.genome;
                    }
                }

                let random_pheno = parse_genome(&mutated_dna, None, None);
                let px = rng.gen_range(500.0..logical_width - 500.0);
                let py = rng.gen_range(500.0..logical_height - 500.0);
                let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                let mut new_agent = spawn_from_phenotype(&mutated_dna, random_pheno.clone(), SpawnOptions {
                    id: next_creature_id,
                    px,
                    py,
                    heading_angle,
                    generation: 1,
                    methylations: None,
                    synapse_weights: None,
                });
                new_agent.vx = rng.gen_range(-0.4..0.4);
                new_agent.vy = rng.gen_range(-0.4..0.4);

                creatures.push(new_agent.clone());

                let now_millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                conn.execute(
                    "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                    params![&mutated_dna, &random_pheno.latin_name, &mutated_dna, None::<String>, now_millis, random_pheno.carnivory],
                ).log_db_err("INSERT OR IGNORE INTO species_records");
                species_cache_dirty = true;

                next_creature_id += 1;
            }

            // Inject Initial Spores with patchy density distribution (nutrient centers!)
            for _ in 0..app_config.food_spore_count {
                let (x, y) = {
                    let mut rng = rand::thread_rng();
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

                food_pellets.push(FoodSpore {
                    id: next_spore_id,
                    type_id: 1, // Plant/Algae
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
        let emit_interval = Duration::from_millis(40); // 25Hz broadcast rate
        let food_emit_interval = Duration::from_millis(500); // 2Hz food broadcast rate

        loop {
            let loop_start = Instant::now();

            // Process UI Actions
            while let Ok(action_str) = rx.try_recv() {
                if let Ok(action) = serde_json::from_str::<serde_json::Value>(&action_str) {
                    if let Some(act_type) = action["type"].as_str() {
                        match act_type {
                            "CLIENT_READY" => {
                                // The mode stays with the router (SET_MODE): forcing Ocean here raced a fast
                                // navigation to the trainer after startup and left the trainer inactive
                                println!("[SIMULATION] Client Handshake successful! Synchronizing {} creatures and {} spores...", creatures.len(), food_pellets.len());
                                let world = ocean_world.clone();
                                let init_json = json!({
                                    "type": "INIT_STATE",
                                    "highestGeneration": highest_generation,
                                    "running": is_running,
                                    "seed": app_config.seed,
                                    "world": world,
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
                            "INJECT_URZELLE" | "SPAWN_CLONE" if creatures.len() >= app_config.max_creatures => {
                                emit_state(json!({
                                    "type": "LOG_EVENT",
                                    "message": format!("Population cap reached ({} creatures), spawn rejected.", app_config.max_creatures),
                                    "logType": "system"
                                }));
                            }
                            "INJECT_URZELLE" => {
                                let mut rng = rand::thread_rng();
                                let random_dna = generate_structured_genome(&app_config.genome_composition, &mut rng);
                                let random_pheno = parse_genome(&random_dna, None, None);

                                let px = rng.gen_range(500.0..logical_width - 500.0);
                                let py = rng.gen_range(500.0..logical_height - 500.0);
                                let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                                let mut new_agent = spawn_from_phenotype(&random_dna, random_pheno.clone(), SpawnOptions {
                                    id: next_creature_id,
                                    px,
                                    py,
                                    heading_angle,
                                    generation: 1,
                                    methylations: None,
                                    synapse_weights: None,
                                });
                                new_agent.vx = rng.gen_range(-0.4..0.4);
                                new_agent.vy = rng.gen_range(-0.4..0.4);

                                creatures.push(new_agent.clone());
                                newly_spawned_creatures.push(new_agent.clone());

                                let now_millis = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64;
                                conn.execute(
                                    "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                    params![&random_dna, &random_pheno.latin_name, &random_dna, None::<String>, now_millis, random_pheno.carnivory],
                                ).log_db_err("INSERT OR IGNORE INTO species_records");
                                species_cache_dirty = true;

                                emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                emit_state(json!({
                                    "type": "LOG_EVENT",
                                    "message": format!("Released wildtype organism: Strain #{} '{}'.", next_creature_id, random_pheno.latin_name),
                                    "logType": "mutation"
                                }));
                                next_creature_id += 1;
                            }
                            "SPAWN_CLONE" => {
                                let mut rng = rand::thread_rng();
                                if let Some(genome) = action["genome"].as_str() {
                                    let load_learned_synapses = action["load_learned_synapses"].as_bool().unwrap_or(false);
                                    
                                    // Parse parent methylations if any
                                    let m_vec: Vec<f32> = serde_json::from_value(action["methylations"].clone()).unwrap_or_default();
                                    let m_opt = if m_vec.is_empty() { None } else { Some(&m_vec[..]) };
                                    
                                    let pheno = parse_genome(genome, None, m_opt);
                                    
                                    let px = rng.gen_range(500.0..logical_width - 500.0);
                                    let py = rng.gen_range(500.0..logical_height - 500.0);
                                    let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                    
                                    let learned_synapses = if load_learned_synapses {
                                        serde_json::from_value::<Vec<f32>>(action["synapse_weights"].clone()).ok()
                                    } else {
                                        None
                                    };

                                    let mut new_agent = spawn_from_phenotype(genome, pheno.clone(), SpawnOptions {
                                        id: next_creature_id,
                                        px,
                                        py,
                                        heading_angle,
                                        generation: 1,
                                        methylations: m_opt,
                                        synapse_weights: learned_synapses,
                                    });
                                    new_agent.vx = rng.gen_range(-0.4..0.4);
                                    new_agent.vy = rng.gen_range(-0.4..0.4);

                                    creatures.push(new_agent.clone());
                                    newly_spawned_creatures.push(new_agent);

                                    // Record the newly released clone in species_records if not exists
                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64;
                                    conn.execute(
                                        "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                        params![genome, &pheno.latin_name, genome, None::<String>, now_millis, pheno.carnivory],
                                    ).log_db_err("INSERT OR IGNORE INTO species_records");
                                    species_cache_dirty = true;

                                    emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                    emit_state(json!({
                                        "type": "LOG_EVENT",
                                        "message": format!("Released catalogue specimen: Strain #{} '{}' (Pre-trained: {}).", next_creature_id, pheno.latin_name, load_learned_synapses),
                                        "logType": "system"
                                    }));
                                    next_creature_id += 1;
                                }
                            }
                            "RESET_EVOLUTION" => {
                                println!("[SIMULATION] Performing complete environmental restoration reset...");
                                
                                // Wipe old species records, histories, and state caches from SQLite database
                                conn.execute("DELETE FROM species_records", []).log_db_err("DELETE FROM species_records");
                                species_cache_dirty = true;
                                conn.execute("DELETE FROM simulation_history", []).log_db_err("DELETE FROM simulation_history");
                                conn.execute("DELETE FROM simulation_state", []).log_db_err("DELETE FROM simulation_state");

                                creatures.clear();
                                food_pellets.clear();
                                newly_spawned_creatures.clear(); // Clear newborns to prevent phantom shivering creatures!
                                next_creature_id = 1;
                                highest_generation = 1;

                                let mut rng = rand::thread_rng();

                                // Query all saved catalogue creatures
                                let mut catalogue_creatures = Vec::new();
                                if let Ok(mut stmt) = conn.prepare("SELECT genome, name, methylations, synapse_weights FROM creature_catalogue") {
                                    if let Ok(rows) = stmt.query_map([], |row| {
                                        Ok((
                                            row.get::<_, String>(0)?,
                                            row.get::<_, String>(1)?,
                                            row.get::<_, String>(2)?,
                                            row.get::<_, String>(3)?
                                        ))
                                    }) {
                                        for row in rows {
                                            if let Ok(data) = row {
                                                catalogue_creatures.push(data);
                                            }
                                        }
                                    }
                                }

                                if !catalogue_creatures.is_empty() {
                                    for _ in 0..25 {
                                        let idx = rng.gen_range(0..catalogue_creatures.len());
                                        let (genome, _name, m_str, s_str) = &catalogue_creatures[idx];
                                        
                                        let m_arr: Vec<f32> = serde_json::from_str(m_str).unwrap_or_default();
                                        let s_arr: Vec<f32> = serde_json::from_str(s_str).unwrap_or_default();
                                        let m_opt = if m_arr.is_empty() { None } else { Some(&m_arr[..]) };
                                        
                                        let pheno = parse_genome(genome, None, m_opt);
                                        
                                        let px = rng.gen_range(500.0..logical_width - 500.0);
                                        let py = rng.gen_range(500.0..logical_height - 500.0);
                                        let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                                        let mut new_agent = spawn_from_phenotype(genome, pheno.clone(), SpawnOptions {
                                            id: next_creature_id,
                                            px,
                                            py,
                                            heading_angle,
                                            generation: 1,
                                            methylations: m_opt,
                                            synapse_weights: Some(s_arr),
                                        });
                                        new_agent.vx = rng.gen_range(-0.4..0.4);
                                        new_agent.vy = rng.gen_range(-0.4..0.4);

                                        creatures.push(new_agent.clone());

                                        let now_millis = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs() as i64;
                                        conn.execute(
                                            "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                            params![genome, &pheno.latin_name, genome, None::<String>, now_millis, pheno.carnivory],
                                        ).log_db_err("INSERT OR IGNORE INTO species_records");
                                        species_cache_dirty = true;
                                        next_creature_id += 1;
                                    }
                                    emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                    emit_state(json!({ "type": "LOG_EVENT", "message": format!("Evolution reset. Biosphere repopulated with 25 clones from Catalogue."), "logType": "system" }));
                                } else {
                                    // Fallback to structured random genomes if the catalogue is empty (TCK-184)
                                    for _ in 0..25 {
                                        let founder_dna = generate_structured_genome(&app_config.genome_composition, &mut rng);

                                        let random_pheno = parse_genome(&founder_dna, None, None);
                                        let px = rng.gen_range(500.0..logical_width - 500.0);
                                        let py = rng.gen_range(500.0..logical_height - 500.0);
                                        let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                                        let mut new_agent = spawn_from_phenotype(&founder_dna, random_pheno.clone(), SpawnOptions {
                                            id: next_creature_id,
                                            px,
                                            py,
                                            heading_angle,
                                            generation: 1,
                                            methylations: None,
                                            synapse_weights: None,
                                        });
                                        new_agent.vx = rng.gen_range(-0.4..0.4);
                                        new_agent.vy = rng.gen_range(-0.4..0.4);

                                        creatures.push(new_agent.clone());

                                        let now_millis = std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs() as i64;
                                        conn.execute(
                                            "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                            params![founder_dna, &random_pheno.latin_name, founder_dna, None::<String>, now_millis, random_pheno.carnivory],
                                        ).log_db_err("INSERT OR IGNORE INTO species_records");
                                        species_cache_dirty = true;
                                        next_creature_id += 1;
                                    }
                                    emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                    emit_state(json!({ "type": "LOG_EVENT", "message": "Evolution reset. Biosphere repopulated with 25 structured random founder cells.", "logType": "system" }));
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

                                    food_pellets.push(FoodSpore {
                                        id: next_spore_id,
                                        type_id: 1, // Plant/Algae
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
                                let world = ocean_world.clone();
                                let init_json = json!({
                                    "type": "INIT_STATE",
                                    "highestGeneration": highest_generation,
                                    "running": is_running,
                                    "seed": app_config.seed,
                                    "world": world,
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
                            "SET_SIMULATION_SPEED" => {
                                if let Some(speed) = action["speed"].as_u64() {
                                    ocean_warp_factor = speed as usize;
                                }
                            }
                            "SET_TRAINER_MODE" => {
                                if let Some(m) = action["mode"].as_str() {
                                    let scenario = Scenario::from_mode(m);
                                    if scenario != trainer.params.scenario {
                                        trainer.set_scenario(scenario);
                                        let size = scenario.plugin().chamber_size();
                                        println!("[TRAINER] Switched training mode to '{}' (Chamber size: {}x{})", m, size, size);
                                    }
                                }
                            }
                            "SET_MODE" => {
                                if let Some(mode) = action["mode"].as_str() {
                                    if mode == "trainer" {
                                        is_trainer_active = true;
                                        // Rebuild the population from the DB when entering the trainer
                                        trainer = restore_trainer(trainer.params.clone(), &trainer_run_id, trainer.generation, &conn);
                                        println!("[TRAINER] Rebuilt sandbox grid with {} chambers for run '{}', restored Gen {}", trainer.params.population, trainer_run_id, trainer.generation);

                                        // Broadcast full hyperparams truth back to the UI!
                                        emit_state(trainer_state_payload("TRAINER_STATE_CHANGED", trainer_is_running, &trainer.params, trainer_warp_speed, trainer_is_headless, &trainer_run_id));
                                    } else {
                                        is_trainer_active = false;
                                        println!("[TRAINER] Leaving trainer mode. Resuming Ocean substrate.");
                                    }
                                }
                            }
                            "START_TRAINING" => {
                                trainer_is_running = true;
                                emit_state(trainer_state_payload("TRAINER_STATE_CHANGED", true, &trainer.params, trainer_warp_speed, trainer_is_headless, &trainer_run_id));
                            }
                            "PAUSE_TRAINING" => {
                                trainer_is_running = false;
                                emit_state(trainer_state_payload("TRAINER_STATE_CHANGED", false, &trainer.params, trainer_warp_speed, trainer_is_headless, &trainer_run_id));
                            }
                            "TRAINER_RESET" => {
                                trainer_is_running = false;

                                // Delete all saved training genomes for the current run_id in SQLite (hard reset!)
                                conn.execute("DELETE FROM trainer_genomes WHERE run_id = ?1", params![trainer_run_id]).log_db_err("DELETE FROM trainer_genomes");
                                trainer = TrainerSim::new(trainer.params.clone(), trainer_seed(&trainer_run_id), trainer_retention);

                                let mut payload = trainer_state_payload("TRAINER_RESET_COMPLETED", false, &trainer.params, trainer_warp_speed, trainer_is_headless, &trainer_run_id);
                                payload["generation"] = json!(1);
                                emit_state(payload);
                            }
                            "SELECT_TRAINER_SANDBOX" => {
                                if let Some(sb_id) = action["id"].as_u64() {
                                    trainer_selected_sandbox_id = Some(sb_id as u32);
                                } else {
                                    trainer_selected_sandbox_id = None;
                                }
                            }
                            "ASSIGN_SANDBOX_CREATURE" => {
                                if let Some(sb_id) = action["sandbox_id"].as_u64() {
                                    if let Some(genome) = action["genome"].as_str() {
                                        let load_syn = action["load_learned_synapses"].as_bool().unwrap_or(false);
                                        let m_vec: Vec<f32> = serde_json::from_value(action["methylations"].clone()).unwrap_or_default();
                                        let m_opt = if m_vec.is_empty() { None } else { Some(&m_vec[..]) };
                                        // Learned weights only on request; the factory falls back to the genome weights on a length mismatch
                                        let s_opt = if load_syn {
                                            Some(serde_json::from_value::<Vec<f32>>(action["synapse_weights"].clone()).unwrap_or_default())
                                        } else {
                                            None
                                        };

                                        let generation = trainer.generation;
                                        if let Some(sb) = trainer.assign_creature(sb_id as u32, genome, m_opt, s_opt) {
                                            // Also save this assigned creature as active in trainer_genomes SQLite database for this run!
                                            let unique_id = format!("{}-{}-assigned-{}", trainer_run_id, generation, sb_id);
                                            let methylations_json = serde_json::to_string(&sb.agent.phenotype.methylations).unwrap_or_else(|_| "[]".to_string());
                                            let synapse_weights_json = serde_json::to_string(&sb.agent.synapse_weights).unwrap_or_else(|_| "[]".to_string());
                                            let now_millis = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

                                            conn.execute(
                                                "INSERT OR REPLACE INTO trainer_genomes (id, run_id, generation, name, genome, fitness, active, created_at, methylations, synapse_weights)
                                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8, ?9)",
                                                params![
                                                    unique_id,
                                                    trainer_run_id,
                                                    generation as i32,
                                                    sb.agent.phenotype.latin_name,
                                                    sb.agent.genome,
                                                    0.0,
                                                    now_millis,
                                                    methylations_json,
                                                    synapse_weights_json
                                                ],
                                            ).log_db_err("INSERT OR REPLACE INTO trainer_genomes");
                                        }
                                    }
                                }
                                emit_state(json!({ "type": "DATABASE_CHANGED" }));
                            }
                            "UPDATE_TRAINER_HYPERPARAMS" => {
                                let mut grid_changed = false;
                                let mut generation_base = trainer.generation;
                                let p = &mut trainer.params;
                                if let Some(n) = action["N"].as_u64() {
                                    if n as usize != p.population {
                                        p.population = n as usize;
                                        grid_changed = true;
                                    }
                                }
                                if let Some(warp) = action["warpSpeed"].as_u64() {
                                    trainer_warp_speed = warp as usize;
                                }
                                if let Some(e_r) = action["eliteRatio"].as_f64() {
                                    p.elite_ratio = e_r as f32;
                                }
                                if let Some(m_r) = action["mutationRate"].as_f64() {
                                    p.mutation_rate = m_r as f32;
                                }
                                if let Some(i_r) = action["inflowRate"].as_f64() {
                                    p.inflow_rate = i_r as f32;
                                }
                                if let Some(h_r) = action["hofRate"].as_f64() {
                                    p.hof_rate = h_r as f32;
                                }
                                if let Some(mt) = action["multiTrial"].as_bool() {
                                    p.multi_trial = mt;
                                }
                                if let Some(hl) = action["isHeadless"].as_bool() {
                                    trainer_is_headless = hl;
                                }
                                if let Some(lam) = action["lamarckian"].as_bool() {
                                    if lam != p.lamarckian {
                                        p.lamarckian = lam;
                                        grid_changed = true;
                                    }
                                }
                                if let Some(r_id) = action["runId"].as_str() {
                                    if r_id != trainer_run_id {
                                        trainer_run_id = r_id.to_string();
                                        generation_base = 1;
                                        grid_changed = true;
                                    }
                                }

                                if grid_changed {
                                    trainer = restore_trainer(trainer.params.clone(), &trainer_run_id, generation_base, &conn);
                                }

                                // Broadcast the validated hyperparameters back to the UI (Self-Healing UI!)
                                emit_state(trainer_state_payload("TRAINER_STATE_CHANGED", trainer_is_running, &trainer.params, trainer_warp_speed, trainer_is_headless, &trainer_run_id));
                            }
                            other => {
                                eprintln!("[SIMULATION] Ignoring unknown client action type '{}'", other);
                            }
                        }
                    }
                }
            }

            // A. OCEAN ECOSYSTEM TICK (60Hz Substrate Physics & Biology)
            if !is_trainer_active {
                if is_running {
                    // Cap warp steps per tick to prevent thread starvation/IPC locking!
                    let steps_this_frame = if ocean_warp_factor > 35 { 35 } else { ocean_warp_factor };
                    for _ in 0..steps_this_frame {
                        // Populate Spatial Grid for collisions and physical pushes (buckets hold vector indices)
                        grid.clear();
                        for (idx, pellet) in food_pellets.iter().enumerate() {
                            grid.insert_food(idx as u32, pellet.x, pellet.y);
                        }
                        for (idx, agent) in creatures.iter().enumerate() {
                            grid.insert_creature(idx as u32, agent.px, agent.py);
                        }

                        // 1. Move, drift, and physically push food spores (using SpatialGrid for O(1) performance!)
                        use crate::shared::physics::{step_food_spore_physics, SporePusher};
                        for pellet in &mut food_pellets {
                            // Sorted indices keep the original creature order of the push sequence
                            let mut nearby_idx = grid.get_nearby_creatures(pellet.x, pellet.y, 80.0);
                            nearby_idx.sort_unstable();
                            let nearby_creatures: Vec<SporePusher> = nearby_idx.iter()
                                .map(|&i| SporePusher::from(&creatures[i as usize]))
                                .collect();
                            step_food_spore_physics(pellet, &nearby_creatures, &ocean_world.obstacles, logical_width, logical_height);
                        }

                        // Lightweight snapshot of all creatures before this step (prey search and bite targets)
                        let alive_clones: Vec<PeerSnapshot> = creatures.iter().map(PeerSnapshot::from).collect();
                        let mut damage_map = std::collections::HashMap::<u32, f32>::new();

                    let mut next_creatures = Vec::new();
                    let population = creatures.len();

                    // 2. Physics & Biology Loop over all creatures
                    for mut agent in creatures {
                        let mean_radius = agent.phenotype.spinal_harmonics.mean_radius;
                        let base_length = agent.phenotype.spinal_harmonics.base_length;

                        // A. Compute unified, rich 5-channel sensory inputs (matching Trainer 100%!)
                        let mut nearest_plant: Option<&FoodSpore> = None;
                        let mut min_plant_dist = f32::MAX;

                        // 1. Find the closest plant spore (algae, type_id == 1)
                        for spore in &food_pellets {
                            if spore.type_id == 2 {
                                continue;
                            }
                            let dx = spore.x - agent.px;
                            let dy = spore.y - agent.py;
                            let dist_sq = dx * dx + dy * dy;
                            if dist_sq < min_plant_dist {
                                min_plant_dist = dist_sq;
                                nearest_plant = Some(spore);
                            }
                        }

                        // 2. Find the closest meat target (either a carcass pellet type_id == 2, or a living prey peer)
                        let mut nearest_meat_spore: Option<&FoodSpore> = None;
                        let mut min_meat_spore_dist = f32::MAX;

                        for spore in &food_pellets {
                            if spore.type_id == 2 {
                                let dx = spore.x - agent.px;
                                let dy = spore.y - agent.py;
                                let dist_sq = dx * dx + dy * dy;
                                if dist_sq < min_meat_spore_dist {
                                    min_meat_spore_dist = dist_sq;
                                    nearest_meat_spore = Some(spore);
                                }
                            }
                        }

                        let mut nearest_prey_peer: Option<&PeerSnapshot> = None;
                        let mut min_prey_peer_dist = f32::MAX;

                        for peer in &alive_clones {
                            if peer.id == agent.id || peer.species_id == agent.species_id {
                                continue;
                            }
                            let dx = peer.px - agent.px;
                            let dy = peer.py - agent.py;
                            let dist_sq = dx * dx + dy * dy;
                            if dist_sq < min_prey_peer_dist {
                                min_prey_peer_dist = dist_sq;
                                nearest_prey_peer = Some(peer);
                            }
                        }

                        // Determine the single closest meat target for the carnivore's sensory inputs
                        let meat_target = if min_meat_spore_dist < min_prey_peer_dist {
                            nearest_meat_spore.cloned().unwrap_or(FoodSpore {
                                id: 2,
                                type_id: 2,
                                x: -99999.0,
                                y: -99999.0,
                                amount: 0.0,
                                vx: 0.0,
                                vy: 0.0,
                            })
                        } else {
                            nearest_prey_peer.map(|peer| FoodSpore {
                                id: peer.id,
                                type_id: 2,
                                x: peer.px,
                                y: peer.py,
                                amount: 15.0,
                                vx: peer.vx,
                                vy: peer.vy,
                            }).unwrap_or(FoodSpore {
                                id: 2,
                                type_id: 2,
                                x: -99999.0,
                                y: -99999.0,
                                amount: 0.0,
                                vx: 0.0,
                                vy: 0.0,
                            })
                        };

                        // 3. Assemble a virtual foods slice matching the trainer's expected structure:
                        // foods[0] = Plant, foods[1] = Meat
                        let plant_target = nearest_plant.cloned().unwrap_or(FoodSpore {
                            id: 1,
                            type_id: 1,
                            x: -99999.0, // out of range if none
                            y: -99999.0,
                            amount: 0.0,
                            vx: 0.0,
                            vy: 0.0,
                        });

                        let virtual_foods = vec![plant_target, meat_target];

                        // Calculate the Central Pattern Generator clock (same 0.1 scaling as Trainer for consistent brain temporal integration)
                        let clock_val = 0.5 + 0.5 * ((agent.age as f32) * 0.1).sin();

                        // Call the identical sensory compiler from brain.rs!
                        let inputs = crate::shared::brain::compute_sensory_inputs(
                            &agent,
                            clock_val,
                            &virtual_foods,
                            &ocean_world.obstacles,
                            logical_width,
                            logical_height,
                        );

                        // Execute Recurrent CTRNN Brain Euler Integration with live Hebbian learning
                        use crate::shared::brain::execute_brain_with_learning;
                        let outputs = execute_brain_with_learning(
                            &agent.phenotype.brain,
                            &inputs,
                            &mut agent.neuron_states,
                            &mut agent.neuron_activations,
                            &mut agent.synapse_weights,
                            &mut agent.brain_runtime,
                            app_config.rules.hebbian_learning_rate_base,
                        );
                        let out_thrust = outputs[0];
                        let out_left = outputs[1];

                        // Passive metabolism
                        // Reduce Ozean basal metabolic rate decay by half (0.45 coefficient) for long, organic, and gentle lifespans!
                        let bmr_decay = agent.phenotype.basal_metabolic_rate * app_config.rules.bmr_base_scale * app_config.basal_metabolic_rate_multiplier * 0.45;
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

                        let max_age_ticks = (agent.phenotype.mature_age * app_config.rules.creature_lifespan_mature_age_multiplier)
                            .min(app_config.rules.creature_max_age_ticks);
                        if agent.energy <= 0.0 || agent.age >= max_age_ticks {
                            // Decompose corpse into nutrient spores (elastic spore relocation, no array deletion!)
                            let num_pellets = (((base_length * mean_radius) / app_config.rules.decomposition_size_ratio).floor() as i32)
                                .clamp(app_config.rules.decomposition_spore_min, app_config.rules.decomposition_spore_max);

                            let mut rng = rand::thread_rng();
                            for _ in 0..num_pellets {
                                if !food_pellets.is_empty() {
                                    let p_idx = rng.gen_range(0..food_pellets.len());
                                    food_pellets[p_idx].type_id = 2; // 🍖 Relocated pellet is converted into a meat spore!
                                    food_pellets[p_idx].x = agent.px + rng.gen_range(-16.0..16.0);
                                    food_pellets[p_idx].y = agent.py + rng.gen_range(-16.0..16.0);
                                    food_pellets[p_idx].vx = rng.gen_range(-0.2..0.2);
                                    food_pellets[p_idx].vy = rng.gen_range(-0.2..0.2);
                                }
                            }

                            emit_state(json!({
                                "type": "LOG_EVENT",
                                "message": format!("💀 Species '{}' (Strain: #{}) deceased due to starvation/senescence.", agent.phenotype.latin_name, agent.id),
                                "logType": "system"
                            }));
                        } else {
                            // Apply unified biological and physical kinetics
                            use crate::shared::physics::step_creature_kinematics;
                            step_creature_kinematics(&mut agent, out_thrust, out_left, &app_config, &ocean_world.obstacles, logical_width, logical_height);

                            agent.age += 1;

                            // Grazing (Eat Food)
                            let eat_radius = mean_radius * app_config.rules.grazing_radius_multiplier + app_config.rules.grazing_radius_offset;
                            let nearby_foods = grid.get_nearby_food(agent.px, agent.py, eat_radius);
                            for f_idx in nearby_foods {
                                if let Some(idx) = Some(f_idx as usize).filter(|&i| i < food_pellets.len()) {
                                    let f = &food_pellets[idx];
                                    let dx = f.x - agent.px;
                                    let dy = f.y - agent.py;
                                    if (dx*dx + dy*dy).sqrt() <= eat_radius {
                                        let is_meat = f.type_id == 2;
                                        let efficiency = if is_meat {
                                            agent.phenotype.carnivory
                                        } else {
                                            1.0 - agent.phenotype.carnivory
                                        };

                                        let old_x = f.x;
                                        let old_y = f.y;
                                        let scale_factor = if is_meat {
                                            app_config.rules.biting_efficiency_carnivore_scale
                                        } else {
                                            app_config.rules.grazing_efficiency_herbivore_scale
                                        };
                                        let energy_gain = 15.0 * efficiency * scale_factor;
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

                                        {
                                            let mutable_f = &mut food_pellets[idx];
                                            if is_meat {
                                                mutable_f.type_id = 1; // Recycled back to a green plant spore!
                                            }
                                            mutable_f.x = new_x;
                                            mutable_f.y = new_y;
                                            mutable_f.vx = new_vx;
                                            mutable_f.vy = new_vy;
                                        }

                                        // Broadcast Eat event for client-side algae sparks
                                        emit_state(json!({
                                            "type": "EAT_EVENT",
                                            "x": old_x,
                                            "y": old_y
                                        }));
                                    }
                                }
                            }

                            // Fight Biting (Predation)
                            if agent.phenotype.carnivory >= app_config.rules.biting_carnivory_threshold {
                                let bite_range = mean_radius * app_config.rules.biting_radius_multiplier + app_config.rules.biting_radius_offset;
                                let nearby_peers = grid.get_nearby_creatures(agent.px, agent.py, bite_range);
                                for v_idx in nearby_peers {
                                    if alive_clones.get(v_idx as usize).map_or(true, |c| c.id == agent.id) { continue; }
                                    if let Some(victim) = alive_clones.get(v_idx as usize) {
                                        if victim.species_id != agent.species_id {
                                            let dx = victim.px - agent.px;
                                            let dy = victim.py - agent.py;
                                            let dist = (dx*dx + dy*dy).sqrt();
                                            if dist <= bite_range {
                                                // Record damage (configured in config.json!)
                                                *damage_map.entry(victim.id).or_insert(0.0) += app_config.rules.biting_energy_damage;

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
                                                    "message": format!("⚡ [BITE ATTACK] {} #{} bites #{}! (-{:.0}nJ damage)", agent.phenotype.latin_name.chars().take(16).collect::<String>(), agent.id, victim.id, app_config.rules.biting_energy_damage),
                                                    "logType": "mutation"
                                                }));
                                            }
                                        }
                                    }
                                }
                            }

                            // Asexual Reproduction (Mitosis)
                            let mature_age = agent.phenotype.mature_age.max(app_config.rules.creature_mature_age_floor);
                            let repro_threshold = agent.phenotype.repro_threshold.max(app_config.rules.reproduction_stomach_threshold_floor);
                            let can_reproduce = agent.age >= mature_age && agent.has_eaten && population < app_config.max_creatures;
                            let threshold = agent.phenotype.stomach_capacity * repro_threshold;
                            
                            if can_reproduce && agent.energy >= threshold {
                                agent.energy *= app_config.rules.reproduction_split_loss_ratio;
                                agent.has_eaten = false;
                                
                                let recoil = agent.phenotype.split_loss * app_config.rules.reproduction_recoil_velocity_scale;
                                agent.vx += recoil * (agent.heading_angle + std::f32::consts::PI).cos();
                                agent.vy += recoil * (agent.heading_angle + std::f32::consts::PI).sin();

                                // Child (TCK-152): a fresh agent from the factory (fresh neuron states) at the parent's place,
                                // with the parent's post-split energy. With 35 % probability the genome mutates.
                                let mut rng = rand::thread_rng();
                                let is_mutation = rng.gen_range(0.0..1.0) < 0.35;
                                let (child_genome, child_phenotype) = if is_mutation {
                                    let genome = mutate_genome(&agent.genome, &app_config.mutation_operators, &mut rng).map_or_else(|| agent.genome.clone(), |m| m.genome);
                                    let phenotype = parse_genome(&genome, None, None);
                                    (genome, phenotype)
                                } else {
                                    (agent.genome.clone(), agent.phenotype.clone())
                                };
                                // Lamarckian assimilation (TCK-152): per plastic synapse with the same key, with probability
                                // `lamarckianAssimilationChance`, the parent's learned offset; only from parents old enough to
                                // have learned something. Clones and mutants alike.
                                let assimilate = agent.age >= app_config.rules.lamarckian_assimilation_age_floor;
                                let (parent_synapses, parent_weights): (&[_], &[_]) = if assimilate { (&agent.phenotype.brain.synapses, &agent.synapse_weights) } else { (&[], &[]) };
                                let child_weights = inherit_learned_weights(
                                    parent_synapses,
                                    parent_weights,
                                    &child_phenotype.brain.synapses,
                                    app_config.rules.lamarckian_assimilation_chance,
                                    &mut rng,
                                );
                                let mut child = spawn_from_phenotype(&child_genome, child_phenotype, SpawnOptions {
                                    id: next_creature_id,
                                    px: agent.px,
                                    py: agent.py,
                                    heading_angle: agent.heading_angle,
                                    generation: agent.generation + 1,
                                    methylations: None,
                                    synapse_weights: Some(child_weights),
                                });
                                child.energy = agent.energy;

                                if is_mutation {
                                    // Save new mutated child species into SQLite database
                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64;
                                    conn.execute(
                                        "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
                                        params![&child.genome, &child.phenotype.latin_name, &child.genome, Some(agent.phenotype.latin_name.clone()), now_millis, child.generation, child.phenotype.carnivory],
                                    ).log_db_err("INSERT OR IGNORE INTO species_records");
                                    species_cache_dirty = true;

                                    emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                    emit_state(json!({
                                        "type": "LOG_EVENT",
                                        "message": format!("🧬 Mitosis Mutation! Specimen #{} birthed new species '{}' (Gen. {}).", agent.id, child.phenotype.latin_name, child.generation),
                                        "logType": "mutation"
                                    }));
                                } else {
                                    // Mirror asexual clone birth log
                                    emit_state(json!({
                                        "type": "LOG_EVENT",
                                        "message": format!("🌱 Mitosis Clone! Specimen #{} duplicated itself successfully.", agent.id),
                                        "logType": "repair"
                                    }));
                                }

                                if child.generation > highest_generation {
                                    highest_generation = child.generation;
                                }

                                next_creature_id += 1;
                                newly_spawned_creatures.push(child);
                            }

                            next_creatures.push(agent);
                        }
                    }

                    // Apply registered damages and filter out deceased prey
                    let mut final_creatures = Vec::new();
                    for mut agent in next_creatures {
                        if let Some(&damage) = damage_map.get(&agent.id) {
                            agent.energy -= damage;
                        }
                        if agent.energy > 0.1 {
                            final_creatures.push(agent);
                        } else {
                            // Decompose carcass into nutrient meat spores (elastic spore relocation, no array deletion!)
                            let base_length = agent.phenotype.spinal_harmonics.base_length;
                            let mean_radius = agent.phenotype.spinal_harmonics.mean_radius;
                            let num_pellets = (((base_length * mean_radius) / app_config.rules.decomposition_size_ratio).floor() as i32)
                                .clamp(app_config.rules.decomposition_spore_min, app_config.rules.decomposition_spore_max);

                            let mut rng = rand::thread_rng();
                            for _ in 0..num_pellets {
                                if !food_pellets.is_empty() {
                                    let p_idx = rng.gen_range(0..food_pellets.len());
                                    food_pellets[p_idx].type_id = 2; // 🍖 Relocated pellet is converted into a meat spore!
                                    food_pellets[p_idx].x = agent.px + rng.gen_range(-16.0..16.0);
                                    food_pellets[p_idx].y = agent.py + rng.gen_range(-16.0..16.0);
                                    food_pellets[p_idx].vx = rng.gen_range(-0.2..0.2);
                                    food_pellets[p_idx].vy = rng.gen_range(-0.2..0.2);
                                }
                            }

                            emit_state(json!({
                                "type": "LOG_EVENT",
                                "message": format!("✝️ Specimen #{} '{}' was hunted down by predators.", agent.id, agent.phenotype.latin_name),
                                "logType": "mutation"
                            }));
                        }
                    }
                    creatures = final_creatures;

                    // Extinction and Peak Population Tracking (matches TS index.ts!)
                    let mut db_changed = false;

                    // The engine is the only writer of species_records, so the table is only re-read after inserts/resets
                    if species_cache_dirty {
                        species_cache.clear();
                        if let Ok(mut stmt) = conn.prepare("SELECT id, status, peak_population FROM species_records") {
                            if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i32>(2)?))) {
                                for row in rows {
                                    if let Ok(item) = row {
                                        species_cache.push(item);
                                    }
                                }
                            }
                        }
                        species_cache_dirty = false;
                    }

                    let mut species_counts = std::collections::HashMap::<&str, i32>::new();
                    for c in &creatures {
                        *species_counts.entry(c.species_id.as_str()).or_insert(0) += 1;
                    }

                    let mut peak_updates = Vec::new();
                    let mut extinction_updates = Vec::new();

                    for (id, status, peak_pop) in species_cache.iter_mut() {
                        let active_count = species_counts.get(id.as_str()).copied().unwrap_or(0);
                        if active_count > 0 {
                            if active_count > *peak_pop {
                                peak_updates.push((id.clone(), active_count));
                                *peak_pop = active_count;
                            }
                            if status == "extinct" {
                                // Re-sprout resurrected species
                                conn.execute("UPDATE species_records SET status = 'alive', extinction_time = NULL WHERE id = ?1", params![id.as_str()]).log_db_err("UPDATE species_records");
                                *status = "alive".to_string();
                                db_changed = true;
                            }
                        } else {
                            if status == "alive" {
                                extinction_updates.push(id.clone());
                                *status = "extinct".to_string();
                            }
                        }
                    }

                    for (id, peak) in peak_updates {
                        conn.execute("UPDATE species_records SET peak_population = ?1 WHERE id = ?2", params![peak, id]).log_db_err("UPDATE species_records");
                        db_changed = true;
                    }

                    for id in extinction_updates {
                        let now_millis = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        conn.execute("UPDATE species_records SET status = 'extinct', extinction_time = ?1 WHERE id = ?2", params![now_millis, id]).log_db_err("UPDATE species_records");
                        db_changed = true;

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
                    }

                    if db_changed {
                        emit_state(json!({ "type": "DATABASE_CHANGED" }));
                    }

                    // Maintain population (restocking founder cells up to config, matches TS index.ts!)
                    let target_population = app_config.target_population;

                    // Query all saved catalogue creatures once per restock pass
                    let mut catalogue_creatures = Vec::new();
                    if creatures.len() < target_population {
                        if let Ok(mut stmt) = conn.prepare("SELECT genome, name, methylations, synapse_weights FROM creature_catalogue") {
                            if let Ok(rows) = stmt.query_map([], |row| {
                                Ok((
                                    row.get::<_, String>(0)?,
                                    row.get::<_, String>(1)?,
                                    row.get::<_, String>(2)?,
                                    row.get::<_, String>(3)?
                                ))
                            }) {
                                for row in rows {
                                    if let Ok(data) = row {
                                        catalogue_creatures.push(data);
                                    }
                                }
                            }
                        }
                    }

                    while creatures.len() < target_population {
                        let mut rng = rand::thread_rng();

                        let (g, random_pheno, _m_vec, s_vec, source_tag, generation_val) = if !catalogue_creatures.is_empty() {
                            let idx = rng.gen_range(0..catalogue_creatures.len());
                            let (genome, name, m_str, s_str) = &catalogue_creatures[idx];
                            
                            let m_arr: Vec<f32> = serde_json::from_str(m_str).unwrap_or_default();
                            let s_arr: Vec<f32> = serde_json::from_str(s_str).unwrap_or_default();
                            let m_opt = if m_arr.is_empty() { None } else { Some(&m_arr[..]) };
                            
                            let pheno = parse_genome(genome, None, m_opt);
                            (genome.clone(), pheno, m_arr, s_arr, format!("📚 Catalog Clonal '{}'", name), 1)
                        } else {
                            // Fallback to a structured random founder if the catalogue is empty
                            let genome = generate_structured_genome(&app_config.genome_composition, &mut rng);
                            let pheno = parse_genome(&genome, None, None);
                            (genome, pheno, Vec::new(), Vec::new(), "🌱 Random Founder".to_string(), 1)
                        };

                        let px = rng.gen_range(500.0..logical_width - 500.0);
                        let py = rng.gen_range(500.0..logical_height - 500.0);
                        let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                        let mut new_restocked = spawn_from_phenotype(&g, random_pheno.clone(), SpawnOptions {
                            id: next_creature_id,
                            px,
                            py,
                            heading_angle,
                            generation: generation_val,
                            methylations: None,
                            synapse_weights: Some(s_vec),
                        });
                        new_restocked.vx = rng.gen_range(-0.4..0.4);
                        new_restocked.vy = rng.gen_range(-0.4..0.4);

                        creatures.push(new_restocked.clone());
                        newly_spawned_creatures.push(new_restocked.clone());

                        // Seed each unique species into local SQLite database if not exists
                        let now_millis = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        conn.execute(
                            "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
                            params![&g, &random_pheno.latin_name, &g, None::<String>, now_millis, generation_val, random_pheno.carnivory],
                        ).log_db_err("INSERT OR IGNORE INTO species_records");
                        species_cache_dirty = true;

                        emit_state(json!({ "type": "DATABASE_CHANGED" }));
                        emit_state(json!({
                            "type": "LOG_EVENT",
                            "message": format!("Restocked population: Spawned {}.", source_tag),
                            "logType": "system"
                        }));

                        next_creature_id += 1;
                    }
                } // End of ocean_warp_factor loop
            } // End of if is_running

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

                        conn.execute(
                            "INSERT INTO simulation_history (session_id, generation, average_fitness, max_fitness, population_count) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![
                                "desktop-session-001",
                                highest_generation as i32,
                                avg_energy,
                                max_energy,
                                pop_count
                            ],
                        ).log_db_err("INSERT INTO simulation_history");

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
                            conn.execute(
                                "INSERT OR REPLACE INTO simulation_state (key, state_json, updated_at) VALUES ('current_state', ?1, ?2)",
                                params![state_str, now_millis],
                            ).log_db_err("INSERT OR REPLACE INTO simulation_state");
                        }
                    }
                    last_history_log_time = Instant::now();
                }
            }

            // B. TRAINER EVOLUTION TICK
            if is_trainer_active {
                if trainer_is_running {
                    // Run trainer_warp_speed steps (cap steps per 60Hz tick to prevent thread blocking/IPC flooding)
                    let steps_this_frame = if trainer_warp_speed > 35 { 35 } else { trainer_warp_speed };
                    for _step in 0..steps_this_frame {
                        if let Some(event) = trainer.step(true) { // parallel == sequential (seeded per-sandbox and per-slot streams)
                            let TrainerEvent::GenerationCompleted { generation, best_fitness, avg_fitness, best_genome, elites, .. } = event;
                            println!("[TRAINER RUN] Gen {} completed! Best Fit: {:.1}, Avg Fit: {:.1}", generation, best_fitness, avg_fitness);

                            // Broadcast completion metadata back to UI instantly
                            emit_state(json!({
                                "type": "TRAINER_GENERATION_COMPLETED",
                                "generation": generation,
                                "bestFitness": best_fitness,
                                "avgFitness": avg_fitness,
                                "bestGenome": best_genome,
                            }));

                            // Save the elite champions (once per generation) and re-trigger roster refreshes
                            save_trainer_elites(&conn, &trainer_run_id, generation, &elites, trainer_retention);
                            emit_state(json!({ "type": "DATABASE_CHANGED" }));
                        }
                    }
                }

                // C. Stream Trainer Telemetry to UI at 25Hz (lightweight frames)
                if last_emit_time.elapsed() >= emit_interval {
                    let plugin = trainer.params.scenario.plugin();
                    let telemetry_sandboxes: Vec<TrainerTelemetrySandbox> = trainer
                        .sandboxes
                        .iter()
                        .map(|sb| {
                            TrainerTelemetrySandbox {
                                id: sb.id,
                                chamber_size: plugin.chamber_size(),
                                px: sb.agent.px,
                                py: sb.agent.py,
                                vx: sb.agent.vx,
                                vy: sb.agent.vy,
                                heading_angle: sb.agent.heading_angle,
                                omega_rot: sb.agent.omega_rot,
                                finished: sb.finished,
                                current_fitness: sb.current_fitness,
                                origin_type: sb.origin_type,
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
                                scenario: sb.scenario,
                                cumulative_rotation: sb.cumulative_rotation,
                                coverage_mask: sb.coverage_mask,
                            }
                        })
                        .collect();

                    let time_str = format!("{:.1}s", ((plugin.epoch_ticks() as f32 - trainer.epoch_ticks as f32) / 60.0).max(0.0));

                    // Compute selected sandbox live neural activations for real-time brain rendering
                    let selected_brain_json = if let Some(sel_id) = trainer_selected_sandbox_id {
                        if let Some(sel_sb) = trainer.sandboxes.iter().find(|s| s.id == sel_id) {
                            json!({
                                "id": sel_id,
                                "activations": sel_sb.agent.neuron_activations
                            })
                        } else {
                            json!(null)
                        }
                    } else {
                        json!(null)
                    };

                    emit_state(json!({
                        "type": "TRAINER_TELEMETRY_TICK",
                        "generation": trainer.generation,
                        "isRunning": trainer_is_running,
                        "timeStr": time_str,
                        "sandboxes": telemetry_sandboxes,
                        "selectedBrain": selected_brain_json
                    }));

                    last_emit_time = Instant::now();
                }
            }

            flush_events();

            // Cap the physical thread at ~60Hz to prevent thread starvation and keep learning rates stable
            let elapsed = loop_start.elapsed();
            let frame_duration = Duration::from_micros(16667); // 16.67ms
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
    });
}
