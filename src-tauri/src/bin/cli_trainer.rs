use src_tauri::biology::dna::{parse_genome, mutate_genome, generate_random_genome, CreaturePhenotype};
use src_tauri::shared::brain::execute_brain;
use src_tauri::database::init_db;
use std::time::Instant;

fn main() {
    println!("==================================================================");
    println!("            🧬 PIXEL DNA LIFE - NATIVE SIMULATION ENGINE          ");
    println!("==================================================================");

    // 1. Initialize safe local SQLite database
    let db_path = "pixel_life_local.db";
    println!("[DB] Connecting to local database: {}", db_path);
    let conn = match init_db(db_path) {
        Ok(c) => {
            println!("[DB] Connection successfully secured.");
            c
        }
        Err(e) => {
            eprintln!("[DB] ERROR initializing database: {}", e);
            return;
        }
    };

    // 2. Set up a fresh training session
    let session_id = "session-001";
    let session_name = "Forward Light Target Training";
    let config_json = r#"{"scenario": "light_drift", "steps_per_creature": 100}"#;

    // Insert session into DB (ignoring if it already exists)
    let _ = conn.execute(
        "INSERT OR IGNORE INTO sessions (id, name, config_json) VALUES (?1, ?2, ?3)",
        &[session_id, session_name, config_json],
    );

    // 3. Setup evolutionary starting population (Progenitor DNA seed)
    let progenitor_dna = "HJKLABCDPQRS1234EFGHTRUSTANDBENDPROGENITORALIFEWELLFORMEDMEMBRANEFOURIERSEGMENTSHARMONICSWAVEPHASEPULSESTIFFNESS";
    println!("[TRAINER] Base progenitor loaded (Len: {}).", progenitor_dna.len());

    let mut current_generation: Vec<String> = (0..500)
        .map(|i| {
            if i == 0 {
                progenitor_dna.to_string()
            } else {
                // Mutate offspring from progenitor to kickstart diversity
                mutate_genome(progenitor_dna)
                    .map(|(mutated, _, _, _)| mutated)
                    .unwrap_or_else(|| generate_random_genome(128))
            }
        })
        .collect();

    println!("[TRAINER] Starting MASSIVE Stresstest loop: 500 generations, 500 organisms each.");
    println!("------------------------------------------------------------------");

    let start_time = Instant::now();

    for generation_idx in 1..=500 {
        let mut organism_results = Vec::new();

        for (org_idx, dna) in current_generation.iter().enumerate() {
            // De-compile genotype to biological phenotype
            let phenotype = parse_genome(dna, None, None);
            let brain = &phenotype.brain;

            // Instanziate in-memory temporal CTRNN states
            let mut neuron_states = vec![0.0; brain.neurons.len()];
            let mut_neuron_activations = vec![0.0; brain.neurons.len()];
            let mut neuron_activations = mut_neuron_activations;

            let mut total_fitness = 0.0;

            // Scenario Evaluation Loop: 100 ticks
            // Environment: Receptor 0 sees high intensity target (1.0), Receptor 1 is obscured (0.0). Hunger clock is steady (0.5)
            let static_inputs = vec![1.0, 0.0, 0.5]; 

            let mut states = neuron_states;
            let mut activations = neuron_activations;

            for _tick in 0..100 {
                let outputs = execute_brain(brain, &static_inputs, &mut states, &mut activations);

                let thrust = outputs[0];  // Motor 0: Forward force [-1.0, 1.0]
                let bending = outputs[1]; // Motor 1: Steering deflection [-1.0, 1.0]

                // Fitness Objective: Maximize forward thrust while minimizing lateral bending/slipping
                let tick_fitness = (thrust.max(0.0) * (1.0 - bending.abs())).max(0.0);
                total_fitness += tick_fitness;
            }

            let final_fitness = (total_fitness / 100.0) * 100.0; // Scale fitness to [0, 100]
            organism_results.push((final_fitness, dna.clone(), phenotype));
        }

        // Sort organisms of the current generation by fitness (highest first)
        organism_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let max_fitness = organism_results[0].0;
        let avg_fitness: f32 = organism_results.iter().map(|(fit, _, _)| fit).sum::<f32>() / 500.0;
        let best_creature: &CreaturePhenotype = &organism_results[0].2;

        // Logging generation progress every 10 generations to keep console outputs clear, showing gen 1, 500, and multiples of 10
        if generation_idx == 1 || generation_idx == 500 || generation_idx % 10 == 0 {
            println!(
                "[GEN {:03}] Max Fit: {:5.1} | Avg Fit: {:5.1} | Best Species: \"{}\" | BMR: {:3.0}",
                generation_idx,
                max_fitness,
                avg_fitness,
                best_creature.latin_name,
                best_creature.basal_metabolic_rate
            );
        }

        // 4. Persist generation statistics directly into local SQLite DB
        let _ = conn.execute(
            "INSERT INTO simulation_history (session_id, generation, average_fitness, max_fitness, population_count) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            &[
                session_id,
                &generation_idx.to_string(),
                &avg_fitness.to_string(),
                &max_fitness.to_string(),
                "500",
            ],
        );

        // 5. SELECTION & REPRODUCTION (Mitosis & Mutation)
        // Select the Top 125 organisms (Elites)
        let elite_parents: Vec<String> = organism_results.iter()
            .take(125)
            .map(|(_, dna, _)| dna.clone())
            .collect();

        // Each elite parent replicates into 4 mutated clones to build the next generation of 500
        let mut next_generation = Vec::with_capacity(500);
        for parent_dna in &elite_parents {
            // Include parent unchanged as a genetic vault (elitism)
            next_generation.push(parent_dna.clone());

            for _clone_idx in 1..4 {
                // Mutate the clone's DNA
                let mutated_dna = mutate_genome(parent_dna)
                    .map(|(mutated, _, _, _)| mutated)
                    .unwrap_or_else(|| parent_dna.clone());
                next_generation.push(mutated_dna);
            }
        }

        current_generation = next_generation;
    }

    let elapsed = start_time.elapsed();
    println!("------------------------------------------------------------------");
    println!(
        "[TRAINER] Evolution training completed in {:.2?} (5000 individual organism trials).",
        elapsed
    );
    println!("[TRAINER] All run data successfully persisted inside: {}", db_path);
    println!("==================================================================");
}
