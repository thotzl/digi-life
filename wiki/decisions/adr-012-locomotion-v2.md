---
sources: [tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-171-training-track-epic.md, wiki/simulation/physics.md]
---

# ADR-012: Locomotion v2 — Speed Band, Turning at Rest, Heading Reflection at Walls

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-22
- Tickets: TCK-150 (scope decision 2026-10-21), TCK-180, TCK-149 (physics keys); epic TCK-171 phase P5 (v1.12.19)

## Context

The TCK-172 speed probe measured a terminal speed spread of ≈ ×60 at full thrust, driven by `pulse²`; 48 % of random genomes could not cover 1200 px in an 1800-tick epoch. Turning was coupled to forward speed. At a wall the velocity was reflected but not the heading, so a creature thrusting into a wall could stall at ≈ 0 forward speed and could not turn away (TCK-180). Decision of 2026-10-21 (TCK-150): the ticket carries the fix; the Ocean ecology part (RV-S07) stays for the Ocean phase.

## Decision

1. **Speed band instead of `pulse²`** (TCK-150): `g = 0.4·pulse_n + 0.4·stiffness_n + 0.2·appendages`, `v_target = locomotionSpeedMin + (locomotionSpeedMax − locomotionSpeedMin)·g`; thrust is inverted from the straight-line update. Chosen after a time-boxed calibration of two adjustments: 1.0 / 5.0 px/tick, `turnRateAtRest` 0.008.
2. **Turning at rest:** `delta_heading = bend·(0.015·v_forward + turnRateAtRest)`; `steerTorqueBaseMultiplier` and `thrustBaseMultiplier` removed.
3. **Walls: reflect the heading** at the contact normal when moving into the obstacle, same code path for the four walls and reefs (TCK-180 offered three options: reflect the heading, slide along the wall, minimal turning at zero speed). Reason for choosing reflection over sliding not recorded in the sources. `dragLateralCoefficient` removed (lock-on heading leaves no lateral velocity).

## Alternatives considered (calibration, 5 seeds, small / large success, TCK-150)

| Setting (min / max / turn at rest) | small | large |
| :--- | :--- | :--- |
| start 0.8 / 3.0 / 0.013 | 0.095 | 0.023 |
| adj 1: 1.0 / 4.0 / 0.008 | 0.168 | 0.055 |
| adj 2 (chosen): 1.0 / 5.0 / 0.008 | 0.218 | 0.033 |

## Consequences

- Speed probe: p90/p10 ×12.5 / ×81.6 → ×1.45 / ×2.0; genomes unable to cover 1200 px 20.2 % / 45.4 % → 0 % (letter soup / structured).
- Wall probe: forced full thrust freed 0 % → 100 %; with the real brain freed letter soup 38 / 28 / 42 % → 72 / 76 / 69 %, structured 33 / 24 / 36 % → 79 / 85 / 74 % (30 / 60 / 90°).
- Golden master unchanged (physics only); trainer baselines re-measured. Large-room success on 5 seeds 0.046 → 0.033 was documented as within noise (10 seeds: 0.027 → 0.060).
- Current physics: [[wiki/simulation/physics]].
