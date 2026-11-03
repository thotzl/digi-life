//! Lateral line: water flow along the organ's axis (rheotaxis, turn damping) and local pain
//! (affinity < 0.25, style 0.35..0.70).

use super::{CHANNELS_PER_ORGANELLE, Organ, OrganKind, OrganSpec, SenseCtx, common, contact};
use crate::shared::fmath;

/// Channel 1 flow, channel 4 pain.
fn live_channels(_: &Organ) -> [bool; CHANNELS_PER_ORGANELLE] {
    [false, true, false, false, true, false]
}

fn sense(ctx: &SenseCtx, organ: &Organ, out: &mut [f32; CHANNELS_PER_ORGANELLE]) {
    let agent = ctx.agent;
    let organ_heading = agent.heading_angle + organ.angle * (std::f32::consts::PI / 180.0);
    let flow_x = -agent.vx;
    let flow_y = -agent.vy;
    let flow_proj = flow_x * fmath::cos(organ_heading) + flow_y * fmath::sin(organ_heading);

    let organ_rad = organ.angle * (std::f32::consts::PI / 180.0);
    let rot_flow = -agent.omega_rot * 12.0 * fmath::sin(organ_rad) * 0.4;

    // Signed flow; the input clamp to [0, 1] keeps only the head-on component. Intended (reviewed in TCK-202): each
    // organ is a directional receptor for flow arriving at its own axis; flow from behind reaches organs that face
    // backwards (organ angle up to ±170°, mirrored pairs with HOX), so a body with several organs still resolves the
    // direction, as the proprioceptor's flexion does for the bend side.
    out[1] = (flow_proj + rot_flow).clamp(-1.0, 1.0);
    out[4] = contact::pain(ctx, contact::boundary_pressure(ctx, organ.range()));
}

pub const SPEC: OrganSpec = OrganSpec {
    kind: OrganKind::LateralLine,
    name: "Lateral line",
    channel_names: ["Hardness", "Flow", "Temperature", "Strain", "Pain"],
    distance: false,
    live_channels,
    sense,
    cost_load: common::cost_load,
    limb: true,
    ballast: 1.0,
    latin_suffix: "vibrans",
    latin_rank: 2,
};
