use crate::shared::types::{CreatureAgent, FoodSpore, AppConfig};
use crate::shared::map_generator::ProceduralObstacle;

pub fn step_food_spore_physics(
    pellet: &mut FoodSpore,
    nearby_creatures: &[CreatureAgent],
    obstacles: &[ProceduralObstacle],
    world_width: f32,
    world_height: f32,
) {
    let spore_radius = 8.0;

    // 1. Spore physical displacement impulse pushes from nearby creatures (Impulse bounce)
    for agent in nearby_creatures {
        let mean_radius = agent.phenotype.spinal_harmonics.mean_radius;
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

pub fn step_creature_kinematics(
    agent: &mut CreatureAgent,
    out_thrust: f32,
    out_left: f32,
    app_config: &AppConfig,
    obstacles: &[ProceduralObstacle],
    world_width: f32,
    world_height: f32,
) -> bool {
    let stiffness = agent.phenotype.stiffness;
    let pulse = agent.phenotype.pulse_speed;
    let mean_radius = agent.phenotype.spinal_harmonics.mean_radius;
    let base_length = agent.phenotype.spinal_harmonics.base_length;

    // 1. Calculate physical thrust scaled by biological characteristics (pulse speed, stiffness)
    let mut thrust_mag = stiffness * (pulse * 1000.0 * pulse * 1000.0) * app_config.rules.thrust_base_multiplier;
    let wave_phase = agent.phenotype.wave_phase;
    let eta_swim = ((base_length / (mean_radius * 3.5)) * wave_phase.sin().max(0.01) * stiffness).clamp(0.1, 3.2);
    thrust_mag *= eta_swim;

    let limbs_count = agent.phenotype.organelles.iter().filter(|o| o.expression_style >= 0.72).count() as f32;
    thrust_mag *= 1.0 + limbs_count * 0.12;
    thrust_mag *= 1.0 + agent.phenotype.spinal_harmonics.parapodia_amp * 1.0;

    let net_thrust_force = out_thrust * thrust_mag;

    // 2. Mass & Ballast
    let mass = mean_radius.powf(1.5) * (base_length / 25.0);
    let receptor_ballast = agent.phenotype.organelles.len() as f32 * app_config.rules.receptor_ballast_scale;
    let drag_forward = (mean_radius * app_config.rules.drag_forward_coefficient + receptor_ballast) 
        * (1.0 - stiffness * app_config.rules.drag_forward_stiffness_decay);

    // 3. Apply native boundary reflections and physics kinematics
    apply_creature_physics(
        agent,
        net_thrust_force,
        out_left,
        mass,
        drag_forward,
        0.0,
        0.0,
        obstacles,
        world_width,
        world_height,
    )
}

pub fn apply_creature_physics(
    agent: &mut CreatureAgent,
    net_thrust_force: f32,
    out_bending: f32,
    mass: f32,
    drag_forward: f32,
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
                let bounce_impulse = -vel_dot_norm * (1.0 + restitution);
                agent.vx += bounce_impulse * norm_x;
                agent.vy += bounce_impulse * norm_y;
            }
            hit_wall = true; // Treats reef collisions as a wall collision for cooldown/penalties!
        }
    }

    hit_wall
}
