//! Eye: light-sensing distance sense (affinity >= 0.80).

use super::{OrganKind, OrganSpec, common, spectral};

pub const SPEC: OrganSpec = OrganSpec {
    kind: OrganKind::Eye,
    name: "Eye",
    channel_names: ["UV", "Blue", "Green", "Red", "IR"],
    distance: true,
    live_channels: spectral::live_channels,
    sense: spectral::sense,
    cost_load: common::cost_load,
    limb: false,
    ballast: 1.0,
    latin_suffix: "ocularium",
    latin_rank: 0,
};
