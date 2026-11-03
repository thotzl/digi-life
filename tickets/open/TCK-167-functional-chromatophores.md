---
id: TCK-167
title: Functional Chromatophores (Tissue Colour Metabolism)
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
epic_phase: Roadmap TCK-169 Phase 2 (feature epics, to be planned)
---

# TCK-167: Functional Chromatophores (Tissue Colour Metabolism)

## Update (2026-11-03, after TCK-196)
- Seams since TCK-196: a chromatophore can be an organ kind (`wiki/guides/add-organ-kind.md`) or a body trait (`BODY_GENES`); photosynthesis is an `EnergyModifier` in `src/sim/systems/metabolize.rs` and applies to herbivores only (`src/biology/diet.rs`, TCK-202). Colour reaches the UI through the phenotype bindings and the organ painters (`frontend/render/organs.ts`). Re-scope on these seams before starting.

> Pool ticket from `ideas.md` (section II.2, epoch 15). Context: `wiki/research/porting-concepts.md`.

## Description
Biogenesis-style tissue colours with a metabolic function (photosynthetic, carnivorous, propulsive segments) instead of cosmetic colours. Photosynthesis is a flat config value today and biomes have no simulation effect (RV-S01).

## Proposed Scope
- Decide together with the biome keys in TCK-149 (light/depth as an environmental factor).
- Re-specify against `COL` genes and spectral affinity; ideas.md uses positional `START ... STOP` genes.

## Measurement Criterion
- Emergence of distinct autotroph/predator niches in headless Ocean runs (after TCK-143).
