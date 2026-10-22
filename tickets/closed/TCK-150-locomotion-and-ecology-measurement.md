---
id: TCK-150
title: Locomotion Thrust Scaling and Ocean Ecology Measurement
status: closed
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

## Verification (P5, v1.12.19)
- Speed band instead of `pulse^2`: `g = 0.4*pulse_n + 0.4*stiffness_n + 0.2*appendages`, `v_target = locomotionSpeedMin + (locomotionSpeedMax - locomotionSpeedMin)*g` (1.0 / 5.0 px/tick); thrust `F = v_target*(0.06*m + 0.94*d)/0.94` inverted from the straight-line update, still through `step_creature_kinematics`. `thrustBaseMultiplier` removed.
- Turning at rest: `delta_heading = bend*(0.015*v_forward + turnRateAtRest)` (0.008; 90 deg from rest ~165 ticks); `steerTorqueBaseMultiplier` removed (TCK-149).
- Speed probe (seed tck172, letter soup / structured): p90/p10 x12.5 / x81.6 -> x1.45 / x2.0 (target <= x5); unable to cover 1200 px in 1800 ticks 20.2 % / 45.4 % -> 0 % (target <= 10 %); p50 1.33 / 0.81 -> 2.82 / 2.95 px/tick.
- Calibration (time-boxed, 2 adjustments, 5 seeds small / large success): start 0.8/3.0/0.013 -> 0.095/0.023; adj 1 1.0/4.0/0.008 -> 0.168/0.055; adj 2 (chosen) 1.0/5.0/0.008 -> 0.218/0.033.
- Success (pop 16, 30 gens, last 5 gens): small 0.207 -> 0.218; large 0.046 -> 0.033 on 5 seeds — documented exception: within noise (5-seed SE ~0.05 small, ~0.01-0.02 large), on 10 seeds large 0.027 -> 0.060 and small 0.150 -> 0.193. Best fitness large 934 -> 1427. Throughput pop 300 standard 857k -> 882k ticks/s.
- Tests: speed band for extreme bodies within 2 %, g monotone, turning at rest. Golden master unchanged (physics only), `MODEL_VERSION` unchanged, CI reference regenerated. `cargo test` 103 passed (1 ignored), 0 warnings.
- Ecology part (RV-S07, food density in the Ocean) is out of the training track and stays for the Ocean phase (TCK-169).
