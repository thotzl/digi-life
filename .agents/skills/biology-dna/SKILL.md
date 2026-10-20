---
name: biology-dna
description: Decodes the biological mechanics, genome parsing, Continuous-Time Recurrent Neural Networks (CTRNN) brain, and phenotype properties of creatures in Pixel DNA Life.
---
# biology-dna: Biology, Genetics & CTRNN Brain Simulation

## I. Genome Compiler (`src/biology/dna.rs`)
- **Sense strand:** uppercase `A-Z` string. **Antisense:** `get_complementary_string`; passed to `parse_genome` as optional second strand.
- **Entry point:** `parse_genome(genome, antisense_input, parent_methylations) -> CreaturePhenotype` — pure and deterministic. Exposed to the frontend via the `get_fossil_phenotype` command.
- **Promoter/terminator genes:** `extract_raw_gene_payloads` scans for promoter motifs (with degenerate biochemical-class matching, `matches_degenerate`) and terminators; payloads are hashed into normalized values (`get_payload_linear_value_offset`). Organelles (eyes, noses, tactile, photophores) and `NEU`/`SYN` brain genes are compiled this way.
- **Chromatin:** `chromatin_state` marks positions covered by recognized promoter..terminator blocks (`COL`, `STF`, `PUL`, `SIZ`, `WAV`, `EYE`, `NOS`, `TAC`, `LUM`, `NEU`, ...; terminators `EN`/`SP`). `active_dna` is the concatenation of those positions and feeds brain-parameter hashing. Trait payloads (`COL`, `SIZ`, `STF`, ...) are extracted from the full cleaned genome.
- **Epigenetics:** parent methylations are an input to `parse_genome` and are persisted with genomes (`methylations` columns).
- **Mutation:** `mutate_genome` (point mutations); in the Ocean, mitosis mutates the child genome with 35% probability (`engine.rs`).

## II. CTRNN Brain (`src/shared/brain.rs`)
- `execute_brain_with_learning` integrates node states with Euler steps; `compute_sensory_inputs` builds the input vector from organelles, environment and internal state. A Thalamus layer sits between sensors and hidden nodes.
- **Activations are mixed per neuron:** `tanh` (default), `relu`, `sigmoid`, `sin`.
- **Time constants:** `tau = (0.2 + hash * 1.8).clamp(0.1, 2.5)`.
- **Neuron index layout** depends on organelle count: see `get_input_neurons_count`, `get_motor_thrust_id`, `get_motor_bending_id`, `get_hidden_neurons_start_id` in `dna.rs`.
- **Learning:** Hebbian weight updates with forgetting (`hebbianLearning*` / `hebbianForgettingDecay` in `config.json`); learned weights are stored as `synapse_weights`.

## III. Evolutionary Dynamics
- **Species identity:** `species_id` is the genome string itself; any mutation creates a new species record (`species_records`, keyed by id, with Latin name, parent name, carnivory). There is no distance-based speciation (RV-B10, design decision pending).
- **Mitosis condition (Ocean):** `age >= phenotype.mature_age && has_eaten && energy >= stomach_capacity * repro_threshold`; parent energy is multiplied by `reproductionSplitLossRatio`.
- **Known defect (RV-B01, deferred):** mitosis children are only pushed to `newly_spawned_creatures` and never enter the simulated creature list. The Ocean therefore has no surviving offspring; population is maintained by restocking. Do not draw evolutionary conclusions from Ocean generation counters until this is fixed.
