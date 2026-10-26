---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/state/active-task.md, tickets/closed/TCK-103-improve-petri-dish.md, tickets/closed/TCK-104-better-steering-and-neurons.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-135-wall-reflection-sign-fix.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md]
---

# Physics

`src/shared/physics.rs`, shared by Ocean and Trainer. `sin`/`cos`/`atan2`/`powf` go through `src/shared/fmath.rs` (pure-Rust `libm`), so kinematics are bit-identical across platforms (TCK-173).

## Functions

- `step_creature_kinematics` — motor outputs → motion. Derives the thrust magnitude from the genome's speed band and multiplies it by `out_thrust` (motor 0) ([[wiki/architecture/invariants]] rule 4).
- `locomotion_drive`, `target_terminal_speed`, `thrust_for_terminal_speed` — speed band and thrust inversion (TCK-150, below).
- `apply_creature_physics` — bending, turning, drag, lock-on heading, wall reflection, reef collision (circle-circle, push-out, normal deflection).
- `step_food_spore_physics` — spore drift and push by creatures via `SporePusher` snapshots.

## Steering model (TCK-104, TCK-150)

- Continuous thrust (forward/backward) and continuous body bending (motor 1, flexion). Bend follows `out_bend · 1.2 / max(stiffness, 0.2)` at rate 0.2 per tick, clamped to ±1.2 rad; full bend reaches 1.2 rad for every body.
- **Turning:** `delta_heading = bend · (0.015 · v_forward + turnRateAtRest)` with `turnRateAtRest` 0.008 rad/tick (TCK-150). A body at rest turns 90° in ≈ 165 ticks at full bend; the turning radius at speed `v` is `1 / (1.2 · (0.015 + 0.008 / v))`, measured p50 44–49 px for random bodies (upper bound 55.6 px as `v → ∞`). Bending alone does not move the body. The renderer curves the spine by the bend angle.
- **Lock-on heading:** after drag and damping (`v · 0.94`) the velocity is projected back onto the heading every tick; there is no lateral velocity.

## Thrust: speed band (TCK-150)

Thrust is set so that full thrust settles at a terminal speed chosen from the genome; mass and drag only change the acceleration.

- **Drive** `g = 0.4 · pulse_n + 0.4 · stiffness_n + 0.2 · appendages`, clamped to `[0, 1]`, with each trait normalised to its genome range (`parse_genome`): `pulse_n = (pulse − 0.0015) / 0.0075`, `stiffness_n = (stiffness − 0.15) / 0.85`, `appendages = clamp(0.5 · parapodia / 0.45 + 0.25 · min(limbs, 2), 0, 1)` (limb = organ with `expression_style ≥ 0.72`). Monotone in pulse and stiffness.
- **Target speed** `v_target = locomotionSpeedMin + (locomotionSpeedMax − locomotionSpeedMin) · g` = 1.0 … 5.0 px/tick.
- **Thrust** `F = v_target · (0.06 · m + 0.94 · d) / 0.94`, the inverted fixed point of the straight-line update `v' = 0.94 · (v + (F − d · v) / m)` (`thrust_for_terminal_speed`). Mass `m = r^1.5 · L/25`; forward drag `d = (r · dragForwardCoefficient + organs · receptorBallastScale) · (1 − stiffness · dragForwardStiffnessDecay)`.
- Applied force = `F · out_thrust`; negative `out_thrust` swims backwards at the same scale.
- `wave_phase`, body aspect and organ ballast no longer affect top speed (before: `stiffness · (pulse·1000)² · thrustBaseMultiplier · eta_swim · (1 + 0.12 · limbs) · (1 + parapodia)`, speed ∝ `pulse²`).

## Measured locomotion (TCK-150)

Probe `trainer_runner probe speed --seed tck172`: forced outputs (thrust 1, bend 0) from rest in an open arena, terminal speed after 1,800 ticks; 1,000 genomes per set. Before = P4 state (v1.12.18), after = P5. Reports kept locally in `.agents/artifacts/p5/before|after/`.

| Set | p10 | p50 | p90 | p90/p10 | < 0.2 px/tick | cannot cover 1200 px (pre-P7 large-room re-trial distance) in 1800 ticks |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| letter soup, before | 0.25 | 1.33 | 3.12 | ×12.5 | 8.9 % | 20.2 % |
| letter soup, after | 2.34 | 2.82 | 3.41 | ×1.45 | 0 % | 0 % |
| structured, before | 0.072 | 0.81 | 5.89 | ×81.6 | 23.0 % | 45.4 % |
| structured, after | 1.99 | 2.95 | 3.98 | ×2.0 | 0 % | 0 % |

- No genome fails to cover 200 px in a 300-tick small-room epoch (before 21.5 % / 46.5 %).
- Turning at full thrust and bend, p50 ticks per 90°: 31 / 27 / 23 (slow / median / fast letter soup), 35 / 26 / 20 (structured); before 352 / 66 / 29 and 1,220 / 108 / 16.
- Turning at rest (`turn_at_rest`, thrust 0, full bend): 90° in 165 ticks (p10–p90 164–167), 100 % of genomes in both sets; before impossible.
- The TCK-172 numbers on pre-P3 genomes (×65 spread, 24.5 % below 0.2 px/tick) are superseded.

## Boundaries and collisions

- Closed basin with elastic walls (TCK-103); creature–creature bounces and creature–spore displacement.
- Wall reflection on all four walls points away from the wall: `vx = vx.abs() * restitution` (left) / `-vx.abs()` (right), same for y (TCK-135, RV-B02). Test `test_wall_contact_reflects_velocity_away_from_every_wall`.
- **Heading reflection (TCK-180):** when the velocity moves into a wall or reef, `reflect_heading` mirrors the heading at the contact normal, in addition to the unchanged velocity reflection. One code path for the four walls and reefs. With forward thrust the heading then points away from the obstacle, so the next lock-on projection keeps the body moving away; with backward thrust the heading points in and the motion still goes away.
- Wall probe (`probe wall --seed tck172`, start 2 px from the top wall at rest, 600 ticks):

| | 30° | 60° | 90° |
| :--- | :--- | :--- | :--- |
| Forced full thrust, freed (before → after) | 0 % → 100 % | 0 % → 100 % | 0 % → 100 % |
| Forced, ticks to free p50 (after) | 53 | 37 | 34 |
| Real brain, letter soup (before → after) | 38 % → 72 % | 28 % → 76 % | 42 % → 69 % |
| Real brain, structured (before → after) | 33 % → 79 % | 24 % → 85 % | 36 % → 74 % |

- Reefs block creatures and spores (FIX-131).

## Spatial grid

- `src/shared/spatial_grid.rs`: 80 px cells (240 × 135 for the Ocean), `insert_creature`/`insert_food`, `get_nearby_*`, `clear()`; allocated once per world (TCK-139).
- Nearest-food search does not use the grid (RV-P04).

## Known limitations

- Narrow speed spread (p90/p10 ×1.45 / ×2.0): there is little selection pressure on speed, and body shape, `wave_phase` and organ ballast are locomotion-neutral at top speed (TCK-150).
- With the real brain 15–31 % of random genomes still do not get free from a wall within 600 ticks; since forced thrust always gets free, the remainder is brain output (thrust/bend), not mechanics.
- The trainer report has no per-trial wall-collision count; the wall probe stands in for it (TCK-180).

## Key Takeaways

- Every genome swims 1–5 px/tick at full thrust; mass and drag set only acceleration; thrust is always derived from the speed band, never raw.
- Turning works at rest (≈ 165 ticks for 90°) and at speed (radius ≈ 45 px).
- Wall and reef contact reflect velocity and heading; forced thrust into a wall always gets free.
- The grid is reused; food lookup is linear.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, TCK-104, FIX-131, TCK-135, TCK-149, TCK-150, TCK-172, TCK-173, TCK-180, `docs/revisit/04-simulation-model.md`.
