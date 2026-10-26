use crate::biology::trainer_engine::TrainerSandbox;
use crate::shared::fmath;
use crate::shared::rng::SimRng;
use crate::shared::types::{AppConfig, CreatureAgent, FoodSpore, TrainerFitnessConfig};
use rand::Rng;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Training room mode. `Standard` and `Exploration` are the fixed rooms (difficulty levels 0 and 1 of the one level
/// plugin `LevelScenario`); `Curriculum` runs the room of the trainer's current level, moved by the success gate
/// (TCK-176, `trainer_sim`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Scenario {
    #[default]
    Standard,
    Exploration,
    Curriculum,
}

impl Scenario {
    /// UI mode string (`SET_TRAINER_MODE`); anything but "exploration" or "curriculum" is the standard room.
    pub fn from_mode(mode: &str) -> Self {
        match mode {
            "exploration" => Scenario::Exploration,
            "curriculum" => Scenario::Curriculum,
            _ => Scenario::Standard,
        }
    }

    /// Difficulty level of a fixed room (`None` for the curriculum, whose level is trainer state).
    pub fn fixed_level(self) -> Option<f32> {
        match self {
            Scenario::Standard => Some(0.0),
            Scenario::Exploration => Some(1.0),
            Scenario::Curriculum => None,
        }
    }

    /// Room plugin of a fixed room. For `Curriculum` the room at config `trainerCurriculum.startLevel` without
    /// metabolic ramp; the trainer builds the curriculum rooms itself (`TrainerSim::room`).
    pub fn plugin(self) -> LevelScenario {
        LevelScenario::at(self.fixed_level().unwrap_or(AppConfig::global().trainer_curriculum.start_level))
    }
}

impl From<Scenario> for LevelScenario {
    fn from(scenario: Scenario) -> Self {
        scenario.plugin()
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

/// Interface of a training room (scenario plugin). The shared trainer loop (`step_physics`,
/// `step_trainer_sandbox_physics`, `TrainerSim`) only calls these hooks; it has no room-specific branches.
pub trait TrainingScenarioPlugin: Send + Sync {
    /// Edge length of the square chamber in px
    fn chamber_size(&self) -> f32;

    /// Ticks per epoch (one trial)
    fn epoch_ticks(&self) -> u32;

    /// Minimum distance of every food from the creature, at the start of each trial (the first one included) and for
    /// respawns after a catch (config `trainerSpawnMinDistance`, TCK-154 / RV-S08)
    fn spawn_min_distance(&self) -> f32;

    /// Factor on `metabolism.trainerMetabolicScale` in this room (curriculum pressure ramp, TCK-176; 1 otherwise)
    fn metabolic_scale(&self) -> f32;

    /// Places the scenario's food for a new trial (every food at least `spawn_min_distance` from the creature) and
    /// restarts the approach tracking; draws only the food positions from `rng`. Used by the first trial, every
    /// re-trial (`TrainerSandbox::reset_trial`) and room switches.
    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng);

    /// Food the creature's sensors see this tick (hook of the shared `step_physics`);
    /// `scratch` is a reused buffer for foods built per tick
    fn visible_foods<'a>(&self, agent: &CreatureAgent, foods: &'a [FoodSpore], scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore];
}

/// Geometric interpolation `start · (end / start)^d`, exact at both ends (linear if `start` is not positive).
fn geometric(start: f32, end: f32, d: f32) -> f32 {
    if d <= 0.0 {
        start
    } else if d >= 1.0 {
        end
    } else if start > 0.0 {
        start * fmath::powf(end / start, d)
    } else {
        start + (end - start) * d
    }
}

/// Training room at difficulty level `d` in [0, 1] (TCK-176): the one scenario plugin; the fixed rooms are its
/// instances `Standard` = level 0 and `Exploration` = level 1 (config `trainerCurriculum`, `trainerSpawnMinDistance`).
///
/// Interpolation law: chamber size, epoch ticks and spawn distance are geometric in `d` (`start · (end/start)^d`):
/// all three are length/time scales, so every level step makes the room the same relative amount harder (with the
/// default rooms per 0.1: chamber ×1.13, epoch ×1.20, distance ×1.10) instead of front-loading the jump as a linear
/// law would. Food per type is linear in `d`, rounded (1 below 0.25, 2 up to 0.75, 3 above). The sensors see the
/// nearest food of each type, which with one food per type is all food (the small room). Both ends are exact, so
/// levels 0 and 1 reproduce the fixed rooms bit for bit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LevelScenario {
    level: f32,
    chamber_size: f32,
    epoch_ticks: u32,
    spawn_min_distance: f32,
    food_per_type: u32,
    metabolic_scale: f32,
}

impl LevelScenario {
    /// Room at `level` (clamped to [0, 1]; non-finite = 0) from the global config, metabolic factor 1.
    pub fn at(level: f32) -> Self {
        Self::from_config(level, AppConfig::global())
    }

    pub fn from_config(level: f32, config: &AppConfig) -> Self {
        let d = if level.is_finite() { level.clamp(0.0, 1.0) } else { 0.0 };
        let c = &config.trainer_curriculum;
        let spawn = &config.trainer_spawn_min_distance;
        let (food_lo, food_hi) = (c.food_per_type.standard as f32, c.food_per_type.exploration as f32);
        Self {
            level: d,
            chamber_size: geometric(c.chamber_size.standard, c.chamber_size.exploration, d),
            epoch_ticks: (geometric(c.epoch_ticks.standard as f32, c.epoch_ticks.exploration as f32, d).round() as u32).max(1),
            spawn_min_distance: geometric(spawn.standard, spawn.exploration, d),
            food_per_type: ((food_lo + (food_hi - food_lo) * d).round() as u32).max(1),
            metabolic_scale: 1.0,
        }
    }

    /// Same room with the metabolic factor `scale` (curriculum pressure ramp).
    pub fn with_metabolic_scale(self, scale: f32) -> Self {
        Self { metabolic_scale: scale, ..self }
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    /// Plants and meatballs per trial (each)
    pub fn food_per_type(&self) -> u32 {
        self.food_per_type
    }
}

impl TrainingScenarioPlugin for LevelScenario {
    fn chamber_size(&self) -> f32 {
        self.chamber_size
    }

    fn epoch_ticks(&self) -> u32 {
        self.epoch_ticks
    }

    fn spawn_min_distance(&self) -> f32 {
        self.spawn_min_distance
    }

    fn metabolic_scale(&self) -> f32 {
        self.metabolic_scale
    }

    /// `food_per_type` plants (ids 1..=n) followed by as many meatballs (ids n+1..=2n), each placed in this order at
    /// least `spawn_min_distance` from the agent; restarts the approach tracking.
    fn initialize(&self, sb: &mut TrainerSandbox, rng: &mut SimRng) {
        let n = self.food_per_type;
        let food = |id: u32, type_id: u32| FoodSpore { id, type_id, x: 0.0, y: 0.0, amount: 15.0, vx: 0.0, vy: 0.0 };
        sb.foods = (1..=n).map(|id| food(id, 1)).chain((n + 1..=2 * n).map(|id| food(id, 2))).collect();
        let from = (sb.agent.px, sb.agent.py);
        for f in &mut sb.foods {
            let (x, y) = spawn_at_min_distance(rng, self.chamber_size, from, self.spawn_min_distance, SPAWN_MAX_TRIES);
            f.x = x;
            f.y = y;
        }
        sb.reset_approach();
    }

    fn visible_foods<'a>(&self, agent: &CreatureAgent, foods: &'a [FoodSpore], scratch: &'a mut Vec<FoodSpore>) -> &'a [FoodSpore] {
        // One food per type: the nearest of each type is all food (plant, meat), passed through without a copy
        if self.food_per_type <= 1 {
            return foods;
        }
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
/// Metabolism (TCK-178): the shared energy function drains `trainerMetabolicScale` times the room's `metabolic_scale`
/// (curriculum pressure ramp, TCK-176; 1 in the fixed rooms) times the Ocean cost; at 0 energy
/// the creature starves (`sb.starved`): it stops at once and does nothing else for the rest of the trial.
/// Returns whether the creature hit a wall.
pub fn step_physics(sb: &mut TrainerSandbox, app_config: &AppConfig) -> bool {
    if sb.starved {
        return false;
    }
    let plugin = sb.room;
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

    let metabolic_scale = app_config.metabolism.trainer_metabolic_scale * plugin.metabolic_scale();
    crate::biology::metabolism::drain_energy(&mut sb.agent, out_thrust, &app_config.metabolism, metabolic_scale);
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
    fn test_fixed_rooms_are_the_level_ends_with_their_p7_values() {
        let spawn = &AppConfig::global().trainer_spawn_min_distance;
        let small = Scenario::Standard.plugin();
        assert_eq!((small.level(), small.chamber_size(), small.epoch_ticks(), small.food_per_type()), (0.0, 1000.0, 300, 1));
        assert_eq!((small.spawn_min_distance(), small.metabolic_scale()), (spawn.standard, 1.0));
        let large = Scenario::Exploration.plugin();
        assert_eq!((large.level(), large.chamber_size(), large.epoch_ticks(), large.food_per_type()), (1.0, 3500.0, 1800, 3));
        assert_eq!((large.spawn_min_distance(), large.metabolic_scale()), (spawn.exploration, 1.0));
        assert_eq!(LevelScenario::at(-0.5), small);
        assert_eq!(LevelScenario::at(7.0), large);
        assert_eq!(LevelScenario::at(f32::NAN), small);
        // Food layout of the fixed rooms: plants first, ids in order
        let types = |sb: &TrainerSandbox| sb.foods.iter().map(|f| (f.id, f.type_id)).collect::<Vec<_>>();
        assert_eq!(types(&sandbox(Scenario::Standard)), vec![(1, 1), (2, 2)]);
        assert_eq!(types(&sandbox(Scenario::Exploration)), vec![(1, 1), (2, 1), (3, 1), (4, 2), (5, 2), (6, 2)]);
    }

    #[test]
    fn test_level_rooms_interpolate_monotonically() {
        let rooms: Vec<LevelScenario> = (0..=10).map(|i| LevelScenario::at(i as f32 / 10.0)).collect();
        for pair in rooms.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            assert!(b.chamber_size() > a.chamber_size() && b.epoch_ticks() > a.epoch_ticks() && b.spawn_min_distance() > a.spawn_min_distance(), "{:?} -> {:?}", a, b);
            assert!(b.food_per_type() >= a.food_per_type());
        }
        // Geometric: the middle is the geometric mean of the ends
        assert!((rooms[5].chamber_size() - (1000.0f32 * 3500.0).sqrt()).abs() < 0.1, "{}", rooms[5].chamber_size());
        assert_eq!(rooms.iter().map(|r| r.food_per_type()).collect::<Vec<_>>(), vec![1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3]);
    }

    #[test]
    fn test_sensors_see_the_nearest_food_per_type_and_all_food_with_one_per_type() {
        let mut scratch = Vec::new();
        let small = sandbox(Scenario::Standard);
        let visible = Scenario::Standard.plugin().visible_foods(&small.agent, &small.foods, &mut scratch).to_vec();
        assert_eq!(visible.iter().map(|f| (f.id, f.x.to_bits())).collect::<Vec<_>>(), small.foods.iter().map(|f| (f.id, f.x.to_bits())).collect::<Vec<_>>());
        let mid = LevelScenario::at(0.5);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, mid, None, None, &mut seeded(4));
        assert_eq!(sb.foods.len(), 4);
        // Move the second plant and the first meat next to the creature: those are seen
        let (px, py) = (sb.agent.px, sb.agent.py);
        sb.foods[1].x = px + 1.0;
        sb.foods[1].y = py;
        sb.foods[2].x = px;
        sb.foods[2].y = py + 1.0;
        let visible = mid.visible_foods(&sb.agent, &sb.foods, &mut scratch);
        assert_eq!(visible.iter().map(|f| f.id).collect::<Vec<_>>(), vec![2, 3]);
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
