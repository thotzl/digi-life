---
sources: [README.md, .agents/skills/biology-dna/SKILL.md, .agents/skills/testing-strategies/SKILL.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-121-synaptic-exuberance-and-active-lifetime-pruning.md, tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, docs/revisit/04-simulation-model.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md]
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
- **Euler step size:** `dt = 1`, state update `y += (1/τ)(−y + Σ + θ)`, clamp `±4`. For `τ < 0.5` (`dt/τ > 2`) forward Euler is unstable: such output neurons flip every tick between about ±0.999; 11 of 500 random genomes have a mean thrust of exactly 0 for this reason (TCK-172 steering probe).
- **`BrainRuntime`** (TCK-173): per-agent cache on `CreatureAgent.brain_runtime` (not serialized), built lazily on the first brain step — activation enum per neuron, motion-sensor flag per organ channel, and the list of active (non-zero) synapses in synapse order. The signal pass and the Hebbian pass iterate only active synapses; a synapse pruned during the step is removed from the list. Rebuilt when the synapse-weight vector is re-initialized; code that replaces an agent's phenotype or weights in place resets it (`BrainRuntime::default()`). Results are bit-identical to the uncached step (golden master unchanged).
- Transcendental functions in the brain step (`tanh`, `exp`, `sin`) go through `src/shared/fmath.rs` (pure-Rust `libm`), identical on every platform (TCK-173).

## Topology

- **Neuron index layout** depends on organelle count — see `get_input_neurons_count`, `get_motor_thrust_id`, `get_motor_bending_id`, `get_hidden_neurons_start_id` in `dna.rs`.
- **Outputs:** motor 0 thrust (forward/backward), motor 1 bending (left/right), motor 3 biolum, 4 reserved (both without effect; `bioluminescenceFlashCost` removed, TCK-149).
- **Thalamus** layer between sensors and hidden nodes; hardwired reflex synapses (±1.0); reflex pairs precomputed in a `HashSet` (TCK-140).
- **NEU genes** define hidden neurons (τ, bias, activation, depth `0.15..0.85`); fallback 2 hidden neurons. **SY genes** define strong synapses (`from`/`to` modulo-mapped over absolute indices, weight `[-2, 2]`) (TCK-119).
- **Synaptic exuberance:** at birth fully connected sources (7 base inputs + thalamus + hidden) × destinations (4 motors + hidden), no self-loops (TCK-121). Synapse count ≈ `(7 + 10·thalamus blocks + H) · (4 + H)`, H ≤ 12, organ count uncapped: ~120 (1 block, H 2), ~250 (2 / 4), ~1,000 (4 / 12), ~1,700 (8 / 12). Non-`SY` weights start in `[-0.5, 0.5]` (`dna.rs:1327`).
- **Weight derivation:** a non-`SY` weight is read from `active_dna` at index `(from + to·13) % len` via `get_payload_linear_value_offset` (`dna.rs:1323-1325`). Weights therefore depend on absolute genome positions and absolute neuron ids: an insertion/deletion inside an active gene, a created/destroyed gene (changes `len`) or a changed organ count (shifts all neuron ids) re-maps most weights. Explicit `SY` synapses are mapped by modulo over the current source/destination lists (`dna.rs:1251-1258`) and shift the same way.
- **Silent wiring:** unpaired organs feed only the sum nodes; their 5 difference nodes stay 0 but are fully wired (`dna.rs:1365-1374`). Motors 3 and 4 have no effect but receive all sources. Thalamus nodes of channels an organ is not sensitive to stay constant but are wired as well.
- **Symmetric sensor fusion (HOX):** for mirrored organ pairs, sum interneurons (`L + R`) feed thrust, difference interneurons (`L − R`) feed bending — symmetry by architecture, no synapse mirroring (TCK-133).

## Learning

- Hebbian update per step: `ΔW = η · a_pre · a_post − decay · W`, clamp `±4.0` (`hebbianLearning*`, `hebbianForgettingDecay` in config) (TCK-119).
- **Active pruning:** `|w| < 0.015` → set to exactly 0 and frozen for the lifetime (TCK-121).
- Learned weights stored as `synapse_weights`; Lamarckian inheritance see [[wiki/biology/metabolism-and-reproduction]].
- The rule has no normalization (Oja/BCM), so weights drift toward the clamp over long lives; reflex synapses are plastic and prunable as well; the `_hebbian_decay` parameter is ignored (RV-S04).
- Measured effect in the trainer (TCK-172 Hebbian probe, 32 HoF genomes, 3 trials, learning on vs. off with the same RNG streams): inconclusive — success 2–3 % in all conditions; learning helped in 59 % of genomes in the small room and hurt in 75 % in the large room.

## Measured behaviour (TCK-172)

Probes in `src/biology/probes.rs` (`trainer_runner probe steering`), seed `tck172`, 500 random and 34 HoF genomes:
- **No directed steering:** food fixed at 300 px and ±45°, real brain with learning, heading change after 100 ticks: random brains turn toward the food 3.2 % and away 5.0 % (noise level); toward on both sides 0.4 % of random and 0 of 34 HoF genomes.
- **Swimming backwards:** mean thrust output is negative for 48 % of random and 71 % of HoF brains; initial weights in `[-0.5, 0.5]` and `tanh` give a thrust near 0 on average.
- Heredity of weights under mutation: [[wiki/biology/genome-compiler]].

## Key Takeaways

- Genome-defined CTRNN with mixed activations, τ in `[0.1, 2.5]`, thalamus and HOX-gated fusion.
- Born fully connected, pruned and Hebbian-trained during life.
- Unnormalized learning and the 2-tick sensor-motor latency are known model properties (RV-S04, RV-S06); `τ < 0.5` outputs oscillate under the `dt = 1` Euler step.
- Measured: random and trained brains show no directed turning toward food (TCK-172).

Sources: `README.md`, `.agents/skills/biology-dna/SKILL.md`, TCK-106, TCK-119, TCK-121, TCK-133, TCK-172, TCK-173, `docs/revisit/04-simulation-model.md`.
