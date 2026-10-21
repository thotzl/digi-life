---
sources: [README.md, .agents/skills/biology-dna/SKILL.md, tickets/closed/TCK-116-sequence-based-dynamic-genome-regulatory-network.md, tickets/closed/TCK-117-biochemical-codon-groups-wobble-matching.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-140-trainer-shared-performance.md, tickets/closed/TCK-162-deterministic-latin-name.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md]
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
- Trait payloads (`COL`, `SIZ`, `STF`, ...) are extracted from the full cleaned genome.
- Helpers shared by `parse_genome` and `read_evolution_rates`: `normalize_genome`, `scan_chromatin`, `active_dna_of` (TCK-140).

## Mutation

- `mutate_genome`: point substitutions plus insertions/deletions at unprotected loci, scaled by the genome's own `insertion_rate`/`deletion_rate`, length capped to `[128, 512]` (TCK-117).
- Reads rates via `read_evolution_rates` without compiling the full phenotype (TCK-140).
- Ocean mitosis mutates the child genome with 35 % probability ([[wiki/biology/metabolism-and-reproduction]]).
- `generate_random_genome(256)` for random origins; such genomes rarely contain `NEU`/`SY`/`EYE`, so trainer generation 1 usually starts blind with the fallback brain (RV-S09).

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

- Model semantics in free-text strings: activation, organ type (emoji label), symmetry, scenario, origin, diet class; a typo falls back to the default silently (RV-S10).
- The frontend interprets a legacy positional genome model, so gene tooltips are wrong (RV-F03), see [[wiki/frontend/structure]].

## Key Takeaways

- Sequence-based, promoter-driven, degenerate-matching compiler; pure and deterministic.
- HOX controls bilateral symmetry; chromatin drives brain hashing.
- Any model change shows up in the golden master ([[wiki/quality/testing]]).

Sources: `README.md`, `.agents/skills/biology-dna/SKILL.md`, TCK-116, TCK-117, FIX-131, TCK-140, TCK-162, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`.
