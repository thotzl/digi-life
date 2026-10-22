---
id: TCK-180
title: Wall Contact Behaviour
status: closed
priority: medium
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-172
---

# TCK-180: Wall Contact Behaviour

Part of epic TCK-171.

## Description
Hypothesis from 2026-10-21 (to be confirmed by the TCK-172 wall probe): wall contact reflects the velocity but not the heading (`src/shared/physics.rs:208-228`); the next tick projects the velocity back onto the heading (`physics.rs:194-196`, "lock-on heading, no slip"). A creature thrusting into a wall can stall at ≈ 0 forward speed, and because turning requires forward speed (`physics.rs:182`), it cannot turn away. The large-room failure fitness additionally rewards distance from the centre, i.e. swimming to the wall. Applies to the Ocean world bounds and reef collisions as well.

## Scope
- Choose one: reflect the heading on wall contact, or allow sliding along the wall (tangential velocity kept, normal component removed), or a minimal turning ability at zero speed.
- Same behaviour for world bounds and reefs (`physics.rs` obstacle block).
- Keep the 4-wall reflection test (TCK-135) green or adapt it intentionally.

## Acceptance
- TCK-172 wall probe: creatures with full thrust at an angle into a wall get free within N ticks (before/after).
- Trainer report: wall collisions per trial drop; no regression in success rate.
- `wiki/simulation/physics.md` updated.

## Verification (P5, v1.12.19)
- Chosen: reflect the heading at the contact normal (`reflect_heading`) when moving into the obstacle; same code path for the four walls and reefs; velocity reflection unchanged (`test_wall_contact_reflects_velocity_away_from_every_wall` green unmodified); new tests for forced thrust into all four walls at 30/60/90 deg and for reefs.
- Wall probe (600 ticks): forced full thrust freed 0 % -> 100 % at 30/60/90 deg (p50 53/37/34 ticks; target >= 90 %); real brain freed letter soup 38/28/42 % -> 72/76/69 %, structured 33/24/36 % -> 79/85/74 %.
- The runner report has no per-trial wall-collision count; the wall probe with the real brain stands in for "wall collisions per trial drop". Success: see TCK-150 (no regression on 10 seeds).
- `dragLateralCoefficient` removed (lock-on heading leaves no lateral velocity; TCK-149).
