---
sources: [tickets/ongoing/TCK-171-training-track-epic.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-144-motion-vision-channel-design.md, tickets/closed/TCK-153-brain-evaluation-order.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-187-gene-spans-from-backend.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-188-targets-adr-and-analytics-skill.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, src/biology/dna.rs, src/shared/brain.rs, src/database.rs, src/biology/creature_factory.rs, tickets/closed/TCK-151-bmr-brain-cost-model.md]
---

# ADR-002: Brain v2 — Stable Synapse Identity, Compact Topology, Genome Operators

- Status: accepted
- Date: 2026-10-22
- Tickets: TCK-179, TCK-182, TCK-144, TCK-153, TCK-184, TCK-187, TCK-183 (reset rule), TCK-149 (motors 3/4, adrenaline input), TCK-188; epic TCK-171 phase P3

## Context

The TCK-172 probes (P2, seed `tck172`) located the main obstacles to cumulative training in the genome compiler and the brain, not in the trainer:

- **Heredity:** parent–child correlation of non-`SY` weights after 1 / 5 / 15 trainer mutations was 0.943 / 0.661 / 0.318 (200 random parents). Weights were read from `active_dna` at `(from + to·13) % len`; neuron ids depended on the organ count; `SY` genes mapped modulo over absolute ids; body traits without their own gene fell back to `active_dna`. One indel or one organ more re-mapped most weights; after 15 ops 17 % of random-parent children changed their terminal speed by ≥ 2×.
- **Topology:** every source (7 base inputs, 10 thalamus nodes per organ block, hidden) was wired to every destination (4 motors, hidden): 52 / 142 / 249 synapses (p10/p50/p90, random 256-base genomes). Motors 3 (biolum) and 4 (reserved), difference nodes of unpaired organs and thalamus nodes of insensitive channels were wired but could never carry a signal.
- **Steering:** no directed turning (3.2 % of random brains toward food vs. 5.0 % away). Sensor → motor latency was 2 ticks (outputs evaluated before thalamus/hidden, RV-S06); forward Euler with `dt = 1` oscillated for `τ < 0.5` (11 of 500 genomes with a mean thrust of exactly 0). Motion vision overwrote the meat channel 5 of eyes/noses (RV-B06); organ types were detected by emoji labels (RV-S10).
- **Operators:** generation 1 and immigrants were letter soup (21 % without an organ, 67 % with a food-sensing organ, RV-S09); a point mutation drew a random letter; no operator duplicated or removed a gene.
- Literature (TCK-171 research summary, [[wiki/research/neuroevolution-and-training-literature]]): efficient evolutionary search needs high locality (Rothlauf); NEAT keeps a stable identity per connection (innovation numbers); single-sensor steering works by temporal comparison (klinotaxis, C. elegans).

## Decision

One rebuild of `parse_genome` and the brain step with one golden-master change and one DB reset. Details of the resulting model: [[wiki/biology/genome-compiler]], [[wiki/biology/ctrnn-brain]], [[wiki/biology/sensing]].

1. **Neurons have roles, not positions** (TCK-179). `NeuronRole`: `Systemic{input}`, `Sensor{gene, side, channel, kind}`, `Thalamus{gene, channel, node Sum|Diff}`, `Hidden{gene}`, `Motor{Thrust|Bend}`. The neuron vector order is the evaluation order: sensors (6 slots per organ) → systemic inputs (6) → thalamus → hidden (by depth) → motors.
2. **Gene identity** `GeneRef{promoter, spelling, occurrence}`: the promoter type, its exact (degenerate) spelling in the genome and the occurrence among genes with that spelling. Letter-level mutations never touch promoter letters, and the structured generator draws distinct spellings per gene, so a gene keeps its identity when other genes appear, vanish or move.
3. **Stable synapse key and one owner gene.** `key` = 53-bit FNV-1a over the canonical encodings of source and target role (exact as a JS number); `owner` = index into `phenotype.genes`. Owner rule: organ gene for sensor → thalamus reflexes (fixed ±1) and thalamus → motor; `NEU` gene for thalamus → hidden, systemic → hidden and hidden → motor; the first `OUT` gene (brain base) for systemic → motor and the motor bias/τ; an `SY` gene takes over the synapse it addresses (first `SY` gene per pair wins) or adds it.
4. **Weight tape.** A synapse's initial weight is one letter of its owner gene's payload, `A..Z → [-0.5, 0.5]`; the slot is `hash(source role, target role) mod payload length`, with the owner's own gene identity left out of the hash. A point mutation in a payload changes the weights read from that letter only; nothing outside the owner gene affects them.
5. **Trait locality** (TCK-179). Every body trait is read from the first gene of its own promoter; a missing gene reads the empty payload (every value at `h = 0.5`). `active_dna` is removed.
6. **Compaction instead of exuberance** (TCK-182). Only live structure is wired: thalamus nodes only for channels the organ can sense, difference nodes only for `HOX` pairs, left/right of a pair share one weight set through their sum/difference nodes, no default hidden → hidden, no synapses without an owner payload. Motors 3/4 removed (TCK-149).
7. **ΔI channel and organ enum** (TCK-144). Distance senses (`OrganKind` `Eye`, `Nose`, `Thermal`) get a 6th slot with the signed temporal intensity change; the meat channel keeps its value. `OrganKind` (`Eye`, `Nose`, `Thermal`, `Bristle`, `LateralLine`, `Proprioceptor`) is decided once at compile time; no emoji label matching remains in the model.
8. **Evaluation order and integration** (TCK-153). One ascending pass: inputs → thalamus → hidden (lower depth first) → motors, sensor → motor latency 1 tick. Exact exponential step `y += (1 − exp(−1/τ)) · (Σ + θ − y)`, clamp ±4, τ range `[0.1, 2.5]` unchanged.
9. **Removals** (TCK-149): adrenaline brain input and `CreatureAgent.adrenaline`.
10. **Genome operators** (TCK-184). `generate_structured_genome` (config `genomeComposition`) for trainer generation 1, random immigrants and Ocean random founders; `mutate_genome` draws one of creep (±1–2 letters), point, insertion, deletion, gene duplication, gene deletion by config weights (`mutationOperators`, `EVO` scales the indel weights); `HOX` joins the protected motifs.
11. **Inheritance by key, reset on model change** (TCK-179, TCK-183). Lamarckian offsets move between parent and child synapses with the same key (`inherit_learned_weights`, trainer and Ocean). `MODEL_VERSION = 2` in a `meta` table; a different or missing stored version recreates the model tables empty (no migration).
12. **Frontend renders backend facts** (TCK-187): gene spans and neuron roles come from `parse_genome` via ts-rs; the frontend maps them to text only.

### Deviations from the P3 design contract

- **Tape slot by hash instead of consecutive role-sorted order.** With consecutive slots, adding a synapse to an owner (e.g. a new organ adds thalamus → hidden inputs of every `NEU` gene) shifts all later slots of that owner. A hash slot depends only on the two roles, so other genes never move it. Cost: two synapses of one owner can share a slot (same initial weight) when the payload is short.
- **Organ identity without an angle bucket.** The angle is read from the organ gene's own payload; a mutation of the angle locus would have changed the identity of all the organ's synapses. The gene identity alone is sufficient.
- **Systemic → hidden owned by the `NEU` gene**, not by the brain-base gene: every synapse into or out of a hidden neuron belongs to its gene, so a duplicated or deleted `NEU` gene carries all its inputs and outputs as one unit.
- **ΔI is signed and also exists for thermal organs.** The sign distinguishes approach from retreat (the comparison klinotaxis needs); thermal organs (affinity 0.65–0.80) detect food like eyes and noses, so they get the same channel.
- **Gene identity by spelling + occurrence** (slice C) instead of the ordinal among genes of a promoter type: an ordinal shifts every later gene of that type on a gene duplication or deletion. With spelling + occurrence, gene duplication and deletion keep 100 % of the unrelated synapse keys and weights (test `test_gene_duplication_and_deletion_keep_unrelated_synapses`, measurement `measure_gene_operator_key_shift`); identity still shifts for later genes with the identical spelling.
- **No NEAT minimal start.** Targeted wiring already removes the dead structure (median synapses 142 → 41 on letter-soup genomes) and every synapse has an owner gene, so gene duplication/deletion is the structural growth operator. A minimal start would need add-connection operators and a connection registry; it was not evaluated.

## Alternatives considered

| Alternative | Why not |
| :--- | :--- |
| Keep the position hash, lower the mutation count | Locality stays low: one indel or one organ re-maps most weights (TCK-172); fewer ops only slow the search down |
| NEAT innovation numbers with a global registry | Needs shared state across Ocean, trainer and persisted genomes; the project has no crossover that would use historical markings. Role keys are derived from the genome alone, so `parse_genome` stays pure and deterministic |
| HyperNEAT / indirect encodings (CPPN) | Too large a rebuild for this project (TCK-182 out of scope); loses the gene-level ownership the inspector and Lamarck matching use |
| Relative addressing of `SY` genes (TCK-168) | Kept separate: `SY` genes address within role classes (modulo the class size); further relative addressing only if this proves insufficient |
| NEAT minimal start and complexification | See deviations; targeted connectivity chosen with the measured synapse counts |
| Forward Euler with a τ floor or `dt/τ ≤ 1` clamp | Narrows the evolvable τ range or changes dynamics only for fast neurons; the exponential step is stable for every τ, exact for constant input and keeps the τ range |
| Motion vision as a separate neuron type | A 6th input slot per organ keeps sensors uniform (role `Sensor{channel 5}`) and needs no new neuron class |

## Consequences

- **Golden master** `0xa7b82dc4fe552ac8` → `0x65f701a6937c1e71` (intermediate `0x0c404a33923c8dd9` after slice A, before gene identity by spelling) ([[wiki/quality/testing]]).
- **DB reset:** the first start after the update drops `trainer_genomes`, `creature_catalogue`, `simulation_state`, `simulation_history`, `species_records` (sessions kept) ([[wiki/architecture/persistence]]).
- **CI reference** `tests/reference/trainer-ci.json` regenerated for the new model ([[wiki/quality/ci-and-tooling]]).
- **Bindings:** new ts-rs types `NeuronRole`, `GeneRef`, `GeneSpan`, `OrganKind`, `OrganSide`, `SystemicInput`, `Motor`, `ThalamusKind`; synapses carry `key`/`owner`, the phenotype carries `genes` ([[wiki/architecture/ipc-protocol]]).
- **Metabolism:** the BMR formula is unchanged; its synapse term drops with compaction, the organ term (`organs · 5 · 0.08`) is unchanged ([[wiki/biology/metabolism-and-reproduction]]). An activity-based cost per neuron and organ is decided in TCK-151 (P6), not here.
- **Known limits kept:** `SY` addressing shifts when a role class changes size (TCK-168); a `HOX` gene pairs or unpairs every organ; only the first gene of a body promoter or `OUT` is read; tape slots can collide.

### Measured effects (TCK-172 probes and runner, before → after P3)

Reports in `.agents/artifacts/p3/` (local). "Structured" = `generate_structured_genome` parents with the new operator mix; "letter soup" = random 256-base genomes.

| Metric | Before (P2) | After P3 |
| :--- | :--- | :--- |
| Weight correlation after 1 / 5 / 15 trainer ops | 0.943 / 0.661 / 0.318 (letter soup) | 0.982 / 0.922 / 0.810 (structured); 0.979 / 0.906 / 0.757 (letter soup) |
| Children with ≥ 2× speed change after 1 / 15 ops | 1.4 % / 16.9 % | 0.7 % / 8.1 % (structured) |
| Synapses p10 / p50 / p90 (letter soup) | 52 / 142 / 249 | 14 / 41 / 103 (structured: 48 / 78 / 148) |
| Wildtype genome synapses / neurons | 249 / 72 | 118 / 67 |
| Generation-1 equipment: no organ / food sensor / `HOX` pair | 21 % / 67 % / 21 % | 0 % / 99 % / 31 % (structured) |
| Steering toward / away from food (random brains) | 3.2 % / 5.0 % | 9.0 % / 8.2 % (structured), still undirected |
| Success rate small / large room (pop 16, 30 gens, mean of last 5 gens, 5 seeds) | 0.040 / 0.023 | 0.122 / 0.033 |
| Sandbox-ticks/s, pop 16, seed b1, small / large room | 158k / 425k | 323k / 763k (after slice A) |

- TCK-171 targets for heredity (≥ 0.95 / 0.85 / 0.60) and for speed change after one op (≤ 2 %) are met; the success and steering targets are not (later phases P4–P8).

### Follow-up P4 (TCK-152)

- Synapses also carry a compiler-set `plastic` flag: sensor → thalamus reflexes are fixed (no learning, no pruning), all others learn with Oja's rule ([[wiki/biology/ctrnn-brain]]).
- Lamarckian inheritance by key (point 11) is now a probability per plastic synapse (`lamarckianAssimilationChance`), the same in trainer and Ocean ([[wiki/biology/metabolism-and-reproduction]]).
- Golden master `0x65f701a6937c1e71` → `0xe0a2bfd1c7310306` (only the flag); `MODEL_VERSION` 2 → 3 ([[wiki/architecture/persistence]]).

### Follow-up P6 (TCK-151)

- The activity-based cost named under Consequences is implemented: the BMR formula is replaced by one additive energy function (body mass, organ load, `neuronCost` per thalamus/hidden/motor neuron, `activityCost · Σ |w · a_source|` from the signal pass, thrust), shared by Ocean and trainer ([[wiki/biology/metabolism-and-reproduction]]). Compaction keeps the brain cheap; pruned synapses cost nothing.
- Golden master `0xe0a2bfd1c7310306` → `0x9d206dfd8124f808` (phenotype field `basal_metabolic_rate` removed); `MODEL_VERSION` 3 → 4.

## Sources

- Rothlauf, High Locality Representations: https://gpbib.pmacs.upenn.edu/gp-html/Rothlauf_2011_DMH_cp7.html
- Galván-López et al., locality in GP: https://ncra.ucd.ie/papers/localityingpWCCI2010.pdf
- NEAT innovation numbers: https://neat-python.readthedocs.io/en/latest/innovation_numbers.html
- Stanley & Miikkulainen, NEAT (minimal start, complexification): https://axon.cs.byu.edu/~dan/778/papers/NeuroEvolution/stanley2*.pdf
- Braitenberg vehicles (tropotaxis): https://en.wikipedia.org/wiki/Braitenberg_vehicle ; review: https://frontiersin.org/articles/10.3389/fbioe.2020.565963/full
- Satoh et al., klinotaxis by a single neuron: https://pmc.ncbi.nlm.nih.gov/articles/PMC6608435 ; klinotaxis review: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4132367/
- Doerr, Doerr, Lengler, mutation rates: https://arxiv.org/pdf/1902.02588

## Key Takeaways

- Synapses and genes have stable identities derived from the genome alone; weights and traits come only from their owner gene.
- The brain wires only live structure, evaluates in one pass with latency 1 and integrates stably for every τ.
- Structured random genomes and gene-level/creep operators make heredity meet the TCK-171 targets; steering and success remain open for later phases.

Sources: TCK-171, TCK-172, TCK-179, TCK-182, TCK-144, TCK-153, TCK-184, TCK-187, TCK-183, TCK-188, TCK-149, TCK-151, TCK-152.
