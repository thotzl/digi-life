use crate::biology::trainer_engine::TrainerSandbox;
use crate::shared::fmath;
use crate::shared::rng::SimRng;
use crate::shared::types::{AppConfig, CreatureAgent, FoodSpore};
use rand::Rng;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Training room; selects the scenario plugin (chamber size, epoch length, food layout, fitness).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Scenario {
    #[default]
    Standard,
    Exploration,
}

impl Scenario {
    /// UI mode string (`SET_TRAINER_MODE`); anything but "exploration" is the standard room.
    pub fn from_mode(mode: &str) -> Self {
        if mode == "exploration" { Scenario::Exploration } else { Scenario::Standard }
    }

    pub fn plugin(self) -> &'static dyn TrainingScenarioPlugin {
        get_scenario_plugin(self)
    }
}

/// Carnivory at or above this value: the creature hunts meat only.
pub const CARNIVORE_MIN: f32 = 0.6;
/// Carnivory below this value: the creature grazes plants only (in between: omnivore).
pub const HERBIVORE_MAX: f32 = 0.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DietClass {
    Herbivore,
    Omnivore,
    Carnivore,
}

impl DietClass {
    pub fn from_carnivory(carnivory: f32) -> Self {
        if carnivory >= CARNIVORE_MIN {
            DietClass::Carnivore
        } else if carnivory >= HERBIVORE_MAX {
            DietClass::Omnivore
        } else {
            DietClass::Herbivore
        }
    }
}

/// Upper bound for spawn rejection sampling; afterwards the farthest candidate is used.
pub const SPAWN_MAX_TRIES: u32 = 1000;

/// Random point in the chamber (25 px margin) at least `min_dist` away from `from`.
/// After `max_tries` rejected candidates the farthest one found is returned.
pub fn spawn_at_min_distance(rng: &mut SimRng, chamber_size: f32, from: (f32, f32), min_dist: f32, max_tries: u32) -> (f32, f32) {
    let mut best = (from.0, from.1);
    let mut best_dist = -1.0;
    for _ in 0..max_tries {
        let x = 25.0 + rng.gen_range(0.0..(chamber_size - 50.0));
        let y = 25.0 + rng.gen_range(0.0..(chamber_size - 50.0));
        let dist = ((x - from.0) * (x - from.0) + (y - from.1) * (y - from.1)).sqrt();
        if dist >= min_dist {
            return (x, y);
        }
        if dist > best_dist {
            best = (x, y);
            best_dist = dist;
        }
    }
    best
}

/// Distance from the agent to the nearest food its diet targets (plants = type 1, meat = type 2;
/// omnivores take the nearer of both). `f32::MAX` if no such food exists.
pub fn target_distance_by_diet(agent: &CreatureAgent, foods: &[FoodSpore]) -> f32 {
    let nearest = |type_id: u32| {
        foods
            .iter()
            .filter(|f| f.type_id == type_id)
            .map(|f| ((f.x - agent.px).powi(2) + (f.y - agent.py).powi(2)).sqrt())
            .fold(f32::MAX, f32::min)
    };
    let (dist_plant, dist_meat) = (nearest(1), nearest(2));
    match DietClass::from_carnivory(agent.phenotype.carnivory) {
        DietClass::Carnivore => dist_meat,
        DietClass::Omnivore => dist_meat.min(dist_plant),
        DietClass::Herbivore => dist_plant,
    }
}

/// Slices representing the standard trait-based interface for training scenarios (Curriculum Plugins)
pub trait TrainingScenarioPlugin: Send + Sync {
    /// Edge length of the square chamber in px
    fn chamber_size(&self) -> f32;

    /// Ticks per epoch (one trial)
    fn epoch_ticks(&self) -> u32;

    /// Minimum distance of a food spore respawned after a catch
    fn respawn_min_distance(&self) -> f32;

    /// Minimum food distance when a new trial starts (multi-trial reset)
    fn trial_min_distance(&self) -> f32;

    /// Initializes food spores and sandbox conditions dynamically based on the active scenario
    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng);

    /// Food the creature's sensors see this tick (hook of the shared `step_physics`);
    /// `scratch` is a reused buffer for foods built per tick
    fn visible_foods<'a>(&self, agent: &CreatureAgent, foods: &'a [FoodSpore], scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore];

    /// Calculates scenario-specific evolutionary fitness at the end of the epoch
    fn calculate_fitness(&self, sb: &TrainerSandbox, epoch_duration_ticks: u32) -> f32;
}

/// Helper function to retrieve the active scenario plugin
pub fn get_scenario_plugin(scenario: Scenario) -> &'static dyn TrainingScenarioPlugin {
    match scenario {
        Scenario::Standard => &StandardScenario,
        Scenario::Exploration => &ExplorationScenario,
    }
}

/// Spawns every food of `sb` at least `min_dist` away from the agent (in food order).
fn spawn_foods(sb: &mut TrainerSandbox, rng: &mut SimRng, min_dist: f32) {
    let size = sb.scenario.plugin().chamber_size();
    let from = (sb.agent.px, sb.agent.py);
    for food in &mut sb.foods {
        let (x, y) = spawn_at_min_distance(rng, size, from, min_dist, SPAWN_MAX_TRIES);
        food.x = x;
        food.y = y;
    }
}

/// One tick of sensing, brain, metabolism, kinematics and food drift; shared by all scenarios.
/// Metabolism (TCK-178): the shared energy function drains `trainerMetabolicScale` times the Ocean cost; at 0 energy
/// the creature starves (`sb.starved`): it stops at once and does nothing else for the rest of the trial.
/// Returns whether the creature hit a wall.
pub fn step_physics(sb: &mut TrainerSandbox, app_config: &AppConfig) -> bool {
    if sb.starved {
        return false;
    }
    let plugin = sb.scenario.plugin();
    let size = plugin.chamber_size();

    let clock_val = 0.5 + 0.5 * fmath::sin((sb.agent.age as f32) * 0.1);
    let visible = plugin.visible_foods(&sb.agent, &sb.foods, &mut sb.visible_scratch);
    crate::shared::brain::compute_sensory_inputs_into(&sb.agent, clock_val, visible, &[], size, size, &mut sb.sensor_inputs);

    use crate::shared::brain::execute_brain_with_learning;
    let outputs = execute_brain_with_learning(
        &sb.agent.phenotype.brain,
        &sb.sensor_inputs,
        &mut sb.agent.neuron_states,
        &mut sb.agent.neuron_activations,
        &mut sb.agent.synapse_weights,
        &mut sb.agent.brain_runtime,
        app_config.rules.hebbian_learning_rate_base,
    );
    let out_thrust = outputs[0];
    let out_left = outputs[1];

    crate::biology::metabolism::drain_energy(&mut sb.agent, out_thrust, &app_config.metabolism, app_config.metabolism.trainer_metabolic_scale);
    if sb.agent.energy <= 0.0 {
        sb.agent.energy = 0.0;
        sb.agent.vx = 0.0;
        sb.agent.vy = 0.0;
        sb.starved = true;
        return false;
    }

    let old_heading = sb.agent.heading_angle;
    use crate::shared::physics::step_creature_kinematics;
    let hit_wall = step_creature_kinematics(&mut sb.agent, out_thrust, out_left, app_config, &[], size, size);

    // Track absolute heading change (cumulative rotation in radians)
    let mut diff = sb.agent.heading_angle - old_heading;
    while diff > std::f32::consts::PI { diff -= std::f32::consts::TAU; }
    while diff < -std::f32::consts::PI { diff += std::f32::consts::TAU; }
    sb.cumulative_rotation += diff.abs();

    let movement = (sb.agent.vx.powi(2) + sb.agent.vy.powi(2)).sqrt();
    sb.distance_traveled += movement;

    use crate::shared::physics::{step_food_spore_physics, SporePusher};
    let temp_creatures = [SporePusher::from(&sb.agent)];
    for pellet in &mut sb.foods {
        step_food_spore_physics(pellet, &temp_creatures, &[], size, size);
    }

    hit_wall
}

// ==========================================================================
// 1. STANDARD SCENARIO (Chamber 1000x1000, food always within view)
// ==========================================================================
pub struct StandardScenario;

impl TrainingScenarioPlugin for StandardScenario {
    fn chamber_size(&self) -> f32 {
        1000.0
    }

    fn epoch_ticks(&self) -> u32 {
        300
    }

    fn respawn_min_distance(&self) -> f32 {
        200.0
    }

    fn trial_min_distance(&self) -> f32 {
        200.0
    }

    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng) {
        sb.foods = vec![
            FoodSpore { id: 1, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant
            FoodSpore { id: 2, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meatball
        ];
        spawn_foods(sb, rng, 200.0);

        sb.start_distance = target_distance_by_diet(&sb.agent, &sb.foods);
        sb.min_distance = sb.start_distance;
    }

    fn visible_foods<'a>(&self, _agent: &CreatureAgent, foods: &'a [FoodSpore], _scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore] {
        // Standard homing runs 100% barrier-free in standard training, all food is in view
        foods
    }

    fn calculate_fitness(&self, sb: &TrainerSandbox, epoch_duration_ticks: u32) -> f32 {
        let size = self.chamber_size();
        let wall_penalty = (1.0 - (sb.wall_collisions as f32) * 0.10).max(0.1);
        let fit;

        if sb.accumulated_yield > 0.0 {
            // Path efficiency: ratio of ideal straight-line distance to actual distance traveled
            let path_efficiency = sb.start_distance / (sb.start_distance.max(sb.distance_traveled)).max(0.1);
            // Quadratic penalty on path efficiency completely annihilates spiral/curving paths!
            let path_efficiency_score = 1000.0 * path_efficiency.powi(2);
            let speed_bonus = epoch_duration_ticks.saturating_sub(sb.finish_tick.unwrap_or(epoch_duration_ticks)) as f32 * 0.2;
            fit = (sb.accumulated_yield * 1000.0 + path_efficiency_score + speed_bonus) * wall_penalty;
        } else {
            let mut penalty_multiplier = 1.0;
            if sb.distance_traveled < 120.0 && sb.min_distance >= 40.0 {
                penalty_multiplier *= 0.3;
            }
            let displacement = ((sb.agent.px - (size / 2.0)).powi(2) + (sb.agent.py - (size / 2.0)).powi(2)).sqrt();
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
    fn chamber_size(&self) -> f32 {
        3500.0
    }

    fn epoch_ticks(&self) -> u32 {
        1800
    }

    fn respawn_min_distance(&self) -> f32 {
        // Out-of-sight spawn limit: 550.0 pixels away from the creature's current position!
        550.0
    }

    fn trial_min_distance(&self) -> f32 {
        1200.0
    }

    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng) {
        sb.foods = vec![
            FoodSpore { id: 1, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant 1
            FoodSpore { id: 2, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant 2
            FoodSpore { id: 3, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant 3
            FoodSpore { id: 4, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meat 1
            FoodSpore { id: 5, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meat 2
            FoodSpore { id: 6, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meat 3
        ];
        spawn_foods(sb, rng, 650.0); // Dynamic curriculum blind search limit

        // Start distance to the single closest target food based on diet
        sb.start_distance = target_distance_by_diet(&sb.agent, &sb.foods);
        sb.min_distance = sb.start_distance;
    }

    fn visible_foods<'a>(&self, agent: &CreatureAgent, foods: &'a [FoodSpore], scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore] {
        // Sensors see only the closest plant and the closest meat spore
        let mut nearest_plant: Option<&FoodSpore> = None;
        let mut min_plant_dist = f32::MAX;
        let mut nearest_meat: Option<&FoodSpore> = None;
        let mut min_meat_dist = f32::MAX;

        for f in foods {
            let dx = f.x - agent.px;
            let dy = f.y - agent.py;
            let dist_sq = dx * dx + dy * dy;
            if f.type_id == 1 {
                if dist_sq < min_plant_dist {
                    min_plant_dist = dist_sq;
                    nearest_plant = Some(f);
                }
            } else if f.type_id == 2 {
                if dist_sq < min_meat_dist {
                    min_meat_dist = dist_sq;
                    nearest_meat = Some(f);
                }
            }
        }

        let absent = |id: u32, type_id: u32| FoodSpore { id, type_id, x: -99999.0, y: -99999.0, amount: 0.0, vx: 0.0, vy: 0.0 };
        let plant_target = nearest_plant.cloned().unwrap_or_else(|| absent(1, 1));
        let meat_target = nearest_meat.cloned().unwrap_or_else(|| absent(2, 2));

        scratch.clear();
        scratch.push(plant_target);
        scratch.push(meat_target);
        scratch
    }

    fn calculate_fitness(&self, sb: &TrainerSandbox, epoch_duration_ticks: u32) -> f32 {
        let size = self.chamber_size();
        let wall_penalty = (1.0 - (sb.wall_collisions as f32) * 0.15).max(0.1);
        let fit;

        let sectors_visited = sb.coverage_mask.count_ones() as f32;

        if sb.accumulated_yield > 0.0 {
            // Zone 3: Sated / Consumed Target Spore successfully (add minor coverage bonus)
            let path_efficiency = sb.start_distance / (sb.start_distance.max(sb.distance_traveled)).max(0.1);
            let speed_bonus = epoch_duration_ticks.saturating_sub(sb.finish_tick.unwrap_or(epoch_duration_ticks)) as f32 * 0.5;
            let coverage_bonus = sectors_visited * 10.0;
            fit = (sb.accumulated_yield * 3000.0 + 1000.0 * path_efficiency + speed_bonus + coverage_bonus) * wall_penalty;
        } else {
            let mut penalty_multiplier = 1.0;
            // Standstill/Laid back penalty
            if sb.distance_traveled < 100.0 {
                penalty_multiplier *= 0.1;
            }

            let displacement = ((sb.agent.px - (size / 2.0)).powi(2) + (sb.agent.py - (size / 2.0)).powi(2)).sqrt();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::rng::seeded;

    #[test]
    fn test_spawn_at_min_distance_respects_limit_and_terminates() {
        let mut rng = seeded(7);
        for _ in 0..50 {
            let (x, y) = spawn_at_min_distance(&mut rng, 1000.0, (500.0, 500.0), 200.0, SPAWN_MAX_TRIES);
            assert!(((x - 500.0).powi(2) + (y - 500.0).powi(2)).sqrt() >= 200.0);
            assert!((25.0..=975.0).contains(&x) && (25.0..=975.0).contains(&y));
        }
        // Unreachable distance: returns the farthest candidate instead of looping forever
        let (x, y) = spawn_at_min_distance(&mut rng, 1000.0, (500.0, 500.0), 5000.0, 20);
        assert!((25.0..=975.0).contains(&x) && (25.0..=975.0).contains(&y));
    }

    #[test]
    fn test_diet_class_thresholds() {
        assert_eq!(DietClass::from_carnivory(0.39), DietClass::Herbivore);
        assert_eq!(DietClass::from_carnivory(0.4), DietClass::Omnivore);
        assert_eq!(DietClass::from_carnivory(0.59), DietClass::Omnivore);
        assert_eq!(DietClass::from_carnivory(0.6), DietClass::Carnivore);
    }
}
