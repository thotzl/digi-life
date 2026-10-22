---
sources: [README.md, .agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-116-sequence-based-dynamic-genome-regulatory-network.md, tickets/closed/TCK-117-biochemical-codon-groups-wobble-matching.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-140-trainer-shared-performance.md, tickets/closed/TCK-162-deterministic-latin-name.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-144-motion-vision-channel-design.md, tickets/closed/TCK-187-gene-spans-from-backend.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md]
---

# Genome Compiler

`src/biology/dna.rs` — turns a genome string into a `CreaturePhenotype` (body, organs, brain). Model since Brain v2 (TCK-171 P3, [[wiki/decisions/adr-002-brain-v2]]).

## Genome

- **Sense strand:** uppercase `A-Z` string; `normalize_genome` uppercases and pads/truncates to `[128, 512]` before compiling.
- **Antisense:** `get_complementary_string`; passed to `parse_genome` as optional second strand. Traits are read from the sense strand only.
- **Entry point:** `parse_genome(genome, antisense_input, parent_methylations) -> CreaturePhenotype` — pure and deterministic. Exposed to the frontend via the `get_fossil_phenotype` command.

## Promoter/terminator genes (since TCK-116)

- A gene is a promoter motif, a payload and the terminator `EN` (degenerate; searched within 40 letters, otherwise a 15-letter payload is read). There are no positional loci; anything referring to "locus N" predates TCK-116.
- Expressed promoters: body `COL`, `SIZ`, `STF`, `WAV`, `PUL`, `STM`, `TEM`, `CAR`, `REP`, `EVO`; symmetry `HOX`; organs `EYE`, `NOS`, `TAC`, `LUM`; brain `NEU`, `SY`, `OUT`.
- Payload values: `get_payload_linear_value_offset(payload, shift)` reads one letter (`shift mod len`), returns `h ∈ [0, 1]`; an empty payload gives `h = 0.5`.
- `phenotype.genes`: every recognised gene as `GeneSpan{gene: GeneRef, start, end}` in genome order; synapse owners index into this list, the frontend renders it (TCK-187, [[wiki/frontend/structure]]).
- There is no regulatory cascade (transcription factors binding operators); TCK-116 described one, the code does not implement it.
- The legacy chromatin scan (`chromatin_state`, also accepting `SP` as terminator) only marks positions for the DNA helix view and logs; it feeds no trait or weight.

## Gene identity (TCK-179, slice C)

- `GeneRef{promoter, spelling, occurrence}`: promoter type, the exact promoter spelling in the genome (a degenerate variant such as `EWA` for `EYE`) and the occurrence among genes with that spelling in genome order.
- Letter-level mutations never change promoter letters (`unprotected_indices` protects every motif match and the promoter of every expressed gene; test `test_letter_mutations_never_change_a_promoter_spelling`); gene duplication inserts the copy after the original. A gene therefore keeps its identity unless a gene with the identical spelling before it is duplicated or deleted.
- Neuron roles, synapse keys and tape slots are built from `GeneRef`, never from genome positions or global counts ([[wiki/biology/ctrnn-brain]]).

## Degenerate matching / codon groups (TCK-117)

- A–Z split into 5 biochemical groups: Alpha `[A,E,I,O,U]`, Beta `[Y,W,F,H,K,R]`, Gamma `[L,M,V,P,T]`, Delta `[D,N,Q,S,C]`, Epsilon `[B,G,J,X,Z]`.
- `matches_degenerate(segment, promoter)`: the first letter must match exactly, the others by group (e.g. `EWA` matches `EYE`); `NEU`, `SY` and one-letter motifs match fully by group.

## Trait locality (TCK-179)

- Every body trait is read from the first gene of its own promoter only (`COL` colour/symmetry, `SIZ` radius/length/seed, `STF` stiffness/hydraulics, `WAV` spine waves, `PUL` pulse/phase/wiggle/head, `STM` stomach, `TEM` thermal tolerance, `CAR` carnivory, `REP` maturity/threshold/split loss, `EVO` indel rates/repair fidelity). Later copies of these promoters are silent.
- A missing gene reads the empty payload: every value at the middle of its range (`h = 0.5`), never another gene. `active_dna` and its fallback are removed, so a brain-gene mutation no longer changes body traits.
- Organs: one per `EYE`/`NOS`/`TAC`/`LUM` gene (organ order: all `EYE` genes, then `NOS`, `TAC`, `LUM`); affinity, bandwidth, expression style, scale, spinal position, angle and hue from fixed payload loci. `OrganKind` is decided once from affinity and expression style ([[wiki/biology/sensing]]).
- Property tests: a point mutation outside every gene changes no weight, organ or trait; a point mutation inside gene G changes only synapses owned by or touching G, neurons of G and traits of G (if G is the expressed copy) ([[wiki/quality/testing]]).

## HOX symmetry (FIX-131)

- Angle space `[-170°, 170°]`, 0° = ahead, from the organ gene's angle locus.
- No `HOX` gene: one organ at its angle (`OrganSide::Single`).
- `HOX` present: an organ within ±7.5° of the midline becomes a single organ at 0°; any other organ gene yields a left/right pair at `∓|θ|` (`OrganSide::Left/Right`). A `HOX` gene therefore pairs or unpairs every organ at once.
- Pairs feed sum/difference thalamus nodes with one shared weight set, see [[wiki/biology/ctrnn-brain]].

## Random genomes (TCK-184)

- `generate_structured_genome(composition, rng)` (config `genomeComposition`, [[wiki/architecture/configuration]]): all body genes plus one `OUT` gene, 1–3 organ genes (the first is a food organ — eye or nose 50/50 — with probability 0.9, otherwise random organ promoters), `HOX` with probability 0.3, 1–3 `NEU`, 0–2 `SY`; genes shuffled, payloads 8–16 letters, intergenic filler towards 256 bases (no filler when the genes alone are longer; mean length 288). Each promoter gets a random degenerate spelling not used before in the genome; payloads and filler never form a stray motif, so `parse_genome` finds exactly the planned genes (test `test_structured_genome_parses_to_exactly_the_planned_genes`).
- Used for trainer generation 1 and random immigrants ([[wiki/simulation/trainer]]) and for Ocean `INJECT_URZELLE`, the empty-catalogue restock founder and the `RESET_EVOLUTION` fallback ([[wiki/simulation/ocean]]). The Ocean's fresh-start seeding still mutates the wildtype literal.
- `generate_random_genome(len)` (uniform letters, "letter soup") remains only for probes and tests as the comparison population.
- Equipment (TCK-172 probe, 1,000 genomes each):

| Property | Letter soup | Structured |
| :--- | :--- | :--- |
| No organ | 21 % | 0 % |
| ≥ 1 eye / ≥ 1 nose | 29 % / 43 % | 58 % / 66 % |
| ≥ 1 food-sensing organ (affinity ≥ 0.25) | 67 % | 99 % |
| `HOX` organ pair | 21 % | 31 % |
| Food in range / non-zero stimulus at spawn, small room | 57 % / 31 % | 86 % / 46 % |
| Food in range / non-zero stimulus at spawn, large room | 14 % / 4.8 % | 24 % / 7.5 % |

## Mutation (TCK-184)

- `mutate_genome(genome, ops, rng) -> Option<Mutation{genome, kind, index}>` applies one operator, drawn by the weights of config `mutationOperators`: creep 0.6 (letter moved 1–2 alphabet steps, no wrap-around), point 0.15 (random other letter), insertion 0.05, deletion 0.05, gene duplication 0.075 (a whole terminated gene copied directly after the original), gene deletion 0.075 (a whole non-essential gene removed; body genes and `OUT` only while another copy exists). The `EVO` gene scales the indel weights by `2h` (`h = 0.5` keeps the configured weight). A non-applicable operator falls back to creep.
- Letter operators act only on unprotected positions; protected: every motif match (all expressed promoters incl. `HOX`, legacy `SYM`, terminator `EN`) and the promoter letters of every expressed gene. Gene operators splice only when the result changes exactly the intended promoter's gene count, with at most one spacer letter.
- Length stays in `[128, 512]` (test `test_mutate_genome_keeps_length_bounds`).
- `mutate_genome_with_kind` applies one fixed operator for probes and tests. All generators and operators take the caller's RNG (seeded `SimRng` in trainer, runner and probes; `thread_rng` in the Ocean; TCK-173).
- Trainer mutants apply `round(genome.len() · mutation_rate)` operations (`breed_genome`; 0.06 → ≈ 15 at 256 bases). Ocean mitosis mutates the child genome with 35 % probability ([[wiki/biology/metabolism-and-reproduction]]).
- `read_evolution_rates` reads the `EVO` rates without compiling the phenotype (TCK-140).

## Measured heredity (TCK-172 probe, P3)

Probe `trainer_runner probe heredity` (seed `tck172`, 200 parents per set): parent–child Pearson correlation of non-`SY`, non-reflex synapse weights on homologous synapses (same key).

| Mutation | 1 op | 5 ops | 15 ops |
| :--- | :--- | :--- | :--- |
| Before P3: letter-soup parents, character mix | 0.943 | 0.661 | 0.318 |
| Structured parents, `mutate_genome` mix | 0.982 | 0.922 | 0.810 |
| Letter-soup parents, `mutate_genome` mix | 0.979 | 0.906 | 0.757 |
| Structured parents, legacy character mix | 0.985 | 0.916 | 0.792 |

- Children with ≥ 2× terminal-speed change (structured parents): 0.7 % after 1 op, 3.0 % after 5, 8.1 % after 15 (before P3: 1.4 % / 7.0 % / 16.9 %).
- Gene duplication and deletion keep 100 % of the keys and weights of synapses that neither belong to nor touch the mutated gene (except `SY`-owned synapses and `HOX` genes); test `test_gene_duplication_and_deletion_keep_unrelated_synapses`, measurement `measure_gene_operator_key_shift` (ignored test).
- Most of the gain over the character mix comes from Brain v2 locality, not from the operator mix (legacy and new mix differ by ≤ 0.02 on structured parents).

## Epigenetics

- Parent methylations are an input to `parse_genome` and are persisted with genomes (`methylations` columns), see [[wiki/architecture/persistence]].

## Latin name

- Derived in `derive_ecological_metrics` from organ-type counts; ties resolve in fixed order light, chem, kinetic, thermal (TCK-162, RV-B11).
- Only ~6,760 strain combinations per prefix/suffix; collisions are silently dropped by `INSERT OR IGNORE` on the UNIQUE `latin_name` (RV-B10).

## Known limitations

- Activation and symmetry are still free-text strings (`activation_type`, `symmetry`); a typo falls back to the default silently (RV-S10 rest). Organ type is an enum since TCK-144.
- `SY` genes address roles modulo the size of their role class, so a changed class size re-addresses them (relative addressing: TCK-168).
- Only the first gene of each body promoter and of `OUT` is expressed; duplicates are silent until the original is deleted.

## Key Takeaways

- Sequence-based, promoter-driven, degenerate-matching compiler; pure and deterministic; every trait and weight comes from its own gene.
- Genes have a stable identity (promoter spelling + occurrence); structured random genomes and gene-level/creep operators replace letter soup and character-only mutation.
- Heredity after 15 trainer ops rose from 0.32 to 0.81 (TCK-179, TCK-184).
- Any model change shows up in the golden master ([[wiki/quality/testing]]).

Sources: `README.md`, `.agents/skills/biology-dna/SKILL.md`, TCK-116, TCK-117, FIX-131, TCK-140, TCK-144, TCK-162, TCK-172, TCK-173, TCK-179, TCK-184, TCK-187, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
