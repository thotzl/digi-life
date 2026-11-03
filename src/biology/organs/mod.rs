//! Organ registry (TCK-199): one file per organ kind (`eye.rs`, `nose.rs`, ...), each exporting a `SPEC: OrganSpec`
//! with everything the simulation needs from that kind (sensing, live channels, cost, locomotion, ballast, names).
//! `OrganKind::spec` is the one dispatch point; the exhaustive `match` there makes the compiler list a new kind's gaps.
//! Contract: `wiki/contracts/organ-kind.md`; guide: `wiki/guides/add-organ-kind.md`.

pub mod bristle;
pub mod contact;
pub mod eye;
pub mod lateral_line;
pub mod nose;
pub mod proprioceptor;
pub mod spectral;
pub mod thermal;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::biology::genome::payload::Payload;
use crate::biology::genome::promoters::{self, GeneRole};
use crate::biology::genome::table::{GeneRef, GeneTable};
use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::shared::map_generator::ProceduralObstacle;

pub use spectral::{CHANNEL_FREQUENCIES, MEAT_EMISSION, MIN_CHANNEL_MATCH, PLANT_EMISSION, channel_sensitivity};

/// Spectral receptive-field channels per organ (UV/pressure .. IR/pheromone).
pub const SPECTRAL_CHANNELS: usize = 5;
/// Channel of the temporal intensity change ΔI (distance senses only, TCK-144).
pub const DELTA_CHANNEL: usize = SPECTRAL_CHANNELS;
/// Input slots per organ: 5 spectral channels + ΔI. Input slot of organ `i`, channel `c`: `i · CHANNELS_PER_ORGANELLE + c`.
pub const CHANNELS_PER_ORGANELLE: usize = SPECTRAL_CHANNELS + 1;
/// Sensor range of an organ per unit organ scale (px): range = `scale · SENSOR_RANGE_PER_SCALE`.
pub const SENSOR_RANGE_PER_SCALE: f32 = 550.0;
/// Payload locus of an organ gene's spectral affinity (decides its OrganKind with the expression style).
pub const AFFINITY_LOCUS: usize = 1;
/// Expression style from which an organ counts as a limb in the locomotion drive (`physics::locomotion_drive`).
pub const LIMB_STYLE: f32 = 0.72;

/// Organ type, decided once at compile time from affinity and expression style (TCK-144; replaces label matching).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub enum OrganKind {
    /// Affinity >= 0.80 (light)
    Eye,
    /// Affinity 0.25..=0.65 (chemical)
    Nose,
    /// Affinity 0.65..0.80: long-wave (red/IR) distance sense
    Thermal,
    /// Affinity < 0.25, style < 0.35: pressure + pain
    Bristle,
    /// Affinity < 0.25, style 0.35..0.70: flow + pain
    LateralLine,
    /// Affinity < 0.25, style >= 0.70: temperature + spine flexion
    Proprioceptor,
}

impl OrganKind {
    pub const ALL: [OrganKind; 6] = [
        OrganKind::Eye,
        OrganKind::Nose,
        OrganKind::Thermal,
        OrganKind::Bristle,
        OrganKind::LateralLine,
        OrganKind::Proprioceptor,
    ];

    /// Everything the simulation knows about this kind.
    pub fn spec(self) -> &'static OrganSpec {
        match self {
            OrganKind::Eye => &eye::SPEC,
            OrganKind::Nose => &nose::SPEC,
            OrganKind::Thermal => &thermal::SPEC,
            OrganKind::Bristle => &bristle::SPEC,
            OrganKind::LateralLine => &lateral_line::SPEC,
            OrganKind::Proprioceptor => &proprioceptor::SPEC,
        }
    }

    /// Kind of a decoded organ gene. The thresholds partition the (affinity, style) square; they are part of the
    /// model (TCK-202 reviews them; the trait thresholds of other code must not repeat them, use the kind).
    pub fn classify(spectral_affinity: f32, expression_style: f32) -> Self {
        if spectral_affinity >= 0.80 {
            OrganKind::Eye
        } else if spectral_affinity > 0.65 {
            OrganKind::Thermal
        } else if spectral_affinity >= 0.25 {
            OrganKind::Nose
        } else if expression_style < 0.35 {
            OrganKind::Bristle
        } else if expression_style < 0.70 {
            OrganKind::LateralLine
        } else {
            OrganKind::Proprioceptor
        }
    }

    /// Distance senses detect food spectrally and carry the ΔI channel.
    pub fn is_distance(self) -> bool {
        self.spec().distance
    }
}

/// Static description of one organ kind. Function pointers instead of a trait (epic TCK-196 principle 4).
pub struct OrganSpec {
    pub kind: OrganKind,
    /// Display name (frontend, `bindings/organRegistry.ts`)
    pub name: &'static str,
    /// Display names of the spectral channels 0..5 (channel `DELTA_CHANNEL` is always "ΔI")
    pub channel_names: [&'static str; SPECTRAL_CHANNELS],
    /// Spectral distance sense: sees `foods[0]`/`foods[1]`, carries the ΔI channel, counts for `food_sense_range`
    pub distance: bool,
    /// Input channels that can ever carry a signal; only these are wired (TCK-182). Must agree with `sense`
    /// (parity test `test_live_channels_match_sense`).
    pub live_channels: fn(&Organ) -> [bool; CHANNELS_PER_ORGANELLE],
    /// Writes this tick's stimuli (before the [0, 1] clamp) into `out` (zeroed); ΔI is left to the brain step.
    pub sense: fn(&SenseCtx, &Organ, &mut [f32; CHANNELS_PER_ORGANELLE]),
    /// Organ load of the metabolic sense cost (`metabolism::organ_load`)
    pub cost_load: fn(&Organ) -> f32,
    /// Counts as a limb in the locomotion drive (`physics::locomotion_drive`)
    pub limb: fn(&Organ) -> bool,
    /// Forward drag units this organ adds (times `physics.receptorBallastScale`)
    pub ballast: f32,
    /// Latin name suffix when this kind dominates and its tie-break rank (lower wins a tie)
    pub latin_suffix: &'static str,
    pub latin_rank: u8,
}

/// Position of an organ relative to its gene: unpaired, or the left/right copy of a HOX pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum OrganSide {
    Single,
    Left,
    Right,
}

/// One expressed organ. Every kind decodes the same parameters (`decode_organs`); a kind-specific parameter would
/// be added here when the first kind needs one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Organ {
    #[serde(rename = "spectralAffinity")]
    #[ts(rename = "spectralAffinity")]
    pub spectral_affinity: f32,
    pub bandwidth: f32,
    #[serde(rename = "expressionStyle")]
    #[ts(rename = "expressionStyle")]
    pub expression_style: f32,
    pub scale: f32,
    #[serde(rename = "spinalPos")]
    #[ts(rename = "spinalPos")]
    pub spinal_pos: f32,
    pub angle: f32,
    #[serde(rename = "hueShift")]
    #[ts(rename = "hueShift")]
    pub hue_shift: f32,
    #[serde(rename = "geneStartIndex")]
    #[ts(rename = "geneStartIndex")]
    pub gene_start_index: usize,
    #[serde(rename = "geneEndIndex")]
    #[ts(rename = "geneEndIndex")]
    pub gene_end_index: usize,
    pub kind: OrganKind,
    pub side: OrganSide,
}

impl Organ {
    /// Sensor range (px).
    pub fn range(&self) -> f32 {
        self.scale * SENSOR_RANGE_PER_SCALE
    }

    /// Stimulus gain: big, broadly tuned organs are strong.
    pub fn power(&self) -> f32 {
        self.scale * (1.1 - self.bandwidth)
    }
}

/// World as one organ senses it in one tick.
pub struct SenseCtx<'a> {
    pub agent: &'a CreatureAgent,
    /// Visible foods: `foods[0]` nearest plant, `foods[1]` nearest meat (layout of the scenarios and the Ocean)
    pub foods: &'a [FoodSpore],
    pub obstacles: &'a [ProceduralObstacle],
    pub width: f32,
    pub height: f32,
}

/// Live input channels of an organ (`OrganSpec::live_channels`).
pub fn live_channels(organ: &Organ) -> [bool; CHANNELS_PER_ORGANELLE] {
    (organ.kind.spec().live_channels)(organ)
}

/// Shared rules the kinds point to.
pub mod common {
    use super::{LIMB_STYLE, Organ};

    /// `scale² · (1.1 − bandwidth)` (shape of the former vestigial organ tax, TCK-151).
    pub fn cost_load(o: &Organ) -> f32 {
        o.scale * o.scale * (1.1 - o.bandwidth)
    }

    /// Limb by expression style alone, whatever the kind.
    // TCK-202: eyes and other distance senses with a high style count as limbs too (model fix pending).
    pub fn limb_by_style(o: &Organ) -> bool {
        o.expression_style >= LIMB_STYLE
    }
}

/// An organ gene and the organs it expressed (`organelles[organs]`: one, or left + right of a HOX pair).
pub(crate) struct OrganGene<'a> {
    pub gene: GeneRef,
    pub owner: usize,
    pub payload: &'a str,
    pub organs: std::ops::Range<usize>,
}

/// Organs of all organ genes in registry promoter order, then genome order: one organ per gene, a mirrored pair or a
/// midline organ with HOX (FIX-131).
pub(crate) fn decode_organs<'a>(genes: &GeneTable<'a>) -> (Vec<Organ>, Vec<OrganGene<'a>>) {
    let has_hox = !genes.genes(promoters::motif_of(GeneRole::Hox)).is_empty();
    let mut organelles = Vec::new();
    let mut organ_genes = Vec::new();
    for promoter in promoters::with_role(GeneRole::Organ) {
        for gene in genes.genes(promoter) {
            let p = Payload(gene.payload);
            let spectral_affinity = p.h(AFFINITY_LOCUS);
            let bandwidth = p.range(4, 0.05, 0.85);
            let expression_style = p.h(7);
            let scale = p.range(10, 0.35, 1.45);
            let spinal_pos = p.range(13, 0.05, 0.9);
            let hue_shift = p.range(19, -180.0, 360.0).round();
            let kind = OrganKind::classify(spectral_affinity, expression_style);

            // Symmetrical Angle Space: 0.0 is straight ahead, Left is negative [-170 .. 0], Right is positive [0 .. 170]
            let base_angle = p.range(16, -170.0, 340.0);
            let organ = |angle: f32, side: OrganSide| Organ {
                spectral_affinity,
                bandwidth,
                expression_style,
                scale,
                spinal_pos,
                angle,
                hue_shift,
                gene_start_index: gene.start,
                gene_end_index: gene.end,
                kind,
                side,
            };

            let first = organelles.len();
            if !has_hox {
                // Primitive asymmetric distribution: anywhere in the range
                organelles.push(organ(base_angle, OrganSide::Single));
            } else if base_angle.abs() <= 7.5 {
                // Midline coalescence/fusion to a single unpaired central organelle
                organelles.push(organ(0.0, OrganSide::Single));
            } else {
                organelles.push(organ(-base_angle.abs(), OrganSide::Left));
                organelles.push(organ(base_angle.abs(), OrganSide::Right));
            }
            let owner = genes.owner(promoter, gene.start);
            organ_genes.push(OrganGene {
                gene: genes.gene_ref(owner),
                owner,
                payload: gene.payload,
                organs: first..organelles.len(),
            });
        }
    }
    (organelles, organ_genes)
}

/// Display data of every kind for the frontend.
#[cfg(test)]
#[derive(Serialize)]
struct OrganKindInfo {
    name: &'static str,
    channels: [&'static str; SPECTRAL_CHANNELS],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::brain::BrainRuntime;
    use rand::Rng;

    fn organ_of(kind: OrganKind, rng: &mut crate::shared::rng::SimRng) -> Organ {
        loop {
            let payload: String = (0..24)
                .map(|_| crate::biology::genome::ALPHABET[rng.gen_range(0..26)] as char)
                .collect();
            let p = Payload(&payload);
            let (affinity, style) = (p.h(AFFINITY_LOCUS), p.h(7));
            if OrganKind::classify(affinity, style) == kind {
                return Organ {
                    spectral_affinity: affinity,
                    bandwidth: p.range(4, 0.05, 0.85),
                    expression_style: style,
                    scale: p.range(10, 0.35, 1.45),
                    spinal_pos: 0.5,
                    angle: p.range(16, -170.0, 340.0),
                    hue_shift: 0.0,
                    gene_start_index: 0,
                    gene_end_index: 0,
                    kind,
                    side: OrganSide::Single,
                };
            }
        }
    }

    fn agent(rng: &mut crate::shared::rng::SimRng) -> CreatureAgent {
        let phenotype =
            crate::biology::phenotype::parse_genome("EYEABCDEFEN", None, None, &crate::config::test_config().brain);
        CreatureAgent {
            id: 1,
            species_id: "t".into(),
            px: rng.gen_range(0.0..1000.0),
            py: rng.gen_range(0.0..1000.0),
            vx: rng.gen_range(-5.0..5.0),
            vy: rng.gen_range(-5.0..5.0),
            heading_angle: rng.gen_range(-3.2..3.2),
            bend_angle: rng.gen_range(-1.5..1.5),
            omega_rot: rng.gen_range(-3.0..3.0),
            energy: 50.0,
            age: 0,
            generation: 1,
            has_eaten: false,
            genome: String::new(),
            antisense: String::new(),
            phenotype,
            neuron_states: Vec::new(),
            neuron_activations: Vec::new(),
            synapse_weights: Vec::new(),
            brain_runtime: BrainRuntime::default(),
        }
    }

    /// Parity of `live_channels` and `sense` per kind (TCK-182 wiring): a channel `sense` ever writes a non-zero
    /// stimulus to is live, and every live channel (but ΔI, filled by the brain step) carries a stimulus in some state.
    #[test]
    fn test_live_channels_match_sense() {
        let mut rng = crate::shared::rng::seeded(199);
        let obstacles = [ProceduralObstacle {
            id: 1,
            x: 500.0,
            y: 500.0,
            radius: 40.0,
            obstacle_type: "rock".into(),
            color: String::new(),
            vertices: Vec::new(),
        }];
        for kind in OrganKind::ALL {
            for _ in 0..40 {
                let organ = organ_of(kind, &mut rng);
                let live = live_channels(&organ);
                let mut fired = [false; CHANNELS_PER_ORGANELLE];
                for _ in 0..300 {
                    let a = agent(&mut rng);
                    // Foods straight along the organ's axis, inside its range (the spectral senses' best case)
                    let dir = a.heading_angle + organ.angle.to_radians();
                    let d = organ.range() * rng.gen_range(0.05..0.9);
                    let food = |type_id: u32| FoodSpore {
                        id: type_id,
                        type_id,
                        x: a.px + d * dir.cos(),
                        y: a.py + d * dir.sin(),
                        amount: 1.0,
                        vx: 0.0,
                        vy: 0.0,
                    };
                    let foods = [food(1), food(2)];
                    let ctx = SenseCtx {
                        agent: &a,
                        foods: &foods,
                        obstacles: &obstacles,
                        width: 1000.0,
                        height: 1000.0,
                    };
                    let mut out = [0.0; CHANNELS_PER_ORGANELLE];
                    (kind.spec().sense)(&ctx, &organ, &mut out);
                    for c in 0..CHANNELS_PER_ORGANELLE {
                        if out[c] != 0.0 {
                            assert!(live[c], "{:?} wrote channel {} that is not live: {:?}", kind, c, organ);
                            fired[c] |= out[c] > 0.0;
                        }
                    }
                }
                for c in (0..CHANNELS_PER_ORGANELLE).filter(|&c| c != DELTA_CHANNEL) {
                    assert!(
                        !live[c] || fired[c],
                        "{:?} live channel {} never fired: {:?}",
                        kind,
                        c,
                        organ
                    );
                }
                assert_eq!(
                    live[DELTA_CHANNEL],
                    kind.is_distance() && live[..SPECTRAL_CHANNELS].iter().any(|&l| l),
                    "{:?}",
                    kind
                );
            }
        }
    }

    #[test]
    fn test_specs_belong_to_their_kind() {
        for kind in OrganKind::ALL {
            assert_eq!(kind.spec().kind, kind);
        }
    }

    /// Writes the display data for the frontend (`bindings/organRegistry.ts`, committed; CI checks drift).
    #[test]
    fn export_organ_registry() {
        let map: std::collections::BTreeMap<String, OrganKindInfo> = OrganKind::ALL
            .iter()
            .map(|k| {
                (
                    serde_json::to_value(k).unwrap().as_str().unwrap().to_string(),
                    OrganKindInfo {
                        name: k.spec().name,
                        channels: k.spec().channel_names,
                    },
                )
            })
            .collect();
        let ts = format!(
            "// Generated by `cargo test` from src/biology/organs (OrganSpec per kind). Do not edit this file manually.\nimport type {{ OrganKind }} from \"./OrganKind\";\n\nexport const ORGAN_KINDS: Record<OrganKind, {{ name: string; channels: string[] }}> = {};\n",
            serde_json::to_string_pretty(&map).unwrap()
        );
        crate::biology::genome::write_binding("organRegistry.ts", &ts);
    }
}
