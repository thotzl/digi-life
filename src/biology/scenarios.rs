use crate::biology::trainer_engine::TrainerSandbox;
use crate::shared::types::{AppConfig, FoodSpore};
use rand::Rng;

/// Slices representing the standard trait-based interface for training scenarios (Curriculum Plugins)
pub trait TrainingScenarioPlugin: Send + Sync {
    fn name(&self) -> &'static str;
    
    /// Initializes food spores and sandbox conditions dynamically based on the active scenario
    fn initialize(&self, sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32);
    
    /// Executes scenario-specific physics, currents, or drift updates per tick
    fn step_physics(&self, sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32, app_config: &AppConfig) -> bool;
    
    /// Calculates scenario-specific evolutionary fitness at the end of the epoch
    fn calculate_fitness(
        &self,
        sb: &TrainerSandbox,
        epoch_duration_ticks: u32,
        canvas_width: f32,
        canvas_height: f32,
    ) -> f32;
}

/// Helper function to retrieve the active scenario plugin by name
pub fn get_scenario_plugin(name: &str) -> Box<dyn TrainingScenarioPlugin> {
    match name {
        "exploration" => Box::new(ExplorationScenario),
        _ => Box::new(StandardScenario),
    }
}

// ==========================================================================
// 1. STANDARD SCENARIO (Chamber 1000x1000, food always within view)
// ==========================================================================
pub struct StandardScenario;

impl TrainingScenarioPlugin for StandardScenario {
    fn name(&self) -> &'static str {
        "standard"
    }

    fn initialize(&self, sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32) {
        let mut rng = rand::thread_rng();
        sb.foods = vec![
            FoodSpore { id: 1, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant
            FoodSpore { id: 2, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meatball
        ];

        let min_dist = 200.0;

        // Spawn foods[0] (Plant)
        let mut valid_spawn_0 = false;
        while !valid_spawn_0 {
            sb.foods[0].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
            sb.foods[0].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
            let dx = sb.foods[0].x - sb.agent.px;
            let dy = sb.foods[0].y - sb.agent.py;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist >= min_dist {
                valid_spawn_0 = true;
            }
        }

        // Spawn foods[1] (Meat)
        let mut valid_spawn_1 = false;
        while !valid_spawn_1 {
            sb.foods[1].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
            sb.foods[1].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
            let dx = sb.foods[1].x - sb.agent.px;
            let dy = sb.foods[1].y - sb.agent.py;
            let dist = (dx*dx + dy*dy).sqrt();
            if dist >= min_dist {
                valid_spawn_1 = true;
            }
        }

        let dist_plant = ((sb.foods[0].x - sb.agent.px).powi(2) + (sb.foods[0].y - sb.agent.py).powi(2)).sqrt();
        let dist_meat = ((sb.foods[1].x - sb.agent.px).powi(2) + (sb.foods[1].y - sb.agent.py).powi(2)).sqrt();
        
        let carnivory = sb.agent.phenotype.carnivory;
        let target_food = if carnivory >= 0.60 {
            &sb.foods[1]
        } else if carnivory >= 0.40 {
            if dist_meat <= dist_plant { &sb.foods[1] } else { &sb.foods[0] }
        } else {
            &sb.foods[0]
        };

        sb.start_distance = ((target_food.x - sb.agent.px).powi(2) + (target_food.y - sb.agent.py).powi(2)).sqrt();
        sb.min_distance = sb.start_distance;
    }

    fn step_physics(&self, sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32, app_config: &AppConfig) -> bool {
        // Standard homing runs 100% barrier-free in standard training
        let clock_val = 0.5 + 0.5 * ((sb.agent.age as f32) * 0.1).sin();
        let inputs = crate::shared::brain::compute_sensory_inputs(&sb.agent, clock_val, &sb.foods, &[], canvas_width, canvas_height);

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

        let old_heading = sb.agent.heading_angle;
        use crate::shared::physics::step_creature_kinematics;
        let hit_wall = step_creature_kinematics(&mut sb.agent, out_thrust, out_left, app_config, &[], canvas_width, canvas_height);

        // Track absolute heading change (cumulative rotation in radians)
        let mut diff = sb.agent.heading_angle - old_heading;
        while diff > std::f32::consts::PI { diff -= std::f32::consts::TAU; }
        while diff < -std::f32::consts::PI { diff += std::f32::consts::TAU; }
        sb.cumulative_rotation += diff.abs();

        let movement = (sb.agent.vx.powi(2) + sb.agent.vy.powi(2)).sqrt();
        sb.distance_traveled += movement;

        use crate::shared::physics::step_food_spore_physics;
        let temp_creatures = vec![sb.agent.clone()];
        for pellet in &mut sb.foods {
            step_food_spore_physics(pellet, &temp_creatures, &[], canvas_width, canvas_height);
        }

        hit_wall
    }

    fn calculate_fitness(
        &self,
        sb: &TrainerSandbox,
        epoch_duration_ticks: u32,
        canvas_width: f32,
        canvas_height: f32,
    ) -> f32 {
        let wall_penalty = (1.0 - (sb.wall_collisions as f32) * 0.10).max(0.1);
        let fit;

        if sb.accumulated_yield > 0.0 {
            // Path efficiency: ratio of ideal straight-line distance to actual distance traveled
            let path_efficiency = sb.start_distance / (sb.start_distance.max(sb.distance_traveled)).max(0.1);
            // Quadratic penalty on path efficiency completely annihilates spiral/curving paths!
            let path_efficiency_score = 1000.0 * path_efficiency.powi(2);
            let speed_bonus = (epoch_duration_ticks - sb.finish_tick.unwrap_or(epoch_duration_ticks)) as f32 * 0.2;
            fit = (sb.accumulated_yield * 1000.0 + path_efficiency_score + speed_bonus) * wall_penalty;
        } else {
            let mut penalty_multiplier = 1.0;
            if sb.distance_traveled < 120.0 && sb.min_distance >= 40.0 {
                penalty_multiplier *= 0.3;
            }
            let displacement = ((sb.agent.px - (canvas_width / 2.0)).powi(2) + (sb.agent.py - (canvas_height / 2.0)).powi(2)).sqrt();
            if sb.distance_traveled > 150.0 && displacement < 120.0 {
                penalty_multiplier *= 0.05; // Brutal 95% penalty for localized looping on the spot!
            }
            let base_fit = if sb.min_distance < sb.start_distance {
                1000.0 * (1.0 - sb.min_distance / sb.start_distance)
            } else {
                0.0
            };
            let kinetic_waste = sb.distance_traveled * 0.20; // Quadrupled kinetic tax to suppress aimless swimming
            fit = ((base_fit - kinetic_waste).max(0.0) * penalty_multiplier * wall_penalty).max(0.0);
        }

        if fit.is_nan() || fit.is_infinite() { 0.0 } else { fit }
    }
}

// ==========================================================================
// 2. EXPLORATION SCENARIO (Chamber 3500x3500, 3-Zone sensor-gated exploration)
// ==========================================================================
pub struct ExplorationScenario;

impl TrainingScenarioPlugin for ExplorationScenario {
    fn name(&self) -> &'static str {
        "exploration"
    }

    fn initialize(&self, sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32) {
        let mut rng = rand::thread_rng();
        sb.foods = vec![
            FoodSpore { id: 1, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant 1
            FoodSpore { id: 2, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant 2
            FoodSpore { id: 3, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant 3
            FoodSpore { id: 4, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meat 1
            FoodSpore { id: 5, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meat 2
            FoodSpore { id: 6, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meat 3
        ];

        let min_dist = 1200.0; // Out-of-sight startup limit (1200px blind search)!

        for target_idx in 0..6 {
            let mut valid_spawn = false;
            while !valid_spawn {
                sb.foods[target_idx].x = 25.0 + rng.gen_range(0.0..(canvas_width - 50.0));
                sb.foods[target_idx].y = 25.0 + rng.gen_range(0.0..(canvas_height - 50.0));
                let dx = sb.foods[target_idx].x - sb.agent.px;
                let dy = sb.foods[target_idx].y - sb.agent.py;
                let dist = (dx*dx + dy*dy).sqrt();
                if dist >= min_dist {
                    valid_spawn = true;
                }
            }
        }

        // Calculate start distance to the single closest target food based on diet
        let dist_plant_0 = ((sb.foods[0].x - sb.agent.px).powi(2) + (sb.foods[0].y - sb.agent.py).powi(2)).sqrt();
        let dist_plant_1 = ((sb.foods[1].x - sb.agent.px).powi(2) + (sb.foods[1].y - sb.agent.py).powi(2)).sqrt();
        let dist_plant_2 = ((sb.foods[2].x - sb.agent.px).powi(2) + (sb.foods[2].y - sb.agent.py).powi(2)).sqrt();
        let min_plant_dist = dist_plant_0.min(dist_plant_1).min(dist_plant_2);

        let dist_meat_0 = ((sb.foods[3].x - sb.agent.px).powi(2) + (sb.foods[3].y - sb.agent.py).powi(2)).sqrt();
        let dist_meat_1 = ((sb.foods[4].x - sb.agent.px).powi(2) + (sb.foods[4].y - sb.agent.py).powi(2)).sqrt();
        let dist_meat_2 = ((sb.foods[5].x - sb.agent.px).powi(2) + (sb.foods[5].y - sb.agent.py).powi(2)).sqrt();
        let min_meat_dist = dist_meat_0.min(dist_meat_1).min(dist_meat_2);

        let carnivory = sb.agent.phenotype.carnivory;
        let target_dist = if carnivory >= 0.60 {
            min_meat_dist
        } else if carnivory >= 0.40 {
            min_meat_dist.min(min_plant_dist)
        } else {
            min_plant_dist
        };

        sb.start_distance = target_dist;
        sb.min_distance = sb.start_distance;
    }

    fn step_physics(&self, sb: &mut TrainerSandbox, canvas_width: f32, canvas_height: f32, app_config: &AppConfig) -> bool {
        // Find closest plant and meat spores among all foods for sensory inputs
        let mut nearest_plant: Option<FoodSpore> = None;
        let mut min_plant_dist = f32::MAX;
        let mut nearest_meat: Option<FoodSpore> = None;
        let mut min_meat_dist = f32::MAX;

        for f in &sb.foods {
            let dx = f.x - sb.agent.px;
            let dy = f.y - sb.agent.py;
            let dist_sq = dx * dx + dy * dy;
            if f.type_id == 1 {
                if dist_sq < min_plant_dist {
                    min_plant_dist = dist_sq;
                    nearest_plant = Some(f.clone());
                }
            } else if f.type_id == 2 {
                if dist_sq < min_meat_dist {
                    min_meat_dist = dist_sq;
                    nearest_meat = Some(f.clone());
                }
            }
        }

        let plant_target = nearest_plant.unwrap_or_else(|| FoodSpore {
            id: 1,
            type_id: 1,
            x: -99999.0,
            y: -99999.0,
            amount: 0.0,
            vx: 0.0,
            vy: 0.0,
        });
        let meat_target = nearest_meat.unwrap_or_else(|| FoodSpore {
            id: 2,
            type_id: 2,
            x: -99999.0,
            y: -99999.0,
            amount: 0.0,
            vx: 0.0,
            vy: 0.0,
        });

        let virtual_foods = vec![plant_target, meat_target];

        let clock_val = 0.5 + 0.5 * ((sb.agent.age as f32) * 0.1).sin();
        let inputs = crate::shared::brain::compute_sensory_inputs(&sb.agent, clock_val, &virtual_foods, &[], canvas_width, canvas_height);

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

        let old_heading = sb.agent.heading_angle;
        use crate::shared::physics::step_creature_kinematics;
        let hit_wall = step_creature_kinematics(&mut sb.agent, out_thrust, out_left, app_config, &[], canvas_width, canvas_height);

        // Track absolute heading change (cumulative rotation in radians)
        let mut diff = sb.agent.heading_angle - old_heading;
        while diff > std::f32::consts::PI { diff -= std::f32::consts::TAU; }
        while diff < -std::f32::consts::PI { diff += std::f32::consts::TAU; }
        sb.cumulative_rotation += diff.abs();

        let movement = (sb.agent.vx.powi(2) + sb.agent.vy.powi(2)).sqrt();
        sb.distance_traveled += movement;

        use crate::shared::physics::step_food_spore_physics;
        let temp_creatures = vec![sb.agent.clone()];
        for pellet in &mut sb.foods {
            step_food_spore_physics(pellet, &temp_creatures, &[], canvas_width, canvas_height);
        }

        hit_wall
    }

    fn calculate_fitness(
        &self,
        sb: &TrainerSandbox,
        epoch_duration_ticks: u32,
        canvas_width: f32,
        canvas_height: f32,
    ) -> f32 {
        let wall_penalty = (1.0 - (sb.wall_collisions as f32) * 0.15).max(0.1);
        let fit;

        let sectors_visited = sb.coverage_mask.count_ones() as f32;

        if sb.accumulated_yield > 0.0 {
            // Zone 3: Sated / Consumed Target Spore successfully (add minor coverage bonus)
            let path_efficiency = sb.start_distance / (sb.start_distance.max(sb.distance_traveled)).max(0.1);
            let speed_bonus = (epoch_duration_ticks - sb.finish_tick.unwrap_or(epoch_duration_ticks)) as f32 * 0.5;
            let coverage_bonus = sectors_visited * 10.0;
            fit = (sb.accumulated_yield * 3000.0 + 1000.0 * path_efficiency + speed_bonus + coverage_bonus) * wall_penalty;
        } else {
            let mut penalty_multiplier = 1.0;
            // Standstill/Laid back penalty
            if sb.distance_traveled < 100.0 {
                penalty_multiplier *= 0.1;
            }

            let displacement = ((sb.agent.px - (canvas_width / 2.0)).powi(2) + (sb.agent.py - (canvas_height / 2.0)).powi(2)).sqrt();

            // Kontinuierliche Suchbasis + additiver Homing-Bonus (keine Klippe!)
            let exploration_reward = displacement * 1.2;
            let homing_bonus = if sb.min_distance <= 550.0 {
                1500.0 * (1.0 - sb.min_distance / 550.0)
            } else {
                0.0
            };

            let base_fit = exploration_reward + homing_bonus + sectors_visited * 20.0;

            // Mild kinetic waste tax in Exploration mode to suppress extremely redundant, aimless wandering
            // but remains negligible for clean, wide-scale search walks (only 2% tax)
            let kinetic_waste = sb.distance_traveled * 0.02;
            fit = ((base_fit - kinetic_waste).max(0.0) * penalty_multiplier * wall_penalty).max(0.0);
        }

        let mut final_fit = fit;
        if sb.cumulative_rotation > 10.0 {
            let avg_radius = sb.distance_traveled / sb.cumulative_rotation;
            if avg_radius < 180.0 {
                final_fit *= 0.25; // Heavily penalize (75% reduction) localized looping on the spot, but don't K.o. completely on success!
            }
        }

        if final_fit.is_nan() || final_fit.is_infinite() { 0.0 } else { final_fit }
    }
}
