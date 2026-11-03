//! The level room (TCK-176, as a `Room` since TCK-201): a training room at difficulty level `d` in [0, 1]. The
//! registry rooms `standard` and `exploration` are its levels 0 and 1; the curriculum builds the rooms in between.

use std::sync::Arc;

use rand::Rng;

use crate::biology::trainer_config::TrainerFitnessConfig;
use crate::config::AppConfig;
use crate::shared::dto::FoodSpore;
use crate::shared::fmath;
use crate::shared::rng::SimRng;
use crate::sim::systems::death::DeathPolicy;
use crate::sim::systems::eat::{EatRadiusRule, RespawnPolicy, SPAWN_MAX_TRIES, spawn_at_min_distance};
use crate::sim::systems::food_drift::FoodDriftOrder;
use crate::sim::systems::sense::{ClockVariant, TargetSelector};
use crate::sim::{TRAINER_SYSTEMS, WorldSettings};

use super::{EntityId, Room};
use crate::trainer::chamber::{Chamber, EntityTrial};

/// Id of a level room built for a curriculum level (not a registry entry).
pub const LEVEL_ROOM_ID: &str = "level";

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

/// Training room at difficulty level `d` in [0, 1] (config `trainerCurriculum`, `trainerSpawnMinDistance`,
/// `trainerFitness`).
///
/// Interpolation law: chamber size, epoch ticks and spawn distance are geometric in `d` (`start · (end/start)^d`):
/// all three are length/time scales, so every level step makes the room the same relative amount harder (with the
/// default rooms per 0.1: chamber ×1.13, epoch ×1.20, distance ×1.10) instead of front-loading the jump as a linear
/// law would. Food per type is linear in `d`, rounded (1 below 0.25, 2 up to 0.75, 3 above). The sensors see the
/// nearest food of each type, which with one food per type is all food (the small room). Both ends are exact, so
/// levels 0 and 1 are the registry rooms `standard` and `exploration` bit for bit.
#[derive(Debug, Clone, PartialEq)]
pub struct LevelRoom {
    id: &'static str,
    level: f32,
    chamber_size: f32,
    epoch_ticks: u32,
    spawn_min_distance: f32,
    food_per_type: u32,
    metabolic_scale: f32,
    fitness: TrainerFitnessConfig,
}

/// Registry constructor of `standard` (level 0).
pub fn standard(config: &AppConfig) -> Arc<dyn Room> {
    Arc::new(LevelRoom {
        id: "standard",
        ..LevelRoom::at(0.0, config)
    })
}

/// Registry constructor of `exploration` (level 1).
pub fn exploration(config: &AppConfig) -> Arc<dyn Room> {
    Arc::new(LevelRoom {
        id: "exploration",
        ..LevelRoom::at(1.0, config)
    })
}

impl LevelRoom {
    /// Room at `level` (clamped to [0, 1]; non-finite = 0), metabolic factor 1, id `LEVEL_ROOM_ID`.
    pub fn at(level: f32, config: &AppConfig) -> Self {
        let d = if level.is_finite() { level.clamp(0.0, 1.0) } else { 0.0 };
        let c = &config.trainer.trainer_curriculum;
        let spawn = &config.trainer.trainer_spawn_min_distance;
        let (food_lo, food_hi) = (c.food_per_type.standard as f32, c.food_per_type.exploration as f32);
        Self {
            id: LEVEL_ROOM_ID,
            level: d,
            chamber_size: geometric(c.chamber_size.standard, c.chamber_size.exploration, d),
            epoch_ticks: (geometric(c.epoch_ticks.standard as f32, c.epoch_ticks.exploration as f32, d).round() as u32)
                .max(1),
            spawn_min_distance: geometric(spawn.standard, spawn.exploration, d),
            food_per_type: ((food_lo + (food_hi - food_lo) * d).round() as u32).max(1),
            metabolic_scale: 1.0,
            fitness: config.trainer.trainer_fitness.clone(),
        }
    }

    /// Same room with the metabolic factor `scale` (curriculum pressure ramp).
    pub fn with_metabolic_scale(self, scale: f32) -> Self {
        Self {
            metabolic_scale: scale,
            ..self
        }
    }

    /// Plants and meatballs per trial (each)
    pub fn food_per_type(&self) -> u32 {
        self.food_per_type
    }

    /// Minimum distance of every food from the creature at the start of a trial and for respawns after a catch
    /// (config `trainerSpawnMinDistance`, TCK-154 / RV-S08)
    pub fn spawn_min_distance(&self) -> f32 {
        self.spawn_min_distance
    }

    /// Factor on `metabolism.trainerMetabolicScale` (curriculum pressure ramp, TCK-176; 1 otherwise)
    pub fn metabolic_scale(&self) -> f32 {
        self.metabolic_scale
    }
}

impl Room for LevelRoom {
    fn id(&self) -> &str {
        self.id
    }

    fn chamber_size(&self) -> f32 {
        self.chamber_size
    }

    fn episode_ticks(&self) -> u32 {
        self.epoch_ticks
    }

    /// Every creature in the chamber centre with a heading drawn from `rng`, then `food_per_type` plants (ids
    /// 1..=n) followed by as many meatballs (ids n+1..=2n), each placed in this order at least `spawn_min_distance`
    /// from creature 0.
    fn layout(&self, chamber: &mut Chamber, rng: &mut SimRng) {
        let centre = self.chamber_size / 2.0;
        for agent in &mut chamber.creatures {
            agent.px = centre;
            agent.py = centre;
            agent.heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        }
        let n = self.food_per_type;
        let food = |id: u32, type_id: u32| FoodSpore {
            id,
            type_id,
            x: 0.0,
            y: 0.0,
            amount: 15.0,
            vx: 0.0,
            vy: 0.0,
        };
        chamber.foods = (1..=n)
            .map(|id| food(id, 1))
            .chain((n + 1..=2 * n).map(|id| food(id, 2)))
            .collect();
        let from = (chamber.creatures[0].px, chamber.creatures[0].py);
        for f in &mut chamber.foods {
            let (x, y) = spawn_at_min_distance(rng, self.chamber_size, from, self.spawn_min_distance, SPAWN_MAX_TRIES);
            f.x = x;
            f.y = y;
        }
    }

    /// Sensors see the nearest food of each type; with one food per type that is all food, passed through without a
    /// copy. Drain `trainerMetabolicScale · metabolic_scale`; trainer divergences: `fmath::sin` clock, food drift after
    /// the creature moved, diet eat radius, starvation freezes the creature; eaten food respawns at
    /// `spawn_min_distance` from it.
    fn world_settings(&self, config: &AppConfig) -> WorldSettings {
        WorldSettings {
            systems: TRAINER_SYSTEMS,
            clock: ClockVariant::Fmath,
            targets: if self.food_per_type <= 1 {
                TargetSelector::All
            } else {
                TargetSelector::NearestPerType
            },
            metabolic_scale: config.metabolism.trainer_metabolic_scale * self.metabolic_scale,
            energy_modifiers: Vec::new(),
            death: DeathPolicy::Freeze,
            food_drift: FoodDriftOrder::AfterEachCreature,
            eat_radius: EatRadiusRule::TrainerDiet,
            respawn: RespawnPolicy::MinDistance {
                min_distance: self.spawn_min_distance,
            },
        }
    }

    fn score(&self, chamber: &Chamber, entity: EntityId) -> f32 {
        calculate_fitness(&chamber.trials[entity], self.epoch_ticks, &self.fitness)
    }

    fn level(&self) -> f32 {
        self.level
    }
}

/// Fitness of one trial in catch units (TCK-175), the score of the level rooms (UI telemetry per tick, trial end in
/// the trainer, the runner and the probes):
///
/// `fitness = yield + first_catch_bonus · (1 − finish_tick / epoch_ticks) + approach_bonus · approach`
///
/// - `yield` (primary): sum over the trial's catches of the diet fit, meat `carnivory`, plant `1 − carnivory` (at most
///   1.0 per catch, for a pure herbivore or carnivore eating its food), so 1.0 = one catch.
/// - time to first catch (secondary): only with a catch; `finish_tick` in `1..=epoch_ticks`, so the bonus lies in
///   `[0, first_catch_bonus)` and earlier is better.
/// - `approach` (shaping) in `[0, 1]`: relative approach `1 − d_min / d_sight` towards the nearest diet target
///   (`chamber::target_distance_by_diet`), counted from the tick it first came within the creature's food-sensing
///   range (`food_sense_range`); `d_sight` = distance at that tick, `d_min` = smallest distance since. The episode
///   restarts after every catch (it then rewards approach to the next food); 0 if no target was ever in range.
///
/// Both bonus weights are below 1 (config `trainerFitness`), so neither bonus is worth a full catch. No penalties:
/// wall contact, looping, standstill and distance from the centre do not enter; starvation ends the trial's catches
/// (TCK-178) and is no separate term. Non-finite results count as 0.
pub fn calculate_fitness(trial: &EntityTrial, epoch_ticks: u32, weights: &TrainerFitnessConfig) -> f32 {
    let time_bonus = match trial.finish_tick {
        Some(tick) if epoch_ticks > 0 => {
            weights.first_catch_bonus * (1.0 - (tick.min(epoch_ticks) as f32 / epoch_ticks as f32))
        }
        _ => 0.0,
    };
    let fit = trial.accumulated_yield + time_bonus + weights.approach_bonus * trial.approach_fraction();
    if fit.is_finite() { fit } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::genome::WILDTYPE_GENOME;
    use crate::shared::rng::seeded;
    use crate::trainer::room::registry;
    use crate::trainer::sandbox::{SandboxOrigin, TrainerSandbox};

    fn room(id: &str) -> Arc<dyn Room> {
        registry::build(id, crate::config::test_config()).unwrap()
    }

    fn sandbox(id: &str) -> TrainerSandbox {
        TrainerSandbox::spawn(
            1,
            WILDTYPE_GENOME,
            1,
            SandboxOrigin::Elite,
            room(id),
            None,
            None,
            &mut seeded(3),
            crate::config::test_config(),
        )
    }

    fn weights() -> TrainerFitnessConfig {
        TrainerFitnessConfig {
            first_catch_bonus: 0.5,
            approach_bonus: 0.3,
        }
    }

    fn fitness(sb: &TrainerSandbox, epoch: u32) -> f32 {
        calculate_fitness(sb.trial(), epoch, &weights())
    }

    #[test]
    fn test_configured_bonuses_stay_below_one_catch() {
        let w = &crate::config::test_config().trainer.trainer_fitness;
        assert!(
            (0.0..1.0).contains(&w.first_catch_bonus) && (0.0..1.0).contains(&w.approach_bonus),
            "{:?}",
            w
        );
    }

    #[test]
    fn test_one_catch_is_one_and_bonuses_are_bounded() {
        for id in ["standard", "exploration"] {
            let epoch = room(id).episode_ticks();
            let mut sb = sandbox(id);
            assert_eq!(fitness(&sb, epoch), 0.0, "nothing done, nothing earned");

            // One catch with full diet fit at the last tick, no approach episode open: exactly 1.0
            sb.trial_mut().accumulated_yield = 1.0;
            sb.trial_mut().finish_tick = Some(epoch);
            assert_eq!(fitness(&sb, epoch), 1.0);
            // The earliest possible first catch stays below one more catch
            sb.trial_mut().finish_tick = Some(1);
            let early = fitness(&sb, epoch);
            assert!(early > 1.0 && early < 1.5, "{}", early);
            // Earlier is better
            sb.trial_mut().finish_tick = Some(epoch / 2);
            assert!(fitness(&sb, epoch) < early);

            // No catch, full approach: below one catch
            let mut miss = sandbox(id);
            miss.trial_mut().sight_distance = Some(400.0);
            miss.trial_mut().approach_min_distance = 0.0;
            let shaped = fitness(&miss, epoch);
            assert!((shaped - 0.3).abs() < 1e-6 && shaped < 1.0, "{}", shaped);
            // Half the distance: half the shaping
            miss.trial_mut().approach_min_distance = 200.0;
            assert!((fitness(&miss, epoch) - 0.15).abs() < 1e-6);
            // Two catches beat one catch with every bonus
            let mut two = sandbox(id);
            two.trial_mut().accumulated_yield = 2.0;
            two.trial_mut().finish_tick = Some(epoch);
            let mut one = sandbox(id);
            one.trial_mut().accumulated_yield = 1.0;
            one.trial_mut().finish_tick = Some(1);
            one.trial_mut().sight_distance = Some(400.0);
            one.trial_mut().approach_min_distance = 1.0;
            assert!(fitness(&two, epoch) > fitness(&one, epoch));
        }
    }

    #[test]
    fn test_no_cliffs_for_walls_looping_standstill_or_centre_distance() {
        for id in ["standard", "exploration"] {
            let epoch = room(id).episode_ticks();
            let mut sb = sandbox(id);
            sb.trial_mut().accumulated_yield = 1.0;
            sb.trial_mut().finish_tick = Some(100);
            sb.trial_mut().sight_distance = Some(300.0);
            sb.trial_mut().approach_min_distance = 150.0;
            let reference = fitness(&sb, epoch);
            let mut variant = sb.clone();
            variant.trial_mut().wall_collisions = 50;
            variant.trial_mut().cumulative_rotation = 500.0;
            variant.trial_mut().distance_traveled = 0.0;
            variant.trial_mut().coverage_mask = u64::MAX;
            variant.agent_mut().px = 30.0;
            variant.agent_mut().py = 30.0;
            assert_eq!(fitness(&variant, epoch), reference, "{}", id);
        }
    }

    #[test]
    fn test_room_score_is_the_configured_fitness_of_the_entity() {
        let mut sb = sandbox("standard");
        sb.trial_mut().accumulated_yield = 1.0;
        sb.trial_mut().finish_tick = Some(30);
        let r = room("standard");
        let expected = calculate_fitness(
            sb.trial(),
            r.episode_ticks(),
            &crate::config::test_config().trainer.trainer_fitness,
        );
        assert_eq!(r.score(&sb.chamber, 0), expected);
        assert!(expected > 1.0);
    }

    #[test]
    fn test_fixed_rooms_are_the_level_ends_with_their_p7_values() {
        let config = crate::config::test_config();
        let spawn = &config.trainer.trainer_spawn_min_distance;
        let small = LevelRoom::at(0.0, config);
        assert_eq!(
            (
                small.level(),
                small.chamber_size(),
                small.episode_ticks(),
                small.food_per_type()
            ),
            (0.0, 1000.0, 300, 1)
        );
        assert_eq!(
            (small.spawn_min_distance(), small.metabolic_scale()),
            (spawn.standard, 1.0)
        );
        let large = LevelRoom::at(1.0, config);
        assert_eq!(
            (
                large.level(),
                large.chamber_size(),
                large.episode_ticks(),
                large.food_per_type()
            ),
            (1.0, 3500.0, 1800, 3)
        );
        assert_eq!(
            (large.spawn_min_distance(), large.metabolic_scale()),
            (spawn.exploration, 1.0)
        );
        assert_eq!(LevelRoom::at(-0.5, config), small);
        assert_eq!(LevelRoom::at(7.0, config), large);
        assert_eq!(LevelRoom::at(f32::NAN, config), small);
        // The registry rooms are the level ends (only the id differs)
        for (id, level) in [("standard", &small), ("exploration", &large)] {
            let r = room(id);
            assert_eq!(
                (r.id(), r.chamber_size(), r.episode_ticks()),
                (id, level.chamber_size(), level.episode_ticks())
            );
            assert_eq!(r.world_settings(config), level.world_settings(config));
        }
        // Food layout of the fixed rooms: plants first, ids in order
        let types = |sb: &TrainerSandbox| sb.chamber.foods.iter().map(|f| (f.id, f.type_id)).collect::<Vec<_>>();
        assert_eq!(types(&sandbox("standard")), vec![(1, 1), (2, 2)]);
        assert_eq!(
            types(&sandbox("exploration")),
            vec![(1, 1), (2, 1), (3, 1), (4, 2), (5, 2), (6, 2)]
        );
    }

    #[test]
    fn test_level_rooms_interpolate_monotonically() {
        let rooms: Vec<LevelRoom> = (0..=10)
            .map(|i| LevelRoom::at(i as f32 / 10.0, crate::config::test_config()))
            .collect();
        for pair in rooms.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            assert!(
                b.chamber_size() > a.chamber_size()
                    && b.episode_ticks() > a.episode_ticks()
                    && b.spawn_min_distance() > a.spawn_min_distance(),
                "{:?} -> {:?}",
                a,
                b
            );
            assert!(b.food_per_type() >= a.food_per_type());
        }
        // Geometric: the middle is the geometric mean of the ends
        assert!(
            (rooms[5].chamber_size() - (1000.0f32 * 3500.0).sqrt()).abs() < 0.1,
            "{}",
            rooms[5].chamber_size()
        );
        assert_eq!(
            rooms.iter().map(|r| r.food_per_type()).collect::<Vec<_>>(),
            vec![1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3]
        );
    }

    #[test]
    fn test_sensors_see_the_nearest_food_per_type_and_all_food_with_one_per_type() {
        let mut scratch = Vec::new();
        let config = crate::config::test_config();
        let small = sandbox("standard");
        let targets = room("standard").world_settings(config).targets;
        assert_eq!(targets, TargetSelector::All);
        let visible = targets
            .select(small.agent(), &small.chamber.foods, &[], &mut scratch)
            .to_vec();
        assert_eq!(
            visible.iter().map(|f| (f.id, f.x.to_bits())).collect::<Vec<_>>(),
            small
                .chamber
                .foods
                .iter()
                .map(|f| (f.id, f.x.to_bits()))
                .collect::<Vec<_>>()
        );
        let mid = LevelRoom::at(0.5, config);
        let mut sb = TrainerSandbox::spawn(
            1,
            WILDTYPE_GENOME,
            1,
            SandboxOrigin::Elite,
            Arc::new(mid.clone()),
            None,
            None,
            &mut seeded(4),
            config,
        );
        assert_eq!(sb.chamber.foods.len(), 4);
        // Move the second plant and the first meat next to the creature: those are seen
        let (px, py) = (sb.agent().px, sb.agent().py);
        sb.chamber.foods[1].x = px + 1.0;
        sb.chamber.foods[1].y = py;
        sb.chamber.foods[2].x = px;
        sb.chamber.foods[2].y = py + 1.0;
        let visible = mid
            .world_settings(config)
            .targets
            .select(sb.agent(), &sb.chamber.foods, &[], &mut scratch);
        assert_eq!(visible.iter().map(|f| f.id).collect::<Vec<_>>(), vec![2, 3]);
    }

    #[test]
    fn test_same_fitness_scale_in_both_rooms() {
        // Same trial outcome, same fitness, although chamber and epoch differ
        let mut small = sandbox("standard");
        let mut large = sandbox("exploration");
        for (sb, epoch) in [(&mut small, 300u32), (&mut large, 1800u32)] {
            sb.trial_mut().accumulated_yield = 2.0;
            sb.trial_mut().finish_tick = Some(epoch / 4);
        }
        assert_eq!(fitness(&small, 300), fitness(&large, 1800));
    }
}
