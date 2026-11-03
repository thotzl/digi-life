//! Body traits: one decode function per body gene (`GeneRole::Body` promoters), each reading only the first gene of
//! its promoter (TCK-179, trait locality). A missing gene reads the empty payload (h = 0.5 at every locus).

use super::{HSLColor, SpinalHarmonics, Symmetry};
use crate::biology::genome::alphabet::normalize_genome;
use crate::biology::genome::payload::Payload;
use crate::biology::genome::table::{GeneTable, first_payload};

/// Insertion/deletion rate at EVO h = 1 (rates are h * MAX_INDEL_RATE).
pub const MAX_INDEL_RATE: f32 = 0.12;

/// Every value the body genes set.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BodyTraits {
    pub symmetry: Symmetry,
    pub primary_color: HSLColor,
    pub secondary_color: HSLColor,
    pub body_seed: u32,
    pub mean_radius: f32,
    pub base_length: f32,
    pub amplitudes: Vec<f32>,
    pub phases: Vec<f32>,
    pub stiffness: f32,
    pub spinal_curve: f32,
    pub spinal_curve_freq: f32,
    pub parapodia_amp: f32,
    pub parapodia_freq: f32,
    pub flattening_head: f32,
    pub pulse_speed: f32,
    pub wave_phase: f32,
    pub wiggle_amplitude: f32,
    pub stomach_capacity: f32,
    pub thermal_tolerance_min: f32,
    pub thermal_tolerance_max: f32,
    pub carnivory: f32,
    pub is_predator: bool,
    pub mature_age: u32,
    pub repro_threshold: f32,
    pub split_loss: f32,
    pub insertion_rate: f32,
    pub deletion_rate: f32,
}

type Decoder = fn(Payload, &mut BodyTraits);

/// Body genes and their decoders (test `test_every_body_promoter_has_a_decoder` checks it against the registry).
pub const BODY_GENES: &[(&str, Decoder)] = &[
    ("COL", decode_col),
    ("SIZ", decode_siz),
    ("STF", decode_stf),
    ("WAV", decode_wav),
    ("PUL", decode_pul),
    ("STM", decode_stm),
    ("TEM", decode_tem),
    ("CAR", decode_car),
    ("REP", decode_rep),
    ("EVO", decode_evo),
];

/// Symmetry, hue, saturation of the primary and lightness of the secondary colour.
fn decode_col(p: Payload, t: &mut BodyTraits) {
    let (h_sym, h_color1, h_color2) = (p.h(11), p.h(0), p.h(5));
    t.symmetry = if h_sym >= 0.5 {
        Symmetry::Quad
    } else {
        Symmetry::Vertical
    };
    t.primary_color.h = (h_color1 * 360.0).round();
    t.primary_color.s = (55.0 + h_color2 * 45.0).round();
    t.secondary_color.h = (((h_color1 * 360.0) + 180.0) % 360.0).round(); // complementary secondary color
    t.secondary_color.l = (35.0 + h_color2 * 45.0).round();
}

/// Body size, body seed, lightness of the primary and saturation of the secondary colour.
fn decode_siz(p: Payload, t: &mut BodyTraits) {
    let (h_radius, h_length, h_seed) = (p.h(0), p.h(4), p.h(8));
    t.primary_color.l = (35.0 + h_seed * 45.0).round();
    t.secondary_color.s = (55.0 + h_seed * 45.0).round();
    t.body_seed = (h_seed * 100_000.0) as u32;
    t.mean_radius = (16.0f32).max((28.0f32).min(16.0 + h_radius * 12.0));
    t.base_length = (90.0f32).max((200.0f32).min(90.0 + h_length * 110.0));
}

fn decode_stf(p: Payload, t: &mut BodyTraits) {
    t.stiffness = (0.15f32).max((1.0f32).min(0.15 + p.h(0) * 0.85));
}

/// Spinal curve, parapodia and the radius harmonics (amplitudes and phases).
fn decode_wav(p: Payload, t: &mut BodyTraits) {
    t.spinal_curve = p.h(0) * 44.0 - 22.0;
    t.spinal_curve_freq = 1.0 + (p.h(3) * 3.0).floor();
    t.parapodia_amp = p.h(6) * 0.45;
    t.parapodia_freq = 2.0 + (p.h(9) * 3.0).floor();
    t.amplitudes = (0..4).map(|j| p.h(j + 15) * 0.3 - 0.15).collect();
    t.phases = (0..4).map(|j| p.h(j + 25) * std::f32::consts::PI * 2.0).collect();
}

fn decode_pul(p: Payload, t: &mut BodyTraits) {
    t.flattening_head = p.h(0) * 1.4 - 0.4;
    t.pulse_speed = 0.0015 + p.h(2) * 0.0075;
    t.wave_phase = p.h(5) * 1.6;
    t.wiggle_amplitude = p.h(9) * 0.22;
}

fn decode_stm(p: Payload, t: &mut BodyTraits) {
    t.stomach_capacity = 50.0 + p.h(0) * 450.0;
}

fn decode_tem(p: Payload, t: &mut BodyTraits) {
    let (h_thermal_c, h_thermal_w) = (p.h(0), p.h(6));
    // TCK-202: `-5.0f32.max(x)` parses as `-(5.0f32.max(x))`, so the minimum is never above -5 (model fix pending)
    t.thermal_tolerance_min = -5.0f32.max((h_thermal_c * 60.0 - h_thermal_w * 15.0).round());
    t.thermal_tolerance_max = 105.0f32.min((h_thermal_c * 60.0 + h_thermal_w * 15.0).round());
}

fn decode_car(p: Payload, t: &mut BodyTraits) {
    t.carnivory = p.h(0);
    t.is_predator = t.carnivory >= 0.50;
}

fn decode_rep(p: Payload, t: &mut BodyTraits) {
    t.mature_age = (300.0 + p.h(0) * 2400.0).round() as u32;
    t.repro_threshold = 0.60 + p.h(4) * 0.35;
    t.split_loss = 0.05 + p.h(8) * 0.35;
}

fn decode_evo(p: Payload, t: &mut BodyTraits) {
    t.insertion_rate = p.h(0) * MAX_INDEL_RATE;
    t.deletion_rate = p.h(5) * MAX_INDEL_RATE;
}

/// All body traits of a genome.
pub fn decode_body(genes: &GeneTable) -> BodyTraits {
    let mut traits = BodyTraits::default();
    for (promoter, decode) in BODY_GENES {
        decode(genes.first_payload(promoter), &mut traits);
    }
    traits
}

/// Insertion/deletion rates exactly as `parse_genome` derives them, without compiling the full phenotype.
pub fn read_evolution_rates(genome: &str) -> (f32, f32) {
    let clean_genome = normalize_genome(genome);
    let mut traits = BodyTraits::default();
    decode_evo(first_payload(&clean_genome, "EVO"), &mut traits);
    (traits.insertion_rate, traits.deletion_rate)
}

/// Body chambers: radius peaks along the spine (at least 1).
// TCK-202: std `cos` instead of `fmath` (cross-platform determinism; switching is a model change)
pub fn chamber_count(sh: &SpinalHarmonics) -> usize {
    let mut peaks_count = 0;
    let mut was_rising = false;
    let mut prev_r = 0.0;
    let radius_at = |s: f32| -> f32 {
        let mut modulation = 0.0;
        for j in 0..4 {
            let frequency_factor = (j + 1) as f32 * std::f32::consts::PI;
            modulation += sh.amplitudes[j] * (frequency_factor * s + sh.phases[j]).cos();
        }
        sh.mean_radius * (1.0 + modulation)
    };
    for j in 0..=50 {
        let s = j as f32 / 50.0;
        let r_current = radius_at(s);
        if j > 0 {
            let is_rising = r_current > prev_r;
            if was_rising && !is_rising {
                peaks_count += 1;
            }
            was_rising = is_rising;
        }
        prev_r = r_current;
    }
    peaks_count.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::genome::promoters::{self, GeneRole};

    #[test]
    fn test_every_body_promoter_has_a_decoder() {
        let registry: Vec<&str> = promoters::with_role(GeneRole::Body).collect();
        let table: Vec<&str> = BODY_GENES.iter().map(|(p, _)| *p).collect();
        assert_eq!(registry, table);
    }
}
