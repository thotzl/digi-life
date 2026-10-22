---
sources: [README.md, .agents/skills/biology-dna/SKILL.md, .agents/skills/testing-strategies/SKILL.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-119-dynamic-ctrnn-neuro-generation-and-epigenetic-learning.md, tickets/closed/TCK-121-synaptic-exuberance-and-active-lifetime-pruning.md, tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, docs/revisit/04-simulation-model.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-144-motion-vision-channel-design.md, tickets/closed/TCK-153-brain-evaluation-order.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md]
---

# CTRNN Brain

`src/shared/brain.rs` (execution) and `compile_brain` in `src/biology/dna.rs` (topology). Model since Brain v2 (TCK-171 P3, [[wiki/decisions/adr-002-brain-v2]]).

## Neuron roles and order (TCK-179, TCK-153)

Every `CTRNNNeuron` carries a `role` (`NeuronRole`, ts-rs); the vector order is the evaluation order:

| Block | Role | Count | Parameters |
| :--- | :--- | :--- | :--- |
| 1. Sensors | `Sensor{gene, side, channel, kind}` | 6 per organ (5 spectral + ΔI), organ order of `phenotype.organelles` | input, no integration |
| 2. Systemic | `Systemic{input}`: `Clock`, `LinearSpeed`, `RotationalSpeed`, `Energy`, `Pain`, `SearchArousal` | 6 | input |
| 3. Thalamus | `Thalamus{gene, channel, node Sum\|Diff}` | per organ gene and live channel one `Sum` node, plus one `Diff` node for `HOX` pairs | τ = 0.1 (≈ pass-through), bias 0; `Sum` sigmoid, `Diff` tanh |
| 4. Hidden | `Hidden{gene}` | one per `NEU` gene (first 12), sorted by depth | τ `0.2 + 1.8h`, bias `[-1, 1]`, activation `tanh`/`relu`/`sigmoid`/`sin`, depth `0.55–0.85`, all from the `NEU` payload |
| 5. Motors | `Motor{Thrust\|Bend}` | 2 | bias/τ from the first `OUT` gene; without one bias 0, τ 1.1 |

- No `NEU` gene → no hidden neuron (the former fallback of 2 is gone). Motors 3 (biolum) and 4 (reserved) and the adrenaline input are removed (TCK-149).
- Labels are display text only; the model and the frontend use roles ([[wiki/frontend/structure]]).

## Wiring (TCK-182, TCK-179)

Only live structure is wired; every synapse has a stable `key` and exactly one `owner` gene.

| Synapse | Weight | Owner |
| :--- | :--- | :--- |
| Sensor → thalamus `Sum` (L and R of a pair: +1 each) / `Diff` (L +1, R −1) | fixed reflex (`plastic: false`) | organ gene |
| Thalamus → each motor | tape | organ gene |
| Thalamus → hidden, systemic → hidden | tape | `NEU` gene of the hidden neuron |
| Hidden → each motor | tape | `NEU` gene |
| Systemic → each motor | tape | first `OUT` gene (not wired without one) |
| Any (source systemic/thalamus/hidden, target hidden/motor) | `SY` payload, `[-2, 2]` | `SY` gene (overrides or adds) |

- **Live channels** (`live_channels`): a distance sense gets a thalamus node only for spectral channels whose sensitivity × max emission exceeds 0.05, plus ΔI if any is live; tactile kinds have 2 fixed live channels each ([[wiki/biology/sensing]]). `Diff` nodes exist only for `HOX` pairs; a pair shares one weight set through its `Sum`/`Diff` nodes (bilateral weight sharing).
- No default hidden → hidden and no sensor → motor bypass; such synapses arise only from `SY` genes.
- **Synapse key:** 53-bit FNV-1a over the canonical encodings of source and target role (gene identity `GeneRef`, side, channel, node, input, motor). Independent of neuron indices and genome positions: adding or removing a gene changes no key of a synapse that does not touch it (test `test_adding_an_organ_gene_keeps_unrelated_synapses`).
- **Weight tape:** a tape weight is one letter of the owner gene's payload mapped `A..Z → [-0.5, 0.5]`; the slot is `hash(roles, owner identity left out) mod payload length`. An empty payload wires nothing. Slots of one owner can collide (same initial weight).
- **`SY` genes** (at most 30): payload loci choose a source class (systemic/thalamus/hidden), an index modulo the class size, a target class (hidden/motor) and index; the first `SY` gene per pair wins. Class sizes change with organs and `NEU` genes, so `SY` addressing is not fully stable (TCK-168).
- **`plastic` flag** (TCK-152): set by the compiler on every `CTRNNSynapse`. Sensor → thalamus reflexes (±1) are `plastic: false` — they never learn and are never pruned, so the sum/difference semantics of the thalamus hold for the whole life. Every other synapse is plastic. Test `test_reflexes_are_fixed_and_the_rest_plastic`. The inspector tooltip marks a fixed synapse "fixed reflex (does not learn)" (`describeSynapse`, `frontend/shared/genome.ts`).
- Synapses are sorted by target for the single linear pass.
- **Size** (TCK-172 topology probe, 1,000 genomes): synapses p10/p50/p90 52/142/249 before P3 → 14/41/103 on letter-soup genomes and 48/78/148 on structured genomes (mean 150 → 52 / 88); wildtype 249 → 118 synapses, 72 → 67 neurons.

## Execution

- `execute_brain_with_learning` integrates neurons in index order in one pass per tick: inputs → thalamus → hidden (lower depth first) → motors. A synapse from a lower index reads this tick's activation, from a higher index the previous tick's. Sensor → motor latency is 1 tick (test `test_delta_channel_and_one_tick_latency`); hidden → hidden (only via `SY`) is same-tick from lower to higher depth.
- **Integration** (TCK-153): exact exponential step `y += α · (Σ w·a + θ − y)` with `α = 1 − exp(−1/τ)` (`dt = 1`, `integration_alpha`), state clamped to ±4, then the activation. `α ∈ (0, 1)` for every τ in `[0.1, 2.5]`, so the state never overshoots its target; the forward-Euler oscillation for `τ < 0.5` is gone (test `test_exponential_integration_is_bounded`).
- Output ranges: `tanh`/`sin` `[-1, 1]`, `sigmoid` `[0, 1]`, `relu` `[0, 4]` (state clamp). Motor outputs are `[thrust, bend]`.
- The ΔI input of distance senses is computed in the brain step from consecutive ticks ([[wiki/biology/sensing]]).
- **`BrainRuntime`** (TCK-173): per-agent cache on `CreatureAgent.brain_runtime` (not serialized), built lazily on the first brain step: activation enum and `α` per neuron, input count, the ΔI organs (from roles, no label matching), motor indices and the list of active (non-zero) synapses. The signal pass iterates only active synapses, the learning pass only the active plastic ones (`plastic_synapses`); a synapse pruned during the step is removed from both lists. Rebuilt when the synapse-weight vector is re-initialized; code that replaces an agent's phenotype or weights in place resets it (`BrainRuntime::default()`).
- Transcendental functions (`tanh`, `exp`, `sin`) go through `src/shared/fmath.rs` (pure-Rust `libm`), identical on every platform (TCK-173).

## Learning (TCK-152)

- **Oja rule** per tick for every active plastic synapse: `Δw = η · post · (pre − post · w)`, `η = hebbianLearningRateBase` (0.00015; 0 disables learning). The `−η · post² · w` term is activity-dependent forgetting that keeps weights bounded; there is no separate decay key. `±4` (`WEIGHT_CLAMP`) stays as a safety bound only.
- **Active pruning** (TCK-121): a plastic weight with `|w| < 0.015` (`PRUNE_THRESHOLD`) is set to exactly 0 and frozen for the lifetime. Fixed reflexes are never pruned.
- Learned weights are stored as `synapse_weights` (synapse order of the phenotype); Lamarckian inheritance passes them per synapse key with a probability, plastic synapses only, see [[wiki/biology/metabolism-and-reproduction]].
- Test `test_oja_learning_stays_bounded_and_reflexes_stay_fixed`: 24 structured genomes, 36,000 ticks of slowly varying random inputs; all weights finite and within ±4, reflexes unchanged, under 5 % of plastic weights at `|w| ≥ 3.9`, mean `|w|` < 2.

### Measured (P4, `probe hebbian`, learning on, best 14–15 elites of a seeded trainer run)

| | Before P4 (unnormalized Hebbian + forgetting) | Oja + fixed reflexes |
| :--- | :--- | :--- |
| Long life 36,000 ticks: mean \|w\| (plastic) | 1.318 | 0.342 |
| Long life: share at the clamp (\|w\| ≥ 3.9) | 10.3 % | 0.22 % |
| Long life: share pruned | 39.7 % | 51.4 % |
| Long life: reflex drift (mean \|learned − genome\|) | 0.097 | 0 |
| 900-tick trial (standard room): mean \|w\| / pruned | 0.270 / 5.3 % | 0.258 / 8.7 % |

- **Benefit of lifetime learning:** not measurable at the current success level. Paired fitness delta learning on − off (median): standard −1.4, exploration −8.4; the means are dominated by single genomes. Success is 0–2.4 % in all conditions.
- Rejected variants (5-seed runs, see [[wiki/simulation/trainer]]): no pruning (lower success), pruning only with active presynaptic input (no measurable change).

## Measured behaviour (TCK-172 steering probe)

Food fixed at 300 px and ±45°, real brain with learning, heading change after 100 ticks; 500 genomes per set. "Bearing reduced" = the minimum bearing to the food fell below 35° for both food sides.

| | Before P3 (letter soup) | P3–P4, letter soup | P3–P4, structured | After P5, letter soup | After P5, structured |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Turns toward / away from food | 3.2 % / 5.0 % | 3.8 % / 4.4 % | 9.0 % / 8.2 % | 7.0 % / 6.2 % | 16.0 % / 14.6 % |
| Toward food on both sides | 0.4 % | 0.6 % | 0.2 % | 0.4 % | 1.0 % |
| Bearing reduced on both sides | – | 23.6 % | 27.4 % | 56.4 % | 64.0 % |
| Mean thrust negative | 48 % | 52 % | 49 % | 52 % | 49 % |

- Steering is still undirected at random initialisation (toward ≈ away); the ΔI channel and latency 1 make directed turning learnable but not innate. The P5 rise in turning and bearing reduction comes from the mechanics (turning at rest and faster turning of slow bodies, TCK-150, [[wiki/simulation/physics]]); the brain model is unchanged in P5.
- Heredity of weights under mutation: [[wiki/biology/genome-compiler]].

## Key Takeaways

- Role-based CTRNN: sensors (6 per organ incl. ΔI) → 6 systemic inputs → thalamus → hidden → thrust/bend, evaluated in one pass with latency 1 and a stable exponential step.
- Targeted wiring of live structure only; every synapse has a role key and one owner gene whose payload tape gives its weight.
- Plastic synapses learn with Oja's rule (bounded, no drift to the clamp); sensor → thalamus reflexes are fixed (TCK-152). Lifetime learning has no measurable benefit yet; random brains do not steer toward food.

Sources: `README.md`, `.agents/skills/biology-dna/SKILL.md`, TCK-106, TCK-119, TCK-121, TCK-133, TCK-144, TCK-149, TCK-150, TCK-152, TCK-153, TCK-172, TCK-173, TCK-179, TCK-182, `docs/revisit/04-simulation-model.md`.
