---
id: TCK-187
title: Gene Spans and Neural Inspector from Backend
status: open
priority: medium
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-179
---

# TCK-187: Gene Spans and Neural Inspector from Backend

Part of epic TCK-171, phase P3. Subset of TCK-160 (RV-F03).

## Description
`frontend/signals.ts` describes a positional genome (`getLocusDescription(i)`: locus 0 = symmetry, 7 = radius, ...) and searches `ST`/`GO`, although the compiler is promoter-based since TCK-116; tooltips and the genome view show wrong meanings (RV-F03). Brain v2 (TCK-179, TCK-182, TCK-144) changes topology and neuron roles, which the neural inspector displays.

## Scope
- `parse_genome` returns gene spans incl. promoter name and role (it knows them); exported via ts-rs; the client only renders.
- Remove the legacy positional locus logic in the frontend.
- Neural inspector renders the Brain v2 topology (roles instead of emoji labels, enum organ types).

## Acceptance
- Gene tooltips match the backend gene spans for sample genomes.
- Neural inspector shows Brain v2 correctly (manual check, part of the TCK-186 smoke checklist).
- `npm run build` green.
