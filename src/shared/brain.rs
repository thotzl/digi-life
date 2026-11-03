use crate::biology::brain::{
    BrainTopology, Motor, MotorOutputs, NeuronRole, OUTPUT_MOTOR_NODES_COUNT, SYSTEMIC_BASE_INPUTS_COUNT,
    SystemicInput, TAU_MAX, TAU_MIN,
};
use crate::biology::organs::{CHANNELS_PER_ORGANELLE, DELTA_CHANNEL, SPECTRAL_CHANNELS, SenseCtx};
use serde::{Deserialize, Serialize};

use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::shared::fmath;
use crate::shared::map_generator::ProceduralObstacle;

pub use crate::biology::brain::Activation;
pub use crate::biology::organs::SENSOR_RANGE_PER_SCALE;

/// Config section `brain`: brain compilation (`phenotype::parse_genome`) and learning (`execute_brain_with_learning`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrainConfig {
    /// Activation of the thalamus sum nodes: "sigmoid" or "tanh" (TCK-190)
    pub thalamus_sum_activation: ThalamusSumActivation,
    /// η of the Oja rule of plastic synapses (TCK-152)
    pub hebbian_learning_rate_base: f32,
}

/// Activation of the thalamus sum nodes (TCK-190). Sensor inputs are non-negative, so `Sigmoid` turns a silent
/// channel into a constant 0.5 drive; `Tanh` keeps a silent channel at 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThalamusSumActivation {
    Sigmoid,
    #[default]
    Tanh,
}

impl ThalamusSumActivation {
    /// Activation of the compiled sum neurons
    pub fn activation(self) -> Activation {
        match self {
            ThalamusSumActivation::Sigmoid => Activation::Sigmoid,
            ThalamusSumActivation::Tanh => Activation::Tanh,
        }
    }
}

/// Gain of the ΔI channel: input = clamp(gain · (I_now − I_prev), −1, 1) (TCK-133, signed since TCK-144).
const DELTA_GAIN: f32 = 2.5;

/// Safety bound of learned weights (Oja keeps them well inside it, TCK-152).
pub const WEIGHT_CLAMP: f32 = 4.0;
/// A plastic weight learned below this magnitude is pruned: set to exactly 0 for the rest of the life (TCK-121).
pub const PRUNE_THRESHOLD: f32 = 0.015;

/// Largest range (px) at which the phenotype's distance organs (eye, nose, thermal: the organs that see food in
/// `compute_sensory_inputs`) can detect food; 0 without such an organ. Ignores the view cone.
pub fn food_sense_range(phenotype: &crate::biology::phenotype::CreaturePhenotype) -> f32 {
    phenotype
        .organelles
        .iter()
        .filter(|o| o.kind.is_distance())
        .map(|o| o.scale * SENSOR_RANGE_PER_SCALE)
        .fold(0.0, f32::max)
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
    /// Indices into `active_synapses` of the plastic ones (the only ones that learn and can be pruned)
    plastic_synapses: Vec<usize>,
    /// Synaptic activity of the last brain step: `Σ |w · a_source|` over the active synapses (metabolic cost, TCK-151)
    activity: f32,
}

#[derive(Debug, Clone, Copy)]
struct ActiveSynapse {
    idx: usize,
    from: usize,
    to: usize,
}

impl BrainRuntime {
    fn build(&mut self, brain: &BrainTopology, synapse_weights: &[f32]) {
        // A neuron without an activation (inputs, motors) integrates with tanh
        self.activations = brain
            .neurons
            .iter()
            .map(|n| n.activation_type.unwrap_or(Activation::Tanh))
            .collect();
        self.alpha = brain.neurons.iter().map(|n| integration_alpha(n.tau)).collect();
        self.input_count = brain
            .neurons
            .iter()
            .take_while(|n| matches!(n.role, NeuronRole::Sensor { .. } | NeuronRole::Systemic { .. }))
            .count();
        self.delta_organs = brain
            .neurons
            .iter()
            .filter(|n| matches!(n.role, NeuronRole::Sensor { channel: DELTA_CHANNEL, kind, .. } if kind.is_distance()))
            .map(|n| n.id - DELTA_CHANNEL)
            .collect();
        for (slot, motor) in Motor::ALL.into_iter().enumerate() {
            self.motors[slot] = brain
                .neurons
                .iter()
                .position(|n| n.role == NeuronRole::Motor { motor })
                .unwrap_or(0);
        }
        self.active_synapses = brain
            .synapses
            .iter()
            .enumerate()
            .filter(|(idx, _)| synapse_weights[*idx] != 0.0)
            .map(|(idx, s)| ActiveSynapse {
                idx,
                from: s.from_node,
                to: s.to_node,
            })
            .collect();
        self.index_plastic(brain);
        self.built = true;
    }

    /// Synaptic activity `Σ |w · a_source|` of the last brain step (0 before the first step); accumulated in the
    /// signal pass, so pruned and zero weights cost nothing (TCK-151).
    pub fn activity(&self) -> f32 {
        self.activity
    }

    fn index_plastic(&mut self, brain: &BrainTopology) {
        self.plastic_synapses = self
            .active_synapses
            .iter()
            .enumerate()
            .filter(|(_, s)| brain.synapses[s.idx].plastic)
            .map(|(i, _)| i)
            .collect();
    }
}

/// One CTRNN tick with lifetime learning. Neurons are integrated in index order, which `parse_genome` makes the
/// evaluation order Inputs → Thalamus → Hidden (by depth) → Motors: a synapse from a lower index reads this tick's
/// activation, one from a higher index the previous tick's. Returns the motor outputs.
///
/// Learning (TCK-152): every plastic synapse follows Oja's rule `Δw = η · post · (pre − post · w)` with
/// `η = hebbian_rate` (0 disables learning). The `−η · post² · w` term is activity-dependent forgetting that keeps the
/// weights bounded (no drift to the clamp); `±WEIGHT_CLAMP` stays as a safety bound. A plastic weight that falls
/// below `PRUNE_THRESHOLD` is pruned for the rest of the life. Fixed synapses (sensor → thalamus reflexes) never change.
pub fn execute_brain_with_learning(
    brain: &BrainTopology,
    inputs: &[f32],
    neuron_states: &mut Vec<f32>,
    neuron_activations: &mut Vec<f32>,
    synapse_weights: &mut Vec<f32>,
    runtime: &mut BrainRuntime,
    hebbian_rate: f32,
) -> MotorOutputs {
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
    let mut activity = 0.0;
    for i in input_count..total_nodes {
        // Sum synaptic weights from all active connections in a single linear O(S) pass!
        let mut sum = 0.0;
        while syn_idx < active.len() && active[syn_idx].to == i {
            let syn = active[syn_idx];
            let signal = neuron_activations[syn.from] * synapse_weights[syn.idx];
            sum += signal;
            activity += signal.abs();
            syn_idx += 1;
        }

        let target = sum + brain.neurons[i].bias;
        neuron_states[i] += runtime.alpha[i] * (target - neuron_states[i]);

        // Clamping to prevent numerical divergent explosion (NaN prevention)
        neuron_states[i] = neuron_states[i].clamp(-4.0, 4.0);

        // Heterogeneous Activation Mapping
        neuron_activations[i] = runtime.activations[i].apply(neuron_states[i]);
    }

    runtime.activity = activity;

    // 3. Oja learning of the plastic synapses, with active lifetime pruning (pruned synapses are no longer active)
    if hebbian_rate > 0.0 {
        let mut pruned = false;
        for &a in &runtime.plastic_synapses {
            let syn = runtime.active_synapses[a];
            let (pre, post) = (neuron_activations[syn.from], neuron_activations[syn.to]);
            let w = &mut synapse_weights[syn.idx];
            *w += hebbian_rate * post * (pre - post * *w);
            if w.abs() < PRUNE_THRESHOLD {
                *w = 0.0;
                pruned = true;
            } else {
                *w = w.clamp(-WEIGHT_CLAMP, WEIGHT_CLAMP);
            }
        }
        if pruned {
            runtime.active_synapses.retain(|s| synapse_weights[s.idx] != 0.0);
            runtime.index_plastic(brain);
        }
    }

    // 4. Motor outputs
    MotorOutputs::from_fn(|motor| neuron_activations[runtime.motors[motor as usize]])
}

/// Input layer of one tick: per organ `CHANNELS_PER_ORGANELLE` slots (organ order, written by the organ kind's
/// `OrganSpec::sense`, clamped to [0, 1]), then the systemic inputs in `SystemicInput::ALL` order.
pub fn compute_sensory_inputs(
    agent: &CreatureAgent,
    clock_val: f32,
    foods: &[FoodSpore],
    obstacles: &[ProceduralObstacle],
    canvas_width: f32,
    canvas_height: f32,
) -> Vec<f32> {
    let mut inputs = Vec::new();
    compute_sensory_inputs_into(
        agent,
        clock_val,
        foods,
        obstacles,
        canvas_width,
        canvas_height,
        &mut inputs,
    );
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

    let ctx = SenseCtx {
        agent,
        foods,
        obstacles,
        width: canvas_width,
        height: canvas_height,
    };
    for (idx, organ) in agent.phenotype.organelles.iter().enumerate() {
        // Stimuli per input slot; the ΔI slot stays 0 here (computed in the brain step from consecutive ticks)
        let mut channel_stimuli = [0.0f32; CHANNELS_PER_ORGANELLE];
        (organ.kind.spec().sense)(&ctx, organ, &mut channel_stimuli);
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
) -> MotorOutputs {
    let mut temp_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
    execute_brain_with_learning(
        brain,
        inputs,
        neuron_states,
        neuron_activations,
        &mut temp_weights,
        &mut BrainRuntime::default(),
        0.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::organs::OrganKind;
    use crate::biology::phenotype::parse_genome;

    #[test]
    fn test_execute_brain_determinism() {
        let d_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";
        let phenotype = parse_genome(d_dna, None, None, &crate::config::test_config().brain);
        let brain = phenotype.brain;

        let inputs = vec![1.0, 0.5, -0.2, 0.0]; // shorter than the input layer: the rest stays 0
        let mut states = Vec::new();
        let mut activations = Vec::new();

        // Run integration for multiple steps
        for _ in 0..10 {
            let outputs = execute_brain(&brain, &inputs, &mut states, &mut activations);
            for out in outputs.values() {
                assert!(out.is_finite());
                assert!(out >= -1.0 && out <= 4.0); // Allowed up to 4.0 for ReLU activations
            }
        }

        // Assert that activations correspond to nodes
        assert_eq!(activations.len(), brain.neurons.len());
        assert_eq!(states.len(), brain.neurons.len());
    }

    #[test]
    fn test_compute_sensory_inputs_correctness() {
        let d_dna = "EYEABCDEFEN"; // Has at least 1 organelle (EYE)
        let phenotype = parse_genome(d_dna, None, None, &crate::config::test_config().brain);
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
            FoodSpore {
                id: 1000,
                type_id: 1,
                x: 550.0,
                y: 500.0,
                amount: 15.0,
                vx: 0.0,
                vy: 0.0,
            }, // near plant Spore
            FoodSpore {
                id: 9999,
                type_id: 2,
                x: -99999.0,
                y: -99999.0,
                amount: 0.0,
                vx: 0.0,
                vy: 0.0,
            }, // far meat Spore
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
    fn run_ir(ir: &[f32]) -> (Vec<f32>, MotorOutputs) {
        let phenotype = parse_genome(EYE_DNA, None, None, &crate::config::test_config().brain);
        let mut inputs = vec![0.0; CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT];
        let (mut states, mut activations) = (Vec::new(), Vec::new());
        let mut weights = phenotype.brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
        let mut runtime = BrainRuntime::default();
        let mut outputs = MotorOutputs::default();
        for &v in ir {
            inputs[4] = v;
            outputs = execute_brain_with_learning(
                &phenotype.brain,
                &inputs,
                &mut states,
                &mut activations,
                &mut weights,
                &mut runtime,
                0.0,
            );
        }
        (activations, outputs)
    }

    #[test]
    fn test_delta_channel_and_one_tick_latency() {
        assert_eq!(
            parse_genome(EYE_DNA, None, None, &crate::config::test_config().brain).organelles[0].kind,
            OrganKind::Eye
        );

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

    /// Runs `ticks` brain steps with Oja learning (η from config) on irregular inputs in [0, 1] (sensor slots per
    /// organ + systemic inputs, the ΔI slots are filled by the brain); returns the learned weights.
    fn learn_with_noise(
        genome: &str,
        ticks: u32,
        seed: u64,
    ) -> (crate::biology::phenotype::CreaturePhenotype, Vec<f32>) {
        use rand::Rng;
        let phenotype = parse_genome(genome, None, None, &crate::config::test_config().brain);
        let input_count = phenotype.organelles.len() * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT;
        let mut rng = crate::shared::rng::seeded(seed);
        let (mut states, mut activations) = (Vec::new(), Vec::new());
        let mut weights = phenotype.brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
        let mut runtime = BrainRuntime::default();
        let rate = crate::config::test_config().brain.hebbian_learning_rate_base;
        // Slowly varying random inputs (a new random level every 20 ticks), so pre and post correlate over time
        let mut inputs = vec![0.0; input_count];
        for tick in 0..ticks {
            if tick % 20 == 0 {
                inputs.iter_mut().for_each(|v| *v = rng.gen_range(0.0..1.0));
            }
            execute_brain_with_learning(
                &phenotype.brain,
                &inputs,
                &mut states,
                &mut activations,
                &mut weights,
                &mut runtime,
                rate,
            );
        }
        (phenotype, weights)
    }

    #[test]
    fn test_oja_learning_stays_bounded_and_reflexes_stay_fixed() {
        let composition = &crate::config::test_config().genome.genome_composition;
        let (mut plastic, mut at_clamp, mut changed, mut abs_sum) = (0usize, 0usize, 0usize, 0.0f32);
        for seed in 0..24 {
            let genome =
                crate::biology::genome::generate_structured_genome(composition, &mut crate::shared::rng::seeded(seed));
            // A long Ocean life (36,000 ticks)
            let (phenotype, weights) = learn_with_noise(&genome, 36_000, seed);
            for (s, &w) in phenotype.brain.synapses.iter().zip(&weights) {
                assert!(w.is_finite() && w.abs() <= WEIGHT_CLAMP);
                if s.plastic {
                    plastic += 1;
                    at_clamp += (w.abs() >= 3.9) as usize;
                    changed += (w != s.weight) as usize;
                    abs_sum += w.abs();
                } else {
                    // Sensor → thalamus reflexes: never learned, never pruned (TCK-152)
                    assert_eq!(w, s.weight, "reflex {:?} changed", s);
                }
            }
        }
        assert!(plastic > 500, "{} plastic synapses", plastic);
        assert!(
            changed * 2 > plastic,
            "learning active: {} of {} plastic weights changed",
            changed,
            plastic
        );
        let (clamp_share, mean_abs) = (at_clamp as f32 / plastic as f32, abs_sum / plastic as f32);
        println!(
            "plastic {} mean |w| {:.3} share at clamp {:.4}",
            plastic, mean_abs, clamp_share
        );
        assert!(clamp_share < 0.05, "{:.3} of plastic weights at the clamp", clamp_share);
        assert!(mean_abs < 2.0, "mean |w| {}", mean_abs);
    }

    #[test]
    fn test_reflexes_are_fixed_and_the_rest_plastic() {
        let composition = &crate::config::test_config().genome.genome_composition;
        for seed in 0..20 {
            let genome =
                crate::biology::genome::generate_structured_genome(composition, &mut crate::shared::rng::seeded(seed));
            let brain = parse_genome(&genome, None, None, &crate::config::test_config().brain).brain;
            for s in &brain.synapses {
                let reflex = matches!(
                    (&brain.neurons[s.from_node].role, &brain.neurons[s.to_node].role),
                    (NeuronRole::Sensor { .. }, NeuronRole::Thalamus { .. })
                );
                assert_eq!(s.plastic, !reflex, "{:?}", s);
                if reflex {
                    assert_eq!(s.weight.abs(), 1.0);
                }
            }
        }
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
                assert!(
                    (next - y) * (target - y) >= 0.0 && (next - target).abs() <= (y - target).abs(),
                    "tau {}",
                    tau
                );
                y = next;
                assert!(y.abs() <= 4.0);
            }
        }
    }
}
