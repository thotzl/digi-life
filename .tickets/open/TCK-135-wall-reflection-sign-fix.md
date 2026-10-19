---
id: TCK-135
title: Fix Wall Reflection Sign on Left/Top Boundaries
status: open
assigned: Claude Code
created_at: 2026-10-19
revisit: RV-B02
---

# TCK-135: Fix Wall Reflection Sign on Left/Top Boundaries

## Description
`apply_creature_physics` (`src/shared/physics.rs`) sets `vx = -vx.abs() * restitution` when `px < mean_radius` (same for `py`). The negative velocity points further into the wall, so creatures stick to left/top walls. Right/bottom walls are correct.

## Scope
- Reproduction test for all four walls (velocity points away from the wall after contact).
- Fix the sign for left/top boundaries.

## Acceptance
- New tests fail before and pass after the fix. Shared physics, so it applies to Trainer and Ocean.
