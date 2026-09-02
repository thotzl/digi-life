use crate::shared::types::CreatureAgent;

pub fn apply_creature_physics(
    agent: &mut CreatureAgent,
    net_thrust_force: f32,
    out_bending: f32,
    mass: f32,
    drag_forward: f32,
    external_force_x: f32,
    external_force_y: f32,
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
    let v_forward = agent.vx * agent.heading_angle.cos() + agent.vy * agent.heading_angle.sin();
    let fx = net_thrust_force * agent.heading_angle.cos();
    let fy = net_thrust_force * agent.heading_angle.sin();

    // Kinematic Curve Turn coupling: turning is strictly dependent on forward/backward movement and flexion!
    let curvature_factor = 0.015;
    let delta_heading = v_forward * agent.bend_angle * curvature_factor;
    agent.heading_angle += delta_heading;
    agent.heading_angle = agent.heading_angle.sin().atan2(agent.heading_angle.cos());

    // omegaRot acts as an alias for visual bending amount in the client renderer
    agent.omega_rot = agent.bend_angle / 12.0;

    // Fluid friction drag calculation
    let drag_force_forward = -drag_forward * v_forward;
    let ax = (fx + drag_force_forward * agent.heading_angle.cos()) / mass;
    let ay = (fy + drag_force_forward * agent.heading_angle.sin()) / mass;

    agent.vx = (agent.vx + ax) * 0.94;
    agent.vy = (agent.vy + ay) * 0.94;

    // Lock-on heading movement (No slip!)
    let net_speed = agent.vx * agent.heading_angle.cos() + agent.vy * agent.heading_angle.sin();
    agent.vx = net_speed * agent.heading_angle.cos();
    agent.vy = net_speed * agent.heading_angle.sin();

    // Apply external environmental forces (e.g. currents)
    agent.vx += external_force_x;
    agent.vy += external_force_y;

    // Boundary bounces (Hard boundaries, aligned with config rules!)
    let app_config = crate::shared::types::AppConfig::global();
    let mean_radius = pheno.spinal_harmonics.mean_radius;
    let restitution = app_config.rules.elastic_wall_restitution;
    let mut hit_wall = false;

    // X-Axis bounds
    agent.px += agent.vx;
    if agent.px < mean_radius {
        agent.px = mean_radius;
        agent.vx = -agent.vx.abs() * restitution;
        hit_wall = true;
    } else if agent.px > world_width - mean_radius {
        agent.px = world_width - mean_radius;
        agent.vx = -agent.vx.abs() * restitution;
        hit_wall = true;
    }

    // Y-Axis bounds
    agent.py += agent.vy;
    if agent.py < mean_radius {
        agent.py = mean_radius;
        agent.vy = -agent.vy.abs() * restitution;
        hit_wall = true;
    } else if agent.py > world_height - mean_radius {
        agent.py = world_height - mean_radius;
        agent.vy = -agent.vy.abs() * restitution;
        hit_wall = true;
    }

    hit_wall
}
