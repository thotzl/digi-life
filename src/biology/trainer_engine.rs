use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::shared::rng::{derive, SimRng};
use crate::shared::types::{AppConfig, CreatureAgent, FoodSpore, MutationChannelMode};
use crate::biology::dna::{
    classify_mutation, generate_structured_genome, mutate_genome, mutate_genome_in_channel, parse_genome, GeneChannel, GenomeComposition, MutationTouch, OrganKind,
};
use crate::biology::creature_factory::{spawn_creature, start_energy, SpawnOptions};
use crate::biology::scenarios::{spawn_at_min_distance, target_distance_by_diet, Scenario, SPAWN_MAX_TRIES};
use crate::shared::brain::food_sense_range;
use crate::shared::rng::seeded;
use rand::{Rng, RngCore};

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
    pub current_fitness: f32,
    pub distance_traveled: f32,
    pub wall_collisions: u32,
    pub wall_collision_cooldown: u32,
    pub consumed_spore_type: Option<String>,
    pub origin_type: SandboxOrigin,
    pub epoch_ticks: u32,
    /// Approach episode of the fitness shaping (`scenarios::calculate_fitness`): distance to the nearest diet target
    /// at the tick it first came within food-sensing range; `None` until then. Restarts with every trial and catch.
    pub sight_distance: Option<f32>,
    /// Smallest distance to the nearest diet target since `sight_distance` was set
    pub approach_min_distance: f32,
    pub accumulated_yield: f32,
    pub consumed_count: u32,
    pub scenario: Scenario,
    pub cumulative_rotation: f32,
    pub coverage_mask: u64,
    /// Energy ran out during the current trial: the creature stops and cannot eat until the trial ends (TCK-178)
    pub starved: bool,
    /// Seed of the respawns after eating in the current trial: catch `k` (1-based, `consumed_count`) respawns its food
    /// from `derive(respawn_seed, [k])`. Drawn from the trial's layout stream, so in the trainer every sandbox of a
    /// generation has the same seed (TCK-174, see `TrainerSim`).
    #[serde(default)]
    pub respawn_seed: u64,
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

/// Number of mutation operator calls for one child: uniform in `[ops_min, max(ops_max, ops_min)]` (TCK-174;
/// config `mutationOpsMin` / `mutationOpsMax`, replaces `round(len * mutation_rate)`).
pub fn draw_mutation_ops(ops_min: u32, ops_max: u32, rng: &mut SimRng) -> u32 {
    rng.gen_range(ops_min..=ops_max.max(ops_min))
}

/// Genome of a new sandbox: mutants and hall-of-fame copies get `draw_mutation_ops(ops_min, ops_max)` calls of
/// `mutate_genome` (operator mix `mutationOperators`), random immigrants a fresh structured genome (config
/// `genomeComposition`; `random_genome` with the minimal criterion), elites an unmodified copy. Hof entries are mutated
/// since TCK-174 (before: unmodified copies).
pub fn breed_genome(parent_genome: &str, origin: SandboxOrigin, ops_min: u32, ops_max: u32, minimal_criterion: bool, rng: &mut SimRng) -> String {
    breed_genome_traced(parent_genome, origin, ops_min, ops_max, minimal_criterion, MutationChannelMode::Mixed, rng).0
}

/// Probability that a child in mutation channel mode "separate" mutates its body genes (otherwise its brain genes).
pub const SEPARATE_BODY_SHARE: f64 = 0.5;

/// `breed_genome` with the gene channels its mutations touched (TCK-181; `dna::classify_mutation` per operator call,
/// unioned over the child's calls): `Some` for mutated origins (mutant, hof), `None` otherwise. Mode `Mixed` draws
/// exactly what `breed_genome` draws (the classification uses no RNG). Mode `Separate` first draws the child's
/// channel (`SEPARATE_BODY_SHARE`) from `rng`, then restricts every call to it (`dna::mutate_genome_in_channel`).
pub fn breed_genome_traced(
    parent_genome: &str,
    origin: SandboxOrigin,
    ops_min: u32,
    ops_max: u32,
    minimal_criterion: bool,
    mode: MutationChannelMode,
    rng: &mut SimRng,
) -> (String, Option<MutationTouch>) {
    let config = AppConfig::global();
    let mut genome = parent_genome.to_string();
    match origin {
        SandboxOrigin::Mutant | SandboxOrigin::Hof => {
            let channel = match mode {
                MutationChannelMode::Mixed => None,
                MutationChannelMode::Separate => Some(if rng.gen_bool(SEPARATE_BODY_SHARE) { GeneChannel::Body } else { GeneChannel::Brain }),
            };
            let mut touch = MutationTouch::default();
            for _ in 0..draw_mutation_ops(ops_min, ops_max, rng) {
                let mutation = match channel {
                    None => mutate_genome(&genome, &config.mutation_operators, rng),
                    Some(c) => mutate_genome_in_channel(&genome, &config.mutation_operators, c, rng),
                };
                if let Some(mutation) = mutation {
                    touch = touch.union(classify_mutation(&genome, &mutation));
                    genome = mutation.genome;
                }
            }
            return (genome, Some(touch));
        }
        SandboxOrigin::Random => genome = random_genome(minimal_criterion, rng),
        SandboxOrigin::Elite => {}
    }
    (genome, None)
}

/// Random trainer genome (generation 1 and immigrants). Without the minimal criterion: `generate_structured_genome`
/// with config `genomeComposition`. With it (RV-S09, config `trainerMinimalCriterion`): the same generator with a
/// food organ gene in every genome (`pFoodOrgan` = 1.0), redrawn up to `maxRedraws` times until
/// `meets_minimal_criterion`; if no candidate passes, the last one is kept. All draws come from `rng`, so the result
/// is deterministic.
pub fn random_genome(minimal_criterion: bool, rng: &mut SimRng) -> String {
    let config = AppConfig::global();
    if !minimal_criterion {
        return generate_structured_genome(&config.genome_composition, rng);
    }
    let criterion = &config.trainer_minimal_criterion;
    let composition = GenomeComposition { p_food_organ: 1.0, ..config.genome_composition.clone() };
    let mut genome = generate_structured_genome(&composition, rng);
    for _ in 0..criterion.max_redraws {
        if meets_minimal_criterion(&genome, config) {
            break;
        }
        genome = generate_structured_genome(&composition, rng);
    }
    genome
}

/// Fixed seed of the mobility check's standard-room layout (independent of the run seed).
const MOBILITY_CHECK_SEED: u64 = 0x4D43;

/// Minimal criterion (RV-S09): the phenotype has at least one food-sensing organ (eye or nose, as `food_organ_share`)
/// and the creature moves: in a fixed standard-room trial (centre start, `MOBILITY_CHECK_SEED` layout) its own brain
/// moves it at least `mobilityMinDistance` px within `mobilityTicks` ticks.
pub fn meets_minimal_criterion(genome: &str, config: &AppConfig) -> bool {
    let criterion = &config.trainer_minimal_criterion;
    let phenotype = parse_genome(genome, None, None);
    if !phenotype.organelles.iter().any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose)) {
        return false;
    }
    let mut sb = init_rust_sandbox(1, genome, 1, SandboxOrigin::Random, Scenario::Standard, None, None, &mut seeded(MOBILITY_CHECK_SEED));
    for _ in 0..criterion.mobility_ticks {
        step_trainer_sandbox_physics_with_config(&mut sb, config);
        if sb.distance_traveled >= criterion.mobility_min_distance {
            return true;
        }
    }
    false
}

/// Fresh sandbox in the chamber centre with a random heading and scenario food layout; draws heading, food positions
/// and the respawn seed from `rng` in this order (the trainer passes the trial's shared layout stream, TCK-174).
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
        current_fitness: 0.0,
        distance_traveled: 0.0,
        wall_collisions: 0,
        wall_collision_cooldown: 0,
        consumed_spore_type: None,
        origin_type,
        epoch_ticks: 0,
        sight_distance: None,
        approach_min_distance: f32::MAX,
        accumulated_yield: 0.0,
        consumed_count: 0,
        scenario,
        cumulative_rotation: 0.0,
        coverage_mask: 0,
        starved: false,
        respawn_seed: 0,
        sensor_inputs: Vec::new(),
        visible_scratch: Vec::new(),
    };

    scenario.plugin().initialize(&mut sb, rng);
    sb.respawn_seed = rng.next_u64();

    sb
}

impl TrainerSandbox {
    /// Starts the next trial of a multi-trial epoch: agent back to the centre with a random heading,
    /// fresh energy, age 0 and cleared neuron state, food respawned (learned synapse weights are kept).
    /// Draws heading, food positions and the respawn seed from `rng` in this order (the trainer passes the trial's
    /// shared layout stream, TCK-174).
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
        self.starved = false;

        // Same food placement as the first trial (plugin spawn distance, TCK-154)
        plugin.initialize(self, rng);
        self.respawn_seed = rng.next_u64();
        self.current_fitness = 0.0;
    }

    /// Restarts the approach episode of the fitness shaping (trial start and after every catch).
    pub fn reset_approach(&mut self) {
        self.sight_distance = None;
        self.approach_min_distance = f32::MAX;
    }

    /// Relative approach `1 - d_min / d_sight` of the current episode in `[0, 1]`; 0 before a diet target was in
    /// food-sensing range (`scenarios::calculate_fitness`).
    pub fn approach_fraction(&self) -> f32 {
        match self.sight_distance {
            Some(sight) if sight > 0.0 => (1.0 - self.approach_min_distance / sight).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    /// Per-tick approach tracking: opens the episode when the nearest diet target is within the creature's
    /// food-sensing range (`food_sense_range`), then keeps the smallest distance to the nearest diet target.
    fn track_approach(&mut self) {
        let dist = target_distance_by_diet(&self.agent, &self.foods);
        if self.sight_distance.is_none() && dist <= food_sense_range(&self.agent.phenotype) {
            self.sight_distance = Some(dist);
        }
        if self.sight_distance.is_some() {
            self.approach_min_distance = self.approach_min_distance.min(dist);
        }
    }
}

/// One tick of one sandbox. Deterministic without an RNG argument: respawns after eating use `sb.respawn_seed`.
pub fn step_trainer_sandbox_physics(sb: &mut TrainerSandbox) {
    step_trainer_sandbox_physics_with_config(sb, crate::shared::types::AppConfig::global());
}

/// `step_trainer_sandbox_physics` with an explicit config (probes, e.g. Hebbian learning off; TCK-172).
pub fn step_trainer_sandbox_physics_with_config(sb: &mut TrainerSandbox, app_config: &crate::shared::types::AppConfig) {
    sb.epoch_ticks += 1;
    let plugin = sb.scenario.plugin();
    let size = plugin.chamber_size();
    let hit_wall = crate::biology::scenarios::step_physics(sb, app_config);
    // Age per trainer tick after the brain step, as in the Ocean (drives the clock input; reset per trial, TCK-174)
    sb.agent.age += 1;
    if sb.starved {
        // Out of energy: no movement, no catches for the rest of the trial (TCK-178)
        return;
    }

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

    // Approach towards the nearest diet target in sensor range (fitness shaping, TCK-175)
    sb.track_approach();

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
            // Refill with the shared food energy rule of the Ocean (TCK-178)
            let gain = crate::biology::metabolism::food_energy(carnivory, is_meat, &app_config.rules);
            sb.agent.energy = (sb.agent.energy + gain).min(sb.agent.phenotype.stomach_capacity);

            if !sb.finished {
                sb.finished = true; // Still mark finished as true for green SUCCESS label!
                sb.finish_tick = Some(sb.epoch_ticks);
                sb.consumed_spore_type = Some(if is_meat { "meat" } else { "plant" }.to_string());
                sb.agent.has_eaten = true;
            }

            // Spore respawn respecting the scenario bounds. Candidate points come from `derive(respawn_seed, [k])` for
            // catch k of the trial: sandboxes with the same respawn seed draw the same candidates for their k-th catch;
            // the accepted point also depends on the creature's position (min distance), so identical positions give
            // identical respawns (TCK-174)
            let mut respawn_rng = derive(sb.respawn_seed, &[sb.consumed_count as u64]);
            let from = (sb.agent.px, sb.agent.py);
            let (x, y) = spawn_at_min_distance(&mut respawn_rng, size, from, plugin.spawn_min_distance(), SPAWN_MAX_TRIES);
            sb.foods[target_idx].x = x;
            sb.foods[target_idx].y = y;
            sb.foods[target_idx].vx = 0.0;
            sb.foods[target_idx].vy = 0.0;
            // The shaping now rewards approach to the next food
            sb.reset_approach();
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
            step_trainer_sandbox_physics(&mut sb);
        }
        assert!(!sb.finished);

        // Drop the plant onto the creature: it is consumed on the 6th tick
        sb.foods[0].x = sb.agent.px;
        sb.foods[0].y = sb.agent.py;
        step_trainer_sandbox_physics(&mut sb);

        assert!(sb.finished);
        assert_eq!(sb.finish_tick, Some(6));
    }

    #[test]
    fn test_trainer_drain_equals_the_ocean_energy_function() {
        // Trainer path: the shared `step_physics`. Ocean path: the same sensing and brain step followed by the Ocean's
        // `drain_energy(.., 1.0)` call. Same agent, tick and outputs → identical drain (trainer scale 1).
        let mut config = AppConfig::global().clone();
        config.metabolism.trainer_metabolic_scale = 1.0;
        let mut rng = seeded(4);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut rng);
        for _ in 0..20 {
            let mut ocean = sb.agent.clone();
            let foods = sb.foods.clone();
            let start = sb.agent.energy;
            crate::biology::scenarios::step_physics(&mut sb, &config);

            let size = Scenario::Standard.plugin().chamber_size();
            let clock_val = 0.5 + 0.5 * crate::shared::fmath::sin((ocean.age as f32) * 0.1);
            let inputs = crate::shared::brain::compute_sensory_inputs(&ocean, clock_val, &foods, &[], size, size);
            let out = crate::shared::brain::execute_brain_with_learning(
                &ocean.phenotype.brain,
                &inputs,
                &mut ocean.neuron_states,
                &mut ocean.neuron_activations,
                &mut ocean.synapse_weights,
                &mut ocean.brain_runtime,
                config.rules.hebbian_learning_rate_base,
            );
            let drain = crate::biology::metabolism::drain_energy(&mut ocean, out[0], &config.metabolism, 1.0);
            assert!(drain > 0.0);
            assert_eq!(sb.agent.energy.to_bits(), ocean.energy.to_bits(), "trainer drain {} vs ocean drain {}", start - sb.agent.energy, drain);
        }
    }

    #[test]
    fn test_starved_creature_stops_and_cannot_eat() {
        let mut rng = seeded(5);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut rng);
        let size = Scenario::Standard.plugin().chamber_size();
        for f in &mut sb.foods {
            f.x = size - 10.0;
            f.y = size - 10.0;
        }
        sb.agent.energy = 1e-6;
        step_trainer_sandbox_physics(&mut sb);
        assert!(sb.starved);
        assert_eq!((sb.agent.energy, sb.agent.vx, sb.agent.vy), (0.0, 0.0, 0.0));

        // Frozen for the rest of the trial: no movement, food on top of it is not eaten
        let (px, py) = (sb.agent.px, sb.agent.py);
        sb.foods[0].x = px;
        sb.foods[0].y = py;
        for _ in 0..10 {
            step_trainer_sandbox_physics(&mut sb);
        }
        assert_eq!((sb.agent.px, sb.agent.py), (px, py));
        assert!(!sb.finished && sb.consumed_count == 0);
        assert_eq!(sb.epoch_ticks, 11);

        sb.reset_trial(&mut rng);
        assert!(!sb.starved);
        assert_eq!(sb.agent.energy, crate::biology::creature_factory::start_energy(&sb.agent.phenotype));
    }

    #[test]
    fn test_eating_refills_energy_with_the_shared_food_rule() {
        let mut rng = seeded(6);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut rng);
        let size = Scenario::Standard.plugin().chamber_size();
        for f in &mut sb.foods {
            f.x = size - 10.0;
            f.y = size - 10.0;
        }
        step_trainer_sandbox_physics(&mut sb);
        let before = sb.agent.energy;
        assert!(before < crate::biology::creature_factory::start_energy(&sb.agent.phenotype), "energy drains every tick");

        sb.agent.energy = 10.0;
        sb.foods[0].x = sb.agent.px;
        sb.foods[0].y = sb.agent.py;
        step_trainer_sandbox_physics(&mut sb);
        assert_eq!(sb.consumed_count, 1);
        let gain = crate::biology::metabolism::food_energy(sb.agent.phenotype.carnivory, false, &AppConfig::global().rules);
        assert!(sb.agent.energy > 10.0 + gain - 1.0 && sb.agent.energy < 10.0 + gain, "energy {} after eating (gain {})", sb.agent.energy, gain);
    }

    #[test]
    fn test_reset_trial_restores_trial_start_state() {
        let mut rng = seeded(2);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Mutant, Scenario::Exploration, None, None, &mut rng);
        for _ in 0..30 {
            step_trainer_sandbox_physics(&mut sb);
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
            assert!(((f.x - sb.agent.px).powi(2) + (f.y - sb.agent.py).powi(2)).sqrt() >= Scenario::Exploration.plugin().spawn_min_distance());
        }
        assert_eq!((sb.sight_distance, sb.approach_min_distance), (None, f32::MAX));
    }
    #[test]
    fn test_age_increments_every_tick_and_resets_per_trial() {
        let mut rng = seeded(8);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut rng);
        assert_eq!(sb.agent.age, 0);
        for tick in 1..=7 {
            step_trainer_sandbox_physics(&mut sb);
            assert_eq!(sb.agent.age, tick);
        }
        sb.reset_trial(&mut rng);
        assert_eq!(sb.agent.age, 0);
    }

    #[test]
    fn test_respawn_depends_only_on_respawn_seed_catch_index_and_position() {
        let mut a = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut seeded(9));
        let mut b = init_rust_sandbox(2, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut seeded(9));
        assert_eq!(a.respawn_seed, b.respawn_seed);
        let eat_plant = |sb: &mut TrainerSandbox| {
            sb.foods[0].x = sb.agent.px;
            sb.foods[0].y = sb.agent.py;
            step_trainer_sandbox_physics(sb);
            (sb.foods[0].x, sb.foods[0].y)
        };
        // Same seed, same catch index, same position: same respawn point; the next catch draws a new point
        let first = eat_plant(&mut a);
        assert_eq!(first, eat_plant(&mut b));
        assert_eq!(a.consumed_count, 1);
        let second = eat_plant(&mut a);
        assert_ne!(first, second);
        // Another respawn seed draws other candidates
        let mut c = init_rust_sandbox(3, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut seeded(10));
        assert_ne!(first, eat_plant(&mut c));
    }

    #[test]
    fn test_mutation_ops_drawn_within_bounds() {
        let mut rng = seeded(11);
        let mut seen = [0u32; 4];
        for _ in 0..600 {
            let k = draw_mutation_ops(1, 3, &mut rng);
            assert!((1..=3).contains(&k), "{} ops", k);
            seen[k as usize] += 1;
        }
        assert!(seen[1..].iter().all(|&c| c > 100), "{:?}", seen);
        // max below min collapses to min
        assert_eq!(draw_mutation_ops(2, 1, &mut rng), 2);
    }

    #[test]
    fn test_breed_genome_mutates_hof_and_mutants_but_not_elites() {
        let mut changed = [0; 3];
        for seed in 0..20 {
            for (i, origin) in [SandboxOrigin::Hof, SandboxOrigin::Mutant, SandboxOrigin::Elite].into_iter().enumerate() {
                changed[i] += (breed_genome(WILDTYPE_GENOME, origin, 1, 3, false, &mut seeded(seed)) != WILDTYPE_GENOME) as usize;
            }
        }
        assert!(changed[0] >= 15 && changed[1] >= 15, "{:?}", changed);
        assert_eq!(changed[2], 0);
        // Hof and mutant follow the same rule: same stream, same child
        assert_eq!(breed_genome(WILDTYPE_GENOME, SandboxOrigin::Hof, 1, 3, false, &mut seeded(3)), breed_genome(WILDTYPE_GENOME, SandboxOrigin::Mutant, 1, 3, false, &mut seeded(3)));
    }

    fn food_distance(sb: &TrainerSandbox, f: &FoodSpore) -> f32 {
        ((f.x - sb.agent.px).powi(2) + (f.y - sb.agent.py).powi(2)).sqrt()
    }

    #[test]
    fn test_spawn_distance_identical_for_first_trial_later_trials_and_respawns() {
        // RV-S08: one plugin distance for trial 0, re-trials and respawns after eating
        for scenario in [Scenario::Standard, Scenario::Exploration] {
            let min_dist = scenario.plugin().spawn_min_distance();
            let expected = match scenario {
                Scenario::Standard => AppConfig::global().trainer_spawn_min_distance.standard,
                Scenario::Exploration => AppConfig::global().trainer_spawn_min_distance.exploration,
            };
            assert_eq!(min_dist, expected);
            for seed in 0..20 {
                let mut rng = seeded(100 + seed);
                let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, scenario, None, None, &mut rng);
                assert!(sb.foods.iter().all(|f| food_distance(&sb, f) >= min_dist), "{:?} trial 0", scenario);
                sb.reset_trial(&mut rng);
                assert!(sb.foods.iter().all(|f| food_distance(&sb, f) >= min_dist), "{:?} trial 1", scenario);
                // Respawn after a catch at the creature's position
                sb.foods[0].x = sb.agent.px;
                sb.foods[0].y = sb.agent.py;
                step_trainer_sandbox_physics(&mut sb);
                assert_eq!(sb.consumed_count, 1);
                assert!(food_distance(&sb, &sb.foods[0]) >= min_dist, "{:?} respawn", scenario);
            }
        }
    }

    #[test]
    fn test_approach_tracking_opens_in_sensor_range_and_restarts_after_a_catch() {
        let mut rng = seeded(12);
        let mut sb = init_rust_sandbox(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, Scenario::Standard, None, None, &mut rng);
        let range = food_sense_range(&sb.agent.phenotype);
        assert!(range > 0.0, "wildtype has a food-sensing organ");
        let (cx, cy) = (sb.agent.px, sb.agent.py);
        // All food out of range: no episode, no shaping
        for f in &mut sb.foods {
            f.x = cx + range + 50.0;
            f.y = cy;
        }
        sb.track_approach();
        assert_eq!((sb.sight_distance, sb.approach_fraction()), (None, 0.0));
        // Food at 0.8 range: the episode opens at that distance; halving the distance gives approach 0.5
        for f in &mut sb.foods {
            f.x = cx + 0.8 * range;
        }
        sb.track_approach();
        assert!((sb.sight_distance.unwrap() - 0.8 * range).abs() < 1e-3, "{:?}", sb.sight_distance);
        for f in &mut sb.foods {
            f.x = cx + 0.4 * range;
        }
        sb.track_approach();
        assert!((sb.approach_fraction() - 0.5).abs() < 1e-5, "{}", sb.approach_fraction());
        // Moving away keeps the best approach
        for f in &mut sb.foods {
            f.x = cx + 0.7 * range;
        }
        sb.track_approach();
        assert!((sb.approach_fraction() - 0.5).abs() < 1e-5);
        // A catch restarts the episode
        sb.foods[0].x = sb.agent.px;
        sb.foods[0].y = sb.agent.py;
        sb.foods[1].x = sb.agent.px;
        sb.foods[1].y = sb.agent.py;
        step_trainer_sandbox_physics(&mut sb);
        assert!(sb.consumed_count >= 1);
        assert!(sb.approach_fraction() < 0.5);
    }

    #[test]
    fn test_minimal_criterion_yields_food_organ_and_mobility() {
        let config = AppConfig::global();
        let mut passed = 0;
        for seed in 0..40 {
            let genome = random_genome(true, &mut seeded(seed));
            // Deterministic: same stream, same genome
            assert_eq!(genome, random_genome(true, &mut seeded(seed)));
            let phenotype = parse_genome(&genome, None, None);
            assert!(phenotype.organelles.iter().any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose)), "seed {} without food organ", seed);
            passed += meets_minimal_criterion(&genome, config) as usize;
        }
        assert_eq!(passed, 40, "every minimal-criterion genome moves");
        // Off: the plain structured generator (same stream as before TCK-175)
        assert_eq!(random_genome(false, &mut seeded(3)), generate_structured_genome(&config.genome_composition, &mut seeded(3)));
        assert_eq!(breed_genome(WILDTYPE_GENOME, SandboxOrigin::Random, 1, 3, true, &mut seeded(5)), random_genome(true, &mut seeded(5)));
    }

    #[test]
    fn test_minimal_criterion_rejects_organless_genomes() {
        // A genome without organs fails the criterion
        let organless = (0..20)
            .map(|seed| crate::biology::probes::letter_soup_genomes(seed, 50))
            .flatten()
            .find(|g| parse_genome(g, None, None).organelles.is_empty())
            .expect("letter soup contains organless genomes");
        assert!(!meets_minimal_criterion(&organless, AppConfig::global()));
    }

    #[test]
    fn test_breed_genome_traced_mixed_matches_breed_genome_and_separate_stays_in_one_channel() {
        use crate::biology::dna::MutationChannel;
        let mut channels = std::collections::BTreeMap::new();
        for seed in 0..80 {
            let parent = random_genome(false, &mut seeded(900 + seed));
            let (mixed, touch) = breed_genome_traced(&parent, SandboxOrigin::Mutant, 1, 3, false, MutationChannelMode::Mixed, &mut seeded(seed));
            assert_eq!(mixed, breed_genome(&parent, SandboxOrigin::Mutant, 1, 3, false, &mut seeded(seed)), "mixed draws exactly like breed_genome");
            assert!(touch.is_some());
            let (_, touch) = breed_genome_traced(&parent, SandboxOrigin::Mutant, 1, 3, false, MutationChannelMode::Separate, &mut seeded(seed));
            let channel = touch.expect("mutated origin").channel();
            assert_ne!(channel, MutationChannel::Both, "separate child touched both channels");
            *channels.entry(format!("{:?}", channel)).or_insert(0) += 1;
        }
        assert!(channels.get("Body").copied().unwrap_or(0) > 20 && channels.get("Brain").copied().unwrap_or(0) > 20, "{:?}", channels);
        // Unmutated origins carry no channel
        assert_eq!(breed_genome_traced(WILDTYPE_GENOME, SandboxOrigin::Elite, 1, 3, false, MutationChannelMode::Separate, &mut seeded(1)).1, None);
    }
}
