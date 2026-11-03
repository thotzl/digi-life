//! Promoter registry (TCK-199): the one list of gene promoters the compiler expresses. Every promoter list of the
//! genome tools (expressed, body, organ, essential, protected motifs, mutation channels, the frontend's gene
//! meanings) derives from `PROMOTERS`. Contract: `wiki/contracts/organ-kind.md` (organ promoters),
//! `wiki/biology/genome-compiler.md`.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use ts_rs::TS;

/// What the compiler builds from a gene of this promoter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub enum GeneRole {
    /// Body trait gene (decoded by `phenotype::body`)
    Body,
    /// Presence pairs every organ left/right (`organs::decode_organs`)
    Hox,
    /// One organ (or a HOX pair) per gene; the kind follows from the payload (`organs::OrganKind::classify`)
    Organ,
    /// One hidden neuron and its synapses
    Hidden,
    /// One explicit synapse
    Synapse,
    /// Motor bias/tau and systemic -> motor weights
    BrainBase,
}

/// How many genes of a promoter the compiler reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub enum Expression {
    /// Only the first gene; later copies are silent
    FirstOnly,
    /// Every gene (the brain compiler caps `NEU` and `SY`)
    EveryCopy,
}

/// Degenerate matching of a motif (`alphabet::matches_degenerate`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub enum Wobble {
    /// First letter exact, the rest by biochemical class
    FirstExact,
    /// Every letter by biochemical class
    Full,
}

/// Mutation channel of a gene (TCK-181), by promoter: brain = `NEU` (hidden neuron and its synapses), `SY` (explicit
/// synapse) and `OUT` (motor bias/tau, systemic -> motor weights); body = every other expressed promoter: the body
/// trait genes (`COL` .. `EVO`), `HOX` (organ pairing) and the organ genes `EYE`/`NOS`/`TAC`/`LUM`. Organ genes are
/// mixed in the compiler: their payload sets the organ's traits and is also the tape of the thalamus -> motor weights;
/// they count as body here and are flagged separately (`MutationTouch::organ`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum GeneChannel {
    Body,
    Brain,
}

/// One expressed promoter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct PromoterSpec {
    /// Canonical spelling
    #[ts(type = "string")]
    pub motif: &'static str,
    pub role: GeneRole,
    pub channel: GeneChannel,
    pub expression: Expression,
    /// Every structured genome carries one; gene deletion never removes the last copy
    pub essential: bool,
    pub wobble: Wobble,
    /// Display text (gene tooltips in the frontend)
    #[ts(type = "string")]
    pub meaning: &'static str,
}

const fn body(motif: &'static str, meaning: &'static str) -> PromoterSpec {
    PromoterSpec {
        motif,
        role: GeneRole::Body,
        channel: GeneChannel::Body,
        expression: Expression::FirstOnly,
        essential: true,
        wobble: Wobble::FirstExact,
        meaning,
    }
}

const fn organ(motif: &'static str, meaning: &'static str) -> PromoterSpec {
    PromoterSpec {
        motif,
        role: GeneRole::Organ,
        channel: GeneChannel::Body,
        expression: Expression::EveryCopy,
        essential: false,
        wobble: Wobble::FirstExact,
        meaning,
    }
}

/// Every promoter `parse_genome` expresses, in compiler order: body genes first (the generator's plan order), organ
/// genes in organ (and input slot) order. The order is part of the model (organ order, generator RNG draws).
pub static PROMOTERS: &[PromoterSpec] = &[
    body("COL", "Colour hue and symmetry (body)"),
    body("SIZ", "Body radius, length and body seed (body)"),
    body("STF", "Stiffness (body)"),
    body("WAV", "Spinal curve and parapodia waves (body)"),
    body("PUL", "Pulse speed, wave phase, wiggle, head shape (body)"),
    body("STM", "Stomach capacity (body)"),
    body("TEM", "Thermal tolerance (body)"),
    body("CAR", "Carnivory (diet)"),
    body("REP", "Maturity age, mitosis threshold, split loss (reproduction)"),
    body("EVO", "Indel rates (evolvability)"),
    PromoterSpec {
        motif: "HOX",
        role: GeneRole::Hox,
        channel: GeneChannel::Body,
        expression: Expression::EveryCopy,
        essential: false,
        wobble: Wobble::FirstExact,
        meaning: "Symmetry: organs form left/right pairs (HOX)",
    },
    organ("EYE", "Sensory organ (EYE)"),
    organ("NOS", "Sensory organ (NOS)"),
    organ("TAC", "Sensory organ (TAC)"),
    organ("LUM", "Sensory organ (LUM)"),
    PromoterSpec {
        motif: "NEU",
        role: GeneRole::Hidden,
        channel: GeneChannel::Brain,
        expression: Expression::EveryCopy,
        essential: false,
        wobble: Wobble::Full,
        meaning: "Hidden neuron and its synapses (NEU)",
    },
    PromoterSpec {
        motif: "SY",
        role: GeneRole::Synapse,
        channel: GeneChannel::Brain,
        expression: Expression::EveryCopy,
        essential: false,
        wobble: Wobble::Full,
        meaning: "Explicit synapse (SY)",
    },
    PromoterSpec {
        motif: "OUT",
        role: GeneRole::BrainBase,
        channel: GeneChannel::Brain,
        expression: Expression::FirstOnly,
        essential: true,
        wobble: Wobble::FirstExact,
        meaning: "Brain base: motor bias/tau, systemic to motor weights (OUT)",
    },
];

/// Gene terminator as `parse_genome` scans it (degenerate: E + any Delta-class letter).
pub const TERMINATOR: &str = "EN";
/// Motifs that are no longer expressed but stay protected from point mutation (`SYM`: legacy symmetry gene).
pub const LEGACY_MOTIFS: &[&str] = &["SYM"];

/// Every motif the genome tools know: expressed promoters, legacy motifs and the terminator. Protected from
/// point-level mutation; never formed by accident in structured genomes.
pub static GENE_MOTIFS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    PROMOTERS
        .iter()
        .map(|p| p.motif)
        .chain(LEGACY_MOTIFS.iter().copied())
        .chain([TERMINATOR])
        .collect()
});

/// Motifs matched with full wobble (`Wobble::Full`).
static FULL_WOBBLE: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    PROMOTERS
        .iter()
        .filter(|p| p.wobble == Wobble::Full)
        .map(|p| p.motif)
        .collect()
});

/// Whether `motif` is matched letter by letter by biochemical class: full-wobble promoters and one-letter motifs.
pub fn full_wobble(motif: &str) -> bool {
    motif.len() <= 1 || FULL_WOBBLE.contains(&motif)
}

/// The spec of an expressed promoter.
pub fn spec(motif: &str) -> Option<&'static PromoterSpec> {
    PROMOTERS.iter().find(|p| p.motif == motif)
}

/// Promoters of `role` in registry order.
pub fn with_role(role: GeneRole) -> impl Iterator<Item = &'static str> {
    PROMOTERS.iter().filter(move |p| p.role == role).map(|p| p.motif)
}

/// The promoter of a role that has exactly one (`Hox`, `Hidden`, `Synapse`, `BrainBase`).
pub fn motif_of(role: GeneRole) -> &'static str {
    with_role(role).next().expect("role has a promoter")
}

/// Channel of an expressed promoter (`GeneChannel`); `None` for anything else.
pub fn promoter_channel(promoter: &str) -> Option<GeneChannel> {
    spec(promoter).map(|p| p.channel)
}

/// Whether `promoter` is an organ gene.
pub fn is_organ(promoter: &str) -> bool {
    spec(promoter).is_some_and(|p| p.role == GeneRole::Organ)
}

/// Whether `promoter` is essential (`PromoterSpec::essential`).
pub fn is_essential(promoter: &str) -> bool {
    spec(promoter).is_some_and(|p| p.essential)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_is_consistent() {
        let motifs: Vec<&str> = PROMOTERS.iter().map(|p| p.motif).collect();
        let unique: std::collections::BTreeSet<&str> = motifs.iter().copied().collect();
        assert_eq!(unique.len(), motifs.len(), "duplicate promoter");
        for p in PROMOTERS {
            assert_eq!(
                p.channel == GeneChannel::Brain,
                matches!(p.role, GeneRole::Hidden | GeneRole::Synapse | GeneRole::BrainBase),
                "{}",
                p.motif
            );
            assert_eq!(
                p.essential,
                matches!(p.role, GeneRole::Body | GeneRole::BrainBase),
                "{}",
                p.motif
            );
        }
        for role in [GeneRole::Hox, GeneRole::Hidden, GeneRole::Synapse, GeneRole::BrainBase] {
            assert_eq!(with_role(role).count(), 1, "{:?}", role);
        }
    }

    #[test]
    fn test_promoter_channels() {
        for p in ["NEU", "SY", "OUT"] {
            assert_eq!(promoter_channel(p), Some(GeneChannel::Brain), "{}", p);
        }
        for p in with_role(GeneRole::Body)
            .chain(with_role(GeneRole::Organ))
            .chain(["HOX"])
        {
            assert_eq!(promoter_channel(p), Some(GeneChannel::Body), "{}", p);
        }
        assert_eq!(promoter_channel("SYM"), None);
        assert_eq!(promoter_channel("EN"), None);
    }

    /// Writes the registry as data for the frontend (`bindings/promoterRegistry.ts`, committed; CI checks drift).
    #[test]
    fn export_promoter_registry() {
        let json = serde_json::to_string_pretty(PROMOTERS).unwrap();
        let ts = format!(
            "// Generated by `cargo test` from src/biology/genome/promoters.rs (PROMOTERS). Do not edit this file manually.\nimport type {{ PromoterSpec }} from \"./PromoterSpec\";\n\nexport const PROMOTERS: PromoterSpec[] = {};\n",
            json
        );
        crate::biology::genome::write_binding("promoterRegistry.ts", &ts);
    }
}
