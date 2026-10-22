use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::shared::rng::SimRng;
use crate::shared::types::{CreatureAgent, FoodSpore};
use crate::biology::dna::{mutate_genome, generate_random_genome};
use crate::biology::creature_factory::{spawn_creature, start_energy, SpawnOptions};
use crate::biology::scenarios::{spawn_at_min_distance, target_distance_by_diet, Scenario, SPAWN_MAX_TRIES};
use rand::Rng;

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
    pub origin_type: SandboxOrigin,
    pub epoch_ticks: u32,
    pub min_distance: f32,
    pub accumulated_yield: f32,
    pub consumed_count: u32,
    pub scenario: Scenario,
    pub cumulative_rotation: f32,
    pub coverage_mask: u64,
    /// Per-tick buffers of `scenarios::step_physics` (reused, not state)
    #[serde(skip)]
    pub sensor_inputs: Vec<f32>,
    #[serde(skip)]
    pub visible_scratch: Vec<FoodSpore>,
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
    pub origin_type: SandboxOrigin,
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
    pub scenario: Scenario,
    pub cumulative_rotation: f32,
    pub coverage_mask: u64,
}

/// Genome of a new sandbox: mutants get `round(len * mutation_rate)` point mutations,
/// random immigrants a fresh random genome, elites and hof entries an unmodified copy.
pub fn breed_genome(parent_genome: &str, origin: SandboxOrigin, mutation_rate: f32, rng: &mut SimRng) -> String {
    let mut genome = parent_genome.to_string();
    if origin == SandboxOrigin::Mutant && mutation_rate > 0.0 {
        let mutations_count = (genome.len() as f32 * mutation_rate).round() as usize;
        for _ in 0..mutations_count {
            if let Some((mutated, _, _, _)) = mutate_genome(&genome, rng) {
                genome = mutated;
            }
        }
    } else if origin == SandboxOrigin::Random {
        genome = generate_random_genome(256, rng);
    }
    genome
}

/// Fresh sandbox in the chamber centre with a random heading and scenario food layout.
/// `methylations` / `synapse_weights` carry Lamarckian inheritance into the factory.
pub fn init_rust_sandbox(
    id: u32,
    genome: &str,
    current_generation: u32,
    origin_type: SandboxOrigin,
    scenario: Scenario,
    methylations: Option<&[f32]>,
    synapse_weights: Option<Vec<f32>>,
    rng: &mut SimRng,
) -> TrainerSandbox {
    let size = scenario.plugin().chamber_size();
    let agent = spawn_creature(genome, SpawnOptions {
        id,
        px: size / 2.0,
        py: size / 2.0,
        heading_angle: rng.gen_range(0.0..std::f32::consts::TAU),
        generation: current_generation,
        methylations,
        synapse_weights,
    });

    let mut sb = TrainerSandbox {
        id,
        agent,
        foods: Vec::new(),
        finished: false,
        finish_tick: None,
        start_distance: 0.0,
        current_fitness: 0.0,
        distance_traveled: 0.0,
        wall_collisions: 0,
        wall_collision_cooldown: 0,
        consumed_spore_type: None,
        origin_type,
        epoch_ticks: 0,
        min_distance: 0.0,
        accumulated_yield: 0.0,
        consumed_count: 0,
        scenario,
        cumulative_rotation: 0.0,
        coverage_mask: 0,
        sensor_inputs: Vec::new(),
        visible_scratch: Vec::new(),
    };

    scenario.plugin().initialize(&mut sb, rng);

    sb
}

impl TrainerSandbox {
    /// Starts the next trial of a multi-trial epoch: agent back to the centre with a random heading,
    /// fresh energy and cleared neuron state, food respawned (learned synapse weights are kept).
    pub fn reset_trial(&mut self, rng: &mut SimRng) {
        let plugin = self.scenario.plugin();
        let size = plugin.chamber_size();

        self.agent.px = size / 2.0;
        self.agent.py = size / 2.0;
        self.agent.vx = 0.0;
        self.agent.vy = 0.0;
        self.agent.heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        self.agent.bend_angle = 0.0;
        self.agent.omega_rot = 0.0;
        self.agent.energy = start_energy(&self.agent.phenotype);
        self.agent.adrenaline = 1.0;
        self.agent.age = 0;
        self.agent.has_eaten = false;

        self.agent.neuron_states.clear();
        self.agent.neuron_activations.clear();

        self.finished = false;
        self.finish_tick = None;
        self.distance_traveled = 0.0;
        self.wall_collisions = 0;
        self.wall_collision_cooldown = 0;
        self.consumed_spore_type = None;
        self.epoch_ticks = 0;
        self.accumulated_yield = 0.0;
        self.consumed_count = 0;
        self.cumulative_rotation = 0.0;
        self.coverage_mask = 0;

        // Randomize spores respecting the trial distance and chamber bounds
        let from = (self.agent.px, self.agent.py);
        for spore in &mut self.foods {
            let (x, y) = spawn_at_min_distance(rng, size, from, plugin.trial_min_distance(), SPAWN_MAX_TRIES);
            spore.x = x;
            spore.y = y;
            spore.vx = 0.0;
            spore.vy = 0.0;
        }

        self.start_distance = target_distance_by_diet(&self.agent, &self.foods);
        self.min_distance = self.start_distance;
        self.current_fitness = 0.0;
    }
}

pub fn step_trainer_sandbox_physics(sb: &mut TrainerSandbox, rng: &mut SimRng) {
    step_trainer_sandbox_physics_with_config(sb, rng, crate::shared::types::AppConfig::global());
}

/// `step_trainer_sandbox_physics` with an explicit config (probes, e.g. Hebbian learning off; TCK-172).
pub fn step_trainer_sandbox_physics_with_config(sb: &mut TrainerSandbox, rng: &mut SimRng, app_config: &crate::shared::types::AppConfig) {
    sb.epoch_ticks += 1;
    let plugin = sb.scenario.plugin();
    let size = plugin.chamber_size();
    let hit_wall = crate::biology::scenarios::step_physics(sb, app_config);

    // Track visited 8x8 grid sectors for unique canvas coverage
    let sec_x = ((sb.agent.px / size) * 8.0).floor().clamp(0.0, 7.0) as u32;
    let sec_y = ((sb.agent.py / size) * 8.0).floor().clamp(0.0, 7.0) as u32;
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
            let from = (sb.agent.px, sb.agent.py);
            let (x, y) = spawn_at_min_distance(rng, size, from, plugin.respawn_min_distance(), SPAWN_MAX_TRIES);
            sb.foods[target_idx].x = x;
            sb.foods[target_idx].y = y;
            sb.foods[target_idx].vx = 0.0;
            sb.foods[target_idx].vy = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::dna::WILDTYPE_GENOME;
    use crate::shared::rng::seeded;

    #[test]
    fn test_finish_tick_records_the_tick_of_first_consumption() {
        let mut rng = seeded(1);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut rng);
        let size = Scenario::Standard.plugin().chamber_size();

        // Park all food in the far corner so nothing can be eaten during the first ticks
        for f in &mut sb.foods {
            f.x = size - 10.0;
            f.y = size - 10.0;
        }
        for _ in 0..5 {
            step_trainer_sandbox_physics(&mut sb, &mut rng);
        }
        assert!(!sb.finished);

        // Drop the plant onto the creature: it is consumed on the 6th tick
        sb.foods[0].x = sb.agent.px;
        sb.foods[0].y = sb.agent.py;
        step_trainer_sandbox_physics(&mut sb, &mut rng);

        assert!(sb.finished);
        assert_eq!(sb.finish_tick, Some(6));
    }

    #[test]
    fn test_reset_trial_restores_trial_start_state() {
        let mut rng = seeded(2);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Mutant, Scenario::Exploration, None, None, &mut rng);
        for _ in 0..30 {
            step_trainer_sandbox_physics(&mut sb, &mut rng);
        }
        let learned = sb.agent.synapse_weights.clone();
        sb.finished = true;
        sb.reset_trial(&mut rng);

        let size = Scenario::Exploration.plugin().chamber_size();
        assert_eq!((sb.agent.px, sb.agent.py), (size / 2.0, size / 2.0));
        assert_eq!((sb.epoch_ticks, sb.distance_traveled, sb.finished), (0, 0.0, false));
        assert!(sb.agent.neuron_states.is_empty());
        assert_eq!(sb.agent.synapse_weights, learned);
        for f in &sb.foods {
            assert!(((f.x - sb.agent.px).powi(2) + (f.y - sb.agent.py).powi(2)).sqrt() >= 1200.0);
        }
        assert_eq!(sb.start_distance, target_distance_by_diet(&sb.agent, &sb.foods));
    }
}
