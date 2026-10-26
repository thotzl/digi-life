use crate::biology::trainer_engine::TrainerSandbox;
use crate::shared::fmath;
use crate::shared::rng::SimRng;
use crate::shared::types::{AppConfig, CreatureAgent, FoodSpore, TrainerFitnessConfig};
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

    /// Minimum distance of every food from the creature, at the start of each trial (the first one included) and for
    /// respawns after a catch (config `trainerSpawnMinDistance`, TCK-154 / RV-S08)
    fn spawn_min_distance(&self) -> f32;

    /// Places the scenario's food for a new trial (every food at least `spawn_min_distance` from the creature) and
    /// restarts the approach tracking; draws only the food positions from `rng`. Used by the first trial, every
    /// re-trial (`TrainerSandbox::reset_trial`) and room switches.
    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng);

    /// Food the creature's sensors see this tick (hook of the shared `step_physics`);
    /// `scratch` is a reused buffer for foods built per tick
    fn visible_foods<'a>(&self, agent: &CreatureAgent, foods: &'a [FoodSpore], scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore];
}

/// Helper function to retrieve the active scenario plugin
pub fn get_scenario_plugin(scenario: Scenario) -> &'static dyn TrainingScenarioPlugin {
    match scenario {
        Scenario::Standard => &StandardScenario,
        Scenario::Exploration => &ExplorationScenario,
    }
}

/// Spawns every food of `sb` at least the plugin's `spawn_min_distance` away from the agent (in food order) and
/// restarts the approach tracking.
fn spawn_foods(sb: &mut TrainerSandbox, rng: &mut SimRng) {
    let plugin = sb.scenario.plugin();
    let (size, min_dist) = (plugin.chamber_size(), plugin.spawn_min_distance());
    let from = (sb.agent.px, sb.agent.py);
    for food in &mut sb.foods {
        let (x, y) = spawn_at_min_distance(rng, size, from, min_dist, SPAWN_MAX_TRIES);
        food.x = x;
        food.y = y;
    }
    sb.reset_approach();
}

/// Trainer fitness of one trial in catch units (TCK-175); the same function for both rooms, the UI telemetry
/// (per tick) and the epoch end (runner and UI):
///
/// `fitness = yield + first_catch_bonus · (1 − finish_tick / epoch_ticks) + approach_bonus · approach`
///
/// - `yield` (primary): sum over the trial's catches of the diet fit, meat `carnivory`, plant `1 − carnivory` (at most
///   1.0 per catch, for a pure herbivore or carnivore eating its food), so 1.0 = one catch.
/// - time to first catch (secondary): only with a catch; `finish_tick` in `1..=epoch_ticks`, so the bonus lies in
///   `[0, first_catch_bonus)` and earlier is better.
/// - `approach` (shaping) in `[0, 1]`: relative approach `1 − d_min / d_sight` towards the nearest diet target
///   (`target_distance_by_diet`), counted from the tick it first came within the creature's food-sensing range
///   (`food_sense_range`); `d_sight` = distance at that tick, `d_min` = smallest distance since. The episode restarts
///   after every catch (it then rewards approach to the next food); 0 if no target was ever in range.
///
/// Both bonus weights are below 1 (config `trainerFitness`), so neither bonus is worth a full catch. No penalties:
/// wall contact, looping, standstill and distance from the centre do not enter; starvation ends the trial's catches
/// (TCK-178) and is no separate term. Non-finite results count as 0.
pub fn calculate_fitness(sb: &TrainerSandbox, epoch_ticks: u32, weights: &TrainerFitnessConfig) -> f32 {
    let time_bonus = match sb.finish_tick {
        Some(tick) if epoch_ticks > 0 => weights.first_catch_bonus * (1.0 - (tick.min(epoch_ticks) as f32 / epoch_ticks as f32)),
        _ => 0.0,
    };
    let fit = sb.accumulated_yield + time_bonus + weights.approach_bonus * sb.approach_fraction();
    if fit.is_finite() { fit } else { 0.0 }
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

    fn spawn_min_distance(&self) -> f32 {
        AppConfig::global().trainer_spawn_min_distance.standard
    }

    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng) {
        sb.foods = vec![
            FoodSpore { id: 1, type_id: 1, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // plant
            FoodSpore { id: 2, type_id: 2, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // meatball
        ];
        spawn_foods(sb, rng);
    }

    fn visible_foods<'a>(&self, _agent: &CreatureAgent, foods: &'a [FoodSpore], _scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore] {
        // Standard homing runs 100% barrier-free in standard training, all food is in view
        foods
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

    fn spawn_min_distance(&self) -> f32 {
        AppConfig::global().trainer_spawn_min_distance.exploration
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
        spawn_foods(sb, rng);
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

    use crate::biology::dna::WILDTYPE_GENOME;
    use crate::biology::trainer_engine::{init_rust_sandbox, SandboxOrigin};

    fn sandbox(scenario: Scenario) -> TrainerSandbox {
        init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, scenario, None, None, &mut seeded(3))
    }

    fn weights() -> TrainerFitnessConfig {
        TrainerFitnessConfig { first_catch_bonus: 0.5, approach_bonus: 0.3 }
    }

    #[test]
    fn test_configured_bonuses_stay_below_one_catch() {
        let w = &AppConfig::global().trainer_fitness;
        assert!((0.0..1.0).contains(&w.first_catch_bonus) && (0.0..1.0).contains(&w.approach_bonus), "{:?}", w);
    }

    #[test]
    fn test_one_catch_is_one_and_bonuses_are_bounded() {
        for scenario in [Scenario::Standard, Scenario::Exploration] {
            let epoch = scenario.plugin().epoch_ticks();
            let mut sb = sandbox(scenario);
            assert_eq!(calculate_fitness(&sb, epoch, &weights()), 0.0, "nothing done, nothing earned");

            // One catch with full diet fit at the last tick, no approach episode open: exactly 1.0
            sb.accumulated_yield = 1.0;
            sb.finish_tick = Some(epoch);
            assert_eq!(calculate_fitness(&sb, epoch, &weights()), 1.0);
            // The earliest possible first catch stays below one more catch
            sb.finish_tick = Some(1);
            let early = calculate_fitness(&sb, epoch, &weights());
            assert!(early > 1.0 && early < 1.5, "{}", early);
            // Earlier is better
            sb.finish_tick = Some(epoch / 2);
            assert!(calculate_fitness(&sb, epoch, &weights()) < early);

            // No catch, full approach: below one catch
            let mut miss = sandbox(scenario);
            miss.sight_distance = Some(400.0);
            miss.approach_min_distance = 0.0;
            let shaped = calculate_fitness(&miss, epoch, &weights());
            assert!((shaped - 0.3).abs() < 1e-6 && shaped < 1.0, "{}", shaped);
            // Half the distance: half the shaping
            miss.approach_min_distance = 200.0;
            assert!((calculate_fitness(&miss, epoch, &weights()) - 0.15).abs() < 1e-6);
            // Two catches beat one catch with every bonus
            let mut two = sandbox(scenario);
            two.accumulated_yield = 2.0;
            two.finish_tick = Some(epoch);
            let mut one = sandbox(scenario);
            one.accumulated_yield = 1.0;
            one.finish_tick = Some(1);
            one.sight_distance = Some(400.0);
            one.approach_min_distance = 1.0;
            assert!(calculate_fitness(&two, epoch, &weights()) > calculate_fitness(&one, epoch, &weights()));
        }
    }

    #[test]
    fn test_no_cliffs_for_walls_looping_standstill_or_centre_distance() {
        for scenario in [Scenario::Standard, Scenario::Exploration] {
            let epoch = scenario.plugin().epoch_ticks();
            let mut sb = sandbox(scenario);
            sb.accumulated_yield = 1.0;
            sb.finish_tick = Some(100);
            sb.sight_distance = Some(300.0);
            sb.approach_min_distance = 150.0;
            let reference = calculate_fitness(&sb, epoch, &weights());
            let mut variant = sb.clone();
            variant.wall_collisions = 50;
            variant.cumulative_rotation = 500.0;
            variant.distance_traveled = 0.0;
            variant.coverage_mask = u64::MAX;
            variant.agent.px = 30.0;
            variant.agent.py = 30.0;
            assert_eq!(calculate_fitness(&variant, epoch, &weights()), reference, "{:?}", scenario);
        }
    }

    #[test]
    fn test_same_fitness_scale_in_both_rooms() {
        // Same trial outcome, same fitness, although chamber and epoch differ
        let mut small = sandbox(Scenario::Standard);
        let mut large = sandbox(Scenario::Exploration);
        for (sb, epoch) in [(&mut small, 300u32), (&mut large, 1800u32)] {
            sb.accumulated_yield = 2.0;
            sb.finish_tick = Some(epoch / 4);
        }
        assert_eq!(calculate_fitness(&small, 300, &weights()), calculate_fitness(&large, 1800, &weights()));
    }
}
