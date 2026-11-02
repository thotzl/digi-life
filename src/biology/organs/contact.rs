//! Contact quantities shared by the tactile kinds (bristle, lateral line, proprioceptor): wall pressure, touch,
//! water temperature and pain. Each kind reads the ones it uses.

use super::SenseCtx;

/// Wall proximity in [0, 1] within half the sensor range of the chamber edge.
pub fn boundary_pressure(ctx: &SenseCtx, range: f32) -> f32 {
    let agent = ctx.agent;
    let wall_warning_zone = range * 0.5;
    let mut boundary_pressure = 0.0;
    if agent.px < wall_warning_zone {
        boundary_pressure = 1.0 - agent.px / wall_warning_zone;
    } else if agent.px > ctx.width - wall_warning_zone {
        boundary_pressure = 1.0 - (ctx.width - agent.px) / wall_warning_zone;
    }
    if agent.py < wall_warning_zone {
        boundary_pressure = f32::max(boundary_pressure, 1.0 - agent.py / wall_warning_zone);
    } else if agent.py > ctx.height - wall_warning_zone {
        boundary_pressure = f32::max(boundary_pressure, 1.0 - (ctx.height - agent.py) / wall_warning_zone);
    }
    boundary_pressure
}

/// Mechanical hardness and texture: walls and reefs are hard (1.0), food spores soft (0.3).
pub fn touch_pressure(ctx: &SenseCtx, range: f32, organ_power: f32, boundary_pressure: f32) -> f32 {
    let agent = ctx.agent;
    let mut tactile_pressure: f32 = 0.0;
    if boundary_pressure > 0.0 {
        // Wall hardness (1.0)
        tactile_pressure = tactile_pressure.max(boundary_pressure * organ_power * 1.0);
    }

    // Spore tactile proximity (spores foods[0] and foods[1] emit soft tactile feedback of 0.3)
    for f in ctx.foods {
        let dx = f.x - agent.px;
        let dy = f.y - agent.py;
        let d = (dx * dx + dy * dy).sqrt();
        if d <= range {
            let proximity = (1.0 - d / range).clamp(0.0, 1.0);
            tactile_pressure = tactile_pressure.max(proximity * organ_power * 0.3);
        }
    }

    // Obstacle tactile proximity (reefs as circular walls with hardness 1.0)
    for obs in ctx.obstacles {
        let dx = obs.x - agent.px;
        let dy = obs.y - agent.py;
        let dist = (dx * dx + dy * dy).sqrt();
        let hard_radius = obs.radius + range;
        if dist <= hard_radius {
            let pressure = (1.0 - (dist - obs.radius).max(0.0) / range).clamp(0.0, 1.0);
            tactile_pressure = tactile_pressure.max(pressure * organ_power * 1.0);
        }
    }
    tactile_pressure
}

/// Water temperature: warm in the chamber centre, cold at the edges.
pub fn water_temperature(ctx: &SenseCtx, organ_power: f32) -> f32 {
    let agent = ctx.agent;
    let dist_from_center = ((agent.px - ctx.width / 2.0).powi(2) + (agent.py - ctx.height / 2.0).powi(2)).sqrt();
    (1.0 - dist_from_center / (ctx.width * 0.7)).clamp(0.15, 0.95) * organ_power
}

/// Local pain: fast contact with a wall.
// TCK-202: pain thresholds (speed 1.5, pressure 0.3) are unreviewed model constants.
pub fn pain(ctx: &SenseCtx, boundary_pressure: f32) -> f32 {
    let agent = ctx.agent;
    let speed = (agent.vx * agent.vx + agent.vy * agent.vy).sqrt();
    if speed > 1.5 && boundary_pressure > 0.3 { (speed * 0.3).min(1.0) } else { 0.0 }
}
