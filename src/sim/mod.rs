//! Shared simulation kernel (TCK-200): one per-creature tick for every simulation (trainer rooms, diagnosis probes,
//! the Ocean since TCK-156). Contract: `wiki/contracts/creature-system.md`, guide `wiki/guides/add-dynamic.md`.
//!
//! - `world`: `World` (the borrowed tick view), `WorldSettings` (the rules, divergences as named fields), `TickCtx`
//! - `systems`: one module per dynamic, each a plain function over the world
//! - `step_creature`: runs a world's ordered system list for one creature; events are collected in `CreatureStep`
//!   and applied by the owner after the step (respawn, decomposition, bookkeeping)
//! - `ocean`: the IO-free Ocean (`OceanSim`, TCK-156) on these systems

pub mod ocean;
pub mod systems;
pub mod world;

use crate::biology::brain::MotorOutputs;
use crate::shared::dto::FoodSpore;

pub use world::{PeerSnapshot, TemperatureField, TickCtx, World, WorldSettings, BRAIN_ONLY_SYSTEMS, OCEAN_SYSTEMS, TRAINER_SYSTEMS};

/// One step of the per-creature tick. A world lists the systems it runs, in order (`WorldSettings::systems`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum System {
    /// Visible targets and sensory inputs (`systems::sense`)
    Sense,
    /// Brain step with Hebbian learning (`systems::think`)
    Think,
    /// Energy drain and environment modifiers (`systems::metabolize`)
    Metabolize,
    /// Death check and the world's death policy (`systems::death`)
    Death,
    /// Kinematics (`systems::r#move`)
    Move,
    /// Food pushed by this creature (`systems::food_drift`, only with `FoodDriftOrder::AfterEachCreature`)
    FoodDrift,
    /// Eating food in reach (`systems::eat`)
    Eat,
}

/// Something a system did that the owner applies after the step (as the Ocean's bite damage map).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreatureEvent {
    /// The creature ate `foods[food]` (energy already added); the owner respawns the food (`eat::respawn`)
    Ate { food: usize, is_meat: bool, x: f32, y: f32 },
    /// The creature died and decomposes into `spores` meat spores at (x, y) (`death::decompose`)
    Decomposed { x: f32, y: f32, spores: i32 },
}

/// Scratch and result of one creature's tick; reused across ticks (no allocation in the hot loop).
#[derive(Debug, Clone, Default)]
pub struct CreatureStep {
    /// Sensory inputs of this tick
    pub inputs: Vec<f32>,
    /// Buffer for targets built per tick (`TargetSelector`)
    pub visible: Vec<FoodSpore>,
    pub motors: MotorOutputs,
    /// Energy drained by the metabolism (before modifiers)
    pub drain: f32,
    /// Heading before the move
    pub old_heading: f32,
    pub hit_wall: bool,
    /// The death system ended the creature's tick; later systems did not run
    pub died: bool,
    pub events: Vec<CreatureEvent>,
    /// Grid query buffer
    pub nearby: Vec<u32>,
}

impl CreatureStep {
    fn begin(&mut self) {
        self.motors = MotorOutputs::default();
        self.drain = 0.0;
        self.old_heading = 0.0;
        self.hit_wall = false;
        self.died = false;
        self.events.clear();
    }
}

/// Runs the world's systems in order for creature `idx`. Stops after a death. Deterministic: draws no random numbers.
pub fn step_creature(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep) {
    step.begin();
    for &system in world.settings.systems {
        match system {
            System::Sense => systems::sense::sense(world, idx, step),
            System::Think => systems::think::think(&mut world.creatures[idx], &step.inputs, ctx, &mut step.motors),
            System::Metabolize => systems::metabolize::metabolize(world, idx, ctx, step),
            System::Death => systems::death::death(world, idx, step),
            System::Move => systems::r#move::move_creature(world, idx, ctx, step),
            System::FoodDrift => systems::food_drift::drift_after_creature(world, idx),
            System::Eat => systems::eat::eat(world, idx, ctx, step),
        }
        if step.died {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::creature_factory::{spawn_creature, SpawnOptions};
    use crate::biology::genome::WILDTYPE_GENOME;
    use crate::trainer::room::registry;
    use crate::shared::dto::CreatureAgent;
    use crate::sim::systems::death::DeathPolicy;
    use crate::sim::systems::sense::{ClockVariant, TargetSelector};

    fn agent(px: f32, py: f32) -> CreatureAgent {
        spawn_creature(WILDTYPE_GENOME, SpawnOptions { id: 1, px, py, heading_angle: 0.0, generation: 1, methylations: None, synapse_weights: None }, crate::config::test_config())
    }

    fn food(x: f32, y: f32) -> FoodSpore {
        FoodSpore { id: 1, type_id: 1, x, y, amount: 15.0, vx: 0.0, vy: 0.0 }
    }

    #[test]
    fn test_brain_only_world_moves_without_drain_or_food_interaction() {
        let config = crate::config::test_config();
        let settings = WorldSettings { systems: BRAIN_ONLY_SYSTEMS, ..registry::build("standard", config).unwrap().world_settings(config) };
        let mut a = agent(500.0, 500.0);
        let mut foods = vec![food(500.0, 500.0)];
        let (energy, mut step) = (a.energy, CreatureStep::default());
        for tick in 0..20 {
            step_creature(&mut World::single(&mut a, &mut foods, 1000.0, &settings), 0, &TickCtx::new(config, tick), &mut step);
            assert!(step.events.is_empty() && !step.died);
        }
        assert_eq!(a.energy, energy);
        assert_eq!((foods[0].x, foods[0].y), (500.0, 500.0), "no food drift");
        assert!(!step.inputs.is_empty());
    }

    #[test]
    fn test_ocean_settings_run_drain_eat_and_decompose_at_the_lifespan() {
        // The Ocean's world (`OceanSim`, TCK-156): std clock, prey selector, photosynthesis and
        // thermal stress after the drain, food eaten with the config radius, decomposition at the lifespan
        let config = crate::config::test_config();
        let settings = WorldSettings::ocean(&config.ocean, vec![(1000.0, 800.0)]);
        assert_eq!((settings.clock, settings.targets), (ClockVariant::Std, TargetSelector::NearestPerTypeWithPrey));
        let mut a = agent(1000.0, 800.0);
        a.energy = 50.0;
        let mut foods = vec![food(1000.0, 800.0)];
        let peers = [PeerSnapshot::from(&a)];
        let mut step = CreatureStep::default();
        {
            let mut world = World { temperature: Some(TemperatureField::OCEAN), peers: &peers, height: 1600.0, ..World::single(&mut a, &mut foods, 2000.0, &settings) };
            step_creature(&mut world, 0, &TickCtx::new(config, 1), &mut step);
        }
        assert!(step.drain > 0.0 && !step.died);
        assert!(matches!(step.events[..], [CreatureEvent::Ate { food: 0, is_meat: false, .. }]), "{:?}", step.events);

        let DeathPolicy::Decompose { lifespan_multiplier, max_age_ticks, .. } = settings.death else { panic!("the Ocean decomposes") };
        a.age = (a.phenotype.mature_age * lifespan_multiplier).min(max_age_ticks);
        let (px, py) = (a.px, a.py);
        step_creature(&mut World::single(&mut a, &mut foods, 2000.0, &settings), 0, &TickCtx::new(config, 2), &mut step);
        assert!(step.died);
        assert_eq!((a.px, a.py), (px, py), "a dead creature does not move");
        assert!(matches!(step.events[..], [CreatureEvent::Decomposed { spores, .. }] if spores >= config.ocean.decomposition_spore_min));
    }
}
