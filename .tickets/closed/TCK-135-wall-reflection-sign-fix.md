---
id: TCK-135
title: Fix Wall Reflection Sign on Left/Top Boundaries
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
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

## Verification
- Reproduction test `shared::physics::tests::test_wall_contact_reflects_velocity_away_from_every_wall` (`src/shared/physics.rs`) failed before the fix (`left wall must reflect to +x, got vx = -2.35`).
- Fix: left/top boundary use `vx = vx.abs() * restitution` / `vy = vy.abs() * restitution` (`src/shared/physics.rs`, `apply_creature_physics`).
- `cargo test`: 35 passed, 0 failed.
- Effect: shared physics, so creatures no longer stick to left/top walls in Trainer and Ocean. Trainer fitness of new runs is not directly comparable with older runs (fewer wall collisions).
