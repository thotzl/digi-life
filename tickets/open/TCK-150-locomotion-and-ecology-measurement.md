---
id: TCK-150
title: Locomotion Thrust Scaling and Ocean Ecology Measurement
status: open
priority: high
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S02, RV-S07
epic_phase: Training Track
depends_on: TCK-172
---

# TCK-150: Locomotion Thrust Scaling and Ocean Ecology Measurement

> Decision 2026-10-21 (training track, epic TCK-171): measurement comes from the TCK-172 speed probe; this ticket now also carries the fix. Estimate from 2026-10-21 (TCK-172): terminal speed p10 0.08 / p50 0.74 / p90 5.0 px/tick at full thrust, spread ≈ ×60 driven by `pulse²` (`src/shared/physics.rs:113`); 48 % of random genomes cannot cover 1200 px in an 1800-tick epoch. Fix scope: rescale thrust vs. mass/drag so the spread narrows (e.g. linear in pulse), keep turning possible at low speed (`physics.rs:182` couples turn rate to forward speed); decide `steerTorqueBaseMultiplier` / `dragLateralCoefficient` (TCK-149). Golden master unaffected (physics only), trainer fitness baselines change — re-measure. Ecology part (RV-S07) stays for the Ocean phase. Findings (RV-S02, RV-S07) below.

## Description
Hypothesis: thrust scales towards zero for low pulse/stiffness genomes (motionless creatures); food density in the Ocean is very low.

## Proposed Scope
Measure terminal speed distribution over random genomes and food encounter rates before tuning.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S02 · P1 · hypothesis "motionless Ocean"** — `src/shared/physics.rs`: thrust ≈ `stiffness · (pulse·1000)² · 3.4 · eta · out_thrust`. With `pulse = 0.0015` (genome minimum) and `stiffness = 0.15` → ~1.1 · eta; mass `r^1.5 · L/25` ≈ 230–1,180; damping 0.94 → terminal speed 10⁻³–10⁻² px/tick in a 19,200 px world. Such genomes are effectively immobile; `out_thrust` is `tanh` in [-1, 1], negative = backwards. This explains the motionless-Ocean symptom better than the earlier render time-scale hypothesis (`frontend/ocean.ts`).
- Measure before fixing: terminal speed distribution over 1,000 random genomes (headless, needs TCK-158).
- **RV-S07 · P2 · hypothesis** — ecology very sparse: 300 spores on 19,200 × 10,800 px (≈ 1 per 830 × 830 px), sensor range `scale · 550` (190–990 px), 25 agents. Random food finding is rare. Balance only after RV-B01/S02 with measurement series.
