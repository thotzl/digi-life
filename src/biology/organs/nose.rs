//! Nose: chemical distance sense (affinity 0.25..=0.65).

use super::{OrganKind, OrganSpec, common, spectral};

pub const SPEC: OrganSpec = OrganSpec {
    kind: OrganKind::Nose,
    name: "Nose",
    channel_names: ["Acid", "Sugar", "Ketone", "Protein", "Pheromone"],
    distance: true,
    live_channels: spectral::live_channels,
    sense: spectral::sense,
    cost_load: common::cost_load,
    limb: false,
    ballast: 1.0,
    latin_suffix: "ciliatum",
    latin_rank: 1,
};
