---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/state/active-task.md, tickets/closed/TCK-103-improve-petri-dish.md, tickets/closed/TCK-104-better-steering-and-neurons.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-135-wall-reflection-sign-fix.md, docs/revisit/04-simulation-model.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md]
---

# Physics

`src/shared/physics.rs`, shared by Ocean and Trainer. `sin`/`cos`/`atan2`/`powf` go through `src/shared/fmath.rs` (pure-Rust `libm`), so kinematics are bit-identical across platforms (TCK-173).

## Functions

- `step_creature_kinematics` — motor outputs → motion. Thrust `out_thrust` (motor 0) scaled by phenotype stiffness, pulse and `rules.thrustBaseMultiplier` ([[wiki/architecture/invariants]] rule 4).
- `apply_creature_physics` — drag, wall reflection, reef collision (circle-circle, push-out, normal deflection).
- `step_food_spore_physics` — spore drift and push by creatures via `SporePusher` snapshots.

## Steering model (TCK-104)

- Continuous thrust (forward/backward) and continuous body bending (motor 1, flexion).
- Heading changes only proportionally to forward speed × bending: `delta_heading = v_forward · bend · 0.015`, bend ≤ 1.2 rad → minimum turning radius `1 / (1.2 · 0.015)` = 55.6 px for every body (measured, TCK-172); a creature without forward speed cannot turn. No sideways drift, no in-place rotation. The renderer curves the spine by the bend angle.
- **Lock-on heading:** after drag and damping (`v · 0.94`) the velocity is projected back onto the heading every tick (`physics.rs:194-196`).
- **Thrust:** `stiffness · (pulse · 1000)² · thrustBaseMultiplier · eta_swim · (1 + 0.12 · limbs) · (1 + parapodia)` times `out_thrust`; mass `r^1.5 · L/25`; forward drag from radius, receptor ballast (`0.18` per organ) and stiffness (`physics.rs:113-127`). Speed therefore scales with `pulse²`.

## Measured locomotion (TCK-172)

Probe `trainer_runner probe speed`: forced outputs (thrust 1, turn 0) from rest in an open arena, terminal speed after 1,800 ticks.

| Genomes | p10 | p50 | p90 | p90/p10 | < 0.2 px/tick | cannot cover 1200 px in 1800 ticks |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| 1,000 random | 0.064 | 0.66 | 4.21 | ×65 | 24.5 % | 50.4 % |
| 34 HoF | 1.08 | 1.54 | 1.59 | ×1.5 | 2.9 % | 8.8 % |

- 51.8 % of random genomes cannot cover 200 px in a 300-tick small-room epoch.
- Turning at full bend: turn rate scales with speed (p50 0.07 °/tick for slow, 0.68 °/tick for median, 4.2 °/tick for fast random bodies); a slow body needs ≈ 1,360 ticks for 90°.
- Selection in the trainer favours fast bodies: HoF genomes are concentrated around 1.5 px/tick.

## Boundaries and collisions

- Closed basin with elastic walls (TCK-103); creature–creature bounces and creature–spore displacement.
- Wall reflection on all four walls points away from the wall: `vx = vx.abs() * restitution` (left) / `-vx.abs()` (right), same for y (TCK-135, RV-B02). Test `test_wall_contact_reflects_velocity_away_from_every_wall`.
- Wall and reef contact reflects the velocity only, not the heading; combined with the lock-on heading, the next tick projects the velocity onto the unchanged heading again.
- Measured (TCK-172 wall probe, start 2 px from the top wall, 600 ticks): with forced full thrust and no bend, 0 % of random and HoF bodies get free at 30°, 60° or 90°; contact speed p50 0.08 / 0.022 / 0.013 px/tick. With the real brain 22–30 % of random and 68 % of HoF genomes get free.
- Reefs block creatures and spores (FIX-131).

## Spatial grid

- `src/shared/spatial_grid.rs`: 80 px cells (240 × 135 for the Ocean), `insert_creature`/`insert_food`, `get_nearby_*`, `clear()`; allocated once per world (TCK-139).
- Nearest-food search does not use the grid (RV-P04).

## Known limitations

- Low-pulse/low-stiffness genomes reach near-zero terminal speed: a quarter of random genomes stay below 0.2 px/tick, speed spread ×65 (measured, TCK-172; RV-S02, [[wiki/simulation/ocean]]).
- A creature pushing into a wall stalls and cannot turn away by bending (measured, TCK-172).
- `dragLateralCoefficient` and `steerTorqueBaseMultiplier` are unread, exempt in the config read-site test (TCK-149; TCK-150/TCK-180).

## Key Takeaways

- Speed-coupled bending steering; thrust always scaled, never raw.
- Wall reflection is correct and tested on all four walls, but forced thrust into a wall never gets free.
- Terminal speed spans ×65 between p10 and p90 of random genomes; turn radius is 55.6 px for every body.
- The grid is reused; food lookup is linear.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, TCK-104, FIX-131, TCK-135, TCK-149, TCK-172, TCK-173, `docs/revisit/04-simulation-model.md`.
