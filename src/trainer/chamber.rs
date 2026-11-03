//! The chamber: the owned world of one trainer sandbox (TCK-201). It holds N creatures (entity 0 = the creature of
//! the sandbox's breeding slot; today's rooms use N = 1), their food, one trial record per entity, the tick of the
//! trial and its respawn seed. Each tick it lends a `sim::World` over its storage to the shared creature systems
//! (`sim::step_creature`), then applies the trainer's bookkeeping and the events (catches, respawns).
//! Contract: `wiki/contracts/room.md`.

use rand::RngCore;
use serde::Serialize;

use crate::biology::creature_factory::start_energy;
use crate::config::AppConfig;
use crate::shared::brain::food_sense_range;
use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::shared::geom;
use crate::shared::map_generator::ProceduralObstacle;
use crate::shared::rng::{derive, SimRng};
use crate::sim::systems::eat::respawn;
use crate::sim::{step_creature, CreatureEvent, CreatureStep, TickCtx, World, WorldSettings};

use super::room::Room;

/// Carnivory at or above this value: the creature hunts meat only.
pub const CARNIVORE_MIN: f32 = 0.6;
/// Carnivory below this value: the creature grazes plants only (in between: omnivore).
pub const HERBIVORE_MAX: f32 = 0.4;

/// Wall contacts within this many ticks after a counted one are not counted again (rubbing).
const WALL_COLLISION_COOLDOWN: u32 = 20;

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

/// Distance from the agent to the nearest food its diet targets (plants = type 1, meat = type 2;
/// omnivores take the nearer of both). `f32::MAX` if no such food exists.
pub fn target_distance_by_diet(agent: &CreatureAgent, foods: &[FoodSpore]) -> f32 {
    let nearest = |type_id: u32| {
        foods
            .iter()
            .filter(|f| f.type_id == type_id)
            .map(|f| geom::dist(agent.px, agent.py, f.x, f.y))
            .fold(f32::MAX, f32::min)
    };
    let (dist_plant, dist_meat) = (nearest(1), nearest(2));
    match DietClass::from_carnivory(agent.phenotype.carnivory) {
        DietClass::Carnivore => dist_meat,
        DietClass::Omnivore => dist_meat.min(dist_plant),
        DietClass::Herbivore => dist_plant,
    }
}

/// What one entity did in the current trial (reset at every trial start). Read by `Room::score` and the trainer's
/// generation stats.
#[derive(Debug, Clone, Serialize)]
pub struct EntityTrial {
    /// Caught food at least once in this trial
    pub finished: bool,
    /// Trial tick of the first catch
    pub finish_tick: Option<u32>,
    pub distance_traveled: f32,
    pub wall_collisions: u32,
    pub wall_collision_cooldown: u32,
    /// Type of the first catch ("plant" / "meat"), shown by the UI
    pub consumed_spore_type: Option<String>,
    /// Approach episode of the fitness shaping (`room::calculate_fitness`): distance to the nearest diet target at the
    /// tick it first came within food-sensing range; `None` until then. Restarts with every trial and catch.
    pub sight_distance: Option<f32>,
    /// Smallest distance to the nearest diet target since `sight_distance` was set
    pub approach_min_distance: f32,
    /// Sum of the diet fit of the catches (meat `carnivory`, plant `1 − carnivory`)
    pub accumulated_yield: f32,
    pub consumed_count: u32,
    pub cumulative_rotation: f32,
    /// Visited sectors of an 8×8 grid over the chamber
    pub coverage_mask: u64,
    /// Energy ran out during the trial: the creature stops and cannot eat until the trial ends (TCK-178)
    pub starved: bool,
    /// Scratch and result of the shared creature tick (`sim::step_creature`; reused buffers, not state)
    #[serde(skip)]
    pub step: CreatureStep,
}

impl Default for EntityTrial {
    fn default() -> Self {
        Self {
            finished: false,
            finish_tick: None,
            distance_traveled: 0.0,
            wall_collisions: 0,
            wall_collision_cooldown: 0,
            consumed_spore_type: None,
            sight_distance: None,
            approach_min_distance: f32::MAX,
            accumulated_yield: 0.0,
            consumed_count: 0,
            cumulative_rotation: 0.0,
            coverage_mask: 0,
            starved: false,
            step: CreatureStep::default(),
        }
    }
}

impl EntityTrial {
    /// Clears the record for a new trial; keeps the step buffers.
    pub fn reset(&mut self) {
        let step = std::mem::take(&mut self.step);
        *self = Self { step, ..Self::default() };
    }

    /// Restarts the approach episode of the fitness shaping (trial start and after every catch).
    pub fn reset_approach(&mut self) {
        self.sight_distance = None;
        self.approach_min_distance = f32::MAX;
    }

    /// Relative approach `1 - d_min / d_sight` of the current episode in `[0, 1]`; 0 before a diet target was in
    /// food-sensing range (`room::calculate_fitness`).
    pub fn approach_fraction(&self) -> f32 {
        match self.sight_distance {
            Some(sight) if sight > 0.0 => (1.0 - self.approach_min_distance / sight).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    /// Per-tick approach tracking: opens the episode when the nearest diet target is within the creature's
    /// food-sensing range (`food_sense_range`), then keeps the smallest distance to the nearest diet target.
    pub fn track_approach(&mut self, agent: &CreatureAgent, foods: &[FoodSpore]) {
        let dist = target_distance_by_diet(agent, foods);
        if self.sight_distance.is_none() && dist <= food_sense_range(&agent.phenotype) {
            self.sight_distance = Some(dist);
        }
        if self.sight_distance.is_some() {
            self.approach_min_distance = self.approach_min_distance.min(dist);
        }
    }
}

/// The owned world of one sandbox (module doc).
#[derive(Debug, Clone, Serialize)]
pub struct Chamber {
    /// Edge of the square chamber (the room's `chamber_size` of the current trial)
    pub size: f32,
    pub creatures: Vec<CreatureAgent>,
    /// Trial record per entity, aligned with `creatures`
    pub trials: Vec<EntityTrial>,
    pub foods: Vec<FoodSpore>,
    /// Tick of the current trial (1 after the first step)
    pub tick: u32,
    /// Catches of all entities in the current trial
    pub catches: u32,
    /// Seed of the respawns after eating in the current trial: catch `k` (1-based, `catches`) respawns its food from
    /// `derive(respawn_seed, [k])`. Drawn from the trial's layout stream after the layout, so in the trainer every
    /// sandbox of a generation has the same seed (TCK-174).
    pub respawn_seed: u64,
}

impl Chamber {
    /// Chamber of `creatures` for `room` before its first layout (no food, tick 0).
    pub fn new(room: &dyn Room, creatures: Vec<CreatureAgent>) -> Self {
        let trials = creatures.iter().map(|_| EntityTrial::default()).collect();
        Self { size: room.chamber_size(), creatures, trials, foods: Vec::new(), tick: 0, catches: 0, respawn_seed: 0 }
    }

    /// Lays out trial 0 of a fresh chamber: the room's layout, then the respawn seed, both from `rng`.
    pub fn first_layout(&mut self, room: &dyn Room, rng: &mut SimRng) {
        room.layout(self, rng);
        self.respawn_seed = rng.next_u64();
    }

    /// Starts the next trial in `room`: every creature at rest with start energy, age 0 and cleared neuron state
    /// (learned synapse weights are kept), every trial record cleared, tick and catches 0; then the room's layout
    /// (positions, headings, food) and the respawn seed, drawn from `rng` in this order (the trainer passes the trial's
    /// shared layout stream, TCK-174).
    pub fn start_trial(&mut self, room: &dyn Room, rng: &mut SimRng, config: &AppConfig) {
        self.size = room.chamber_size();
        for (agent, trial) in self.creatures.iter_mut().zip(&mut self.trials) {
            agent.vx = 0.0;
            agent.vy = 0.0;
            agent.bend_angle = 0.0;
            agent.omega_rot = 0.0;
            agent.energy = start_energy(&agent.phenotype, &config.metabolism);
            agent.age = 0;
            agent.has_eaten = false;
            agent.neuron_states.clear();
            agent.neuron_activations.clear();
            trial.reset();
        }
        self.tick = 0;
        self.catches = 0;
        room.layout(self, rng);
        self.respawn_seed = rng.next_u64();
    }

    /// One tick: per entity in order, the shared creature systems of the room's world (`settings`, `obstacles`), then
    /// the trainer's bookkeeping (age, coverage, wall counter, approach tracking) and the catches (yield, first-catch
    /// tick, respawn). Deterministic without an RNG argument: respawns use `derive(respawn_seed, [catches])`.
    /// A starved entity (`EntityTrial::starved`, TCK-178) does nothing but age for the rest of the trial.
    pub fn step(&mut self, settings: &WorldSettings, obstacles: &[ProceduralObstacle], config: &AppConfig) {
        self.tick += 1;
        let size = self.size;
        let ctx = TickCtx::new(config, self.tick as u64);
        for idx in 0..self.creatures.len() {
            let trial = &mut self.trials[idx];
            if !trial.starved {
                let mut world = World {
                    creatures: &mut self.creatures,
                    foods: &mut self.foods,
                    obstacles,
                    peers: &[],
                    grid: None,
                    width: size,
                    height: size,
                    temperature: None,
                    settings,
                };
                step_creature(&mut world, idx, &ctx, &mut trial.step);
                let agent = &self.creatures[idx];
                if trial.step.died {
                    trial.starved = true;
                } else {
                    // Absolute heading change (cumulative rotation in radians) and path length
                    trial.cumulative_rotation += geom::wrap_angle(agent.heading_angle - trial.step.old_heading).abs();
                    trial.distance_traveled += (agent.vx.powi(2) + agent.vy.powi(2)).sqrt();
                }
            }
            let agent = &mut self.creatures[idx];
            // Age per tick after the brain step, as in the Ocean (drives the clock input; reset per trial, TCK-174)
            agent.age += 1;
            if trial.starved {
                // Out of energy: no movement, no catches for the rest of the trial (TCK-178)
                continue;
            }

            // Visited 8x8 grid sectors
            let sec_x = ((agent.px / size) * 8.0).floor().clamp(0.0, 7.0) as u32;
            let sec_y = ((agent.py / size) * 8.0).floor().clamp(0.0, 7.0) as u32;
            trial.coverage_mask |= 1 << (sec_y * 8 + sec_x);

            if trial.step.hit_wall && trial.wall_collision_cooldown == 0 {
                trial.wall_collisions += 1;
                trial.wall_collision_cooldown = WALL_COLLISION_COOLDOWN;
            }
            if trial.wall_collision_cooldown > 0 {
                trial.wall_collision_cooldown -= 1;
            }

            // Approach towards the nearest diet target in sensor range (fitness shaping, TCK-175); the eaten food has not
            // respawned yet, so this sees the food as it was when the creature reached it
            trial.track_approach(agent, &self.foods);

            // Catches of this tick (the eat system already added the energy, shared food rule of the Ocean, TCK-178)
            let events = std::mem::take(&mut trial.step.events);
            for event in &events {
                let CreatureEvent::Ate { food, is_meat, .. } = *event else { continue };
                let carnivory = agent.phenotype.carnivory;
                trial.accumulated_yield += if is_meat { carnivory } else { 1.0 - carnivory };
                trial.consumed_count += 1;
                self.catches += 1;

                if !trial.finished {
                    trial.finished = true;
                    trial.finish_tick = Some(self.tick);
                    trial.consumed_spore_type = Some(if is_meat { "meat" } else { "plant" }.to_string());
                    agent.has_eaten = true;
                }

                // Respawn: candidate points from `derive(respawn_seed, [k])` for catch k of the trial; sandboxes with the
                // same respawn seed draw the same candidates for their k-th catch, the accepted point also depends on
                // the creature's position (min distance), so identical positions give identical respawns (TCK-174)
                let mut respawn_rng = derive(self.respawn_seed, &[self.catches as u64]);
                respawn(&mut self.foods[food], &settings.respawn, (agent.px, agent.py), size, size, &mut respawn_rng);
                // The shaping now rewards approach to the next food
                trial.reset_approach();
            }
            trial.step.events = events;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diet_class_thresholds() {
        assert_eq!(DietClass::from_carnivory(0.39), DietClass::Herbivore);
        assert_eq!(DietClass::from_carnivory(0.4), DietClass::Omnivore);
        assert_eq!(DietClass::from_carnivory(0.59), DietClass::Omnivore);
        assert_eq!(DietClass::from_carnivory(0.6), DietClass::Carnivore);
    }

    #[test]
    fn test_entity_trial_reset_clears_the_record_and_keeps_the_buffers() {
        let mut trial = EntityTrial { finished: true, consumed_count: 3, sight_distance: Some(2.0), approach_min_distance: 1.0, starved: true, ..EntityTrial::default() };
        trial.step.inputs = vec![0.5; 8];
        trial.reset();
        assert!(!trial.finished && !trial.starved && trial.consumed_count == 0);
        assert_eq!((trial.sight_distance, trial.approach_min_distance), (None, f32::MAX));
        assert_eq!(trial.step.inputs.len(), 8);
    }
}
