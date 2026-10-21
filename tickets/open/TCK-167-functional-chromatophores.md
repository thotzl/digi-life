---
id: TCK-167
title: Functional Chromatophores (Tissue Colour Metabolism)
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
---

# TCK-167: Functional Chromatophores (Tissue Colour Metabolism)

> Pool ticket from `ideas.md` (section II.2, epoch 15). Context: `wiki/research/porting-concepts.md`.

## Description
Biogenesis-style tissue colours with a metabolic function (photosynthetic, carnivorous, propulsive segments) instead of cosmetic colours. Photosynthesis is a flat config value today and biomes have no simulation effect (RV-S01).

## Proposed Scope
- Decide together with the biome keys in TCK-149 (light/depth as an environmental factor).
- Re-specify against `COL` genes and spectral affinity; ideas.md uses positional `START ... STOP` genes.

## Measurement Criterion
- Emergence of distinct autotroph/predator niches in headless Ocean runs (after TCK-143).
