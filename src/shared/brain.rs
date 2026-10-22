use crate::biology::dna::{
    channel_sensitivity, BrainTopology, Motor, NeuronRole, OrganKind, SystemicInput, CHANNELS_PER_ORGANELLE, DELTA_CHANNEL,
    MEAT_EMISSION, MIN_CHANNEL_MATCH, OUTPUT_MOTOR_NODES_COUNT, PLANT_EMISSION, SPECTRAL_CHANNELS, SYSTEMIC_BASE_INPUTS_COUNT,
    TAU_MAX, TAU_MIN,
};
use crate::shared::types::{CreatureAgent, FoodSpore};
use crate::shared::map_generator::ProceduralObstacle;
use crate::shared::fmath;

/// Gain of the ΔI channel: input = clamp(gain · (I_now − I_prev), −1, 1) (TCK-133, signed since TCK-144).
const DELTA_GAIN: f32 = 2.5;

/// Neuron activation function, decoded once from `CTRNNNeuron::activation_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    Tanh,
    Relu,
    Sigmoid,
    Sin,
}

impl Activation {
    /// Unknown or missing labels fall back to tanh.
    pub fn from_label(label: Option<&str>) -> Self {
        match label.unwrap_or("tanh") {
            "relu" => Activation::Relu,
            "sigmoid" => Activation::Sigmoid,
            "sin" => Activation::Sin,
            _ => Activation::Tanh,
        }
    }

    fn apply(self, x: f32) -> f32 {
        match self {
            Activation::Relu => x.max(0.0),
            Activation::Sigmoid => 1.0 / (1.0 + fmath::exp(-x)),
            Activation::Sin => fmath::sin(x),
            Activation::Tanh => fmath::tanh(x),
        }
    }
}

/// Factor of the exact exponential step `y += (1 − exp(−dt/τ)) · (target − y)` with dt = 1 (TCK-153):
/// in (0, 1) for every τ, so the state never overshoots its target (forward Euler oscillated for τ < 0.5).
pub fn integration_alpha(tau: f32) -> f32 {
    1.0 - fmath::exp(-1.0 / tau.clamp(TAU_MIN, TAU_MAX))
}

/// Per-agent data derived from the brain topology (roles), so the brain step does no decoding.
/// Not serialized; built lazily on the first brain step and rebuilt when the topology or the weight vector changes.
/// Reset it (`BrainRuntime::default()`) whenever the agent's phenotype or synapse weights are replaced in place.
#[derive(Debug, Clone, Default)]
pub struct BrainRuntime {
    built: bool,
    activations: Vec<Activation>,
    /// Per neuron: `integration_alpha(tau)`
    alpha: Vec<f32>,
    /// Sensor and systemic input neurons (the leading neurons)
    input_count: usize,
    /// First input slot of every distance-sense organ (source of its ΔI channel)
    delta_organs: Vec<usize>,
    /// Neuron index per motor, in `Motor::ALL` order
    motors: [usize; OUTPUT_MOTOR_NODES_COUNT],
    /// Synapses with a non-zero weight, in synapse order (sorted by `to_node`); pruned ones are dropped
    active_synapses: Vec<ActiveSynapse>,
}

#[derive(Debug, Clone, Copy)]
struct ActiveSynapse {
    idx: usize,
    from: usize,
    to: usize,
}

impl BrainRuntime {
    fn build(&mut self, brain: &BrainTopology, synapse_weights: &[f32]) {
        self.activations = brain.neurons.iter().map(|n| Activation::from_label(n.activation_type.as_deref())).collect();
        self.alpha = brain.neurons.iter().map(|n| integration_alpha(n.tau)).collect();
        self.input_count = brain.neurons.iter().take_while(|n| matches!(n.role, NeuronRole::Sensor { .. } | NeuronRole::Systemic { .. })).count();
        self.delta_organs = brain
            .neurons
            .iter()
            .filter(|n| matches!(n.role, NeuronRole::Sensor { channel: DELTA_CHANNEL, kind, .. } if kind.is_distance()))
            .map(|n| n.id - DELTA_CHANNEL)
            .collect();
        for (slot, motor) in Motor::ALL.into_iter().enumerate() {
            self.motors[slot] = brain.neurons.iter().position(|n| n.role == NeuronRole::Motor { motor }).unwrap_or(0);
        }
        self.active_synapses = brain
            .synapses
            .iter()
            .enumerate()
            .filter(|(idx, _)| synapse_weights[*idx] != 0.0)
            .map(|(idx, s)| ActiveSynapse { idx, from: s.from_node, to: s.to_node })
            .collect();
        self.built = true;
    }
}

/// One CTRNN tick with Hebbian learning. Neurons are integrated in index order, which `parse_genome` makes the
/// evaluation order Inputs → Thalamus → Hidden (by depth) → Motors: a synapse from a lower index reads this tick's
/// activation, one from a higher index the previous tick's. Returns the motor outputs `[thrust, bend]`.
pub fn execute_brain_with_learning(
    brain: &BrainTopology,
    inputs: &[f32],
    neuron_states: &mut Vec<f32>,
    neuron_activations: &mut Vec<f32>,
    synapse_weights: &mut Vec<f32>,
    runtime: &mut BrainRuntime,
    hebbian_rate: f32,
    _hebbian_decay: f32, // stiffness decay
    forgetting_decay: f32,
) -> [f32; OUTPUT_MOTOR_NODES_COUNT] {
    let total_nodes = brain.neurons.len();
    let total_synapses = brain.synapses.len();

    // Dynamically resize and initialize synapse weights if needed
    let weights_reset = synapse_weights.len() != total_synapses;
    if weights_reset {
        synapse_weights.clear();
        for syn in &brain.synapses {
            synapse_weights.push(syn.weight);
        }
    }

    // Dynamically resize internal arrays to ensure thread-safety and no-allocation runs
    if neuron_states.len() != total_nodes {
        neuron_states.resize(total_nodes, 0.0);
    }
    if neuron_activations.len() != total_nodes {
        neuron_activations.resize(total_nodes, 0.0);
    }
    if !runtime.built || runtime.activations.len() != total_nodes || weights_reset {
        runtime.build(brain, synapse_weights);
    }
    let input_count = runtime.input_count;

    // 1. Sensor and systemic inputs
    let injected = input_count.min(inputs.len());
    neuron_activations[..injected].copy_from_slice(&inputs[..injected]);

    // 1b. ΔI channel of distance senses: signed change of the organ's total spectral intensity since the last tick.
    //     Input neurons do not integrate, so the ΔI neuron's state holds the previous intensity.
    for &base in &runtime.delta_organs {
        if let Some(channels) = inputs.get(base..base + SPECTRAL_CHANNELS) {
            let intensity: f32 = channels.iter().sum();
            let delta = base + DELTA_CHANNEL;
            neuron_activations[delta] = ((intensity - neuron_states[delta]) * DELTA_GAIN).clamp(-1.0, 1.0);
            neuron_states[delta] = intensity;
        }
    }

    // Pruned synapses (weight 0) contribute nothing to the sums and are skipped
    let active = &runtime.active_synapses;
    let mut syn_idx = 0;
    while syn_idx < active.len() && active[syn_idx].to < input_count {
        syn_idx += 1;
    }

    // 2. Exact exponential integration of thalamus, hidden and motor neurons (dt = 1)
    for i in input_count..total_nodes {
        // Sum synaptic weights from all active connections in a single linear O(S) pass!
        let mut sum = 0.0;
        while syn_idx < active.len() && active[syn_idx].to == i {
            let syn = active[syn_idx];
            sum += neuron_activations[syn.from] * synapse_weights[syn.idx];
            syn_idx += 1;
        }

        let target = sum + brain.neurons[i].bias;
        neuron_states[i] += runtime.alpha[i] * (target - neuron_states[i]);

        // Clamping to prevent numerical divergent explosion (NaN prevention)
        neuron_states[i] = neuron_states[i].clamp(-4.0, 4.0);

        // Heterogeneous Activation Mapping
        neuron_activations[i] = runtime.activations[i].apply(neuron_states[i]);
    }

    // 3. Hebbian learning update with Active Lifetime Pruning!
    if hebbian_rate > 0.0 {
        // Permanently pruned synapses are not in the active list
        let mut pruned = false;
        for syn in &runtime.active_synapses {
            let syn_idx = syn.idx;
            let act_pre = neuron_activations[syn.from];
            let act_post = neuron_activations[syn.to];

            // Hebbian Correlation delta
            let correlation = act_pre * act_post;

            // Weight update: dw = learning_rate * correlation - forgetting * weight
            let delta = hebbian_rate * correlation;
            let decay = forgetting_decay * synapse_weights[syn_idx];

            synapse_weights[syn_idx] += delta - decay;

            // Active Pruning: if weight drops below threshold, lock permanently at 0.0
            if synapse_weights[syn_idx].abs() < 0.015 {
                synapse_weights[syn_idx] = 0.0;
                pruned = true;
            } else {
                // Clamp active weights to avoid divergent explosions
                synapse_weights[syn_idx] = synapse_weights[syn_idx].clamp(-4.0, 4.0);
            }
        }
        if pruned {
            runtime.active_synapses.retain(|s| synapse_weights[s.idx] != 0.0);
        }
    }

    // 4. Motor outputs
    runtime.motors.map(|m| neuron_activations[m])
}

/// Computes unified, rich multi-channel sensory inputs (Multispectral vision, olfactory chemoreception, haptics, proprioception)
pub fn compute_sensory_inputs(
    agent: &CreatureAgent,
    clock_val: f32,
    foods: &[FoodSpore],
    obstacles: &[ProceduralObstacle],
    canvas_width: f32,
    canvas_height: f32,
) -> Vec<f32> {
    let mut inputs = Vec::new();
    compute_sensory_inputs_into(agent, clock_val, foods, obstacles, canvas_width, canvas_height, &mut inputs);
    inputs
}

/// `compute_sensory_inputs` into a reused buffer (overwritten and resized to the input count).
pub fn compute_sensory_inputs_into(
    agent: &CreatureAgent,
    clock_val: f32,
    foods: &[FoodSpore],
    obstacles: &[ProceduralObstacle],
    canvas_width: f32,
    canvas_height: f32,
    inputs: &mut Vec<f32>,
) {
    let k = agent.phenotype.organelles.len();
    inputs.clear();
    inputs.resize(k * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT, 0.0);

    for (idx, patch) in agent.phenotype.organelles.iter().enumerate() {
        let range = patch.scale * 550.0;
        let alpha = patch.angle * (std::f32::consts::PI / 180.0);
        let half_cone = (patch.bandwidth * 1.5).max(0.1);
        let organ_power = patch.scale * (1.1 - patch.bandwidth);

        // Stimuli per input slot; the ΔI slot stays 0 here (computed in the brain step from consecutive ticks)
        let mut channel_stimuli = [0.0f32; CHANNELS_PER_ORGANELLE];

        if patch.kind.is_distance() {
            // --- LIGHT AND CHEMICAL SENSORY DETECTION (EYE / NOSE / THERMAL) ---
            // Plant spore foods[0] and meat spore foods[1], each with its emission spectrum
            for (food, emission) in foods.iter().zip([PLANT_EMISSION, MEAT_EMISSION]) {
                let dx = food.x - agent.px;
                let dy = food.y - agent.py;
                let dist = (dx*dx + dy*dy).sqrt();
                if dist > range {
                    continue;
                }
                let mut angle_rel = fmath::atan2(dy, dx) - agent.heading_angle;
                while angle_rel > std::f32::consts::PI { angle_rel -= std::f32::consts::TAU; }
                while angle_rel < -std::f32::consts::PI { angle_rel += std::f32::consts::TAU; }
                let mut delta_beta = angle_rel - alpha;
                while delta_beta > std::f32::consts::PI { delta_beta -= std::f32::consts::TAU; }
                while delta_beta < -std::f32::consts::PI { delta_beta += std::f32::consts::TAU; }

                if delta_beta.abs() <= half_cone {
                    let falloff = (1.0 - dist / range) * fmath::cos(delta_beta);
                    for c in 0..SPECTRAL_CHANNELS {
                        let match_val = channel_sensitivity(patch.spectral_affinity, patch.bandwidth, c) * emission[c];
                        if match_val > MIN_CHANNEL_MATCH {
                            channel_stimuli[c] = channel_stimuli[c].max(match_val * organ_power * falloff);
                        }
                    }
                }
            }
        } else {
            // --- TACTILE AND PROPRIOCEPTIVE SENSORY DETECTION (TACTILE / HAPTIC) ---
            // A. Mechanical Hardness & Texture (Channel 1, Center 0.10)
            // Wall warning is hard (1.0). Spores are soft (0.3).
            let wall_warning_zone = range * 0.5;
            let mut boundary_pressure = 0.0;
            if agent.px < wall_warning_zone {
                boundary_pressure = 1.0 - agent.px / wall_warning_zone;
            } else if agent.px > canvas_width - wall_warning_zone {
                boundary_pressure = 1.0 - (canvas_width - agent.px) / wall_warning_zone;
            }
            if agent.py < wall_warning_zone {
                boundary_pressure = boundary_pressure.max(1.0 - agent.py / wall_warning_zone);
            } else if agent.py > canvas_height - wall_warning_zone {
                boundary_pressure = boundary_pressure.max(1.0 - (canvas_height - agent.py) / wall_warning_zone);
            }

            let mut tactile_pressure: f32 = 0.0;
            if boundary_pressure > 0.0 {
                // Wall hardness (1.0)
                tactile_pressure = tactile_pressure.max(boundary_pressure * organ_power * 1.0);
            }

            // Spore tactile proximity (spores foods[0] and foods[1] emit soft tactile feedback of 0.3)
            for f in foods {
                let dx = f.x - agent.px;
                let dy = f.y - agent.py;
                let d = (dx*dx + dy*dy).sqrt();
                if d <= range {
                    let proximity = (1.0 - d / range).clamp(0.0, 1.0);
                    // Soft food texture (0.3)
                    tactile_pressure = tactile_pressure.max(proximity * organ_power * 0.3);
                }
            }

            // Obstacle tactile proximity (Treat reefs as circular walls with hardness 1.0)
            for obs in obstacles {
                let dx = obs.x - agent.px;
                let dy = obs.y - agent.py;
                let dist = (dx*dx + dy*dy).sqrt();
                let hard_radius = obs.radius + range;
                if dist <= hard_radius {
                    let pressure = (1.0 - (dist - obs.radius).max(0.0) / range).clamp(0.0, 1.0);
                    // Reef hardness (1.0)
                    tactile_pressure = tactile_pressure.max(pressure * organ_power * 1.0);
                }
            }

            // C. Water Temperature
            let dist_from_center = ((agent.px - canvas_width / 2.0).powi(2) + (agent.py - canvas_height / 2.0).powi(2)).sqrt();
            let temp_gradient = (1.0 - dist_from_center / (canvas_width * 0.7)).clamp(0.15, 0.95) * organ_power;

            // D. Tactile subtype by expression style (Phase 4 of TCK-133, `OrganKind`)
            let speed = (agent.vx * agent.vx + agent.vy * agent.vy).sqrt();

            if patch.kind == OrganKind::Bristle {
                // 1. Taktile Borsten (Reine Oberflächen-Härte & lokaler Schmerz)
                channel_stimuli[0] = tactile_pressure;
                channel_stimuli[1] = 0.0;
                channel_stimuli[2] = 0.0;
                channel_stimuli[3] = 0.0;
                if speed > 1.5 && boundary_pressure > 0.3 {
                    channel_stimuli[4] = (speed * 0.3).min(1.0);
                } else {
                    channel_stimuli[4] = 0.0;
                }
            } else if patch.kind == OrganKind::LateralLine {
                // 2. Seitenlinien-Poren (Vektorielle Strömung & lokaler Schmerz - Rheotaxis/Gyrodämpfung)
                let organ_heading = agent.heading_angle + patch.angle * (std::f32::consts::PI / 180.0);
                let flow_x = -agent.vx;
                let flow_y = -agent.vy;
                let flow_proj = flow_x * fmath::cos(organ_heading) + flow_y * fmath::sin(organ_heading);

                let organ_rad = patch.angle * (std::f32::consts::PI / 180.0);
                let rot_flow = -agent.omega_rot * 12.0 * fmath::sin(organ_rad) * 0.4;

                let local_drag = (flow_proj + rot_flow).clamp(-1.0, 1.0);

                channel_stimuli[0] = 0.0;
                channel_stimuli[1] = local_drag;
                channel_stimuli[2] = 0.0;
                channel_stimuli[3] = 0.0;
                if speed > 1.5 && boundary_pressure > 0.3 {
                    channel_stimuli[4] = (speed * 0.3).min(1.0);
                } else {
                    channel_stimuli[4] = 0.0;
                }
            } else {
                // 3. Thermo-Propriozeptoren (Wirbelsäulen-Biegung & Temperatur)
                channel_stimuli[0] = 0.0;
                channel_stimuli[1] = 0.0;
                channel_stimuli[2] = temp_gradient;
                // Normalisierte interne Biegung [-1.0 .. 1.0] (max_flexion = 1.2)
                let flexion = (agent.bend_angle / 1.2).clamp(-1.0, 1.0);
                channel_stimuli[3] = flexion;
                channel_stimuli[4] = 0.0;
            }
        }

        // Write the compiled channel signals into their respective input neurons!
        for c in 0..CHANNELS_PER_ORGANELLE {
            inputs[idx * CHANNELS_PER_ORGANELLE + c] = channel_stimuli[c].clamp(0.0, 1.0);
        }
    }

    // Systemic Base Inputs (Feste propriozeptive Kern-Sensorik)
    let speed = (agent.vx.powi(2) + agent.vy.powi(2)).sqrt();
    let linear_speed = speed * 0.4;
    let rotational_speed = agent.omega_rot.abs() * 0.8;
    let internal_energy = agent.energy / 100.0;
    let pain_signal = if agent.omega_rot.abs() > 2.0 || speed > 4.5 {
        (speed * 0.3).min(1.0)
    } else {
        0.0
    };

    // Calculate Search Arousal based on cumulative exteroceptive stimuli!
    let mut total_exteroceptive = 0.0;
    for s in 0..(k * CHANNELS_PER_ORGANELLE) {
        total_exteroceptive += inputs[s].abs();
    }
    // Arousal is 1.0 if the creature registers no external exteroceptive stimulus at all, allowing it to trigger blind search behaviors
    let search_arousal = if total_exteroceptive < 0.05 { 1.0 } else { 0.0 };

    let base_inputs_start = k * CHANNELS_PER_ORGANELLE;
    for (slot, input) in SystemicInput::ALL.into_iter().enumerate() {
        inputs[base_inputs_start + slot] = match input {
            SystemicInput::Clock => clock_val,
            SystemicInput::LinearSpeed => linear_speed,
            SystemicInput::RotationalSpeed => rotational_speed,
            SystemicInput::Energy => internal_energy,
            SystemicInput::Pain => pain_signal,
            SystemicInput::SearchArousal => search_arousal,
        };
    }
}

/// Legacy wrapper for execute_brain without live learning
pub fn execute_brain(
    brain: &BrainTopology,
    inputs: &[f32],
    neuron_states: &mut Vec<f32>,
    neuron_activations: &mut Vec<f32>,
) -> [f32; OUTPUT_MOTOR_NODES_COUNT] {
    let mut temp_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
    execute_brain_with_learning(brain, inputs, neuron_states, neuron_activations, &mut temp_weights, &mut BrainRuntime::default(), 0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::dna::parse_genome;

    #[test]
    fn test_execute_brain_determinism() {
        let d_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";
        let phenotype = parse_genome(d_dna, None, None);
        let brain = phenotype.brain;

        let inputs = vec![1.0, 0.5, -0.2, 0.0]; // shorter than the input layer: the rest stays 0
        let mut states = Vec::new();
        let mut activations = Vec::new();

        // Run integration for multiple steps
        for _ in 0..10 {
            let outputs = execute_brain(&brain, &inputs, &mut states, &mut activations);
            assert_eq!(outputs.len(), OUTPUT_MOTOR_NODES_COUNT);
            for out in &outputs {
                assert!(out.is_finite());
                assert!(*out >= -1.0 && *out <= 4.0); // Allowed up to 4.0 for ReLU activations
            }
        }

        // Assert that activations correspond to nodes
        assert_eq!(activations.len(), brain.neurons.len());
        assert_eq!(states.len(), brain.neurons.len());
    }

    #[test]
    fn test_compute_sensory_inputs_correctness() {
        let d_dna = "EYEABCDEFEN"; // Has at least 1 organelle (EYE)
        let phenotype = parse_genome(d_dna, None, None);
        let num_neurons = phenotype.brain.neurons.len();

        let agent = CreatureAgent {
            id: 1,
            species_id: "test".to_string(),
            px: 500.0,
            py: 500.0,
            vx: 1.0,
            vy: 0.0,
            heading_angle: 0.0,
            bend_angle: 0.0,
            omega_rot: 0.0,
            energy: 100.0,
            age: 0,
            generation: 1,
            has_eaten: false,
            genome: d_dna.to_string(),
            antisense: String::new(),
            phenotype: phenotype.clone(),
            neuron_states: vec![0.0; num_neurons],
            neuron_activations: vec![0.0; num_neurons],
            synapse_weights: phenotype.brain.synapses.iter().map(|s| s.weight).collect(),
            brain_runtime: BrainRuntime::default(),
        };

        let foods = vec![
            FoodSpore { id: 1000, type_id: 1, x: 550.0, y: 500.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // near plant Spore
            FoodSpore { id: 9999, type_id: 2, x: -99999.0, y: -99999.0, amount: 0.0, vx: 0.0, vy: 0.0 }, // far meat Spore
        ];

        let inputs = compute_sensory_inputs(&agent, 0.5, &foods, &[], 1000.0, 1000.0);
        
        // Input size: CHANNELS_PER_ORGANELLE slots per organ + systemic inputs (clock first)
        let k = phenotype.organelles.len();
        assert_eq!(inputs.len(), k * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT);
        assert_eq!(inputs[k * CHANNELS_PER_ORGANELLE], 0.5);
        // The ΔI slot is filled by the brain step, not by sensing
        assert_eq!(inputs[DELTA_CHANNEL], 0.0);

        // Every value must be finite and within [0, 1]
        for val in &inputs {
            assert!(val.is_finite());
            assert!(*val >= 0.0 && *val <= 1.0);
        }
    }

    /// Eye with live IR (channel 5) and red/green channels: payload index 1 = T gives affinity 0.80.
    const EYE_DNA: &str = "EYEATAAAAAAAAAAAAAAAEN";

    /// Runs the brain with the IR channel of the single organ set to `ir[t]` at tick t; returns the last activations and outputs.
    fn run_ir(ir: &[f32]) -> (Vec<f32>, [f32; OUTPUT_MOTOR_NODES_COUNT]) {
        let phenotype = parse_genome(EYE_DNA, None, None);
        let mut inputs = vec![0.0; CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT];
        let (mut states, mut activations) = (Vec::new(), Vec::new());
        let mut weights = phenotype.brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
        let mut runtime = BrainRuntime::default();
        let mut outputs = [0.0; OUTPUT_MOTOR_NODES_COUNT];
        for &v in ir {
            inputs[4] = v;
            outputs = execute_brain_with_learning(&phenotype.brain, &inputs, &mut states, &mut activations, &mut weights, &mut runtime, 0.0, 0.0, 0.0);
        }
        (activations, outputs)
    }

    #[test]
    fn test_delta_channel_and_one_tick_latency() {
        assert_eq!(parse_genome(EYE_DNA, None, None).organelles[0].kind, OrganKind::Eye);

        // ΔI has its own slot (signed); the IR channel keeps its value (TCK-144)
        let (rising, out_stimulus) = run_ir(&[0.0, 0.2]);
        assert_eq!(rising[4], 0.2);
        assert!((rising[DELTA_CHANNEL] - 0.2 * DELTA_GAIN).abs() < 1e-6);
        let (falling, _) = run_ir(&[0.2, 0.0]);
        assert!((falling[DELTA_CHANNEL] + 0.2 * DELTA_GAIN).abs() < 1e-6);

        // Inputs → Thalamus → Motors in one pass: a stimulus moves the motors in the same tick (TCK-153)
        let (_, out_none) = run_ir(&[0.0, 0.0]);
        assert_ne!(out_stimulus, out_none);
    }

    #[test]
    fn test_exponential_integration_is_bounded() {
        for step in 0..=24 {
            let tau = TAU_MIN + (TAU_MAX - TAU_MIN) * step as f32 / 24.0;
            let alpha = integration_alpha(tau);
            assert!(alpha > 0.0 && alpha < 1.0, "tau {}", tau);
            let mut y = 0.0f32;
            for tick in 0..200 {
                let target = if (tick / 7) % 2 == 0 { 4.0 } else { -4.0 };
                let next = y + alpha * (target - y);
                // Moves towards the target without overshooting it, so no tick-to-tick oscillation for any tau
                assert!((next - y) * (target - y) >= 0.0 && (next - target).abs() <= (y - target).abs(), "tau {}", tau);
                y = next;
                assert!(y.abs() <= 4.0);
            }
        }
    }
}
