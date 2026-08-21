use serde::{Deserialize, Serialize};
use rand::Rng;
use ts_rs::TS;

use crate::shared::types::{CreatureAgent, FoodSpore};
use crate::shared::physics::apply_creature_physics;
use crate::shared::brain::execute_brain;
use crate::biology::dna::{parse_genome, mutate_genome, generate_random_genome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainerSandbox {
    pub id: u32,
    pub agent: CreatureAgent,
    pub foods: Vec<FoodSpore>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TrainerTelemetrySandbox {
    pub id: u32,
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
    pub latin_name: String,
    pub primary_color_h: f32,
    pub primary_color_s: f32,
    pub primary_color_l: f32,
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    pub genome: String,
}

pub fn compute_trainer_sensory_inputs(
    agent: &CreatureAgent,
    clock_val: f32,
    foods: &[FoodSpore],
    canvas_width: f32,
    canvas_height: f32,
) -> Vec<f32> {
    let k = agent.phenotype.organelles.len();
    let mut inputs = vec![0.0; k + 1];
    inputs[k] = clock_val;

    for (idx, patch) in agent.phenotype.organelles.iter().enumerate() {
        let range = patch.scale * 550.0;
        let alpha = (patch.angle - 90.0) * (std::f32::consts::PI / 180.0);
        let half_cone = (patch.bandwidth * 1.5).max(0.1);
        let aff = patch.spectral_affinity;
        let organ_power = patch.scale * (1.1 - patch.bandwidth);

        let mut max_stimulus: f32 = 0.0;

        // A. Food / Algae Scan (Herbivore sensors - plant spore is foods[0])
        let pellet = &foods[0];
        let dx = pellet.x - agent.px;
        let dy = pellet.y - agent.py;
        let dist = (dx*dx + dy*dy).sqrt();

        if dist <= range {
            let mut angle_rel = dy.atan2(dx) - agent.heading_angle;
            while angle_rel > std::f32::consts::PI { angle_rel -= std::f32::consts::TAU; }
            while angle_rel < -std::f32::consts::PI { angle_rel += std::f32::consts::TAU; }
            let mut delta_beta = angle_rel - alpha;
            while delta_beta > std::f32::consts::PI { delta_beta -= std::f32::consts::TAU; }
            while delta_beta < -std::f32::consts::PI { delta_beta += std::f32::consts::TAU; }

            if delta_beta.abs() <= half_cone {
                let match_val: f32;
                if aff >= 0.8 {
                    match_val = (1.0 - (aff - 0.33).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                } else if aff >= 0.25 && aff <= 0.65 {
                    match_val = (1.0 - (aff - 0.35).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                } else if aff < 0.25 {
                    match_val = (1.0 - (aff - 0.05).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                } else {
                    match_val = 0.0;
                }

                if match_val > 0.05 {
                    let strength = match_val * organ_power * (1.0 - dist / range) * delta_beta.cos();
                    max_stimulus = max_stimulus.max(strength);
                }
            }
        }

        // B. Peer / Prey Scan (Carnivore sensors - detecting the mocked MEATBALL spore foods[1])
        let other = &foods[1];
        let dx_peer = other.x - agent.px;
        let dy_peer = other.y - agent.py;
        let dist_peer = (dx_peer*dx_peer + dy_peer*dy_peer).sqrt();

        if dist_peer <= range {
            let mut angle_rel = dy_peer.atan2(dx_peer) - agent.heading_angle;
            while angle_rel > std::f32::consts::PI { angle_rel -= std::f32::consts::TAU; }
            while angle_rel < -std::f32::consts::PI { angle_rel += std::f32::consts::TAU; }
            let mut delta_beta = angle_rel - alpha;
            while delta_beta > std::f32::consts::PI { delta_beta -= std::f32::consts::TAU; }
            while delta_beta < -std::f32::consts::PI { delta_beta += std::f32::consts::TAU; }

            if delta_beta.abs() <= half_cone {
                let match_val: f32;
                if aff >= 0.8 {
                    // Thermal Heat Scan (target is 0.15)
                    match_val = (1.0 - (aff - 0.15).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                } else if aff >= 0.65 && aff < 0.8 {
                    // Vibration Scan (target is 0.0)
                    match_val = (1.0 - (aff - 0.0).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                } else if aff >= 0.25 && aff < 0.65 {
                    // Olfactory/Smell Scan (target is 0.50)
                    match_val = (1.0 - (aff - 0.50).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                } else {
                    // Visual Eye Scan (target is 1.0)
                    match_val = (1.0 - (aff - 1.0).abs() / (patch.bandwidth * 1.8 + 0.12)).max(0.0);
                }

                if match_val > 0.05 {
                    let strength = match_val * organ_power * (1.0 - dist_peer / range) * delta_beta.cos();
                    max_stimulus = max_stimulus.max(strength);
                }
            }
        }

        // C. Boundary wall pressure warning
        if aff < 0.25 {
            let wall_warning_zone = range * 0.5;
            let mut boundary_pressure = 0.0;
            if agent.px < wall_warning_zone {
                boundary_pressure = 1.0 - agent.px / wall_warning_zone;
            } else if agent.px > canvas_width - wall_warning_zone {
                boundary_pressure = 1.0 - (canvas_width - agent.px) / wall_warning_zone;
            }
            if agent.py < wall_warning_zone {
                boundary_pressure = boundary_pressure.max(1.0 - agent.py / wall_warning_zone);
            } else if agent.py > canvas_height - wall_warning_zone {
                boundary_pressure = boundary_pressure.max(1.0 - (canvas_height - agent.py) / wall_warning_zone);
            }

            if boundary_pressure > 0.0 {
                max_stimulus = max_stimulus.max(boundary_pressure * organ_power);
            }
        }

        // D. Proprioceptive centrifugal touch
        if aff < 0.25 {
            let speed = (agent.vx * agent.vx + agent.vy * agent.vy).sqrt();
            let rot_speed = agent.omega_rot.abs();
            let proprioceptive_stimulus = (speed * 0.15 + rot_speed * 0.35).min(1.0);
            if proprioceptive_stimulus > 0.0 {
                max_stimulus = max_stimulus.max(proprioceptive_stimulus * organ_power);
            }
        }

        inputs[idx] = max_stimulus.clamp(0.0, 1.0);
    }

    inputs
}

pub fn calculate_sandbox_fitness(
    finished: bool,
    finish_tick: Option<u32>,
    epoch_duration_ticks: u32,
    start_distance: f32,
    distance_traveled: f32,
    wall_collisions: u32,
    cur_dist: f32,
    end_x: f32,
    end_y: f32,
) -> f32 {
    let wall_penalty = (1.0 - (wall_collisions as f32) * 0.15).max(0.2);
    let fit;

    if finished && finish_tick.is_some() {
        let f_tick = finish_tick.unwrap();
        // Path efficiency: ratio of ideal straight-line distance to actual distance traveled
        let path_efficiency = start_distance / (start_distance.max(distance_traveled)).max(0.1);
        let speed_bonus = (epoch_duration_ticks - f_tick) as f32 * 0.2;
        fit = (1000.0 + 2000.0 * path_efficiency + speed_bonus) * wall_penalty;
    } else {
        // Unsuccessful: proximity reward with standstill, circular & aimless traveling penalties!
        if distance_traveled < 180.0 && cur_dist >= 20.0 {
            return 0.0;
        }

        // Kreisel-Erkennung (Circular movement detection):
        let displacement = ((end_x - 500.0).powi(2) + (end_y - 500.0).powi(2)).sqrt();
        if distance_traveled > 150.0 && displacement < 75.0 {
            return 0.0;
        }

        let base_fit = if cur_dist < start_distance {
            1000.0 * (1.0 - cur_dist / start_distance)
        } else {
            0.0
        };
        let kinetic_waste = distance_traveled * 0.1;
        fit = ((base_fit - kinetic_waste) * wall_penalty).max(0.0);
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
    };

    // Setup exactly two food spores (1 plant green [1000], 1 meat red [9999])
    let mut foods = vec![
        FoodSpore { id: 1000, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant
        FoodSpore { id: 9999, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meatball
    ];

    for spore in &mut foods {
        let mut valid_spawn = false;
        while !valid_spawn {
            spore.x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
            spore.y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
            let dx = spore.x - agent.px;
            let dy = spore.y - agent.py;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist >= 80.0 {
                valid_spawn = true;
            }
        }
    }

    let target_type = if pheno.carnivory >= 0.35 { "meat" } else { "plant" };
    let target_food = if target_type == "meat" { &foods[1] } else { &foods[0] };
    let start_distance = ((target_food.x - agent.px).powi(2) + (target_food.y - agent.py).powi(2)).sqrt();

    TrainerSandbox {
        id,
        agent,
        foods,
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
    }
}

pub fn step_trainer_sandbox_physics(sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32) {
    if sb.finished {
        return;
    }

    sb.agent.age += 1;
    sb.epoch_ticks += 1;

    // 1. Compute sensory chemoreception inputs
    let clock_val = 0.5 + 0.5 * ((sb.agent.age as f32) * 0.1).sin();
    let inputs = compute_trainer_sensory_inputs(&sb.agent, clock_val, &sb.foods, canvas_width, canvas_height);

    // 2. Execute Recurrent CTRNN Brain Euler Integration
    let outputs = execute_brain(&sb.agent.phenotype.brain, &inputs, &mut sb.agent.neuron_states, &mut sb.agent.neuron_activations);
    let out_thrust = outputs[0];
    let out_left = outputs[1];

    // 3. Locomotion Physical Kinematics (decoupled from hardcodes, matching config rules!)
    let app_config = crate::shared::types::AppConfig::global();
    let stiffness = sb.agent.phenotype.stiffness;
    let pulse = sb.agent.phenotype.pulse_speed;
    let mean_radius = sb.agent.phenotype.spinal_harmonics.mean_radius;
    let base_length = sb.agent.phenotype.spinal_harmonics.base_length;

    let mut thrust_mag = stiffness * (pulse * 1000.0 * pulse * 1000.0) * app_config.rules.thrust_base_multiplier;
    let wave_phase = sb.agent.phenotype.wave_phase;
    let eta_swim = ((base_length / (mean_radius * 3.5)) * wave_phase.sin().max(0.01) * stiffness).clamp(0.1, 3.2);
    thrust_mag *= eta_swim;

    let net_thrust_force = out_thrust * thrust_mag;

    let mass = mean_radius.powf(1.5) * (base_length / 25.0);
    let receptor_ballast = sb.agent.phenotype.organelles.len() as f32 * app_config.rules.receptor_ballast_scale;
    let drag_forward = (mean_radius * app_config.rules.drag_forward_coefficient + receptor_ballast) * (1.0 - stiffness * app_config.rules.drag_forward_stiffness_decay);

    // Apply native boundary reflections and physics kinematics
    let hit_wall = apply_creature_physics(&mut sb.agent, net_thrust_force, out_left, mass, drag_forward, 0.0, 0.0, canvas_width, canvas_height);

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

    // 4. Collision and Spore Consumption Checks
    let is_carnivore = sb.agent.phenotype.carnivory >= 0.35;
    let target_idx = if is_carnivore { 1 } else { 0 }; // meat vs plant spore

    let dx = sb.foods[target_idx].x - sb.agent.px;
    let dy = sb.foods[target_idx].y - sb.agent.py;
    let dist = (dx*dx + dy*dy).sqrt();

    let base_eat_dist = if is_carnivore {
        mean_radius * 1.6 * 0.5 + 5.0
    } else {
        mean_radius * 1.5 * 0.5 + 8.0
    };
    let eat_dist = (mean_radius + 10.0).max(base_eat_dist);

    if dist <= eat_dist {
        sb.finished = true;
        sb.finish_tick = Some(sb.epoch_ticks);
        sb.consumed_spore_type = Some(if is_carnivore { "meat" } else { "plant" }.to_string());
        sb.agent.has_eaten = true;
    }

    // 5. Spore physical displacement impulse pushes
    if !sb.finished {
        for pellet in &mut sb.foods {
            let pdx = pellet.x - sb.agent.px;
            let pdy = pellet.y - sb.agent.py;
            let pd = (pdx*pdx + pdy*pdy).sqrt();
            let min_dist = mean_radius + 8.0; // Spore radius = 8

            if pd < min_dist && pd > 0.1 {
                let overlap = min_dist - pd;
                let nx = pdx / pd;
                let ny = pdy / pd;

                pellet.x += nx * overlap;
                pellet.y += ny * overlap;

                // Clip spore to boundary
                pellet.x = pellet.x.clamp(8.0, canvas_width - 8.0);
                pellet.y = pellet.y.clamp(8.0, canvas_height - 8.0);

                pellet.vx = sb.agent.vx + nx * 2.0;
                pellet.vy = sb.agent.vy + ny * 2.0;
            }

            // Spore drift friction
            pellet.x += pellet.vx;
            pellet.y += pellet.vy;
            pellet.vx *= 0.92;
            pellet.vy *= 0.92;

            // Wall bounces for food spores
            if pellet.x < 8.0 { pellet.x = 8.0; pellet.vx = -pellet.vx.abs(); }
            else if pellet.x > canvas_width - 8.0 { pellet.x = canvas_width - 8.0; pellet.vx = pellet.vx.abs(); }
            if pellet.y < 8.0 { pellet.y = 8.0; pellet.vy = -pellet.vy.abs(); }
            else if pellet.y > canvas_height - 8.0 { pellet.y = canvas_height - 8.0; pellet.vy = pellet.vy.abs(); }
        }
    }
}
