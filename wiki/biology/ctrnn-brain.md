---
sources: [README.md, .agents/skills/biology-dna/SKILL.md, .agents/skills/testing-strategies/SKILL.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-121-synaptic-exuberance-and-active-lifetime-pruning.md, tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, docs/revisit/04-simulation-model.md]
---

# CTRNN Brain

`src/shared/brain.rs` (execution) and `src/biology/dna.rs` (topology compiler).

## Execution

- `execute_brain_with_learning` integrates node states with Euler steps: `τ_i dy_i/dt = −y_i + Σ w_ji σ(y_j + θ_j) + I_i`.
- `compute_sensory_inputs` builds the input vector from organelles, environment and internal state (shared by Ocean and Trainer since TCK-129), see [[wiki/biology/sensing]].
- **Activations per neuron:** `tanh` (default), `relu`, `sigmoid`, `sin` (evolvable since TCK-106).
- **Time constants:** `tau = (0.2 + hash * 1.8).clamp(0.1, 2.5)`.
- Output ranges: `tanh`/`sin` `[-1, 1]`, `sigmoid` `[0, 1]`, `relu` `[0, ∞)`.
- **Evaluation order:** neuron IDs are `[Inputs][Outputs][Thalamus][Hidden]` and the Euler pass runs ascending, so outputs read thalamus/hidden activations from the previous tick: sensor → motor latency is 2 ticks (RV-S06).

## Topology

- **Neuron index layout** depends on organelle count — see `get_input_neurons_count`, `get_motor_thrust_id`, `get_motor_bending_id`, `get_hidden_neurons_start_id` in `dna.rs`.
- **Outputs:** motor 0 thrust (forward/backward), motor 1 bending (left/right), motor 3 biolum, 4 reserved (both without effect, RV-S01).
- **Thalamus** layer between sensors and hidden nodes; hardwired reflex synapses (±1.0); reflex pairs precomputed in a `HashSet` (TCK-140).
- **NEU genes** define hidden neurons (τ, bias, activation, depth `0.15..0.85`); fallback 2 hidden neurons. **SY genes** define strong synapses (`from`/`to` modulo-mapped over absolute indices, weight `[-2, 2]`) (TCK-119).
- **Synaptic exuberance:** at birth fully connected sources (inputs + hidden) × destinations (outputs + hidden), no self-loops; non-`SY` synapses start weak in `[-0.075, 0.075]` (TCK-121). Synapse count grows with `(7 + 10·organs + H) × (4 + H)`.
- **Symmetric sensor fusion (HOX):** for mirrored organ pairs, sum interneurons (`L + R`) feed thrust, difference interneurons (`L − R`) feed bending — symmetry by architecture, no synapse mirroring (TCK-133).

## Learning

- Hebbian update per step: `ΔW = η · a_pre · a_post − decay · W`, clamp `±4.0` (`hebbianLearning*`, `hebbianForgettingDecay` in config) (TCK-119).
- **Active pruning:** `|w| < 0.015` → set to exactly 0 and frozen for the lifetime (TCK-121).
- Learned weights stored as `synapse_weights`; Lamarckian inheritance see [[wiki/biology/metabolism-and-reproduction]].
- The rule has no normalization (Oja/BCM), so weights drift toward the clamp over long lives; reflex synapses are plastic and prunable as well; the `_hebbian_decay` parameter is ignored (RV-S04).

## Key Takeaways

- Genome-defined CTRNN with mixed activations, τ in `[0.1, 2.5]`, thalamus and HOX-gated fusion.
- Born fully connected, pruned and Hebbian-trained during life.
- Unnormalized learning and the 2-tick sensor-motor latency are known model properties (RV-S04, RV-S06).

Sources: `README.md`, `.agents/skills/biology-dna/SKILL.md`, TCK-106, TCK-119, TCK-121, TCK-133, `docs/revisit/04-simulation-model.md`.
