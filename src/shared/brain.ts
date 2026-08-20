/**
 * Dynamic Deep CTRNN Recurrent Brain signal execution (Euler temporal memory integration)
 */
export function executeBrain(
  brain: any,
  inputs: number[],
  neuronStates: number[],
  neuronActivations: number[]
): { outputs: number[], allLayerActivations: number[][] } {
  const totalNodes = brain.neurons.length;

  // Initialize or resize temporal membrane states in-memory instantly!
  if (neuronStates.length !== totalNodes) {
    neuronStates.length = totalNodes;
    neuronStates.fill(0.0);
  }
  if (neuronActivations.length !== totalNodes) {
    neuronActivations.length = totalNodes;
    neuronActivations.fill(0.0);
  }

  const K = inputs.length - 1; // last input is the clock

  // 1. Direct Sensory Follow: Assign sensory inputs directly to input nodes
  for (let i = 0; i <= K; i++) {
    neuronActivations[i] = inputs[i];
  }

  // 2. Continuous Euler Integration: Update hidden and output neurons
  for (let i = K + 1; i < totalNodes; i++) {
    const neuron = brain.neurons[i];
    const tau = neuron.tau || 1.0;
    const bias = neuron.bias || 0.0;

    // Accumulate inputs from all incoming temporal synapses
    let sum = 0.0;
    brain.synapses.forEach((syn: any) => {
      if (syn.toNode === i) {
        sum += neuronActivations[syn.fromNode] * syn.weight;
      }
    });

    // Euler integration step (dt = 1.0, tau_i is the genetically encoded decay time)
    neuronStates[i] += (1.0 / tau) * (-neuronStates[i] + sum + bias);

    // Bounded potential clamping to prevent numeric drift explosions
    neuronStates[i] = Math.max(-4.0, Math.min(4.0, neuronStates[i]));

    // Heterogeneous Activation Function (adaptive normalisation)
    const actType = neuron.activationType || "tanh";
    if (actType === "relu") {
      neuronActivations[i] = Math.max(0.0, neuronStates[i]);
    } else if (actType === "sigmoid") {
      neuronActivations[i] = 1.0 / (1.0 + Math.exp(-neuronStates[i]));
    } else if (actType === "sin") {
      neuronActivations[i] = Math.sin(neuronStates[i]);
    } else {
      neuronActivations[i] = Math.tanh(neuronStates[i]);
    }
  }

  // 3. Map output node activations directly to the 4 motor directions
  // Output nodes are compiled at indices K+1 to K+4
  const outputs = [
    neuronActivations[K + 1],
    neuronActivations[K + 2],
    neuronActivations[K + 3],
    neuronActivations[K + 4]
  ];

  // Return the activations as a flat array for fine-grained direct visualization
  return { outputs, allLayerActivations: [neuronActivations] };
}
