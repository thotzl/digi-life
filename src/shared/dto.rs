//! IPC data transfer structs of the simulation state (Ocean agents, food spores, telemetry); exported to `bindings/`
//! via ts-rs.

use serde::{Deserialize, Serialize};
use crate::biology::phenotype::CreaturePhenotype;
use crate::shared::brain::BrainRuntime;
use ts_rs::TS;


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FoodSpore {
    pub id: u32,
    #[serde(rename = "typeId")]
    #[ts(rename = "typeId")]
    pub type_id: u32, // 1 = Plant/Algae, 2 = Meat/Carcass
    pub x: f32,
    pub y: f32,
    pub amount: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreatureAgent {
    pub id: u32,
    #[serde(rename = "speciesId")]
    #[ts(rename = "speciesId")]
    pub species_id: String,
    
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    
    #[serde(rename = "headingAngle")]
    #[ts(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "bendAngle")]
    #[ts(rename = "bendAngle")]
    pub bend_angle: f32,
    #[serde(rename = "omegaRot")]
    #[ts(rename = "omegaRot")]
    pub omega_rot: f32,
    
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
    #[ts(rename = "hasEaten")]
    pub has_eaten: bool,
    
    pub genome: String,
    pub antisense: String,
    pub phenotype: CreaturePhenotype,

    #[serde(rename = "neuronStates")]
    #[ts(rename = "neuronStates")]
    pub neuron_states: Vec<f32>,
    #[serde(rename = "neuronActivations")]
    #[ts(rename = "neuronActivations")]
    pub neuron_activations: Vec<f32>,

    #[serde(rename = "synapseWeights")]
    #[ts(rename = "synapseWeights")]
    pub synapse_weights: Vec<f32>,

    /// Derived brain data (runtime only; reset when phenotype or weights are replaced in place)
    #[serde(skip)]
    #[ts(skip)]
    pub brain_runtime: BrainRuntime,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TelemetryCreature {
    pub id: u32,
    #[serde(rename = "speciesId")]
    #[ts(rename = "speciesId")]
    pub species_id: String,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    #[serde(rename = "headingAngle")]
    #[ts(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "omegaRot")]
    #[ts(rename = "omegaRot")]
    pub omega_rot: f32,
    pub energy: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
    #[ts(rename = "hasEaten")]
    pub has_eaten: bool,
}

impl From<&CreatureAgent> for TelemetryCreature {
    fn from(c: &CreatureAgent) -> Self {
        Self {
            id: c.id,
            species_id: c.species_id.clone(),
            px: c.px,
            py: c.py,
            vx: c.vx,
            vy: c.vy,
            heading_angle: c.heading_angle,
            omega_rot: c.omega_rot,
            energy: c.energy,
            age: c.age,
            generation: c.generation,
            has_eaten: c.has_eaten,
        }
    }
}
