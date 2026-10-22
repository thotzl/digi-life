---
id: TCK-187
title: Gene Spans and Neural Inspector from Backend
status: closed
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

## Verification (P3, v1.12.17)
- `parse_genome` returns `genes: Vec<GeneSpan{gene: GeneRef, start, end}>`; neurons carry `role`, synapses `key`/`owner` (ts-rs bindings `GeneSpan`, `GeneRef`, `NeuronRole`, `OrganKind`, ...).
- Frontend: `frontend/shared/genome.ts` maps promoter -> meaning (one table); the genome view colours backend spans, promoter letters highlighted, "Non-coding" outside spans; legacy positional locus logic removed (`getLocusDescription`, `ST`/`GO` scan; RV-F03 resolved). Neural inspector lays out by role (sensors per organ incl. ΔI, systemic inputs, thalamus sum/diff, hidden by depth, thrust/bend) with owner-gene tooltips; adrenaline UI removed; brain/gene types from `bindings/` (RV-A03 partly).
- Tests: `frontend/shared/genome.test.ts` (5 vitest tests).
- UI check (TCK-186 item 3, P3): Ocean and trainer inspector screenshots and tooltip texts match the backend spans and roles; 0 console errors.
- Golden master `0xa7b82dc4fe552ac8` -> `0x65f701a6937c1e71` (one intended Brain v2 change for TCK-179/182/144/153/184/149; ADR-002). CI reference `tests/reference/trainer-ci.json` regenerated. `cargo test` 94 passed (1 ignored measurement), 0 warnings; `npm run build` and `npm test` green.
