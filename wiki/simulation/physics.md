---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/state/active-task.md, .tickets/closed/TCK-103-improve-petri-dish.md, .tickets/closed/TCK-104-better-steering-and-neurons.md, .tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, .tickets/closed/TCK-135-wall-reflection-sign-fix.md, docs/revisit/04-simulation-model.md]
---

# Physics

`src/shared/physics.rs`, shared by Ocean and Trainer.

## Functions

- `step_creature_kinematics` — motor outputs → motion. Thrust `out_thrust` (motor 0) scaled by phenotype stiffness, pulse and `rules.thrustBaseMultiplier` ([[architecture/invariants]] rule 4).
- `apply_creature_physics` — drag, wall reflection, reef collision (circle-circle, push-out, normal deflection).
- `step_food_spore_physics` — spore drift and push by creatures via `SporePusher` snapshots.

## Steering model (TCK-104)

- Continuous thrust (forward/backward) and continuous body bending (motor 1, flexion).
- Heading changes only proportionally to forward speed × bending; no sideways drift, no in-place rotation. The renderer curves the spine by the bend angle.

## Boundaries and collisions

- Closed basin with elastic walls (TCK-103); creature–creature bounces and creature–spore displacement.
- Wall reflection fixed in TCK-135 (RV-B02): left/top used `-abs` and pushed into the wall; now `vx = vx.abs() * restitution` / `vy = vy.abs() * restitution`. Test `test_wall_contact_reflects_velocity_away_from_every_wall`.
- Reefs block creatures and spores (FIX-131).

## Spatial grid

- `src/shared/spatial_grid.rs`: 80 px cells (240 × 135 for the Ocean), `insert_creature`/`insert_food`, `get_nearby_*`, `clear()`; allocated once per world (TCK-139).
- Nearest-food search still scans all spores (RV-P04).

## Open

- Thrust scaling may make many genomes immobile (RV-S02, TCK-150), see [[simulation/ocean]].
- `dragLateralCoefficient` and `steerTorqueBaseMultiplier` are dead config (RV-S01).

## Key Takeaways

- Speed-coupled bending steering; thrust always scaled, never raw.
- Wall bug fixed and tested on all four walls.
- The grid is reused; food lookup is not yet grid-based.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, TCK-104, FIX-131, TCK-135, `docs/revisit/04-simulation-model.md`.
