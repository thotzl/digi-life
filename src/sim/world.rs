//! The world one creature tick runs in (TCK-200): entities, food, obstacles, bounds, temperature field and the world
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
use super::systems::eat::{EatRadiusRule, RespawnPolicy};
use super::systems::food_drift::FoodDriftOrder;
use super::systems::metabolize::EnergyModifier;
use super::systems::sense::{ClockVariant, TargetSelector};

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

/// Water temperature as a linear function of depth: `surface − (y / height) · span` (the Ocean: 38 °C at the top,
/// 12 °C at the bottom).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TemperatureField {
    pub surface: f32,
    pub span: f32,
}

impl TemperatureField {
    /// The Ocean's stratification (38 °C surface, 26 °C span).
    pub const OCEAN: TemperatureField = TemperatureField {
        surface: 38.0,
        span: 26.0,
    };

    pub fn at(&self, y: f32, height: f32) -> f32 {
        self.surface - (y / height) * self.span
    }
}

/// The rules of one world. Every place where Ocean and trainer behave differently is a named field here, so the
/// divergence is explicit (unifying them is TCK-202).
#[derive(Debug, Clone, PartialEq)]
pub struct WorldSettings {
    /// Ordered systems of the per-creature tick (`sim::step_creature`)
    pub systems: &'static [System],
    /// `sin` variant of the clock input (trainer `fmath::sin`, Ocean `std` sin)
    pub clock: ClockVariant,
    /// Which food (and prey) the sensors see
    pub targets: TargetSelector,
    /// Factor on the shared energy function (trainer `trainerMetabolicScale · room factor`, Ocean 1)
    pub metabolic_scale: f32,
    /// Environment energy terms after the drain, in order (Ocean: photosynthesis, thermal stress)
    pub energy_modifiers: Vec<EnergyModifier>,
    /// What happens at death (trainer: freeze for the rest of the trial; Ocean: decompose into meat spores)
    pub death: DeathPolicy,
    /// When food drifts (trainer: after each creature moved; Ocean: once before all creatures)
    pub food_drift: FoodDriftOrder,
    /// Eat radius (trainer: hard-coded diet rule; Ocean: config `grazingRadiusMultiplier` / `Offset`)
    pub eat_radius: EatRadiusRule,
    /// Where eaten food reappears
    pub respawn: RespawnPolicy,
}

impl WorldSettings {
    /// The Ocean's rules from config section `ocean` (`sim::ocean::OceanSim`).
    pub fn ocean(ocean: &OceanConfig, nutrient_centers: Vec<(f32, f32)>) -> Self {
        Self {
            systems: OCEAN_SYSTEMS,
            clock: ClockVariant::Std,
            targets: TargetSelector::NearestPerTypeWithPrey,
            metabolic_scale: 1.0,
            energy_modifiers: vec![
                EnergyModifier::photosynthesis(ocean),
                EnergyModifier::thermal_stress(ocean),
            ],
            death: DeathPolicy::decompose(ocean),
            food_drift: FoodDriftOrder::BeforeAllCreatures,
            eat_radius: EatRadiusRule::Linear {
                multiplier: ocean.grazing_radius_multiplier,
                offset: ocean.grazing_radius_offset,
            },
            respawn: RespawnPolicy::Patchy {
                nutrient_centers,
                recycle_meat: true,
            },
        }
    }
}

/// Systems of a trainer room: sensing, brain, drain, starvation, movement, food drift, eating.
pub const TRAINER_SYSTEMS: &[System] = &[
    System::Sense,
    System::Think,
    System::Metabolize,
    System::Death,
    System::Move,
    System::FoodDrift,
    System::Eat,
];
/// Systems of the Ocean creature loop (food drift runs once before all creatures, `food_drift::drift_before_creatures`).
/// Bite and mitosis run after these with Ocean state (`systems::bite`, `systems::reproduce`).
pub const OCEAN_SYSTEMS: &[System] = &[
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
    /// `None`: no thermal environment (trainer rooms)
    pub temperature: Option<TemperatureField>,
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
            temperature: None,
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
