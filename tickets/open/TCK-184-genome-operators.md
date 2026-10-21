---
id: TCK-184
title: "Genome Operators: Structured Random Genomes, Gene-Level and Creep Mutation"
status: open
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
