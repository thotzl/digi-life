use tauri::Emitter;
use std::thread;
use std::time::{Duration, Instant};
use serde_json::json;
use std::sync::mpsc::Receiver;
use rusqlite::params;
use rand::Rng;

use crate::shared::types::{CreatureAgent, FoodSpore, TelemetryCreature};
use crate::shared::spatial_grid::SpatialGrid;
use crate::shared::physics::apply_creature_physics;
use crate::biology::dna::{parse_genome, mutate_genome, generate_random_genome};
use crate::biology::trainer_engine::{
    init_rust_sandbox, step_trainer_sandbox_physics, calculate_sandbox_fitness,
    TrainerSandbox, TrainerTelemetrySandbox,
};
use crate::database::init_db;

pub fn spawn_simulation_thread(window: tauri::WebviewWindow, rx: Receiver<String>) {
    thread::spawn(move || {
        let db_path = "pixel_life_local.db";
        let conn = init_db(db_path).expect("Failed to initialize local DB");

        let emit_state = |payload: serde_json::Value| {
            if let Err(e) = window.emit("simulation-state", payload) {
                eprintln!("[SIMULATION] Error emitting 'simulation-state' event: {}", e);
            }
        };

        let logical_width = 19200.0;
        let logical_height = 10800.0;
        
        let app_config = crate::shared::types::AppConfig::global();
        
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
        let mut trainer_elite_ratio = 0.15;
        let mut trainer_mutation_rate = 0.06;
        let mut trainer_inflow_rate = 0.15;
        let mut trainer_hof_rate = 0.15;
        let mut trainer_multi_trial = true;
        let mut trainer_is_headless = false;
        let mut trainer_run_id = "default_run".to_string();
        let mut trainer_selected_sandbox_id: Option<u32> = Some(1);

        let mut trainer_generation = 1;
        let mut trainer_epoch_ticks = 0;

        // Helper to rebuild sandbox grid in Rust!
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

            let base_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";

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
            params!["desktop-session-001", "Tauri Desktop Laboratory Studio", "{}"],
        );

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
                    if let Some((mutated, _, _, _)) = mutate_genome(&base_dna) {
                        mutated_dna = mutated;
                    }
                }

                let random_pheno = parse_genome(&mutated_dna, None, None);
                let px = rng.gen_range(500.0..logical_width - 500.0);
                let py = rng.gen_range(500.0..logical_height - 500.0);
                let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                let new_agent = CreatureAgent {
                    id: next_creature_id,
                    species_id: mutated_dna.clone(),
                    px,
                    py,
                    vx: rng.gen_range(-0.4..0.4),
                    vy: rng.gen_range(-0.4..0.4),
                    heading_angle,
                    bend_angle: 0.0,
                    omega_rot: 0.0,
                    energy: random_pheno.stomach_capacity * 0.60,
                    adrenaline: 1.0,
                    age: 0,
                    generation: 1,
                    has_eaten: false,
                    genome: mutated_dna.clone(),
                    antisense: String::new(),
                    phenotype: random_pheno.clone(),
                    neuron_states: Vec::new(),
                    neuron_activations: Vec::new(),
                    synapse_weights: random_pheno.brain.synapses.iter().map(|s| s.weight).collect(),
                };

                creatures.push(new_agent.clone());

                let now_millis = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                    params![&mutated_dna, &random_pheno.latin_name, &mutated_dna, None::<String>, now_millis, random_pheno.carnivory],
                );

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

        let mut trainer_sandboxes: Vec<TrainerSandbox> = Vec::new();
        let mut trainer_trial_index: usize = 0;
        let mut trainer_accumulated_fitness: Vec<f32> = Vec::new();
        let mut trainer_accumulated_eaten: Vec<u32> = Vec::new();

        loop {
            let loop_start = Instant::now();

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
                            "INJECT_URZELLE" => {
                                let mut rng = rand::thread_rng();
                                let random_dna = generate_random_genome(256);
                                let random_pheno = parse_genome(&random_dna, None, None);

                                let px = rng.gen_range(500.0..logical_width - 500.0);
                                let py = rng.gen_range(500.0..logical_height - 500.0);
                                let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

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
                                    energy: random_pheno.stomach_capacity * 0.60,
                                    adrenaline: 1.0,
                                    age: 0,
                                    generation: 1,
                                    has_eaten: false,
                                    genome: random_dna.clone(),
                                    antisense: String::new(),
                                    phenotype: random_pheno.clone(),
                                    neuron_states: Vec::new(),
                                    neuron_activations: Vec::new(),
                                    synapse_weights: random_pheno.brain.synapses.iter().map(|s| s.weight).collect(),
                                };

                                creatures.push(new_agent.clone());
                                newly_spawned_creatures.push(new_agent.clone());

                                let now_millis = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64;
                                let _ = conn.execute(
                                    "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, 1, ?6)",
                                    params![&random_dna, &random_pheno.latin_name, &random_dna, None::<String>, now_millis, random_pheno.carnivory],
                                );

                                emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                emit_state(json!({
                                    "type": "LOG_EVENT",
                                    "message": format!("Released wildtype organism: Strain #{} '{}'.", next_creature_id, random_pheno.latin_name),
                                    "logType": "mutation"
                                }));
                                next_creature_id += 1;
                            }
                            "RESET_EVOLUTION" => {
                                println!("[SIMULATION] Performing complete environmental restoration reset...");
                                
                                // Wipe old species records, histories, and state caches from SQLite database
                                let _ = conn.execute("DELETE FROM species_records", []);
                                let _ = conn.execute("DELETE FROM simulation_history", []);
                                let _ = conn.execute("DELETE FROM simulation_state", []);

                                creatures.clear();
                                food_pellets.clear();
                                newly_spawned_creatures.clear(); // Clear newborns to prevent phantom shivering creatures!
                                next_creature_id = 1;
                                highest_generation = 1;

                                let mut rng = rand::thread_rng();
                                let base_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";

                                // Seed fresh 25 unique, highly active, and brilliantly colored viable starting wildtypes
                                for _ in 0..25 {
                                    let mut mutated_dna = base_dna.to_string();

                                    // Apply 3 to 8 random sequential mutations to ensure structural and behavioral drift
                                    let num_mutations = rng.gen_range(3..=8);
                                    for _ in 0..num_mutations {
                                        if let Some((mutated, _, _, _)) = mutate_genome(&mutated_dna) {
                                            mutated_dna = mutated;
                                        }
                                    }

                                    // Randomize characters 3, 4, 5 (the "OOO" color payload of "COLOOOEN") to guarantee beautiful color diversity
                                    let mut char_vec: Vec<char> = mutated_dna.chars().collect();
                                    if char_vec.len() >= 6 {
                                        char_vec[3] = rng.gen_range(b'A'..=b'Z') as char;
                                        char_vec[4] = rng.gen_range(b'A'..=b'Z') as char;
                                        char_vec[5] = rng.gen_range(b'A'..=b'Z') as char;
                                    }
                                    mutated_dna = char_vec.into_iter().collect();

                                    let random_pheno = parse_genome(&mutated_dna, None, None);
                                    let px = rng.gen_range(500.0..logical_width - 500.0);
                                    let py = rng.gen_range(500.0..logical_height - 500.0);
                                    let heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);

                                    let new_agent = CreatureAgent {
                                        id: next_creature_id,
                                        species_id: mutated_dna.clone(),
                                        px,
                                        py,
                                        vx: rng.gen_range(-0.4..0.4),
                                        vy: rng.gen_range(-0.4..0.4),
                                        heading_angle,
                                        bend_angle: 0.0,
                                        omega_rot: 0.0,
                                        energy: random_pheno.stomach_capacity * 0.60,
                                        adrenaline: 1.0,
                                        age: 0,
                                        generation: 1,
                                        has_eaten: false,
                                        genome: mutated_dna.clone(),
                                        antisense: String::new(),
                                        phenotype: random_pheno.clone(),
                                        neuron_states: Vec::new(),
                                        neuron_activations: Vec::new(),
                                        synapse_weights: random_pheno.brain.synapses.iter().map(|s| s.weight).collect(),
                                    };

                                    creatures.push(new_agent.clone());

                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64;
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
                                        params![&mutated_dna, &random_pheno.latin_name, &mutated_dna, None::<String>, now_millis, 1, random_pheno.carnivory],
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
                            "SET_MODE" => {
                                if let Some(mode) = action["mode"].as_str() {
                                    if mode == "trainer" {
                                        is_trainer_active = true;
                                        // Initialize sandboxes grid in Rust when entering trainer
                                        let (sbs, restored_gen) = rebuild_sandbox_grid(trainer_N, &trainer_run_id, trainer_generation, trainer_mutation_rate, &conn, trainer_elite_ratio, trainer_inflow_rate, trainer_hof_rate);
                                        trainer_sandboxes = sbs;
                                        trainer_generation = restored_gen;
                                        trainer_trial_index = 0;
                                        trainer_accumulated_fitness = vec![0.0; trainer_N];
                                        trainer_accumulated_eaten = vec![0; trainer_N];
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
                                trainer_trial_index = 0;
                                trainer_accumulated_fitness = vec![0.0; trainer_N];
                                trainer_accumulated_eaten = vec![0; trainer_N];
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
                                } else {
                                    trainer_selected_sandbox_id = None;
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
                                if let Some(warp) = action["warpSpeed"].as_u64() {
                                    trainer_warp_speed = warp as usize;
                                }
                                if let Some(e_r) = action["eliteRatio"].as_f64() {
                                    trainer_elite_ratio = e_r as f32;
                                }
                                if let Some(m_r) = action["mutationRate"].as_f64() {
                                    trainer_mutation_rate = m_r as f32;
                                }
                                if let Some(i_r) = action["inflowRate"].as_f64() {
                                    trainer_inflow_rate = i_r as f32;
                                }
                                if let Some(h_r) = action["hofRate"].as_f64() {
                                    trainer_hof_rate = h_r as f32;
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
                                    trainer_trial_index = 0;
                                    trainer_accumulated_fitness = vec![0.0; trainer_N];
                                    trainer_accumulated_eaten = vec![0; trainer_N];
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

            // A. OCEAN ECOSYSTEM TICK (60Hz Substrate Physics & Biology)
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
                    for mut agent in creatures {
                        let mean_radius = agent.phenotype.spinal_harmonics.mean_radius;
                        let base_length = agent.phenotype.spinal_harmonics.base_length;

                        // A. Compute neural brain integration inputs (original simplified, ultra-fast ocean simulated inputs!)
                        let k = agent.phenotype.organelles.len();
                        let mut inputs = vec![0.0; k + 1];
                        for i in 0..k {
                            inputs[i] = 0.5; // default moderate stimulus
                        }
                        inputs[k] = 0.5; // clock / hunger

                        // Execute Recurrent CTRNN Brain Euler Integration with live Hebbian learning
                        use crate::shared::brain::execute_brain_with_learning;
                        let outputs = execute_brain_with_learning(
                            &agent.phenotype.brain,
                            &inputs,
                            &mut agent.neuron_states,
                            &mut agent.neuron_activations,
                            &mut agent.synapse_weights,
                            app_config.rules.hebbian_learning_rate_base,
                            app_config.rules.hebbian_learning_stiffness_decay,
                            app_config.rules.hebbian_forgetting_decay,
                        );
                        let out_thrust = outputs[0];
                        let out_left = outputs[1];

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
                            let num_pellets = (((base_length * mean_radius) / app_config.rules.decomposition_size_ratio).floor() as i32)
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
                                "message": format!("💀 Species '{}' (Strain: #{}) deceased due to starvation/senescence.", agent.phenotype.latin_name, agent.id),
                                "logType": "system"
                            }));
                        } else {
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
                            apply_creature_physics(
                                &mut agent,
                                net_thrust,
                                out_left,
                                mean_radius.powf(1.5) * (base_length / 25.0), // Mass
                                mean_radius * 0.015, // Drag forward
                                0.0,
                                0.0,
                                logical_width,
                                logical_height,
                            );

                            agent.age += 1;

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
                                            let old_x = f.x;
                                            let old_y = f.y;
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

                                            {
                                                let mutable_f = &mut food_pellets[idx];
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
                                child.age = 0;
                                child.generation += 1;
                                child.has_eaten = false;

                                // Mutate child genome with 35% probability
                                let mut rng = rand::thread_rng();
                                if rng.gen_range(0.0..1.0) < 0.35 {
                                    if let Some((mutated, _, _, _)) = mutate_genome(&agent.genome) {
                                        child.genome = mutated;
                                    }
                                    child.species_id = child.genome.clone();
                                    child.phenotype = parse_genome(&child.genome, None, None);
                                    
                                    // Lamarckian Epigenetic Assimilation: Pass parent's learned weights to child homologous synapses
                                    let parent_synapses = &agent.phenotype.brain.synapses;
                                    let parent_weights = &agent.synapse_weights;
                                    
                                    let mut child_weights = vec![0.0; child.phenotype.brain.synapses.len()];
                                    for (c_idx, c_syn) in child.phenotype.brain.synapses.iter().enumerate() {
                                        let mut w = c_syn.weight;
                                        if let Some(p_idx) = parent_synapses.iter().position(|ps| ps.from_node == c_syn.from_node && ps.to_node == c_syn.to_node) {
                                            if p_idx < parent_weights.len() {
                                                let parent_learned_offset = parent_weights[p_idx] - parent_synapses[p_idx].weight;
                                                w += parent_learned_offset * app_config.rules.lamarckian_assimilation_chance;
                                            }
                                        }
                                        child_weights[c_idx] = w.clamp(-4.0, 4.0);
                                    }
                                    child.synapse_weights = child_weights;
                                    
                                    // Save new mutated child species into SQLite database
                                    let now_millis = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs() as i64;
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO species_records (id, latin_name, genome_string, parent_name, status, peak_population, birth_time, generation, carnivory) VALUES (?1, ?2, ?3, ?4, 'alive', 1, ?5, ?6, ?7)",
                                        params![&child.genome, &child.phenotype.latin_name, &child.genome, Some(agent.phenotype.latin_name.clone()), now_millis, child.generation, child.phenotype.carnivory],
                                    );

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

                    let mut registered_species = Vec::new();
                    if let Ok(mut stmt) = conn.prepare("SELECT id, status, peak_population FROM species_records") {
                        if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i32>(2)?))) {
                            for row in rows {
                                if let Ok(item) = row {
                                    registered_species.push(item);
                                }
                            }
                        }
                    }

                    let mut peak_updates = Vec::new();
                    let mut extinction_updates = Vec::new();

                    for (id, status, peak_pop) in registered_species {
                        let active_count = creatures.iter().filter(|c| c.species_id == id).count() as i32;
                        if active_count > 0 {
                            if active_count > peak_pop {
                                peak_updates.push((id.clone(), active_count));
                            }
                            if status == "extinct" {
                                // Re-sprout resurrected species
                                let _ = conn.execute("UPDATE species_records SET status = 'alive', extinction_time = NULL WHERE id = ?1", params![id]);
                                db_changed = true;
                            }
                        } else {
                            if status == "alive" {
                                extinction_updates.push(id.clone());
                            }
                        }
                    }

                    for (id, peak) in peak_updates {
                        let _ = conn.execute("UPDATE species_records SET peak_population = ?1 WHERE id = ?2", params![peak, id]);
                        db_changed = true;
                    }

                    for id in extinction_updates {
                        let now_millis = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        let _ = conn.execute("UPDATE species_records SET status = 'extinct', extinction_time = ?1 WHERE id = ?2", params![now_millis, id]);
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
                            synapse_weights: random_pheno.brain.synapses.iter().map(|s| s.weight).collect(),
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

            // B. NATIVE PARALLEL TRAINER EVOLUTION TICK
            if is_trainer_active {
                if trainer_is_running {
                    // Run trainer_warp_speed physics steps (cap steps per 60Hz tick to prevent thread blocking/IPC flooding)
                    let steps_this_frame = if trainer_warp_speed > 35 { 35 } else { trainer_warp_speed };
                    for _step in 0..steps_this_frame {
                        if !trainer_is_running {
                            break;
                        }

                        trainer_epoch_ticks += 1;

                        // Step all sandbox physical loops in Rust!
                        for sb in &mut trainer_sandboxes {
                            sb.current_fitness = calculate_sandbox_fitness(
                                sb.accumulated_yield,
                                sb.finish_tick,
                                300,
                                sb.start_distance,
                                sb.distance_traveled,
                                sb.wall_collisions,
                                sb.min_distance,
                                sb.agent.px,
                                sb.agent.py,
                            );

                            // Run 1 tick of continuous physics, neural nets, and collisions in Rust for full 300 ticks!
                            step_trainer_sandbox_physics(sb, 1000.0, 1000.0);
                        }

                        // If we completed the 300-tick epoch:
                        if trainer_epoch_ticks >= 300 {
                            // 1. Calculate fitness for all sandboxes
                            for sb in &mut trainer_sandboxes {
                                sb.current_fitness = calculate_sandbox_fitness(
                                    sb.accumulated_yield,
                                    sb.finish_tick,
                                    300,
                                    sb.start_distance,
                                    sb.distance_traveled,
                                    sb.wall_collisions,
                                    sb.min_distance,
                                    sb.agent.px,
                                    sb.agent.py,
                                );
                            }

                            // If Multi-Trial is active, manage trial runs
                            let mut run_breeding_step = true;
                            if trainer_multi_trial {
                                if trainer_accumulated_fitness.len() != trainer_sandboxes.len() {
                                    trainer_accumulated_fitness = vec![0.0; trainer_sandboxes.len()];
                                }
                                if trainer_accumulated_eaten.len() != trainer_sandboxes.len() {
                                    trainer_accumulated_eaten = vec![0; trainer_sandboxes.len()];
                                }
                                for (idx, sb) in trainer_sandboxes.iter().enumerate() {
                                    trainer_accumulated_fitness[idx] += sb.current_fitness;
                                    if sb.finished {
                                        trainer_accumulated_eaten[idx] += 1;
                                    }
                                }

                                if trainer_trial_index < 2 {
                                    trainer_trial_index += 1;
                                    trainer_epoch_ticks = 0;
                                    
                                    let mut rng = rand::thread_rng();
                                    for sb in &mut trainer_sandboxes {
                                        sb.agent.px = 500.0;
                                        sb.agent.py = 500.0;
                                        sb.agent.vx = 0.0;
                                        sb.agent.vy = 0.0;
                                        sb.agent.heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                        sb.agent.bend_angle = 0.0;
                                        sb.agent.omega_rot = 0.0;
                                        sb.agent.energy = 100.0;
                                        sb.agent.adrenaline = 1.0;
                                        sb.agent.age = 0;
                                        sb.agent.has_eaten = false;
                                        
                                        sb.agent.neuron_states.clear();
                                        sb.agent.neuron_activations.clear();
                                        
                                        sb.finished = false;
                                        sb.finish_tick = None;
                                        sb.distance_traveled = 0.0;
                                        sb.wall_collisions = 0;
                                        sb.wall_collision_cooldown = 0;
                                        sb.consumed_spore_type = None;
                                        sb.epoch_ticks = 0;
                                        sb.accumulated_yield = 0.0;
                                        sb.consumed_count = 0;
                                        
                                        // Randomize spores respecting minimum 200px distance
                                        for spore in &mut sb.foods {
                                            let mut valid_spawn = false;
                                            while !valid_spawn {
                                                spore.x = 25.0 + rng.gen_range(0.0..950.0);
                                                spore.y = 25.0 + rng.gen_range(0.0..950.0);
                                                let dx = spore.x - sb.agent.px;
                                                let dy = spore.y - sb.agent.py;
                                                let dist = (dx*dx + dy*dy).sqrt();
                                                if dist >= 200.0 {
                                                    valid_spawn = true;
                                                }
                                            }
                                            spore.vx = 0.0;
                                            spore.vy = 0.0;
                                        }
                                        
                                        let carnivory = sb.agent.phenotype.carnivory;
                                        let target_food = if carnivory >= 0.65 {
                                            &sb.foods[1] // Strict Carnivore targets meat
                                        } else if carnivory >= 0.35 {
                                            // Omnivore targets whichever is closer on reset!
                                            let dist_plant = ((sb.foods[0].x - sb.agent.px).powi(2) + (sb.foods[0].y - sb.agent.py).powi(2)).sqrt();
                                            let dist_meat = ((sb.foods[1].x - sb.agent.px).powi(2) + (sb.foods[1].y - sb.agent.py).powi(2)).sqrt();
                                            if dist_meat <= dist_plant { &sb.foods[1] } else { &sb.foods[0] }
                                        } else {
                                            &sb.foods[0] // Strict Herbivore targets plant
                                        };
                                        sb.start_distance = ((target_food.x - sb.agent.px).powi(2) + (target_food.y - sb.agent.py).powi(2)).sqrt();
                                        sb.min_distance = sb.start_distance;
                                        sb.current_fitness = 0.0;
                                    }
                                    run_breeding_step = false;
                                } else {
                                    // Average the fitness scores across the 3 trials and apply Lamarckian Viability Gate
                                    for (idx, sb) in trainer_sandboxes.iter_mut().enumerate() {
                                        let avg_fit = trainer_accumulated_fitness[idx] / 3.0;
                                        let eaten = trainer_accumulated_eaten[idx];
                                        
                                        if eaten == 0 {
                                            // Starvation-Filter: 90% penalty if candidate didn't eat in any trial!
                                            sb.current_fitness = avg_fit * 0.1;
                                        } else {
                                            // Success: Keep full average fitness and add +150 per successful catch!
                                            sb.current_fitness = avg_fit + (eaten as f32 * 150.0);
                                        }
                                    }
                                    trainer_trial_index = 0;
                                    trainer_accumulated_fitness.clear();
                                    trainer_accumulated_eaten.clear();
                                }
                            }

                            if run_breeding_step {
                                // Sort by fitness DESC
                                trainer_sandboxes.sort_by(|a, b| b.current_fitness.partial_cmp(&a.current_fitness).unwrap_or(std::cmp::Ordering::Equal));

                                let best_fit = trainer_sandboxes[0].current_fitness;
                                let avg_fit = trainer_sandboxes.iter().map(|s| s.current_fitness).sum::<f32>() / (trainer_N as f32);
                                let best_genome = trainer_sandboxes[0].agent.genome.clone();

                                println!("[TRAINER RUN] Gen {} completed! Best Fit: {:.1}, Avg Fit: {:.1}", trainer_generation, best_fit, avg_fit);

                                // Broadcast completion metadata back to UI instantly
                                emit_state(json!({
                                    "type": "TRAINER_GENERATION_COMPLETED",
                                    "generation": trainer_generation,
                                    "bestFitness": best_fit,
                                    "avgFitness": avg_fit,
                                    "bestGenome": best_genome,
                                }));

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

                                // Re-trigger species selection / listing refreshes
                                emit_state(json!({ "type": "DATABASE_CHANGED" }));
                                
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
                }

                // C. Stream Trainer Telemetry to UI at 25Hz (lightweight frames)
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

                    let time_str = format!("{:.1}s", ((300.0 - trainer_epoch_ticks as f32) / 60.0).max(0.0));

                    // Compute selected sandbox live neural activations for real-time brain rendering
                    let selected_brain_json = if let Some(sel_id) = trainer_selected_sandbox_id {
                        if let Some(sel_sb) = trainer_sandboxes.iter().find(|s| s.id == sel_id) {
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
                        "generation": trainer_generation,
                        "isRunning": trainer_is_running,
                        "timeStr": time_str,
                        "sandboxes": telemetry_sandboxes,
                        "selectedBrain": selected_brain_json
                    }));

                    last_emit_time = Instant::now();
                }
            }

            // Cap the physical thread at ~60Hz to prevent thread starvation
            let elapsed = loop_start.elapsed();
            let frame_duration = Duration::from_micros(16667); // 16.67ms
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }
    });
}
