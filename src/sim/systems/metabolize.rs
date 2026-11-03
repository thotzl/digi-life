//! Metabolism system: the shared energy function (`biology::metabolism::drain_energy`) times the world's
//! `metabolic_scale`, then the world's environment energy modifiers in order.

use crate::biology::metabolism::drain_energy;
use crate::config::OceanConfig;
use crate::shared::dto::CreatureAgent;
use crate::sim::{CreatureStep, TemperatureField, TickCtx, World};

/// Environment energy term applied after the drain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnergyModifier {
    /// Green creatures (`carnivory < max_carnivory`, hue in `[hue_min, hue_max]`) in the light zone (top
    /// `light_zone_depth` share of the world) gain `gain`, capped by the stomach.
    Photosynthesis {
        max_carnivory: f32,
        hue_min: f32,
        hue_max: f32,
        light_zone_depth: f32,
        gain: f32,
    },
    /// Outside the thermal tolerance of the phenotype, by more than `min_stress` degrees of the world's temperature
    /// field, the creature loses `stress · penalty_scale`. No effect in a world without temperature field.
    ThermalStress { min_stress: f32, penalty_scale: f32 },
}

impl EnergyModifier {
    /// The Ocean's photosynthesis (config `ocean.photosynthesisEnergyGain`, `bitingCarnivoryThreshold`).
    pub fn photosynthesis(ocean: &OceanConfig) -> Self {
        EnergyModifier::Photosynthesis {
            max_carnivory: ocean.biting_carnivory_threshold,
            hue_min: 75.0,
            hue_max: 175.0,
            light_zone_depth: 0.35,
            gain: ocean.photosynthesis_energy_gain,
        }
    }

    /// The Ocean's thermal stress (config `ocean.thermalStressPenaltyScale`).
    pub fn thermal_stress(ocean: &OceanConfig) -> Self {
        EnergyModifier::ThermalStress {
            min_stress: 0.1,
            penalty_scale: ocean.thermal_stress_penalty_scale,
        }
    }

    pub fn apply(&self, agent: &mut CreatureAgent, height: f32, temperature: Option<TemperatureField>) {
        match *self {
            EnergyModifier::Photosynthesis {
                max_carnivory,
                hue_min,
                hue_max,
                light_zone_depth,
                gain,
            } => {
                let hue = agent.phenotype.primary_color.h;
                let green = agent.phenotype.carnivory < max_carnivory && (hue >= hue_min && hue <= hue_max);
                if green && agent.py < height * light_zone_depth {
                    agent.energy = (agent.energy + gain).min(agent.phenotype.stomach_capacity);
                }
            }
            EnergyModifier::ThermalStress {
                min_stress,
                penalty_scale,
            } => {
                let Some(field) = temperature else { return };
                let local = field.at(agent.py, height);
                let (lo, hi) = (
                    agent.phenotype.thermal_tolerance_min,
                    agent.phenotype.thermal_tolerance_max,
                );
                let stress = if local < lo {
                    lo - local
                } else if local > hi {
                    local - hi
                } else {
                    0.0
                };
                if stress > min_stress {
                    agent.energy -= stress * penalty_scale;
                }
            }
        }
    }
}

/// Drains one tick of metabolism (after the brain step: uses `step.motors.thrust` and the step's synaptic activity)
/// and applies the world's energy modifiers.
pub fn metabolize(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep) {
    let agent = &mut world.creatures[idx];
    step.drain = drain_energy(
        agent,
        step.motors.thrust,
        ctx.metabolism,
        world.settings.metabolic_scale,
    );
    for modifier in &world.settings.energy_modifiers {
        modifier.apply(agent, world.height, world.temperature);
    }
}
