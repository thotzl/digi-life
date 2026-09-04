use crate::biology::dna::BrainTopology;
use crate::shared::types::{CreatureAgent, FoodSpore};
use crate::shared::map_generator::ProceduralObstacle;

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

/// Computes unified, rich 5-channel sensory inputs (Multispectral vision, olfactory chemoreception, haptics, proprioception)
pub fn compute_sensory_inputs(
    agent: &CreatureAgent,
    clock_val: f32,
    foods: &[FoodSpore],
    obstacles: &[ProceduralObstacle],
    canvas_width: f32,
    canvas_height: f32,
) -> Vec<f32> {
    let k = agent.phenotype.organelles.len();
    let mut inputs = vec![0.0; k * 5 + 1];
    inputs[k * 5] = clock_val;

    // Fixed center frequencies for the 5 receptive channels (Cones)
    let channel_frequencies = [0.10, 0.30, 0.50, 0.70, 0.90];

    for (idx, patch) in agent.phenotype.organelles.iter().enumerate() {
        let range = patch.scale * 550.0;
        let alpha = (patch.angle - 90.0) * (std::f32::consts::PI / 180.0);
        let half_cone = (patch.bandwidth * 1.5).max(0.1);
        let aff = patch.spectral_affinity;
        let organ_power = patch.scale * (1.1 - patch.bandwidth);

        // Calculate channel sensitivities (S_c) based on Gaussian curves for each of the 5 channels
        let mut channel_sensitivities = [0.0f32; 5];
        for c in 0..5 {
            let f_c = channel_frequencies[c];
            let denominator = patch.bandwidth * 1.8 + 0.12;
            channel_sensitivities[c] = (1.0 - (f_c - aff).abs() / denominator).clamp(0.0, 1.0);
        }

        // Initialize stimuli array for the 5 channels
        let mut channel_stimuli = [0.0f32; 5];

        if aff >= 0.25 {
            // --- LIGHT AND CHEMICAL SENSORY DETECTION (EYE / NOSE) ---
            // A. Plant Spore / Algae Scan (foods[0] - emits on channels 2 [Blue, weight 1.0] and 3 [Green, weight 0.5])
            if !foods.is_empty() {
                let pellet = &foods[0];
                let dx = pellet.x - agent.px;
                let dy = pellet.y - agent.py;
                let dist = (dx*dx + dy*dy).sqrt();

                if dist <= range {
                    let mut angle_rel = dy.atan2(dx) - agent.heading_angle;
                    while angle_rel > std::f32::consts::PI { angle_rel -= std::f32::consts::TAU; }
                    while angle_rel < -std::f32::consts::PI { angle_rel += std::f32::consts::TAU; }
                    let mut delta_beta = angle_rel - alpha;
                    while delta_beta > std::f32::consts::PI { delta_beta -= std::f32::consts::TAU; }
                    while delta_beta < -std::f32::consts::PI { delta_beta += std::f32::consts::TAU; }

                    if delta_beta.abs() <= half_cone {
                        let falloff = (1.0 - dist / range) * delta_beta.cos();
                        
                        // Algae emits: 1.0 on Channel 2 (Blue) and 0.5 on Channel 3 (Green)
                        let emission = [0.0, 1.0, 0.5, 0.0, 0.0];
                        for c in 0..5 {
                            let match_val = channel_sensitivities[c] * emission[c];
                            if match_val > 0.05 {
                                channel_stimuli[c] = channel_stimuli[c].max(match_val * organ_power * falloff);
                            }
                        }
                    }
                }
            }

            // B. Meat Spore / Prey Scan (foods[1] - emits on channels 5 [Infrared, weight 1.0] and 4 [Red, weight 0.5])
            if foods.len() > 1 {
                let other = &foods[1];
                let dx_peer = other.x - agent.px;
                let dy_peer = other.y - agent.py;
                let dist_peer = (dx_peer*dx_peer + dy_peer*dy_peer).sqrt();

                if dist_peer <= range {
                    let mut angle_rel = dy_peer.atan2(dx_peer) - agent.heading_angle;
                    while angle_rel > std::f32::consts::PI { angle_rel -= std::f32::consts::TAU; }
                    while angle_rel < -std::f32::consts::PI { angle_rel += std::f32::consts::TAU; }
                    let mut delta_beta = angle_rel - alpha;
                    while delta_beta > std::f32::consts::PI { delta_beta -= std::f32::consts::TAU; }
                    while delta_beta < -std::f32::consts::PI { delta_beta += std::f32::consts::TAU; }

                    if delta_beta.abs() <= half_cone {
                        let falloff_peer = (1.0 - dist_peer / range) * delta_beta.cos();
                        
                        // Meat emits: 1.0 on Channel 5 (IR) and 0.5 on Channel 4 (Red)
                        let emission_peer = [0.0, 0.0, 0.0, 0.5, 1.0];
                        for c in 0..5 {
                            let match_val = channel_sensitivities[c] * emission_peer[c];
                            if match_val > 0.05 {
                                channel_stimuli[c] = channel_stimuli[c].max(match_val * organ_power * falloff_peer);
                            }
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

            if boundary_pressure > 0.0 {
                // Channel 1 receives wall hardness (1.0)
                channel_stimuli[0] = channel_stimuli[0].max(boundary_pressure * organ_power * 1.0);
            }

            // Spore tactile proximity (spores foods[0] and foods[1] emit soft tactile feedback of 0.3)
            for f in foods {
                let dx = f.x - agent.px;
                let dy = f.y - agent.py;
                let d = (dx*dx + dy*dy).sqrt();
                if d <= range {
                    let proximity = (1.0 - d / range).clamp(0.0, 1.0);
                    // Channel 1 receives soft food texture (0.3)
                    channel_stimuli[0] = channel_stimuli[0].max(proximity * organ_power * 0.3);
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
                    // Channel 1 receives reef hardness (1.0)
                    channel_stimuli[0] = channel_stimuli[0].max(pressure * organ_power * 1.0);
                }
            }

            // B. Fluid Drag & Flow (Channel 2, Center 0.30)
            let speed = (agent.vx * agent.vx + agent.vy * agent.vy).sqrt();
            let flow_reception = (speed * 0.4).min(1.0);
            channel_stimuli[1] = channel_stimuli[1].max(flow_reception * organ_power);

            // C. Water Temperature (Channel 3, Center 0.50)
            // Mock temperature gradient: center is warm (0.5), outer bounds are colder
            let dist_from_center = ((agent.px - canvas_width / 2.0).powi(2) + (agent.py - canvas_height / 2.0).powi(2)).sqrt();
            let temp = (1.0 - dist_from_center / (canvas_width * 0.7)).clamp(0.15, 0.95);
            channel_stimuli[2] = channel_stimuli[2].max(temp * organ_power);

            // D. Proprioceptive Strain & Rotation (Channel 4, Center 0.70)
            let rot_speed = agent.omega_rot.abs();
            let strain = (rot_speed * 0.8).min(1.0);
            channel_stimuli[3] = channel_stimuli[3].max(strain * organ_power);

            // E. Physical Pain / Impact Damage (Channel 5, Center 0.90)
            // High-speed wall impact pain triggers a response!
            if speed > 1.5 && boundary_pressure > 0.3 {
                let pain = (speed * 0.3).min(1.0);
                channel_stimuli[4] = channel_stimuli[4].max(pain * organ_power);
            }
        }

        // Write the 5 compiled channel signals into their respective input neurons!
        for c in 0..5 {
            inputs[idx * 5 + c] = channel_stimuli[c].clamp(0.0, 1.0);
        }
    }

    inputs
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
            adrenaline: 1.0,
            age: 0,
            generation: 1,
            has_eaten: false,
            genome: d_dna.to_string(),
            antisense: String::new(),
            phenotype: phenotype.clone(),
            neuron_states: vec![0.0; num_neurons],
            neuron_activations: vec![0.0; num_neurons],
            synapse_weights: phenotype.brain.synapses.iter().map(|s| s.weight).collect(),
        };

        let foods = vec![
            FoodSpore { id: 1000, type_id: 1, x: 550.0, y: 500.0, amount: 15.0, vx: 0.0, vy: 0.0 }, // near plant Spore
            FoodSpore { id: 9999, type_id: 2, x: -99999.0, y: -99999.0, amount: 0.0, vx: 0.0, vy: 0.0 }, // far meat Spore
        ];

        let inputs = compute_sensory_inputs(&agent, 0.5, &foods, &[], 1000.0, 1000.0);
        
        // Input size must be exactly (organelles.len() * 5 + 1)
        let k = phenotype.organelles.len();
        assert_eq!(inputs.len(), k * 5 + 1);
        
        // Last element is the clock
        assert_eq!(inputs[k * 5], 0.5);

        // Every value must be finite and within [0, 1]
        for val in &inputs {
            assert!(val.is_finite());
            assert!(*val >= 0.0 && *val <= 1.0);
        }
    }
}
