use serde::{Deserialize, Serialize};
use rand::Rng;
use ts_rs::TS;

use crate::shared::types::{CreatureAgent, FoodSpore};
use crate::shared::map_generator::{generate_trainer_world, ProceduralWorld};
use crate::biology::dna::{parse_genome, mutate_genome, generate_random_genome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerSandbox {
    pub id: u32,
    pub agent: CreatureAgent,
    pub foods: Vec<FoodSpore>,
    pub world: ProceduralWorld,
    pub finished: bool,
    pub finish_tick: Option<u32>,
    pub start_distance: f32,
    pub current_fitness: f32,
    pub distance_traveled: f32,
    pub wall_collisions: u32,
    pub wall_collision_cooldown: u32,
    pub consumed_spore_type: Option<String>,
    pub origin_type: String, // "elite" | "hof" | "mutant" | "random"
    pub epoch_ticks: u32,
    pub min_distance: f32,
    pub accumulated_yield: f32,
    pub consumed_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TrainerTelemetrySandbox {
    pub id: u32,
    #[serde(rename = "chamberSize")]
    #[ts(rename = "chamberSize")]
    pub chamber_size: f32,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub heading_angle: f32,
    pub omega_rot: f32,
    pub finished: bool,
    pub current_fitness: f32,
    pub origin_type: String,
    pub consumed_spore_type: Option<String>,
    pub foods: Vec<FoodSpore>,
    pub world: ProceduralWorld,
    pub latin_name: String,
    pub primary_color_h: f32,
    pub primary_color_s: f32,
    pub primary_color_l: f32,
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    pub genome: String,
}

pub fn calculate_sandbox_fitness(
    accumulated_yield: f32,
    finish_tick: Option<u32>,
    epoch_duration_ticks: u32,
    start_distance: f32,
    distance_traveled: f32,
    wall_collisions: u32,
    min_dist: f32,
    end_x: f32,
    end_y: f32,
) -> f32 {
    let wall_penalty = (1.0 - (wall_collisions as f32) * 0.10).max(0.1);
    let fit;

    if accumulated_yield > 0.0 {
        // Path efficiency: ratio of ideal straight-line distance to actual distance traveled
        let path_efficiency = start_distance / (start_distance.max(distance_traveled)).max(0.1);
        let speed_bonus = (epoch_duration_ticks - finish_tick.unwrap_or(epoch_duration_ticks)) as f32 * 0.2;
        fit = (accumulated_yield * 1000.0 + 1000.0 * path_efficiency + speed_bonus) * wall_penalty;
    } else {
        // Unsuccessful: proximity reward with standstill, circular & aimless traveling penalties!
        // We use scaling multipliers instead of harsh 0.0 cuts to keep a smooth fitness landscape!
        let mut penalty_multiplier = 1.0;

        // 1. Standstill penalty (applied if candidate barely moved and is still far from target)
        if distance_traveled < 120.0 && min_dist >= 40.0 {
            penalty_multiplier *= 0.3; // 70% penalty
        }

        // 2. Circular movement detection
        let displacement = ((end_x - 500.0).powi(2) + (end_y - 500.0).powi(2)).sqrt();
        if distance_traveled > 150.0 && displacement < 80.0 {
            penalty_multiplier *= 0.2; // 80% penalty
        }

        let base_fit = if min_dist < start_distance {
            1000.0 * (1.0 - min_dist / start_distance)
        } else {
            0.0
        };
        let kinetic_waste = distance_traveled * 0.05; // 50% reduced kinetic penalty to encourage early swimming
        fit = ((base_fit - kinetic_waste).max(0.0) * penalty_multiplier * wall_penalty).max(0.0);
    }

    if fit.is_nan() || fit.is_infinite() {
        return 0.0;
    }
    fit
}

pub fn init_rust_sandbox(
    id: u32,
    parent_genome: &str,
    current_generation: u32,
    mutation_rate: f32,
    origin_type: &str,
    canvas_width: f32,
    canvas_height: f32,
) -> TrainerSandbox {
    let mut rng = rand::thread_rng();

    // Genetically evolve or mutate genome
    let mut genome = parent_genome.to_string();
    if origin_type == "mutant" && mutation_rate > 0.0 {
        let mutations_count = (genome.len() as f32 * mutation_rate).round() as usize;
        for _ in 0..mutations_count {
            if let Some((mutated, _, _, _)) = mutate_genome(&genome) {
                genome = mutated;
            }
        }
    } else if origin_type == "random" {
        genome = generate_random_genome(256);
    }

    let pheno = parse_genome(&genome, None, None);

    let agent = CreatureAgent {
        id,
        species_id: genome.clone(),
        px: canvas_width / 2.0,
        py: canvas_height / 2.0,
        vx: 0.0,
        vy: 0.0,
        heading_angle: rng.gen_range(0.0..std::f32::consts::TAU),
        bend_angle: 0.0,
        omega_rot: 0.0,
        energy: 100.0,
        adrenaline: 1.0,
        age: 0,
        generation: current_generation,
        has_eaten: false,
        genome: genome.clone(),
        antisense: String::new(),
        phenotype: pheno.clone(),
        neuron_states: Vec::new(),
        neuron_activations: Vec::new(),
        synapse_weights: pheno.brain.synapses.iter().map(|s| s.weight).collect(),
    };

    // Setup exactly two food spores (1 plant green [type_id: 1], 1 meat red [type_id: 2])
    let mut foods = vec![
        FoodSpore { id: 1, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant
        FoodSpore { id: 2, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meatball
    ];

    let is_exploration = canvas_width >= 2000.0;
    let min_dist = if is_exploration { 1200.0 } else { 200.0 };

    // Spawn foods[0] (Plant) - uniform distribution across the entire canvas with min_dist limit!
    let mut valid_spawn_0 = false;
    while !valid_spawn_0 {
        foods[0].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
        foods[0].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
        let dx = foods[0].x - agent.px;
        let dy = foods[0].y - agent.py;
        let dist = (dx*dx + dy*dy).sqrt();
        if dist >= min_dist {
            valid_spawn_0 = true;
        }
    }

    // Spawn foods[1] (Meat) - uniform distribution across the entire canvas with min_dist limit!
    let mut valid_spawn_1 = false;
    while !valid_spawn_1 {
        foods[1].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
        foods[1].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
        let dx = foods[1].x - agent.px;
        let dy = foods[1].y - agent.py;
        let dist = (dx*dx + dy*dy).sqrt();
        if dist >= min_dist {
            valid_spawn_1 = true;
        }
    }

    let dist_plant = ((foods[0].x - agent.px).powi(2) + (foods[0].y - agent.py).powi(2)).sqrt();
    let dist_meat = ((foods[1].x - agent.px).powi(2) + (foods[1].y - agent.py).powi(2)).sqrt();
    let start_distance = dist_plant.min(dist_meat);

    let world = generate_trainer_world(&format!("SANDBOX_SEED_{}_GEN_{}", id, current_generation), 19200.0, 10800.0);

    TrainerSandbox {
        id,
        agent,
        foods,
        world,
        finished: false,
        finish_tick: None,
        start_distance,
        current_fitness: 0.0,
        distance_traveled: 0.0,
        wall_collisions: 0,
        wall_collision_cooldown: 0,
        consumed_spore_type: None,
        origin_type: origin_type.to_string(),
        epoch_ticks: 0,
        min_distance: start_distance,
        accumulated_yield: 0.0,
        consumed_count: 0,
    }
}

pub fn step_trainer_sandbox_physics(sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32) {
    let mean_radius = sb.agent.phenotype.spinal_harmonics.mean_radius;
    sb.agent.age += 1;
    sb.epoch_ticks += 1;

    // 1. Compute sensory chemoreception inputs
    let clock_val = 0.5 + 0.5 * ((sb.agent.age as f32) * 0.1).sin();
    let inputs = crate::shared::brain::compute_sensory_inputs(&sb.agent, clock_val, &sb.foods, canvas_width, canvas_height);

    // 2. Execute Recurrent CTRNN Brain Euler Integration with live Hebbian learning
    let app_config = crate::shared::types::AppConfig::global();
    use crate::shared::brain::execute_brain_with_learning;
    let outputs = execute_brain_with_learning(
        &sb.agent.phenotype.brain,
        &inputs,
        &mut sb.agent.neuron_states,
        &mut sb.agent.neuron_activations,
        &mut sb.agent.synapse_weights,
        app_config.rules.hebbian_learning_rate_base,
        app_config.rules.hebbian_learning_stiffness_decay,
        app_config.rules.hebbian_forgetting_decay,
    );
    let out_thrust = outputs[0];
    let out_left = outputs[1];

    // 3. Locomotion Physical Kinematics (decoupled from hardcodes, matching config rules!)
    use crate::shared::physics::step_creature_kinematics;
    let hit_wall = step_creature_kinematics(&mut sb.agent, out_thrust, out_left, &app_config, canvas_width, canvas_height);

    // Track cumulative distance traveled
    let movement = (sb.agent.vx.powi(2) + sb.agent.vy.powi(2)).sqrt();
    sb.distance_traveled += movement;

    if hit_wall {
        if sb.wall_collision_cooldown == 0 {
            sb.wall_collisions += 1;
            sb.wall_collision_cooldown = 20; // Cooldown to prevent rubbing count
        }
    }

    if sb.wall_collision_cooldown > 0 {
        sb.wall_collision_cooldown -= 1;
    }

    // 4. Collision and Spore Consumption Checks (TCK-122: 100% Continuous Foraging without any Eat Blocks!)
    let carnivory = sb.agent.phenotype.carnivory;

    let dx_plant = sb.foods[0].x - sb.agent.px;
    let dy_plant = sb.foods[0].y - sb.agent.py;
    let dist_plant = (dx_plant*dx_plant + dy_plant*dy_plant).sqrt();

    let dx_meat = sb.foods[1].x - sb.agent.px;
    let dy_meat = sb.foods[1].y - sb.agent.py;
    let dist_meat = (dx_meat*dx_meat + dy_meat*dy_meat).sqrt();

    // Update minimum distance to ANY spore reached during this trial run
    let min_dist = dist_plant.min(dist_meat);
    sb.min_distance = sb.min_distance.min(min_dist);

    // Separately check collisions and consumption for BOTH plant (index 0) and meat (index 1) spores!
    for target_idx in 0..2 {
        let dist = if target_idx == 0 { dist_plant } else { dist_meat };

        let base_eat_dist = if target_idx == 1 {
            mean_radius * 1.6 * 0.5 + 5.0
        } else {
            mean_radius * 1.5 * 0.5 + 8.0
        };
        let eat_dist = (mean_radius + 10.0).max(base_eat_dist);

        if dist <= eat_dist {
            // Calculate metabolic yield based on what spore was consumed
            let yield_val = if target_idx == 1 {
                carnivory // Meat gives C
            } else {
                1.0 - carnivory // Plant gives 1 - C
            };

            sb.accumulated_yield += yield_val;
            sb.consumed_count += 1;

            if !sb.finished {
                sb.finished = true; // Still mark finished as true for green SUCCESS label!
                sb.finish_tick = Some(sb.epoch_ticks);
                sb.consumed_spore_type = Some(if target_idx == 1 { "meat" } else { "plant" }.to_string());
                sb.agent.has_eaten = true;
            }

            // Instant Spore Respawn!
            let mut rng = rand::thread_rng();
            let mut valid_spawn = false;
            let is_exploration = canvas_width >= 2000.0;
            let min_dist = if is_exploration { 1200.0 } else { 200.0 };
            while !valid_spawn {
                sb.foods[target_idx].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
                sb.foods[target_idx].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
                let rdx = sb.foods[target_idx].x - sb.agent.px;
                let rdy = sb.foods[target_idx].y - sb.agent.py;
                let rdist = (rdx*rdx + rdy*rdy).sqrt();
                if rdist >= min_dist {
                    valid_spawn = true;
                }
            }
            sb.foods[target_idx].vx = 0.0;
            sb.foods[target_idx].vy = 0.0;
        }
    }

    // 5. Spore physical displacement impulse pushes and drift physics (unified!)
    use crate::shared::physics::step_food_spore_physics;
    let temp_creatures = vec![sb.agent.clone()];
    for pellet in &mut sb.foods {
        step_food_spore_physics(pellet, &temp_creatures, canvas_width, canvas_height);
    }
}
