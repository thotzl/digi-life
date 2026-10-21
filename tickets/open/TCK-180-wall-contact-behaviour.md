---
id: TCK-180
title: Wall Contact Behaviour
status: open
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
