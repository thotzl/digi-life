---
id: TCK-150
title: Locomotion Thrust Scaling and Ocean Ecology Measurement
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S02, RV-S07
---

# TCK-150: Locomotion Thrust Scaling and Ocean Ecology Measurement

> Pool ticket: requires a user decision before implementation. Findings (RV-S02, RV-S07) below.

## Description
Hypothesis: thrust scales towards zero for low pulse/stiffness genomes (motionless creatures); food density in the Ocean is very low.

## Proposed Scope
Measure terminal speed distribution over random genomes and food encounter rates before tuning.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S02 · P1 · hypothesis "motionless Ocean"** — `src/shared/physics.rs`: thrust ≈ `stiffness · (pulse·1000)² · 3.4 · eta · out_thrust`. With `pulse = 0.0015` (genome minimum) and `stiffness = 0.15` → ~1.1 · eta; mass `r^1.5 · L/25` ≈ 230–1,180; damping 0.94 → terminal speed 10⁻³–10⁻² px/tick in a 19,200 px world. Such genomes are effectively immobile; `out_thrust` is `tanh` in [-1, 1], negative = backwards. This explains the motionless-Ocean symptom better than the earlier render time-scale hypothesis (`frontend/ocean.ts`).
- Measure before fixing: terminal speed distribution over 1,000 random genomes (headless, needs TCK-158).
- **RV-S07 · P2 · hypothesis** — ecology very sparse: 300 spores on 19,200 × 10,800 px (≈ 1 per 830 × 830 px), sensor range `scale · 550` (190–990 px), 25 agents. Random food finding is rare. Balance only after RV-B01/S02 with measurement series.
