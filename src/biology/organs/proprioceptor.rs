//! Proprioceptor: water temperature and spine flexion (affinity < 0.25, style >= 0.70).

use super::{common, contact, Organ, OrganKind, OrganSpec, SenseCtx, CHANNELS_PER_ORGANELLE};

/// Spine flexion that reads as a full signal (rad).
const MAX_FLEXION: f32 = 1.2;

/// Channel 2 temperature, channel 3 flexion.
fn live_channels(_: &Organ) -> [bool; CHANNELS_PER_ORGANELLE] {
    [false, false, true, true, false, false]
}

fn sense(ctx: &SenseCtx, organ: &Organ, out: &mut [f32; CHANNELS_PER_ORGANELLE]) {
    out[2] = contact::water_temperature(ctx, organ.power());
    // Signed bend [-1, 1]; the input clamp to [0, 1] keeps one side
    out[3] = (ctx.agent.bend_angle / MAX_FLEXION).clamp(-1.0, 1.0);
}

pub const SPEC: OrganSpec = OrganSpec {
    kind: OrganKind::Proprioceptor,
    name: "Proprioceptor",
    channel_names: ["Hardness", "Flow", "Temperature", "Strain", "Pain"],
    distance: false,
    live_channels,
    sense,
    cost_load: common::cost_load,
    limb: common::limb_by_style,
    ballast: 1.0,
    latin_suffix: "vibrans",
    latin_rank: 2,
};
