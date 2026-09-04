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
    pub scenario: String,
    pub cumulative_rotation: f32,
    pub coverage_mask: u64,
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
    pub scenario: String,
    pub cumulative_rotation: f32,
    pub coverage_mask: u64,
}

pub fn init_rust_sandbox(
    id: u32,
    parent_genome: &str,
    current_generation: u32,
    mutation_rate: f32,
    origin_type: &str,
    canvas_width: f32,
    canvas_height: f32,
    scenario_name: &str,
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

    let world = generate_trainer_world(&format!("SANDBOX_SEED_{}_GEN_{}", id, current_generation), 19200.0, 10800.0);

    let mut sb = TrainerSandbox {
        id,
        agent,
        foods: Vec::new(),
        world,
        finished: false,
        finish_tick: None,
        start_distance: 0.0,
        current_fitness: 0.0,
        distance_traveled: 0.0,
        wall_collisions: 0,
        wall_collision_cooldown: 0,
        consumed_spore_type: None,
        origin_type: origin_type.to_string(),
        epoch_ticks: 0,
        min_distance: 0.0,
        accumulated_yield: 0.0,
        consumed_count: 0,
        scenario: scenario_name.to_string(),
        cumulative_rotation: 0.0,
        coverage_mask: 0,
    };

    let plugin = crate::biology::scenarios::get_scenario_plugin(scenario_name);
    plugin.initialize(&mut sb, canvas_width, canvas_height);

    sb
}

pub fn step_trainer_sandbox_physics(sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32) {
    let plugin = crate::biology::scenarios::get_scenario_plugin(&sb.scenario);
    let app_config = crate::shared::types::AppConfig::global();
    let hit_wall = plugin.step_physics(sb, canvas_width, canvas_height, &app_config);

    // Track visited 8x8 grid sectors for unique canvas coverage
    let sec_x = ((sb.agent.px / canvas_width) * 8.0).floor().clamp(0.0, 7.0) as u32;
    let sec_y = ((sb.agent.py / canvas_height) * 8.0).floor().clamp(0.0, 7.0) as u32;
    let sector_idx = sec_y * 8 + sec_x;
    sb.coverage_mask |= 1 << sector_idx;

    if hit_wall {
        if sb.wall_collision_cooldown == 0 {
            sb.wall_collisions += 1;
            sb.wall_collision_cooldown = 20; // Cooldown to prevent rubbing count
        }
    }

    if sb.wall_collision_cooldown > 0 {
        sb.wall_collision_cooldown -= 1;
    }

    // 4. Update minimum distance to any spore reached during this trial run
    let mut min_dist = f32::MAX;
    for f in &sb.foods {
        let dx = f.x - sb.agent.px;
        let dy = f.y - sb.agent.py;
        let dist = (dx*dx + dy*dy).sqrt();
        min_dist = min_dist.min(dist);
    }
    sb.min_distance = sb.min_distance.min(min_dist);

    // Separately check collisions and consumption for ALL spores in sb.foods
    let num_foods = sb.foods.len();
    for target_idx in 0..num_foods {
        let f = &sb.foods[target_idx];
        let dx = f.x - sb.agent.px;
        let dy = f.y - sb.agent.py;
        let dist = (dx*dx + dy*dy).sqrt();

        let mean_radius = sb.agent.phenotype.spinal_harmonics.mean_radius;
        let is_meat = sb.foods[target_idx].type_id == 2;
        let base_eat_dist = if is_meat {
            mean_radius * 1.6 * 0.5 + 5.0
        } else {
            mean_radius * 1.5 * 0.5 + 8.0
        };
        let eat_dist = (mean_radius + 10.0).max(base_eat_dist);

        if dist <= eat_dist {
            let carnivory = sb.agent.phenotype.carnivory;
            let yield_val = if is_meat {
                carnivory // Meat gives C
            } else {
                1.0 - carnivory // Plant gives 1 - C
            };

            sb.accumulated_yield += yield_val;
            sb.consumed_count += 1;

            if !sb.finished {
                sb.finished = true; // Still mark finished as true for green SUCCESS label!
                sb.finish_tick = Some(sb.epoch_ticks);
                sb.consumed_spore_type = Some(if is_meat { "meat" } else { "plant" }.to_string());
                sb.agent.has_eaten = true;
            }

            // Spore Respawn respecting the scenario bounds
            let mut rng = rand::thread_rng();
            let mut valid_spawn = false;
            let is_exploration = sb.scenario == "exploration";
            // Out-of-sight spawn limit: 550.0 pixels away from the creature's current position!
            let min_dist_limit = if is_exploration { 550.0 } else { 200.0 };

            while !valid_spawn {
                sb.foods[target_idx].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
                sb.foods[target_idx].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
                let rdx = sb.foods[target_idx].x - sb.agent.px;
                let rdy = sb.foods[target_idx].y - sb.agent.py;
                let rdist = (rdx*rdx + rdy*rdy).sqrt();
                if rdist >= min_dist_limit {
                    valid_spawn = true;
                }
            }
            sb.foods[target_idx].vx = 0.0;
            sb.foods[target_idx].vy = 0.0;
        }
    }
}
