//! Brain model types: neuron roles, motors, systemic inputs, activation and the compiled topology. The compiler is
//! `compile.rs`; the runtime (sensing input layout, CTRNN step, learning) is `crate::shared::brain`.

pub mod compile;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::biology::genome::GeneRef;
use crate::biology::organs::{OrganKind, OrganSide};
use crate::shared::fmath;

pub const SYSTEMIC_BASE_INPUTS_COUNT: usize = SystemicInput::ALL.len();
/// Motor outputs: thrust and bend (biolum and reserved removed, TCK-149).
pub const OUTPUT_MOTOR_NODES_COUNT: usize = Motor::ALL.len();
/// Time constant range of all integrating neurons.
pub const TAU_MIN: f32 = 0.1;
pub const TAU_MAX: f32 = 2.5;

/// Neuron activation function (TCK-155: typed instead of a label string).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum Activation {
    #[default]
    Tanh,
    Relu,
    Sigmoid,
    Sin,
}

impl Activation {
    /// Serialized name (also the bytes of the golden master's model-field hash).
    pub fn label(self) -> &'static str {
        match self {
            Activation::Tanh => "tanh",
            Activation::Relu => "relu",
            Activation::Sigmoid => "sigmoid",
            Activation::Sin => "sin",
        }
    }

    pub fn apply(self, x: f32) -> f32 {
        match self {
            Activation::Relu => x.max(0.0),
            Activation::Sigmoid => 1.0 / (1.0 + fmath::exp(-x)),
            Activation::Sin => fmath::sin(x),
            Activation::Tanh => fmath::tanh(x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub enum SystemicInput {
    Clock,
    LinearSpeed,
    RotationalSpeed,
    Energy,
    Pain,
    SearchArousal,
}

impl SystemicInput {
    /// Input order after the organ channels.
    pub const ALL: [SystemicInput; 6] = [
        SystemicInput::Clock,
        SystemicInput::LinearSpeed,
        SystemicInput::RotationalSpeed,
        SystemicInput::Energy,
        SystemicInput::Pain,
        SystemicInput::SearchArousal,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum Motor {
    Thrust,
    Bend,
}

impl Motor {
    /// Neuron order of the motors (and slot order of `MotorOutputs::from_fn`).
    pub const ALL: [Motor; 2] = [Motor::Thrust, Motor::Bend];
}

/// Motor outputs of one brain step (`execute_brain_with_learning`), by name instead of position (TCK-199).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MotorOutputs {
    /// Forward/backward drive, scaled by `physics::step_creature_kinematics` (invariant 4)
    pub thrust: f32,
    /// Bend (turn) drive
    pub bend: f32,
}

impl MotorOutputs {
    /// Outputs with each motor's value from `value`.
    pub fn from_fn(mut value: impl FnMut(Motor) -> f32) -> Self {
        Self { thrust: value(Motor::Thrust), bend: value(Motor::Bend) }
    }

    pub fn get(&self, motor: Motor) -> f32 {
        match motor {
            Motor::Thrust => self.thrust,
            Motor::Bend => self.bend,
        }
    }

    /// Values in `Motor::ALL` order.
    pub fn values(&self) -> [f32; OUTPUT_MOTOR_NODES_COUNT] {
        Motor::ALL.map(|m| self.get(m))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum ThalamusKind {
    /// L + R (or the single organ)
    Sum,
    /// L − R (HOX pairs only)
    Diff,
}

/// Role of a neuron; neuron order, synapse keys and ownership derive from roles, not from indices (TCK-179).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "role", rename_all = "lowercase")]
#[ts(export)]
pub enum NeuronRole {
    Systemic { input: SystemicInput },
    Sensor { gene: GeneRef, side: OrganSide, channel: usize, kind: OrganKind },
    Thalamus { gene: GeneRef, channel: usize, node: ThalamusKind },
    Hidden { gene: GeneRef },
    Motor { motor: Motor },
}

impl NeuronRole {
    /// Canonical bytes for hashing; `anonymous` drops the gene identity (tape slot of the owner's own end).
    pub(crate) fn encode(&self, anonymous: bool, out: &mut Vec<u8>) {
        let gene = |g: &GeneRef, out: &mut Vec<u8>| {
            if !anonymous {
                out.extend_from_slice(g.promoter.as_bytes());
                out.extend_from_slice(g.spelling.as_bytes());
                out.extend_from_slice(&(g.occurrence as u32).to_le_bytes());
            }
        };
        match self {
            NeuronRole::Systemic { input } => out.extend_from_slice(&[1, *input as u8]),
            NeuronRole::Sensor { gene: g, side, channel, .. } => {
                out.push(2);
                gene(g, out);
                out.extend_from_slice(&[*side as u8, *channel as u8]);
            }
            NeuronRole::Thalamus { gene: g, channel, node } => {
                out.push(3);
                gene(g, out);
                out.extend_from_slice(&[*channel as u8, *node as u8]);
            }
            NeuronRole::Hidden { gene: g } => {
                out.push(4);
                gene(g, out);
            }
            NeuronRole::Motor { motor } => out.extend_from_slice(&[5, *motor as u8]),
        }
    }

    /// The gene this neuron belongs to (organ or `NEU` gene); systemic inputs and motors belong to none.
    pub fn gene(&self) -> Option<&GeneRef> {
        match self {
            NeuronRole::Sensor { gene, .. } | NeuronRole::Thalamus { gene, .. } | NeuronRole::Hidden { gene } => Some(gene),
            NeuronRole::Systemic { .. } | NeuronRole::Motor { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CTRNNSynapse {
    #[serde(rename = "fromNode")]
    #[ts(rename = "fromNode")]
    pub from_node: usize,
    #[serde(rename = "toNode")]
    #[ts(rename = "toNode")]
    pub to_node: usize,
    pub weight: f32,
    /// Stable identity: hash of source and target role (53 bits, exact as a JS number)
    #[ts(type = "number")]
    pub key: u64,
    /// Index into `CreaturePhenotype::genes` of the gene that defines this synapse
    pub owner: usize,
    /// Learns during life (Oja rule) and can be pruned; false for the fixed sensor → thalamus reflexes (TCK-152)
    pub plastic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum NeuronType {
    Input,
    Hidden,
    Output,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CTRNNNeuron {
    pub id: usize,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub neuron_type: NeuronType,
    pub role: NeuronRole,
    pub tau: f32,  // time constant [TAU_MIN, TAU_MAX]
    pub bias: f32, // neural bias [-1.0 to 1.0]
    #[serde(rename = "activationType")]
    #[ts(rename = "activationType")]
    pub activation_type: Option<Activation>,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BrainTopology {
    pub neurons: Vec<CTRNNNeuron>,
    pub synapses: Vec<CTRNNSynapse>,
}
