//! The world one creature tick runs in (TCK-200): entities, food, obstacles, bounds and the world
//! settings that make a simulation (Ocean, trainer room, probe) what it is. Contract: `wiki/contracts/creature-system.md`.
//!
//! `World` borrows the owner's storage for one tick: the trainer sandbox owns one creature and its food, the Ocean
//! owns the population and the spore field. Systems read and write only what the world exposes. Randomness is never
//! drawn from a global source: a system that needs it takes an explicit `SimRng` stream from the owner.

use crate::biology::metabolism::MetabolismConfig;
use crate::config::AppConfig;
use crate::config::OceanConfig;
use crate::shared::brain::BrainConfig;
use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::shared::map_generator::ProceduralObstacle;
use crate::shared::physics::PhysicsConfig;
use crate::shared::spatial_grid::SpatialGrid;

use super::System;
use super::systems::death::DeathPolicy;
use super::systems::eat::RespawnPolicy;
use super::systems::metabolize::EnergyModifier;
use super::systems::sense::TargetSelector;

pub use crate::shared::temperature::TemperatureField;

/// Per-tick snapshot of the fields other creatures read (prey targets of the sensing, bite targets).
#[derive(Debug, Clone, PartialEq)]
pub struct PeerSnapshot {
    pub id: u32,
    pub species_id: String,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub energy: f32,
}

impl From<&CreatureAgent> for PeerSnapshot {
    fn from(c: &CreatureAgent) -> Self {
        Self {
            id: c.id,
            species_id: c.species_id.clone(),
            px: c.px,
            py: c.py,
            vx: c.vx,
            vy: c.vy,
            energy: c.energy,
        }
    }
}

/// The rules of one world. Every place where worlds behave differently is a named field here. Since TCK-202 the
/// fields are only real world features (what a trainer room is versus the open Ocean); the creature model itself
/// (clock, eat radius, food drift order, death threshold, temperature) is one rule for every world.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldSettings {
    /// Ordered systems of the per-creature tick (`sim::step_creature`)
    pub systems: &'static [System],
    /// Which food (and prey) the sensors see (room layout with one or several food per type; Ocean with prey)
    pub targets: TargetSelector,
    /// Factor on the shared energy function (trainer `trainerMetabolicScale · room factor`, the curriculum's pressure
    /// ramp; Ocean 1)
    pub metabolic_scale: f32,
    /// Environment energy terms after the drain, in order (Ocean: photosynthesis, thermal stress; a trainer room has
    /// no light zone and no thermal stress)
    pub energy_modifiers: Vec<EnergyModifier>,
    /// What happens with a dead creature (trainer: frozen for the rest of the trial; Ocean: lifespan and
    /// decomposition into meat spores). The death threshold is the same in both (`death::is_dead_energy`).
    pub death: DeathPolicy,
    /// Where eaten food reappears
    pub respawn: RespawnPolicy,
}

impl WorldSettings {
    /// The Ocean's rules from config section `ocean` (`sim::ocean::OceanSim`).
    pub fn ocean(ocean: &OceanConfig, nutrient_centers: Vec<(f32, f32)>) -> Self {
        Self {
            systems: CREATURE_SYSTEMS,
            targets: TargetSelector::NearestPerTypeWithPrey,
            metabolic_scale: 1.0,
            energy_modifiers: vec![
                EnergyModifier::photosynthesis(ocean),
                EnergyModifier::thermal_stress(ocean),
            ],
            death: DeathPolicy::decompose(ocean),
            respawn: RespawnPolicy::Patchy {
                nutrient_centers,
                recycle_meat: true,
            },
        }
    }
}

/// Systems of every simulated world (trainer rooms and Ocean): sensing, brain, drain, death, movement, eating. Food
/// drifts once per tick before the creatures (`food_drift::drift_before_creatures`, called by the owner). The Ocean
/// runs bite and mitosis after these with Ocean state (`systems::bite`, `systems::reproduce`).
pub const CREATURE_SYSTEMS: &[System] = &[
    System::Sense,
    System::Think,
    System::Metabolize,
    System::Death,
    System::Move,
    System::Eat,
];
/// Sensing, brain and movement only: no drain, no death, no food interaction (diagnosis probes).
pub const BRAIN_ONLY_SYSTEMS: &[System] = &[System::Sense, System::Think, System::Move];

/// One tick's view of the simulation. Borrowed from the owner (trainer sandbox, Ocean state, probe).
pub struct World<'a> {
    pub creatures: &'a mut [CreatureAgent],
    pub foods: &'a mut [FoodSpore],
    pub obstacles: &'a [ProceduralObstacle],
    /// Other creatures as seen at the start of the tick (prey targets); empty in single-creature worlds
    pub peers: &'a [PeerSnapshot],
    /// Spatial index of creatures and food (Ocean); `None`: systems scan all entities
    pub grid: Option<&'a SpatialGrid>,
    pub width: f32,
    pub height: f32,
    pub settings: &'a WorldSettings,
}

impl<'a> World<'a> {
    /// World of one creature and its food in a square obstacle-free chamber of edge `size` (trainer room, probes).
    pub fn single(
        agent: &'a mut CreatureAgent,
        foods: &'a mut [FoodSpore],
        size: f32,
        settings: &'a WorldSettings,
    ) -> Self {
        Self {
            creatures: std::slice::from_mut(agent),
            foods,
            obstacles: &[],
            peers: &[],
            grid: None,
            width: size,
            height: size,
            settings,
        }
    }
}

/// Context of one tick: the config sections the systems read and the tick index.
#[derive(Debug, Clone, Copy)]
pub struct TickCtx<'a> {
    pub brain: &'a BrainConfig,
    pub metabolism: &'a MetabolismConfig,
    pub physics: &'a PhysicsConfig,
    /// Tick index of the owner (trainer: tick of the trial, Ocean: global tick)
    pub tick: u64,
}

impl<'a> TickCtx<'a> {
    pub fn new(config: &'a AppConfig, tick: u64) -> Self {
        Self {
            brain: &config.brain,
            metabolism: &config.metabolism,
            physics: &config.physics,
            tick,
        }
    }
}
