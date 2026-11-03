//! Bristle: surface hardness and local pain (affinity < 0.25, style < 0.35).

use super::{CHANNELS_PER_ORGANELLE, Organ, OrganKind, OrganSpec, SenseCtx, common, contact};

/// Channel 0 hardness, channel 4 pain.
fn live_channels(_: &Organ) -> [bool; CHANNELS_PER_ORGANELLE] {
    [true, false, false, false, true, false]
}

fn sense(ctx: &SenseCtx, organ: &Organ, out: &mut [f32; CHANNELS_PER_ORGANELLE]) {
    let range = organ.range();
    let boundary = contact::boundary_pressure(ctx, range);
    out[0] = contact::touch_pressure(ctx, range, organ.power(), boundary);
    out[4] = contact::pain(ctx, boundary);
}

pub const SPEC: OrganSpec = OrganSpec {
    kind: OrganKind::Bristle,
    name: "Bristle",
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
