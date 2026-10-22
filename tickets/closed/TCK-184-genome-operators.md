---
id: TCK-184
title: "Genome Operators: Structured Random Genomes, Gene-Level and Creep Mutation"
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-179
---

# TCK-184: Genome Operators: Structured Random Genomes, Gene-Level and Creep Mutation

Part of epic TCK-171, phase P3 (Brain v2). Intended golden-master change together with TCK-179.

## Description
- `generate_random_genome` draws single letters (`src/biology/dna.rs:379-387`); genes only appear by chance. After the DB reset (TCK-183) there is no starting population, so generation 1 would start from letter soup (mostly blind, RV-S09).
- `mutate_genome` knows only point mutation, single-character insertion and deletion (`dna.rs:389-480`); there is no operator that duplicates or removes a whole gene, which is how new organs and neurons would arise coherently (comparable to NEAT's add-node).
- A point mutation replaces a character with a random letter; values read from that character jump arbitrarily. Even with stable synapse keys (TCK-179) a small mutation stays a large jump.

## Scope
- Structured random genome generator: composes valid genes (promoter + payload + terminator), always the body genes (`PUL`, `STF`, `SIZ`, ...), optionally eyes/noses, `NEU`, `SY`; configurable composition. Used for generation 1 (TCK-175 decides minimum equipment).
- Gene-level mutations: duplicate a gene, delete a gene; rates from config (or the `EVO` gene).
- Creep mutation: point mutations prefer ±1–2 letter steps (small value change); a smaller share stays a random replacement.
- Keep the length bounds [128, 512] and the protected motifs.
- Property tests: generator output always parses with the requested genes; gene duplication adds exactly one gene; creep mutation changes the value by at most the step.

## Acceptance
- TCK-172 heredity probe: parent-child correlation after 1 / 5 / 15 mutations rises compared with character-only mutation (before/after).
- Generation 1 from the generator: share of genomes with eyes/noses and movement as configured.

## Verification (P3, v1.12.17)
- `generate_structured_genome`: body genes COL SIZ STF WAV PUL STM TEM CAR REP EVO and one `OUT` always; 1-3 organs, first a food organ with p 0.9; HOX 0.3; NEU 1-3; SY 0-2; distinct degenerate promoter spellings; filler cannot form stray promoters; config `genomeComposition`. Used for trainer generation 1 and random immigrants and for Ocean INJECT_URZELLE, the reset fallback and the empty-catalogue refill.
- Operators (one per `mutate_genome` call, config `mutationOperators`): creep 0.6 (+-1-2 letters), point 0.15, insertion 0.05, deletion 0.05, gene duplication 0.075, gene deletion 0.075 (never the last body gene or `OUT`); EVO scales the indel weights; length bounds [128, 512] and protected motifs kept (HOX added to the protected motifs).
- Property tests: generator parses to exactly the planned genes (2000 seeds); duplication adds exactly one gene; deletion removes exactly one, never an essential gene; creep changes a tape weight by at most step/25; 10k mutations keep the bounds; seeded determinism.
- Heredity (structured parents, exuberant-weight r 1/5/15 ops): new mix 0.982/0.922/0.810 vs legacy character mix 0.985/0.916/0.792 (r higher at 5/15 ops; unchanged-weight share still higher for the legacy mix because gene operators add/remove whole genes). Creep alone 0.985/0.919/0.789 (slice B ids).
- Generation 1 equipment, structured vs letter soup: no organ 0 % vs 21 %, food sensor 99 % vs 67 %, HOX pair 31 % vs 21 %; food stimulus at spawn small 42 % vs 31 %, large 7.7 % vs 4.8 %.
- Golden master `0xa7b82dc4fe552ac8` -> `0x65f701a6937c1e71` (one intended Brain v2 change for TCK-179/182/144/153/184/149; ADR-002). CI reference `tests/reference/trainer-ci.json` regenerated. `cargo test` 94 passed (1 ignored measurement), 0 warnings; `npm run build` and `npm test` green.
