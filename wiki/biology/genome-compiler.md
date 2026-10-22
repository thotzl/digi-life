---
sources: [README.md, .agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-116-sequence-based-dynamic-genome-regulatory-network.md, tickets/closed/TCK-117-biochemical-codon-groups-wobble-matching.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-140-trainer-shared-performance.md, tickets/closed/TCK-162-deterministic-latin-name.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md]
---

# Genome Compiler

`src/biology/dna.rs` — turns a genome string into a `CreaturePhenotype` (body, organs, brain).

## Genome

- **Sense strand:** uppercase `A-Z` string, length kept in `[128, 512]`.
- **Antisense:** `get_complementary_string`; passed to `parse_genome` as optional second strand. Traits are read from the sense strand only.
- **Entry point:** `parse_genome(genome, antisense_input, parent_methylations) -> CreaturePhenotype` — pure and deterministic. Exposed to the frontend via the `get_fossil_phenotype` command.

## Promoter/terminator genes (since TCK-116)

- Genes are found by promoter motifs followed by a payload and a terminator (`EN`/`SP`); there are no fixed positional loci. Anything referring to "locus N" predates TCK-116.
- Motifs protected from mutation (`mutate_genome`): `COL`, `STF`, `PUL`, `SIZ`, `WAV`, `SYM`, `STM`, `TEM`, `EYE`, `NOS`, `TAC`, `LUM`, `CAR`, `REP`, `EVO`, `NEU`, `SY`, `OUT`, `EN`; `HOX` is scanned for symmetry.
- `extract_raw_gene_payloads` scans for promoters/terminators; payloads of any length are hashed into normalized values (`get_payload_linear_value_offset`) that drive colors, stiffness, size, wave parameters, sensory organelles and brain parameters. Interpretation lives in the calling module (one neutral extraction helper).
- There is no regulatory cascade (transcription factors binding operators); TCK-116 described one, the code does not implement it.
- Starting wildtypes can be organless; organs emerge through mutation.

## Degenerate matching / codon groups (TCK-117)

- A–Z split into 5 biochemical groups: Alpha `[A,E,I,O,U]`, Beta `[Y,W,F,H,K,R]`, Gamma `[L,M,V,P,T]`, Delta `[D,N,Q,S,C]`, Epsilon `[B,G,J,X,Z]`.
- `matches_degenerate(segment, promoter)`: match if every character is in the same group as the promoter's (e.g. `OWO` matches `EYE`). Applies to organ promoters and terminators; makes organs evolvable from random strings.

## Chromatin

- `chromatin_state` marks positions covered by recognized promoter..terminator blocks; `active_dna` is the concatenation of those positions and feeds brain-parameter hashing.
- Trait payloads (`COL`, `SIZ`, `STF`, ...) are extracted from the full cleaned genome. If a trait gene is missing, the value is read from `active_dna` instead (`dna.rs:800-861`), so a mutation in any active gene (e.g. a brain gene) can change body traits such as pulse or stiffness.
- Helpers shared by `parse_genome` and `read_evolution_rates`: `normalize_genome`, `scan_chromatin`, `active_dna_of` (TCK-140).

## Mutation

- `mutate_genome`: point substitutions plus insertions/deletions at unprotected loci, scaled by the genome's own `insertion_rate`/`deletion_rate`, length capped to `[128, 512]` (TCK-117). One call = one operation on a single character; a point substitution draws a random letter (no small "creep" steps); there is no operator that duplicates or removes a whole gene.
- `mutate_genome` and `generate_random_genome` take the caller's RNG (seeded `SimRng` in the trainer and runner, `thread_rng` in the Ocean; TCK-173). `mutate_genome_with_kind` applies one operation of a fixed kind (point/insertion/deletion) for the probes.
- Trainer mutants apply `round(genome.len() · mutation_rate)` operations (`breed_genome`); with the default rate 0.06 that is 15–30 per child.
- Reads rates via `read_evolution_rates` without compiling the full phenotype (TCK-140).
- Ocean mitosis mutates the child genome with 35 % probability ([[wiki/biology/metabolism-and-reproduction]]).
- `generate_random_genome(256)` for random origins draws uniformly distributed single letters; genes appear only by chance via degenerate matching (RV-S09). Measured over 1,000 random 256-base genomes (TCK-172): 21 % have no organ, 29 % at least one eye, 43 % at least one nose, 67 % at least one food-sensing organ (affinity ≥ 0.25), 21 % a `HOX` organ pair ([[wiki/biology/sensing]]).

## Measured heredity (TCK-172)

Probe `trainer_runner probe heredity`: parent–child Pearson correlation of exuberant (non-`SY`, non-reflex) synapse weights on homologous synapses (same from/to), 200 random parents, the wildtype and 34 HoF genomes.

| Mutation | 1 op | 5 ops | 15 ops |
| :--- | :--- | :--- | :--- |
| Trainer operator mix (≈ 94 % point, 2 % insertion, 4 % deletion) | 0.90–0.94 | 0.62–0.66 | 0.21–0.32 |
| Point only | 0.93–0.94 | 0.74–0.77 | 0.40–0.50 |
| Insertion or deletion only | 0.32–0.40 | ≤ 0.08 | ≈ 0 |

- A single indel inside active DNA re-maps most weights (position hash, see [[wiki/biology/ctrnn-brain]]); 15 trainer ops keep the from/to of 65–79 % of parent synapses.
- Explicit `SY` synapses are more stable (r ≈ 0.76 after 15 mixed ops, random parents).
- Body coupling: after 15 trainer ops 17–25 % of children change their terminal speed by ≥ 2× and 19–35 % change their organ count; after one op 1–2 % change speed ≥ 2×.
- Consequence: the trainer's 15–30 ops per child act close to random search; elites survive only as unmodified copies.

## HOX symmetry (FIX-131)

- No `HOX` gene: organs distributed asymmetrically in `10°..350°` (primitive species).
- `HOX` active: organs within ±5° of the midline snap to it (single central organ); lateral organs get a mirrored partner at `360° − θ`.
- Mirrored pairs feed sum/difference interneurons, see [[wiki/biology/ctrnn-brain]].

## Epigenetics

- Parent methylations are an input to `parse_genome` and are persisted with genomes (`methylations` columns), see [[wiki/architecture/persistence]].

## Latin name

- Derived in `derive_ecological_metrics` from organ-type counts; ties resolve in fixed order light, chem, kinetic, thermal (TCK-162, RV-B11). Names stored before the fix may differ for tied species.
- Only ~6,760 strain combinations per prefix/suffix; collisions are silently dropped by `INSERT OR IGNORE` on the UNIQUE `latin_name` (RV-B10).

## Known limitations

- Model semantics in free-text strings: activation, organ type (emoji label), symmetry; a typo falls back to the default silently (RV-S10). Scenario, sandbox origin and diet class are enums since TCK-173; the brain decodes activation and organ type once per agent (`BrainRuntime`, [[wiki/biology/ctrnn-brain]]).
- The frontend interprets a legacy positional genome model, so gene tooltips are wrong (RV-F03), see [[wiki/frontend/structure]].

## Key Takeaways

- Sequence-based, promoter-driven, degenerate-matching compiler; pure and deterministic.
- HOX controls bilateral symmetry; chromatin drives brain hashing.
- Any model change shows up in the golden master ([[wiki/quality/testing]]).
- Heredity is low: one indel or 15 trainer ops leave parent and child weights nearly uncorrelated (TCK-172).

Sources: `README.md`, `.agents/skills/biology-dna/SKILL.md`, TCK-116, TCK-117, FIX-131, TCK-140, TCK-162, TCK-172, TCK-173, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
