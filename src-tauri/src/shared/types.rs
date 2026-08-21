use serde::{Deserialize, Serialize};
use crate::biology::dna::CreaturePhenotype;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSpore {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub amount: f32,
    pub vx: f32,
    pub vy: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureAgent {
    pub id: u32,
    #[serde(rename = "speciesId")]
    pub species_id: String,
    
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    
    #[serde(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "bendAngle")]
    pub bend_angle: f32,
    #[serde(rename = "omegaRot")]
    pub omega_rot: f32,
    
    pub energy: f32,
    pub adrenaline: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
    pub has_eaten: bool,
    
    pub genome: String,
    pub antisense: String,
    pub phenotype: CreaturePhenotype,

    #[serde(rename = "neuronStates")]
    pub neuron_states: Vec<f32>,
    #[serde(rename = "neuronActivations")]
    pub neuron_activations: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryCreature {
    pub id: u32,
    #[serde(rename = "speciesId")]
    pub species_id: String,
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    #[serde(rename = "headingAngle")]
    pub heading_angle: f32,
    #[serde(rename = "omegaRot")]
    pub omega_rot: f32,
    pub energy: f32,
    pub adrenaline: f32,
    pub age: u32,
    pub generation: u32,
    #[serde(rename = "hasEaten")]
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
            adrenaline: c.adrenaline,
            age: c.age,
            generation: c.generation,
            has_eaten: c.has_eaten,
        }
    }
}
