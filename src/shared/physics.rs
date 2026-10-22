use crate::shared::types::{CreatureAgent, FoodSpore, AppConfig};
use crate::shared::map_generator::ProceduralObstacle;
use crate::shared::fmath;

/// Kinematic snapshot of a creature that can push food spores (avoids cloning whole agents).
#[derive(Debug, Clone, Copy)]
pub struct SporePusher {
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub mean_radius: f32,
}

impl From<&CreatureAgent> for SporePusher {
    fn from(agent: &CreatureAgent) -> Self {
        Self {
            px: agent.px,
            py: agent.py,
            vx: agent.vx,
            vy: agent.vy,
            mean_radius: agent.phenotype.spinal_harmonics.mean_radius,
        }
    }
}

pub fn step_food_spore_physics(
    pellet: &mut FoodSpore,
    nearby_creatures: &[SporePusher],
    obstacles: &[ProceduralObstacle],
    world_width: f32,
    world_height: f32,
) {
    let spore_radius = 8.0;

    // 1. Spore physical displacement impulse pushes from nearby creatures (Impulse bounce)
    for agent in nearby_creatures {
        let mean_radius = agent.mean_radius;
        let push_radius = mean_radius + spore_radius;
        let pdx = pellet.x - agent.px;
        let pdy = pellet.y - agent.py;
        let pd = (pdx * pdx + pdy * pdy).sqrt();

        if pd < push_radius && pd > 0.1 {
            let overlap = push_radius - pd;
            let nx = pdx / pd;
            let ny = pdy / pd;

            pellet.x += nx * overlap;
            pellet.y += ny * overlap;

            pellet.x = pellet.x.clamp(8.0, world_width - 8.0);
            pellet.y = pellet.y.clamp(8.0, world_height - 8.0);

            pellet.vx = agent.vx + nx * 2.0;
            pellet.vy = agent.vy + ny * 2.0;
        }
    }

    // 2. Drift, friction and boundary bounces
    pellet.x += pellet.vx;
    pellet.y += pellet.vy;
    pellet.vx *= 0.92;
    pellet.vy *= 0.92;

    if pellet.x < 8.0 { pellet.x = 8.0; pellet.vx = pellet.vx.abs(); }
    else if pellet.x > world_width - 8.0 { pellet.x = world_width - 8.0; pellet.vx = -pellet.vx.abs(); }

    if pellet.y < 8.0 { pellet.y = 8.0; pellet.vy = pellet.vy.abs(); }
    else if pellet.y > world_height - 8.0 { pellet.y = world_height - 8.0; pellet.vy = -pellet.vy.abs(); }

    // 3. Collision with circular obstacles (reefs)
    for obs in obstacles {
        let dx = pellet.x - obs.x;
        let dy = pellet.y - obs.y;
        let dist = (dx*dx + dy*dy).sqrt();
        let min_dist = spore_radius + obs.radius;
        if dist < min_dist {
            let overlap = min_dist - dist;
            let push_x = if dist > 0.1 { (dx / dist) * overlap } else { overlap };
            let push_y = if dist > 0.1 { (dy / dist) * overlap } else { 0.0 };
            
            pellet.x = (pellet.x + push_x).clamp(8.0, world_width - 8.0);
            pellet.y = (pellet.y + push_y).clamp(8.0, world_height - 8.0);
            
            // Simple velocity reversal deflection
            let norm_x = if dist > 0.1 { dx / dist } else { 1.0 };
            let norm_y = if dist > 0.1 { dy / dist } else { 0.0 };
            let vel_dot_norm = pellet.vx * norm_x + pellet.vy * norm_y;
            if vel_dot_norm < 0.0 {
                let bounce_impulse = -vel_dot_norm * 1.5;
                pellet.vx += bounce_impulse * norm_x;
                pellet.vy += bounce_impulse * norm_y;
            }
        }
    }
}

/// Trait ranges of `parse_genome` (`src/biology/dna.rs`): pulse 0.0015 + h * 0.0075, stiffness 0.15 + h * 0.85,
/// parapodia amplitude h * 0.45.
const PULSE_MIN: f32 = 0.0015;
const PULSE_SPAN: f32 = 0.0075;
const STIFFNESS_MIN: f32 = 0.15;
const STIFFNESS_SPAN: f32 = 0.85;
const PARAPODIA_MAX: f32 = 0.45;
/// Velocity damping per tick in `apply_creature_physics`
const VELOCITY_DAMPING: f32 = 0.94;

/// Normalised locomotion drive `g` in [0, 1] (TCK-150): 0.4 pulse + 0.4 stiffness + 0.2 appendages, each
/// normalised to its genome range; appendages = half parapodia amplitude, a quarter per limb (at most two).
/// Monotone in pulse and stiffness; limbs and parapodia are folded in instead of a multiplicative bonus.
pub fn locomotion_drive(phenotype: &crate::biology::dna::CreaturePhenotype) -> f32 {
    let pulse = ((phenotype.pulse_speed - PULSE_MIN) / PULSE_SPAN).clamp(0.0, 1.0);
    let stiffness = ((phenotype.stiffness - STIFFNESS_MIN) / STIFFNESS_SPAN).clamp(0.0, 1.0);
    let limbs = phenotype.organelles.iter().filter(|o| o.expression_style >= 0.72).count().min(2) as f32;
    let appendages = (0.5 * phenotype.spinal_harmonics.parapodia_amp / PARAPODIA_MAX + 0.25 * limbs).clamp(0.0, 1.0);
    (0.4 * pulse + 0.4 * stiffness + 0.2 * appendages).clamp(0.0, 1.0)
}

/// Terminal speed at full thrust (px/tick): `locomotionSpeedMin + (locomotionSpeedMax - locomotionSpeedMin) * g`.
pub fn target_terminal_speed(phenotype: &crate::biology::dna::CreaturePhenotype, app_config: &AppConfig) -> f32 {
    let (min, max) = (app_config.rules.locomotion_speed_min, app_config.rules.locomotion_speed_max);
    min + (max - min) * locomotion_drive(phenotype)
}

/// Mass and forward drag coefficient of a body (receptor ballast raises drag, stiffness lowers it).
fn mass_and_drag(phenotype: &crate::biology::dna::CreaturePhenotype, app_config: &AppConfig) -> (f32, f32) {
    let mean_radius = phenotype.spinal_harmonics.mean_radius;
    let mass = fmath::powf(mean_radius, 1.5) * (phenotype.spinal_harmonics.base_length / 25.0);
    let receptor_ballast = phenotype.organelles.len() as f32 * app_config.rules.receptor_ballast_scale;
    let drag_forward = (mean_radius * app_config.rules.drag_forward_coefficient + receptor_ballast)
        * (1.0 - phenotype.stiffness * app_config.rules.drag_forward_stiffness_decay);
    (mass, drag_forward)
}

/// Thrust that makes full thrust settle at `target_speed`. Straight-line update of `apply_creature_physics`:
/// `v' = 0.94 * (v + (F - d * v) / m)`; its fixed point `v = 0.94 * F / (0.06 * m + 0.94 * d)` inverted gives
/// `F = v * (0.06 * m + 0.94 * d) / 0.94`.
pub fn thrust_for_terminal_speed(target_speed: f32, mass: f32, drag_forward: f32) -> f32 {
    target_speed * ((1.0 - VELOCITY_DAMPING) * mass + VELOCITY_DAMPING * drag_forward) / VELOCITY_DAMPING
}

pub fn step_creature_kinematics(
    agent: &mut CreatureAgent,
    out_thrust: f32,
    out_left: f32,
    app_config: &AppConfig,
    obstacles: &[ProceduralObstacle],
    world_width: f32,
    world_height: f32,
) -> bool {
    // 1. Thrust from the genome's speed band (TCK-150): full thrust reaches the target terminal speed
    let (mass, drag_forward) = mass_and_drag(&agent.phenotype, app_config);
    let thrust_mag = thrust_for_terminal_speed(target_terminal_speed(&agent.phenotype, app_config), mass, drag_forward);
    let net_thrust_force = out_thrust * thrust_mag;

    // 2. Apply native boundary reflections and physics kinematics
    apply_creature_physics(
        agent,
        net_thrust_force,
        out_left,
        mass,
        drag_forward,
        app_config.rules.turn_rate_at_rest,
        0.0,
        0.0,
        obstacles,
        world_width,
        world_height,
    )
}

/// Reflects the heading at a contact normal `(nx, ny)` (unit, pointing away from the obstacle), like the velocity.
/// Called only when the velocity moved into the obstacle, so the lock-on projection of the next tick keeps the
/// creature moving away (forward thrust: heading now points away; backward thrust: heading points in, motion away).
fn reflect_heading(agent: &mut CreatureAgent, nx: f32, ny: f32) {
    let (hx, hy) = (fmath::cos(agent.heading_angle), fmath::sin(agent.heading_angle));
    let dot = hx * nx + hy * ny;
    agent.heading_angle = fmath::atan2(hy - 2.0 * dot * ny, hx - 2.0 * dot * nx);
}

pub fn apply_creature_physics(
    agent: &mut CreatureAgent,
    net_thrust_force: f32,
    out_bending: f32,
    mass: f32,
    drag_forward: f32,
    turn_rate_at_rest: f32,
    external_force_x: f32,
    external_force_y: f32,
    obstacles: &[ProceduralObstacle],
    world_width: f32,
    world_height: f32,
) -> bool {
    let pheno = &agent.phenotype;
    let stiffness = pheno.stiffness;

    // 1. Biomorphic flexion (body bending) steering kinematics:
    let max_flexion = 1.2; // approx 68 degrees max bend
    let target_bending = out_bending * (max_flexion / stiffness.max(0.2));

    // Smooth muscle stiffness body bending interpolation
    let muscle_interpolation_rate = 0.20;
    agent.bend_angle = agent.bend_angle * (1.0 - muscle_interpolation_rate) + target_bending * muscle_interpolation_rate;
    agent.bend_angle = agent.bend_angle.clamp(-max_flexion, max_flexion);

    // 2. Fluid drag & thrust forces
    let v_forward = agent.vx * fmath::cos(agent.heading_angle) + agent.vy * fmath::sin(agent.heading_angle);
    let fx = net_thrust_force * fmath::cos(agent.heading_angle);
    let fy = net_thrust_force * fmath::sin(agent.heading_angle);

    // Turning: a speed-coupled part (curvature) plus a base rate that also works at rest (TCK-150)
    let curvature_factor = 0.015;
    let delta_heading = agent.bend_angle * (v_forward * curvature_factor + turn_rate_at_rest);
    agent.heading_angle += delta_heading;
    agent.heading_angle = fmath::atan2(fmath::sin(agent.heading_angle), fmath::cos(agent.heading_angle));

    // omegaRot acts as an alias for visual bending amount in the client renderer
    agent.omega_rot = agent.bend_angle / 12.0;

    // Fluid friction drag calculation
    let drag_force_forward = -drag_forward * v_forward;
    let ax = (fx + drag_force_forward * fmath::cos(agent.heading_angle)) / mass;
    let ay = (fy + drag_force_forward * fmath::sin(agent.heading_angle)) / mass;

    agent.vx = (agent.vx + ax) * VELOCITY_DAMPING;
    agent.vy = (agent.vy + ay) * VELOCITY_DAMPING;

    // Lock-on heading movement (No slip!)
    let net_speed = agent.vx * fmath::cos(agent.heading_angle) + agent.vy * fmath::sin(agent.heading_angle);
    agent.vx = net_speed * fmath::cos(agent.heading_angle);
    agent.vy = net_speed * fmath::sin(agent.heading_angle);

    // Apply external environmental forces (e.g. currents)
    agent.vx += external_force_x;
    agent.vy += external_force_y;

    // Boundary bounces (Hard boundaries, aligned with config rules!)
    let app_config = crate::shared::types::AppConfig::global();
    let mean_radius = pheno.spinal_harmonics.mean_radius;
    let restitution = app_config.rules.elastic_wall_restitution;
    let mut hit_wall = false;

    // Wall contact (TCK-180): velocity and heading are reflected at the wall normal when moving into the wall
    agent.px += agent.vx;
    if agent.px < mean_radius {
        agent.px = mean_radius;
        if agent.vx < 0.0 { reflect_heading(agent, 1.0, 0.0); }
        agent.vx = agent.vx.abs() * restitution;
        hit_wall = true;
    } else if agent.px > world_width - mean_radius {
        agent.px = world_width - mean_radius;
        if agent.vx > 0.0 { reflect_heading(agent, -1.0, 0.0); }
        agent.vx = -agent.vx.abs() * restitution;
        hit_wall = true;
    }

    agent.py += agent.vy;
    if agent.py < mean_radius {
        agent.py = mean_radius;
        if agent.vy < 0.0 { reflect_heading(agent, 0.0, 1.0); }
        agent.vy = agent.vy.abs() * restitution;
        hit_wall = true;
    } else if agent.py > world_height - mean_radius {
        agent.py = world_height - mean_radius;
        if agent.vy > 0.0 { reflect_heading(agent, 0.0, -1.0); }
        agent.vy = -agent.vy.abs() * restitution;
        hit_wall = true;
    }

    // Circular obstacle (reef) collisions
    for obs in obstacles {
        let dx = agent.px - obs.x;
        let dy = agent.py - obs.y;
        let dist = (dx*dx + dy*dy).sqrt();
        let min_dist = mean_radius + obs.radius;
        if dist < min_dist {
            let overlap = min_dist - dist;
            let push_x = if dist > 0.1 { (dx / dist) * overlap } else { overlap };
            let push_y = if dist > 0.1 { (dy / dist) * overlap } else { 0.0 };
            
            agent.px = (agent.px + push_x).clamp(mean_radius, world_width - mean_radius);
            agent.py = (agent.py + push_y).clamp(mean_radius, world_height - mean_radius);
            
            // Deflect/bounce velocity with restitution
            let norm_x = if dist > 0.1 { dx / dist } else { 1.0 };
            let norm_y = if dist > 0.1 { dy / dist } else { 0.0 };
            let vel_dot_norm = agent.vx * norm_x + agent.vy * norm_y;
            if vel_dot_norm < 0.0 {
                reflect_heading(agent, norm_x, norm_y);
                let bounce_impulse = -vel_dot_norm * (1.0 + restitution);
                agent.vx += bounce_impulse * norm_x;
                agent.vy += bounce_impulse * norm_y;
            }
            hit_wall = true; // Treats reef collisions as a wall collision for cooldown/penalties!
        }
    }

    hit_wall
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::creature_factory::{spawn_creature, SpawnOptions};

    fn agent_moving(px: f32, py: f32, heading_angle: f32, speed: f32) -> CreatureAgent {
        let mut agent = spawn_creature("EYEABCDEFEN", SpawnOptions {
            id: 1, px, py, heading_angle, generation: 1, methylations: None, synapse_weights: None,
        });
        agent.vx = speed * heading_angle.cos();
        agent.vy = speed * heading_angle.sin();
        agent
    }

    fn step_without_forces(agent: &mut CreatureAgent, size: f32) -> bool {
        apply_creature_physics(agent, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, &[], size, size)
    }

    #[test]
    fn test_spore_push_from_snapshot_matches_full_agent() {
        let agent = agent_moving(500.0, 500.0, 0.3, 2.0);
        let start = FoodSpore { id: 1, type_id: 1, x: 505.0, y: 503.0, amount: 15.0, vx: 0.0, vy: 0.0 };

        let mut expected = start.clone();
        // Reference values computed with the pre-snapshot implementation (agent at (500,500), v=(2cos0.3, 2sin0.3))
        let r = agent.phenotype.spinal_harmonics.mean_radius;
        let (dx, dy) = (5.0f32, 3.0f32);
        let d = (dx * dx + dy * dy).sqrt();
        let overlap = r + 8.0 - d;
        expected.x += dx / d * overlap;
        expected.y += dy / d * overlap;
        expected.vx = agent.vx + dx / d * 2.0;
        expected.vy = agent.vy + dy / d * 2.0;
        expected.x += expected.vx;
        expected.y += expected.vy;
        expected.vx *= 0.92;
        expected.vy *= 0.92;

        let mut pellet = start.clone();
        step_food_spore_physics(&mut pellet, &[SporePusher::from(&agent)], &[], 1000.0, 1000.0);

        assert!((pellet.x - expected.x).abs() < 1e-4 && (pellet.y - expected.y).abs() < 1e-4);
        assert!((pellet.vx - expected.vx).abs() < 1e-4 && (pellet.vy - expected.vy).abs() < 1e-4);
    }

    #[test]
    fn test_wall_contact_reflects_velocity_away_from_every_wall() {
        let size = 1000.0;
        let r = agent_moving(0.0, 0.0, 0.0, 0.0).phenotype.spinal_harmonics.mean_radius;
        let pi = std::f32::consts::PI;

        // Left wall: moving towards -x
        let mut left = agent_moving(r + 1.0, size / 2.0, pi, 5.0);
        assert!(step_without_forces(&mut left, size));
        assert!(left.vx > 0.0, "left wall must reflect to +x, got vx = {}", left.vx);
        assert!(left.px >= r);

        // Top wall: moving towards -y
        let mut top = agent_moving(size / 2.0, r + 1.0, -pi / 2.0, 5.0);
        assert!(step_without_forces(&mut top, size));
        assert!(top.vy > 0.0, "top wall must reflect to +y, got vy = {}", top.vy);
        assert!(top.py >= r);

        // Right wall: moving towards +x
        let mut right = agent_moving(size - r - 1.0, size / 2.0, 0.0, 5.0);
        assert!(step_without_forces(&mut right, size));
        assert!(right.vx < 0.0, "right wall must reflect to -x, got vx = {}", right.vx);
        assert!(right.px <= size - r);

        // Bottom wall: moving towards +y
        let mut bottom = agent_moving(size / 2.0, size - r - 1.0, pi / 2.0, 5.0);
        assert!(step_without_forces(&mut bottom, size));
        assert!(bottom.vy < 0.0, "bottom wall must reflect to -y, got vy = {}", bottom.vy);
        assert!(bottom.py <= size - r);
    }

    fn speed_of(agent: &CreatureAgent) -> f32 {
        (agent.vx * agent.vx + agent.vy * agent.vy).sqrt()
    }

    /// Body with the given locomotion traits (limb flag sets every organ to limb or non-limb expression).
    fn body(pulse: f32, stiffness: f32, parapodia: f32, limbs: bool, radius: f32, length: f32, organs: usize) -> CreatureAgent {
        let mut agent = agent_moving(500.0, 500.0, 0.0, 0.0);
        let p = &mut agent.phenotype;
        p.pulse_speed = pulse;
        p.stiffness = stiffness;
        p.spinal_harmonics.parapodia_amp = parapodia;
        p.spinal_harmonics.mean_radius = radius;
        p.spinal_harmonics.base_length = length;
        let template = p.organelles.first().cloned().expect("test genome has an organ");
        p.organelles = vec![template; organs];
        for organ in p.organelles.iter_mut() {
            organ.expression_style = if limbs { 0.9 } else { 0.1 };
        }
        agent
    }

    #[test]
    fn test_full_thrust_terminal_speed_stays_in_config_band() {
        let config = AppConfig::global();
        let (min, max) = (config.rules.locomotion_speed_min, config.rules.locomotion_speed_max);
        // Slowest and fastest locomotion traits, each on light/heavy and few/many-organ bodies
        for &(pulse, stiffness, parapodia, limbs, expected) in &[(0.0015, 0.15, 0.0, false, min), (0.009, 1.0, 0.45, true, max), (0.005, 0.6, 0.2, false, f32::NAN)] {
            for &(radius, length, organs) in &[(16.0, 90.0, 2), (28.0, 200.0, 2), (16.0, 200.0, 12), (28.0, 90.0, 12)] {
                let mut agent = body(pulse, stiffness, parapodia, limbs, radius, length, organs);
                for _ in 0..1800 {
                    step_creature_kinematics(&mut agent, 1.0, 0.0, config, &[], 1.0e7, 1.0e7);
                }
                let v = speed_of(&agent);
                assert!(v >= min * 0.98 && v <= max * 1.02, "terminal speed {v} outside [{min}, {max}] (pulse {pulse}, r {radius}, L {length}, organs {organs})");
                if expected.is_finite() {
                    assert!((v - expected).abs() <= 0.02 * expected, "extreme genome reaches {v}, expected {expected}");
                }
            }
        }
    }

    #[test]
    fn test_locomotion_drive_is_monotone_in_pulse_and_stiffness() {
        let base = body(0.004, 0.5, 0.1, false, 20.0, 120.0, 2);
        let g = locomotion_drive(&base.phenotype);
        assert!(locomotion_drive(&body(0.006, 0.5, 0.1, false, 20.0, 120.0, 2).phenotype) > g);
        assert!(locomotion_drive(&body(0.004, 0.7, 0.1, false, 20.0, 120.0, 2).phenotype) > g);
    }

    #[test]
    fn test_bending_turns_the_heading_at_rest() {
        let config = AppConfig::global();
        let mut agent = agent_moving(500.0, 500.0, 0.0, 0.0);
        for _ in 0..30 {
            step_creature_kinematics(&mut agent, 0.0, 1.0, config, &[], 1000.0, 1000.0);
        }
        assert_eq!(speed_of(&agent), 0.0, "bending alone must not move the body");
        assert!(agent.heading_angle > 0.1, "full bend at rest must turn the heading, got {}", agent.heading_angle);
        let mut mirrored = agent_moving(500.0, 500.0, 0.0, 0.0);
        for _ in 0..30 {
            step_creature_kinematics(&mut mirrored, 0.0, -1.0, config, &[], 1000.0, 1000.0);
        }
        assert!((mirrored.heading_angle + agent.heading_angle).abs() < 1e-5, "turning at rest must be symmetric");
    }

    #[test]
    fn test_wall_contact_reflects_heading_and_forced_thrust_leaves_every_wall() {
        let config = AppConfig::global();
        let size = 1000.0;
        let r = agent_moving(0.0, 0.0, 0.0, 0.0).phenotype.spinal_harmonics.mean_radius;
        let pi = std::f32::consts::PI;
        // (start x, start y, outward normal of the wall)
        let walls = [(r + 2.0, size / 2.0, (1.0, 0.0)), (size - r - 2.0, size / 2.0, (-1.0, 0.0)), (size / 2.0, r + 2.0, (0.0, 1.0)), (size / 2.0, size - r - 2.0, (0.0, -1.0))];
        for &(x, y, (nx, ny)) in &walls {
            for &angle_deg in &[30.0f32, 60.0, 90.0] {
                // Heading into the wall at `angle_deg` to its surface
                let inward = fmath::atan2(-ny, -nx);
                let heading = inward + (90.0 - angle_deg).to_radians();
                let mut agent = agent_moving(x, y, heading, 0.0);
                let mut contact = false;
                let mut freed = false;
                for _ in 0..600 {
                    if step_creature_kinematics(&mut agent, 1.0, 0.0, config, &[], size, size) && !contact {
                        contact = true;
                        let (hx, hy) = (agent.heading_angle.cos(), agent.heading_angle.sin());
                        assert!(hx * nx + hy * ny > 0.0, "heading must point away from the wall after contact (normal {nx},{ny}, angle {angle_deg})");
                    }
                    let distance_from_wall = (agent.px - x) * nx + (agent.py - y) * ny;
                    if distance_from_wall > 50.0 {
                        freed = true;
                        break;
                    }
                }
                assert!(contact && freed, "forced thrust must leave the wall (normal {nx},{ny}, angle {angle_deg}, pi {pi})");
            }
        }
    }

    #[test]
    fn test_reef_contact_reflects_heading_like_a_wall() {
        let config = AppConfig::global();
        let reef = ProceduralObstacle { id: 1, x: 600.0, y: 500.0, radius: 60.0, obstacle_type: "rock".to_string(), color: String::new(), vertices: Vec::new() };
        let r = agent_moving(0.0, 0.0, 0.0, 0.0).phenotype.spinal_harmonics.mean_radius;
        // Head-on and oblique approach towards the reef centre
        for &offset_y in &[0.0f32, 25.0] {
            let start_x = reef.x - reef.radius - r - 2.0;
            let mut agent = agent_moving(start_x, reef.y + offset_y, 0.0, 0.0);
            let mut contact = false;
            for _ in 0..300 {
                if step_creature_kinematics(&mut agent, 1.0, 0.0, config, std::slice::from_ref(&reef), 1.0e4, 1.0e4) && !contact {
                    contact = true;
                    let (nx, ny) = ((agent.px - reef.x), (agent.py - reef.y));
                    let (hx, hy) = (agent.heading_angle.cos(), agent.heading_angle.sin());
                    assert!(hx * nx + hy * ny > 0.0, "heading must point away from the reef after contact (offset {offset_y})");
                }
            }
            let gap = ((agent.px - reef.x).powi(2) + (agent.py - reef.y).powi(2)).sqrt() - reef.radius - r;
            assert!(contact && gap > 50.0, "forced thrust must leave the reef (offset {offset_y}, gap {gap})");
        }
    }
}
