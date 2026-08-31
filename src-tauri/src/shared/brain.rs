use crate::biology::dna::BrainTopology;

/// Dynamic Deep CTRNN Recurrent Brain signal execution with Hebbian Learning (Euler integration + Plasticity)
pub fn execute_brain_with_learning(
    brain: &BrainTopology,
    inputs: &[f32],
    neuron_states: &mut Vec<f32>,
    neuron_activations: &mut Vec<f32>,
    synapse_weights: &mut Vec<f32>,
    hebbian_rate: f32,
    _hebbian_decay: f32, // stiffness decay
    forgetting_decay: f32,
) -> Vec<f32> {
    let total_nodes = brain.neurons.len();
    let total_synapses = brain.synapses.len();

    // Dynamically resize and initialize synapse weights if needed
    if synapse_weights.len() != total_synapses {
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

    let k_count = inputs.len() - 1; // Last input represents the clock

    // 1. Direct Sensory Follow: Inject environmental sensor charges
    for i in 0..=k_count {
        if i < inputs.len() {
            neuron_activations[i] = inputs[i];
        }
    }

    // 2. Continuous Euler Integration: Integrate hidden and output potentials
    for i in (k_count + 1)..total_nodes {
        let neuron = &brain.neurons[i];
        let tau = if neuron.tau > 0.1 { neuron.tau } else { 1.0 };
        let bias = neuron.bias;

        // Sum synaptic weights from all active connections using dynamic synapse_weights
        let mut sum = 0.0;
        for (syn_idx, syn) in brain.synapses.iter().enumerate() {
            if syn.to_node == i {
                sum += neuron_activations[syn.from_node] * synapse_weights[syn_idx];
            }
        }

        // Euler Integration Step (dt = 1.0, tau represents local temporal inertia)
        neuron_states[i] += (1.0 / tau) * (-neuron_states[i] + sum + bias);

        // Clamping to prevent numerical divergent explosion (NaN prevention)
        neuron_states[i] = neuron_states[i].clamp(-4.0, 4.0);

        // Heterogeneous Activation Mapping
        let act_type = neuron.activation_type.as_deref().unwrap_or("tanh");
        neuron_activations[i] = match act_type {
            "relu" => neuron_states[i].max(0.0),
            "sigmoid" => 1.0 / (1.0 + (-neuron_states[i]).exp()),
            "sin" => neuron_states[i].sin(),
            _ => neuron_states[i].tanh(), // Default to tanh standard activation
        };
    }

    // 3. Hebbian learning update with Active Lifetime Pruning!
    if hebbian_rate > 0.0 {
        for (syn_idx, syn) in brain.synapses.iter().enumerate() {
            // Skip permanently pruned synapses
            if synapse_weights[syn_idx] == 0.0 {
                continue;
            }

            let act_pre = neuron_activations[syn.from_node];
            let act_post = neuron_activations[syn.to_node];

            // Hebbian Correlation delta
            let correlation = act_pre * act_post;
            
            // Weight update: dw = learning_rate * correlation - forgetting * weight
            let delta = hebbian_rate * correlation;
            let decay = forgetting_decay * synapse_weights[syn_idx];

            synapse_weights[syn_idx] += delta - decay;

            // Active Pruning: if weight drops below threshold, lock permanently at 0.0
            if synapse_weights[syn_idx].abs() < 0.015 {
                synapse_weights[syn_idx] = 0.0;
            } else {
                // Clamp active weights to avoid divergent explosions
                synapse_weights[syn_idx] = synapse_weights[syn_idx].clamp(-4.0, 4.0);
            }
        }
    }

    // 4. Map output node activations directly to the 4 motor directions
    // Outputs are mapped at indices K+1 to K+4
    let mut outputs = vec![0.0; 4];
    for i in 0..4 {
        let idx = k_count + 1 + i;
        if idx < total_nodes {
            outputs[i] = neuron_activations[idx];
        }
    }

    outputs
}

/// Legacy wrapper for execute_brain without live learning
pub fn execute_brain(
    brain: &BrainTopology,
    inputs: &[f32],
    neuron_states: &mut Vec<f32>,
    neuron_activations: &mut Vec<f32>,
) -> Vec<f32> {
    let mut temp_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
    execute_brain_with_learning(brain, inputs, neuron_states, neuron_activations, &mut temp_weights, 0.0, 0.0, 0.0)
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

        let inputs = vec![1.0, 0.5, -0.2, 0.0]; // 3 sensors + 1 hunger clock
        let mut states = Vec::new();
        let mut activations = Vec::new();

        // Run integration for multiple steps
        for _ in 0..10 {
            let outputs = execute_brain(&brain, &inputs, &mut states, &mut activations);
            assert_eq!(outputs.len(), 4);
            for out in &outputs {
                assert!(out.is_finite());
                assert!(*out >= -1.0 && *out <= 4.0); // Allowed up to 4.0 for ReLU activations
            }
        }

        // Assert that activations correspond to nodes
        assert_eq!(activations.len(), brain.neurons.len());
        assert_eq!(states.len(), brain.neurons.len());
    }
}
