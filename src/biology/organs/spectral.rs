//! Spectral distance sense shared by eye, nose and thermal organ: a tuning curve over 5 channels, a view cone and a
//! range; sees the plant spore `foods[0]` and the meat spore `foods[1]` by their emission spectra.

use super::{CHANNELS_PER_ORGANELLE, DELTA_CHANNEL, Organ, SPECTRAL_CHANNELS, SenseCtx};
use crate::shared::fmath;
use crate::shared::geom;

/// Center frequencies of the 5 spectral channels (cones).
pub const CHANNEL_FREQUENCIES: [f32; SPECTRAL_CHANNELS] = [0.10, 0.30, 0.50, 0.70, 0.90];
/// Algae/plant emission per channel (blue 1.0, green 0.5).
pub const PLANT_EMISSION: [f32; SPECTRAL_CHANNELS] = [0.0, 1.0, 0.5, 0.0, 0.0];
/// Meat/prey emission per channel (IR 1.0, red 0.5).
pub const MEAT_EMISSION: [f32; SPECTRAL_CHANNELS] = [0.0, 0.0, 0.0, 0.5, 1.0];
/// Minimum channel match (sensitivity · emission) that registers a stimulus.
pub const MIN_CHANNEL_MATCH: f32 = 0.05;

/// Sensitivity of a distance-sense organ to spectral channel `c` (Gaussian-like tuning curve).
pub fn channel_sensitivity(spectral_affinity: f32, bandwidth: f32, c: usize) -> f32 {
    (1.0 - (CHANNEL_FREQUENCIES[c] - spectral_affinity).abs() / (bandwidth * 1.8 + 0.12)).clamp(0.0, 1.0)
}

/// Channels whose tuning matches plant or meat emission, plus ΔI when any does.
pub fn live_channels(organ: &Organ) -> [bool; CHANNELS_PER_ORGANELLE] {
    let mut live = [false; CHANNELS_PER_ORGANELLE];
    for c in 0..SPECTRAL_CHANNELS {
        let emission = PLANT_EMISSION[c].max(MEAT_EMISSION[c]);
        live[c] = channel_sensitivity(organ.spectral_affinity, organ.bandwidth, c) * emission > MIN_CHANNEL_MATCH;
    }
    live[DELTA_CHANNEL] = live.iter().any(|&l| l);
    live
}

/// Per channel the strongest food match inside the cone, falling off with distance and bearing; ΔI stays 0 here
/// (computed in the brain step from consecutive ticks).
pub fn sense(ctx: &SenseCtx, organ: &Organ, out: &mut [f32; CHANNELS_PER_ORGANELLE]) {
    let agent = ctx.agent;
    let range = organ.range();
    let alpha = organ.angle * (std::f32::consts::PI / 180.0);
    let half_cone = (organ.bandwidth * 1.5).max(0.1);
    let organ_power = organ.power();

    // Plant spore foods[0] and meat spore foods[1], each with its emission spectrum
    for (food, emission) in ctx.foods.iter().zip([PLANT_EMISSION, MEAT_EMISSION]) {
        let dx = food.x - agent.px;
        let dy = food.y - agent.py;
        let dist = geom::dist(agent.px, agent.py, food.x, food.y);
        if dist > range {
            continue;
        }
        let angle_rel = geom::wrap_angle(fmath::atan2(dy, dx) - agent.heading_angle);
        let delta_beta = geom::wrap_angle(angle_rel - alpha);

        if delta_beta.abs() <= half_cone {
            let falloff = (1.0 - dist / range) * fmath::cos(delta_beta);
            for c in 0..SPECTRAL_CHANNELS {
                let match_val = channel_sensitivity(organ.spectral_affinity, organ.bandwidth, c) * emission[c];
                if match_val > MIN_CHANNEL_MATCH {
                    out[c] = out[c].max(match_val * organ_power * falloff);
                }
            }
        }
    }
}
