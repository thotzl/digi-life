//! Thermal organ: long-wave (red/IR) distance sense (affinity 0.65..0.80).

use super::{OrganKind, OrganSpec, common, spectral};

pub const SPEC: OrganSpec = OrganSpec {
    kind: OrganKind::Thermal,
    name: "Thermal organ",
    channel_names: ["UV heat", "Blue heat", "Green heat", "Red heat", "IR heat"],
    distance: true,
    live_channels: spectral::live_channels,
    sense: spectral::sense,
    cost_load: common::cost_load,
    // TCK-202: distance senses with a high expression style count as limbs (model fix pending)
    limb: common::limb_by_style,
    ballast: 1.0,
    latin_suffix: "thermum",
    latin_rank: 3,
};
