//! A trainer sandbox: one breeding slot (id, origin, fitness) with its chamber and the room of its current trial.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::biology::creature_factory::{spawn_creature, SpawnOptions};
use crate::config::AppConfig;
use crate::shared::dto::CreatureAgent;
use crate::shared::rng::SimRng;

use super::chamber::{Chamber, EntityTrial};
use super::room::Room;

/// How a sandbox's genome was produced by the breeding slot plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SandboxOrigin {
    Elite,
    Hof,
    Mutant,
    Random,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrainerSandbox {
    /// Slot id (1-based breeding slot of the generation)
    pub id: u32,
    pub origin_type: SandboxOrigin,
    /// The slot's fitness: live score of the current trial (UI telemetry), the trial score at its end, the generation
    /// fitness after the last trial (`Evaluator::aggregate`)
    pub current_fitness: f32,
    /// Room of the current trial (shared by all sandboxes of the trial)
    #[serde(skip)]
    pub room: Arc<dyn Room>,
    pub chamber: Chamber,
}

impl TrainerSandbox {
    /// Fresh sandbox in `room`: the slot's creature from the factory (`methylations` / `synapse_weights` carry
    /// Lamarckian inheritance), laid out by the room and given the respawn seed, both from `rng` (the trainer passes the
    /// trial's shared layout stream, TCK-174).
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        id: u32,
        genome: &str,
        generation: u32,
        origin_type: SandboxOrigin,
        room: Arc<dyn Room>,
        methylations: Option<&[f32]>,
        synapse_weights: Option<Vec<f32>>,
        rng: &mut SimRng,
        config: &AppConfig,
    ) -> Self {
        let agent = spawn_creature(genome, SpawnOptions { id, px: 0.0, py: 0.0, heading_angle: 0.0, generation, methylations, synapse_weights }, config);
        let mut chamber = Chamber::new(room.as_ref(), vec![agent]);
        chamber.first_layout(room.as_ref(), rng);
        Self { id, origin_type, current_fitness: 0.0, room, chamber }
    }

    /// The slot's creature (entity 0).
    pub fn agent(&self) -> &CreatureAgent {
        &self.chamber.creatures[0]
    }

    pub fn agent_mut(&mut self) -> &mut CreatureAgent {
        &mut self.chamber.creatures[0]
    }

    /// Trial record of the slot's creature.
    pub fn trial(&self) -> &EntityTrial {
        &self.chamber.trials[0]
    }

    pub fn trial_mut(&mut self) -> &mut EntityTrial {
        &mut self.chamber.trials[0]
    }

    /// Starts the next trial in the current room (`Chamber::start_trial`; learned synapse weights are kept) and
    /// clears the slot's fitness.
    pub fn reset_trial(&mut self, rng: &mut SimRng, config: &AppConfig) {
        self.chamber.start_trial(self.room.as_ref(), rng, config);
        self.current_fitness = 0.0;
    }

    /// `reset_trial` in another room.
    pub fn reset_trial_in(&mut self, room: Arc<dyn Room>, rng: &mut SimRng, config: &AppConfig) {
        self.room = room;
        self.reset_trial(rng, config);
    }

    /// One tick of the chamber in the room's world. Probes pass a modified config (e.g. Hebbian learning off).
    pub fn step(&mut self, config: &AppConfig) {
        let settings = self.room.world_settings(config);
        self.chamber.step(&settings, self.room.obstacles(), config);
    }

    /// The room's score of the slot's creature in the current trial.
    pub fn score(&self) -> f32 {
        self.room.score(&self.chamber, 0)
    }

    /// The room's episode is over.
    pub fn is_done(&self) -> bool {
        self.room.is_done(&self.chamber)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::genome::WILDTYPE_GENOME;
    use crate::shared::brain::food_sense_range;
    use crate::shared::dto::FoodSpore;
    use crate::shared::rng::seeded;
    use crate::trainer::room::{registry, LevelRoom};

    fn room(id: &str) -> Arc<dyn Room> {
        registry::build(id, crate::config::test_config()).unwrap()
    }

    fn spawn_in(id: &str, origin: SandboxOrigin, rng: &mut SimRng) -> TrainerSandbox {
        TrainerSandbox::spawn(1, WILDTYPE_GENOME, 1, origin, room(id), None, None, rng, crate::config::test_config())
    }

    fn park_food(sb: &mut TrainerSandbox) {
        let size = sb.chamber.size;
        for f in &mut sb.chamber.foods {
            f.x = size - 10.0;
            f.y = size - 10.0;
        }
    }

    fn drop_food_on_creature(sb: &mut TrainerSandbox, food: usize) {
        let (px, py) = (sb.agent().px, sb.agent().py);
        sb.chamber.foods[food].x = px;
        sb.chamber.foods[food].y = py;
    }

    #[test]
    fn test_finish_tick_records_the_tick_of_first_consumption() {
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(1));
        // Park all food in the far corner so nothing can be eaten during the first ticks
        park_food(&mut sb);
        for _ in 0..5 {
            sb.step(crate::config::test_config());
        }
        assert!(!sb.trial().finished);

        // Drop the plant onto the creature: it is consumed on the 6th tick
        drop_food_on_creature(&mut sb, 0);
        sb.step(crate::config::test_config());

        assert!(sb.trial().finished);
        assert_eq!(sb.trial().finish_tick, Some(6));
    }

    #[test]
    fn test_trainer_drain_equals_the_ocean_energy_function() {
        // Trainer path: the shared creature tick. Ocean path: the same sensing and brain step followed by the Ocean's
        // `drain_energy(.., 1.0)` call. Same agent, tick and outputs → identical drain (trainer scale 1).
        let mut config = crate::config::test_config().clone();
        config.metabolism.trainer_metabolic_scale = 1.0;
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(4));
        for _ in 0..20 {
            let mut ocean = sb.agent().clone();
            let foods = sb.chamber.foods.clone();
            let start = sb.agent().energy;
            sb.step(&config);
            assert_eq!(sb.trial().consumed_count, 0, "no catch disturbs the energy comparison");

            let size = sb.chamber.size;
            let clock_val = 0.5 + 0.5 * crate::shared::fmath::sin((ocean.age as f32) * 0.1);
            let inputs = crate::shared::brain::compute_sensory_inputs(&ocean, clock_val, &foods, &[], size, size);
            let out = crate::shared::brain::execute_brain_with_learning(
                &ocean.phenotype.brain,
                &inputs,
                &mut ocean.neuron_states,
                &mut ocean.neuron_activations,
                &mut ocean.synapse_weights,
                &mut ocean.brain_runtime,
                config.brain.hebbian_learning_rate_base,
            );
            let drain = crate::biology::metabolism::drain_energy(&mut ocean, out.thrust, &config.metabolism, 1.0);
            assert!(drain > 0.0);
            assert_eq!(sb.agent().energy.to_bits(), ocean.energy.to_bits(), "trainer drain {} vs ocean drain {}", start - sb.agent().energy, drain);
        }
    }

    #[test]
    fn test_room_metabolic_scale_multiplies_the_trainer_drain() {
        // Curriculum ramp (TCK-176): the room factor scales the drain; 0 = no drain, fixed rooms 1
        let mut config = crate::config::test_config().clone();
        config.metabolism.trainer_metabolic_scale = 1.0;
        let drain = |factor: f32| {
            let room = LevelRoom::at(0.0, crate::config::test_config()).with_metabolic_scale(factor);
            let mut sb = TrainerSandbox::spawn(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Arc::new(room), None, None, &mut seeded(13), crate::config::test_config());
            let start = sb.agent().energy;
            sb.step(&config);
            start - sb.agent().energy
        };
        let (full, half, none) = (drain(1.0), drain(0.5), drain(0.0));
        assert!(full > 0.0);
        assert_eq!(none, 0.0);
        assert!((half - 0.5 * full).abs() <= 1e-5 * full.max(1.0), "half {} vs full {}", half, full);
        assert_eq!(room("standard").world_settings(&config).metabolic_scale, config.metabolism.trainer_metabolic_scale);
    }

    #[test]
    fn test_starved_creature_stops_and_cannot_eat() {
        let mut rng = seeded(5);
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut rng);
        park_food(&mut sb);
        sb.agent_mut().energy = 1e-6;
        sb.step(crate::config::test_config());
        assert!(sb.trial().starved);
        assert_eq!((sb.agent().energy, sb.agent().vx, sb.agent().vy), (0.0, 0.0, 0.0));

        // Frozen for the rest of the trial: no movement, food on top of it is not eaten
        let (px, py) = (sb.agent().px, sb.agent().py);
        drop_food_on_creature(&mut sb, 0);
        for _ in 0..10 {
            sb.step(crate::config::test_config());
        }
        assert_eq!((sb.agent().px, sb.agent().py), (px, py));
        assert!(!sb.trial().finished && sb.trial().consumed_count == 0);
        assert_eq!(sb.chamber.tick, 11);

        sb.reset_trial(&mut rng, crate::config::test_config());
        assert!(!sb.trial().starved);
        assert_eq!(sb.agent().energy, crate::biology::creature_factory::start_energy(&sb.agent().phenotype, &crate::config::test_config().metabolism));
    }

    #[test]
    fn test_eating_refills_energy_with_the_shared_food_rule() {
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(6));
        park_food(&mut sb);
        sb.step(crate::config::test_config());
        let before = sb.agent().energy;
        assert!(before < crate::biology::creature_factory::start_energy(&sb.agent().phenotype, &crate::config::test_config().metabolism), "energy drains every tick");

        sb.agent_mut().energy = 10.0;
        drop_food_on_creature(&mut sb, 0);
        sb.step(crate::config::test_config());
        assert_eq!(sb.trial().consumed_count, 1);
        let gain = crate::biology::metabolism::food_energy(sb.agent().phenotype.carnivory, false, &crate::config::test_config().metabolism);
        assert!(sb.agent().energy > 10.0 + gain - 1.0 && sb.agent().energy < 10.0 + gain, "energy {} after eating (gain {})", sb.agent().energy, gain);
    }

    #[test]
    fn test_reset_trial_restores_trial_start_state() {
        let mut rng = seeded(2);
        let mut sb = spawn_in("exploration", SandboxOrigin::Mutant, &mut rng);
        for _ in 0..30 {
            sb.step(crate::config::test_config());
        }
        let learned = sb.agent().synapse_weights.clone();
        sb.trial_mut().finished = true;
        sb.reset_trial(&mut rng, crate::config::test_config());

        let size = room("exploration").chamber_size();
        assert_eq!((sb.agent().px, sb.agent().py), (size / 2.0, size / 2.0));
        assert_eq!((sb.chamber.tick, sb.trial().distance_traveled, sb.trial().finished), (0, 0.0, false));
        assert!(sb.agent().neuron_states.is_empty());
        assert_eq!(sb.agent().synapse_weights, learned);
        let min_dist = LevelRoom::at(1.0, crate::config::test_config()).spawn_min_distance();
        for f in &sb.chamber.foods {
            assert!(food_distance(&sb, f) >= min_dist);
        }
        assert_eq!((sb.trial().sight_distance, sb.trial().approach_min_distance), (None, f32::MAX));
    }

    #[test]
    fn test_age_increments_every_tick_and_resets_per_trial() {
        let mut rng = seeded(8);
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut rng);
        assert_eq!(sb.agent().age, 0);
        for tick in 1..=7 {
            sb.step(crate::config::test_config());
            assert_eq!(sb.agent().age, tick);
        }
        sb.reset_trial(&mut rng, crate::config::test_config());
        assert_eq!(sb.agent().age, 0);
    }

    #[test]
    fn test_respawn_depends_only_on_respawn_seed_catch_index_and_position() {
        let mut a = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(9));
        let mut b = TrainerSandbox::spawn(2, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, room("standard"), None, None, &mut seeded(9), crate::config::test_config());
        assert_eq!(a.chamber.respawn_seed, b.chamber.respawn_seed);
        let eat_plant = |sb: &mut TrainerSandbox| {
            drop_food_on_creature(sb, 0);
            sb.step(crate::config::test_config());
            (sb.chamber.foods[0].x, sb.chamber.foods[0].y)
        };
        // Same seed, same catch index, same position: same respawn point; the next catch draws a new point
        let first = eat_plant(&mut a);
        assert_eq!(first, eat_plant(&mut b));
        assert_eq!((a.trial().consumed_count, a.chamber.catches), (1, 1));
        let second = eat_plant(&mut a);
        assert_ne!(first, second);
        // Another respawn seed draws other candidates
        let mut c = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(10));
        assert_ne!(first, eat_plant(&mut c));
    }

    fn food_distance(sb: &TrainerSandbox, f: &FoodSpore) -> f32 {
        ((f.x - sb.agent().px).powi(2) + (f.y - sb.agent().py).powi(2)).sqrt()
    }

    #[test]
    fn test_spawn_distance_identical_for_first_trial_later_trials_and_respawns() {
        // RV-S08: one room distance for trial 0, re-trials and respawns after eating
        let spawn = &crate::config::test_config().trainer.trainer_spawn_min_distance;
        for (id, level, expected) in [("standard", 0.0, spawn.standard), ("exploration", 1.0, spawn.exploration)] {
            let min_dist = LevelRoom::at(level, crate::config::test_config()).spawn_min_distance();
            assert_eq!(min_dist, expected);
            for seed in 0..20 {
                let mut rng = seeded(100 + seed);
                let mut sb = spawn_in(id, SandboxOrigin::Elite, &mut rng);
                assert!(sb.chamber.foods.iter().all(|f| food_distance(&sb, f) >= min_dist), "{} trial 0", id);
                sb.reset_trial(&mut rng, crate::config::test_config());
                assert!(sb.chamber.foods.iter().all(|f| food_distance(&sb, f) >= min_dist), "{} trial 1", id);
                // Respawn after a catch at the creature's position
                drop_food_on_creature(&mut sb, 0);
                sb.step(crate::config::test_config());
                assert_eq!(sb.trial().consumed_count, 1);
                assert!(food_distance(&sb, &sb.chamber.foods[0]) >= min_dist, "{} respawn", id);
            }
        }
    }

    #[test]
    fn test_approach_tracking_opens_in_sensor_range_and_restarts_after_a_catch() {
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(12));
        let range = food_sense_range(&sb.agent().phenotype);
        assert!(range > 0.0, "wildtype has a food-sensing organ");
        let (cx, cy) = (sb.agent().px, sb.agent().py);
        let track = |sb: &mut TrainerSandbox| {
            let TrainerSandbox { chamber, .. } = sb;
            chamber.trials[0].track_approach(&chamber.creatures[0], &chamber.foods);
        };
        // All food out of range: no episode, no shaping
        for f in &mut sb.chamber.foods {
            f.x = cx + range + 50.0;
            f.y = cy;
        }
        track(&mut sb);
        assert_eq!((sb.trial().sight_distance, sb.trial().approach_fraction()), (None, 0.0));
        // Food at 0.8 range: the episode opens at that distance; halving the distance gives approach 0.5
        for f in &mut sb.chamber.foods {
            f.x = cx + 0.8 * range;
        }
        track(&mut sb);
        assert!((sb.trial().sight_distance.unwrap() - 0.8 * range).abs() < 1e-3, "{:?}", sb.trial().sight_distance);
        for f in &mut sb.chamber.foods {
            f.x = cx + 0.4 * range;
        }
        track(&mut sb);
        assert!((sb.trial().approach_fraction() - 0.5).abs() < 1e-5, "{}", sb.trial().approach_fraction());
        // Moving away keeps the best approach
        for f in &mut sb.chamber.foods {
            f.x = cx + 0.7 * range;
        }
        track(&mut sb);
        assert!((sb.trial().approach_fraction() - 0.5).abs() < 1e-5);
        // A catch restarts the episode
        drop_food_on_creature(&mut sb, 0);
        drop_food_on_creature(&mut sb, 1);
        sb.step(crate::config::test_config());
        assert!(sb.trial().consumed_count >= 1);
        assert!(sb.trial().approach_fraction() < 0.5);
    }

    #[test]
    fn test_score_and_is_done_follow_the_room() {
        let mut sb = spawn_in("standard", SandboxOrigin::Elite, &mut seeded(14));
        park_food(&mut sb);
        let epoch = sb.room.episode_ticks();
        for _ in 0..epoch - 1 {
            sb.step(crate::config::test_config());
        }
        assert!(!sb.is_done());
        sb.step(crate::config::test_config());
        assert!(sb.is_done());
        assert_eq!(sb.score(), sb.room.score(&sb.chamber, 0));
    }
}
