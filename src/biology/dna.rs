use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Spectral receptive-field channels per organ (UV/pressure .. IR/pheromone).
pub const SPECTRAL_CHANNELS: usize = 5;
/// Channel of the temporal intensity change ΔI (distance senses only, TCK-144).
pub const DELTA_CHANNEL: usize = SPECTRAL_CHANNELS;
/// Input slots per organ: 5 spectral channels + ΔI.
pub const CHANNELS_PER_ORGANELLE: usize = SPECTRAL_CHANNELS + 1;
pub const SYSTEMIC_BASE_INPUTS_COUNT: usize = SystemicInput::ALL.len();
/// Motor outputs: thrust and bend (biolum and reserved removed, TCK-149).
pub const OUTPUT_MOTOR_NODES_COUNT: usize = Motor::ALL.len();
/// Time constant range of all integrating neurons.
pub const TAU_MIN: f32 = 0.1;
pub const TAU_MAX: f32 = 2.5;
/// Hidden neurons: at most this many `NEU` genes are expressed.
const MAX_HIDDEN_NEURONS: usize = 12;
/// Explicit `SY` genes: at most this many are expressed.
const MAX_EXPLICIT_SYNAPSES: usize = 30;

/// Center frequencies of the 5 spectral channels (cones).
pub const CHANNEL_FREQUENCIES: [f32; SPECTRAL_CHANNELS] = [0.10, 0.30, 0.50, 0.70, 0.90];
/// Algae/plant emission per channel (blue 1.0, green 0.5).
pub const PLANT_EMISSION: [f32; SPECTRAL_CHANNELS] = [0.0, 1.0, 0.5, 0.0, 0.0];
/// Meat/prey emission per channel (IR 1.0, red 0.5).
pub const MEAT_EMISSION: [f32; SPECTRAL_CHANNELS] = [0.0, 0.0, 0.0, 0.5, 1.0];
/// Minimum channel match (sensitivity · emission) that registers a stimulus.
pub const MIN_CHANNEL_MATCH: f32 = 0.05;

/// Wildtype progenitor genome (trainer slot plan placeholder for random immigrants, tests).
pub const WILDTYPE_GENOME: &str = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";

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
        matches!(self, OrganKind::Eye | OrganKind::Nose | OrganKind::Thermal)
    }
}

/// Sensitivity of a distance-sense organ to spectral channel `c` (Gaussian-like tuning curve).
pub fn channel_sensitivity(spectral_affinity: f32, bandwidth: f32, c: usize) -> f32 {
    (1.0 - (CHANNEL_FREQUENCIES[c] - spectral_affinity).abs() / (bandwidth * 1.8 + 0.12)).clamp(0.0, 1.0)
}

/// Input channels that can ever carry a signal for this organ (mirrors `compute_sensory_inputs`); only these are wired (TCK-182).
pub fn live_channels(patch: &SensoryPatch) -> [bool; CHANNELS_PER_ORGANELLE] {
    let mut live = [false; CHANNELS_PER_ORGANELLE];
    match patch.kind {
        OrganKind::Eye | OrganKind::Nose | OrganKind::Thermal => {
            for c in 0..SPECTRAL_CHANNELS {
                let emission = PLANT_EMISSION[c].max(MEAT_EMISSION[c]);
                live[c] = channel_sensitivity(patch.spectral_affinity, patch.bandwidth, c) * emission > MIN_CHANNEL_MATCH;
            }
            live[DELTA_CHANNEL] = live.iter().any(|&l| l);
        }
        OrganKind::Bristle => (live[0], live[4]) = (true, true),
        OrganKind::LateralLine => (live[1], live[4]) = (true, true),
        OrganKind::Proprioceptor => (live[2], live[3]) = (true, true),
    }
    live
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

/// Stable gene identity (TCK-179): promoter type, the promoter's exact spelling in the genome (a degenerate variant
/// such as `EWA` for `EYE`) and the occurrence among the genes with that spelling, counted in genome order.
/// Letter-level mutations never change promoter letters (`unprotected_indices`) and gene duplication inserts the copy
/// after the original, so a gene keeps its identity unless a gene with the identical spelling before it is
/// duplicated or deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeneRef {
    pub promoter: String,
    pub spelling: String,
    pub occurrence: usize,
}

/// One recognised gene (promoter .. terminator) of the compiled genome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeneSpan {
    pub gene: GeneRef,
    /// First base of the promoter
    pub start: usize,
    /// One past the last base (terminator included when present)
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub enum SystemicInput {
    Clock,
    LinearSpeed,
    RotationalSpeed,
    Energy,
    Pain,
    SearchArousal,
}

impl SystemicInput {
    /// Input order after the organ channels.
    pub const ALL: [SystemicInput; 6] = [
        SystemicInput::Clock,
        SystemicInput::LinearSpeed,
        SystemicInput::RotationalSpeed,
        SystemicInput::Energy,
        SystemicInput::Pain,
        SystemicInput::SearchArousal,
    ];

    fn label(self) -> &'static str {
        match self {
            SystemicInput::Clock => "⌛ Hunger Clock",
            SystemicInput::LinearSpeed => "🌊 Linear Speed",
            SystemicInput::RotationalSpeed => "🔄 Rotational Speed",
            SystemicInput::Energy => "🔋 Internal Energy",
            SystemicInput::Pain => "💥 Pain Signal",
            SystemicInput::SearchArousal => "📡 Search Arousal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum Motor {
    Thrust,
    Bend,
}

impl Motor {
    /// Output order of `execute_brain_with_learning`.
    pub const ALL: [Motor; 2] = [Motor::Thrust, Motor::Bend];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum ThalamusKind {
    /// L + R (or the single organ)
    Sum,
    /// L − R (HOX pairs only)
    Diff,
}

/// Role of a neuron; neuron order, synapse keys and ownership derive from roles, not from indices (TCK-179).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "role", rename_all = "lowercase")]
#[ts(export)]
pub enum NeuronRole {
    Systemic { input: SystemicInput },
    Sensor { gene: GeneRef, side: OrganSide, channel: usize, kind: OrganKind },
    Thalamus { gene: GeneRef, channel: usize, node: ThalamusKind },
    Hidden { gene: GeneRef },
    Motor { motor: Motor },
}

impl NeuronRole {
    /// Canonical bytes for hashing; `anonymous` drops the gene identity (tape slot of the owner's own end).
    fn encode(&self, anonymous: bool, out: &mut Vec<u8>) {
        let gene = |g: &GeneRef, out: &mut Vec<u8>| {
            if !anonymous {
                out.extend_from_slice(g.promoter.as_bytes());
                out.extend_from_slice(g.spelling.as_bytes());
                out.extend_from_slice(&(g.occurrence as u32).to_le_bytes());
            }
        };
        match self {
            NeuronRole::Systemic { input } => out.extend_from_slice(&[1, *input as u8]),
            NeuronRole::Sensor { gene: g, side, channel, .. } => {
                out.push(2);
                gene(g, out);
                out.extend_from_slice(&[*side as u8, *channel as u8]);
            }
            NeuronRole::Thalamus { gene: g, channel, node } => {
                out.push(3);
                gene(g, out);
                out.extend_from_slice(&[*channel as u8, *node as u8]);
            }
            NeuronRole::Hidden { gene: g } => {
                out.push(4);
                gene(g, out);
            }
            NeuronRole::Motor { motor } => out.extend_from_slice(&[5, *motor as u8]),
        }
    }

    /// The gene this neuron belongs to (organ or `NEU` gene); systemic inputs and motors belong to none.
    pub fn gene(&self) -> Option<&GeneRef> {
        match self {
            NeuronRole::Sensor { gene, .. } | NeuronRole::Thalamus { gene, .. } | NeuronRole::Hidden { gene } => Some(gene),
            NeuronRole::Systemic { .. } | NeuronRole::Motor { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HSLColor {
    pub h: f32, // 0 - 360
    pub s: f32, // 0 - 100
    pub l: f32, // 0 - 100
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SensoryPatch {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SpinalHarmonics {
    #[serde(rename = "baseLength")]
    #[ts(rename = "baseLength")]
    pub base_length: f32,
    #[serde(rename = "meanRadius")]
    #[ts(rename = "meanRadius")]
    pub mean_radius: f32,
    pub amplitudes: Vec<f32>,
    pub phases: Vec<f32>,
    #[serde(rename = "spinalCurve")]
    #[ts(rename = "spinalCurve")]
    pub spinal_curve: f32,
    #[serde(rename = "spinalCurveFreq")]
    #[ts(rename = "spinalCurveFreq")]
    pub spinal_curve_freq: f32,
    #[serde(rename = "parapodiaAmp")]
    #[ts(rename = "parapodiaAmp")]
    pub parapodia_amp: f32,
    #[serde(rename = "parapodiaFreq")]
    #[ts(rename = "parapodiaFreq")]
    pub parapodia_freq: f32,
    #[serde(rename = "flatteningHead")]
    #[ts(rename = "flatteningHead")]
    pub flattening_head: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CTRNNSynapse {
    #[serde(rename = "fromNode")]
    #[ts(rename = "fromNode")]
    pub from_node: usize,
    #[serde(rename = "toNode")]
    #[ts(rename = "toNode")]
    pub to_node: usize,
    pub weight: f32,
    /// Stable identity: hash of source and target role (53 bits, exact as a JS number)
    #[ts(type = "number")]
    pub key: u64,
    /// Index into `CreaturePhenotype::genes` of the gene that defines this synapse
    pub owner: usize,
    /// Learns during life (Oja rule) and can be pruned; false for the fixed sensor → thalamus reflexes (TCK-152)
    pub plastic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum NeuronType {
    Input,
    Hidden,
    Output,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CTRNNNeuron {
    pub id: usize,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub neuron_type: NeuronType,
    pub label: String,
    pub role: NeuronRole,
    pub tau: f32,  // time constant [TAU_MIN, TAU_MAX]
    pub bias: f32, // neural bias [-1.0 to 1.0]
    #[serde(rename = "activationType")]
    #[ts(rename = "activationType")]
    pub activation_type: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BrainTopology {
    pub neurons: Vec<CTRNNNeuron>,
    pub synapses: Vec<CTRNNSynapse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreaturePhenotype {
    pub symmetry: String, // "vertical" | "quad"
    #[serde(rename = "primaryColor")]
    #[ts(rename = "primaryColor")]
    pub primary_color: HSLColor,
    #[serde(rename = "secondaryColor")]
    #[ts(rename = "secondaryColor")]
    pub secondary_color: HSLColor,
    #[serde(rename = "bodySeed")]
    #[ts(rename = "bodySeed")]
    pub body_seed: u32,
    #[ts(type = "any[]")]
    pub segments: Vec<serde_json::Value>,
    #[serde(rename = "spinalHarmonics")]
    #[ts(rename = "spinalHarmonics")]
    pub spinal_harmonics: SpinalHarmonics,
    #[serde(rename = "emergentChambersCount")]
    #[ts(rename = "emergentChambersCount")]
    pub emergent_chambers_count: usize,
    pub organelles: Vec<SensoryPatch>,
    /// Recognised genes in genome order; synapse owners index into this list
    pub genes: Vec<GeneSpan>,
    #[serde(rename = "pulseSpeed")]
    #[ts(rename = "pulseSpeed")]
    pub pulse_speed: f32,
    #[serde(rename = "wavePhase")]
    #[ts(rename = "wavePhase")]
    pub wave_phase: f32,
    #[serde(rename = "wiggleAmplitude")]
    #[ts(rename = "wiggleAmplitude")]
    pub wiggle_amplitude: f32,
    pub stiffness: f32,

    #[serde(rename = "matureAge")]
    #[ts(rename = "matureAge")]
    pub mature_age: u32,
    #[serde(rename = "reproThreshold")]
    #[ts(rename = "reproThreshold")]
    pub repro_threshold: f32,
    #[serde(rename = "splitLoss")]
    #[ts(rename = "splitLoss")]
    pub split_loss: f32,

    pub brain: BrainTopology,
    pub carnivory: f32,
    #[serde(rename = "isPredator")]
    #[ts(rename = "isPredator")]
    pub is_predator: bool,

    #[serde(rename = "latinName")]
    #[ts(rename = "latinName")]
    pub latin_name: String,
    #[serde(rename = "sensoryVisus")]
    #[ts(rename = "sensoryVisus")]
    pub sensory_visus: f32,
    #[serde(rename = "sensoryOlfaction")]
    #[ts(rename = "sensoryOlfaction")]
    pub sensory_olfaction: f32,
    #[serde(rename = "sensoryTactility")]
    #[ts(rename = "sensoryTactility")]
    pub sensory_tactility: f32,
    #[serde(rename = "sensoryBiolum")]
    #[ts(rename = "sensoryBiolum")]
    pub sensory_biolum: f32,
    #[serde(rename = "dietClass")]
    #[ts(rename = "dietClass")]
    pub diet_class: String,
    #[serde(rename = "preferredHabitat")]
    #[ts(rename = "preferredHabitat")]
    pub preferred_habitat: String,

    #[serde(rename = "basalMetabolicRate")]
    #[ts(rename = "basalMetabolicRate")]
    pub basal_metabolic_rate: f32,
    #[serde(rename = "stomachCapacity")]
    #[ts(rename = "stomachCapacity")]
    pub stomach_capacity: f32,
    #[serde(rename = "thermalToleranceMin")]
    #[ts(rename = "thermalToleranceMin")]
    pub thermal_tolerance_min: f32,
    #[serde(rename = "thermalToleranceMax")]
    #[ts(rename = "thermalToleranceMax")]
    pub thermal_tolerance_max: f32,
    #[serde(rename = "hydraulicPressure")]
    #[ts(rename = "hydraulicPressure")]
    pub hydraulic_pressure: f32,
    #[serde(rename = "rotationalInertia")]
    #[ts(rename = "rotationalInertia")]
    pub rotational_inertia: f32,

    #[serde(rename = "survivalExpectation")]
    #[ts(rename = "survivalExpectation")]
    pub survival_expectation: f32,
    #[serde(rename = "survivalAnalysis")]
    #[ts(rename = "survivalAnalysis")]
    pub survival_analysis: String,

    #[serde(rename = "chromatinState")]
    #[ts(rename = "chromatinState")]
    pub chromatin_state: Vec<bool>,
    #[serde(rename = "epigeneticLogs")]
    #[ts(rename = "epigeneticLogs")]
    pub epigenetic_logs: Vec<String>,
    pub methylations: Vec<f32>,

    #[serde(rename = "antisenseStrand")]
    #[ts(rename = "antisenseStrand")]
    pub antisense_strand: String,
    #[serde(rename = "repairFidelity")]
    #[ts(rename = "repairFidelity")]
    pub repair_fidelity: f32,
    #[serde(rename = "insertionRate")]
    #[ts(rename = "insertionRate")]
    pub insertion_rate: f32,
    #[serde(rename = "deletionRate")]
    #[ts(rename = "deletionRate")]
    pub deletion_rate: f32,

    #[serde(rename = "genomeString")]
    #[ts(rename = "genomeString")]
    pub genome_string: String,
}

// DNA Base Helper utilities

pub fn char_to_value(c: char) -> usize {
    let uc = c.to_ascii_uppercase();
    ALPHABET.iter().position(|&x| x == uc as u8).unwrap_or(0)
}

pub fn value_to_char(val: f32) -> char {
    let index = (val.floor() as usize).clamp(0, 25);
    ALPHABET[index] as char
}

pub fn get_complementary_char(c: char) -> char {
    let val = char_to_value(c);
    let comp_val = 25 - val;
    value_to_char(comp_val as f32)
}

pub fn get_complementary_string(sense: &str) -> String {
    sense.chars().map(get_complementary_char).collect()
}

pub fn get_bio_class(c: char) -> u8 {
    match c.to_ascii_uppercase() {
        'A' | 'E' | 'I' | 'O' | 'U' => 1,       // Alpha (Polar/Hydrophilic)
        'Y' | 'W' | 'F' | 'H' | 'K' | 'R' => 2, // Beta (Aromatic/Basic)
        'L' | 'M' | 'V' | 'P' | 'T' => 3,       // Gamma (Hydrophobic)
        'D' | 'N' | 'Q' | 'S' | 'C' => 4,       // Delta (Acidic/Amide)
        _ => 5,                                 // Epsilon (Inert/Rare)
    }
}

pub fn matches_degenerate(segment: &str, classic_promoter: &str) -> bool {
    let (s, p) = (segment.as_bytes(), classic_promoter.as_bytes());
    if s.len() != p.len() {
        return false;
    }
    let same_class = |(&a, &b): (&u8, &u8)| get_bio_class(a as char) == get_bio_class(b as char);

    // Brain-related promoters ("NEU", "SY") and one-letter motifs use full biochemical group matching (full wobble)
    if classic_promoter == "NEU" || classic_promoter == "SY" || p.len() <= 1 {
        return s.iter().zip(p).all(same_class);
    }

    // Golden-Middle matching: first character exact, the remaining characters degenerate (wobble)
    s[0] == p[0] && s[1..].iter().zip(&p[1..]).all(same_class)
}

/// One gene found by `scan_genes`.
pub struct ScannedGene<'a> {
    /// First base of the promoter
    pub start: usize,
    /// One past the terminator (or past the payload when no terminator was found)
    pub end: usize,
    pub payload: &'a str,
}

pub fn extract_raw_gene_payloads(genome: &str, start_motif: &str, stop_motif: &str) -> Vec<String> {
    scan_genes(genome, start_motif, stop_motif).into_iter().map(|g| g.payload.to_string()).collect()
}

/// Genes of one promoter type in genome order (non-overlapping, degenerate matching).
pub fn scan_genes<'a>(genome: &'a str, start_motif: &str, stop_motif: &str) -> Vec<ScannedGene<'a>> {
    let mut genes = Vec::new();

    if start_motif.is_empty() || stop_motif.is_empty() || genome.len() < start_motif.len() + stop_motif.len() {
        return genes;
    }

    let mut idx = 0;
    while idx <= genome.len().saturating_sub(start_motif.len()) {
        let segment_start = &genome[idx..idx + start_motif.len()];
        if matches_degenerate(segment_start, start_motif) {
            let payload_start = idx + start_motif.len();
            let mut payload_end = None;
            // Limit the stop motif search to at most 40 characters (gene size limitation)
            let max_search = (payload_start + 40).min(genome.len());
            for j in payload_start..=genome.len().saturating_sub(stop_motif.len()) {
                if j > max_search {
                    break;
                }
                let segment_stop = &genome[j..j + stop_motif.len()];
                if matches_degenerate(segment_stop, stop_motif) {
                    payload_end = Some(j);
                    break;
                }
            }

            if let Some(end_idx) = payload_end {
                genes.push(ScannedGene { start: idx, end: end_idx + stop_motif.len(), payload: &genome[payload_start..end_idx] });
                idx = end_idx + stop_motif.len();
            } else {
                // Point 1 Fallback: If no stop motif found within 40 characters, read a standard 15-char payload
                // and resume scanning after it, instead of swallowing the remaining genome and breaking!
                let end_idx = (payload_start + 15).min(genome.len());
                if end_idx > payload_start {
                    genes.push(ScannedGene { start: idx, end: end_idx, payload: &genome[payload_start..end_idx] });
                }
                idx = end_idx;
            }
        } else {
            idx += 1;
        }
    }
    genes
}

/// Random letter soup (no gene structure); probes keep it as the "letter soup" comparison population.
pub fn generate_random_genome<R: rand::Rng + ?Sized>(length: usize, rng: &mut R) -> String {
    let mut dna = String::with_capacity(length);
    for _ in 0..length {
        let idx = rng.gen_range(0..26);
        dna.push(ALPHABET[idx] as char);
    }
    dna
}

/// Composition of `generate_structured_genome` (config `genomeComposition`, TCK-184).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenomeComposition {
    /// Total length the intergenic filler aims for (clamped to [128, 512]); gene sets longer than this get no filler
    pub target_length: usize,
    /// Payload length range per gene (letters between promoter and terminator)
    pub payload_length_min: usize,
    pub payload_length_max: usize,
    pub organ_count_min: usize,
    pub organ_count_max: usize,
    /// Probability that the first organ gene is food-sensing (OrganKind Eye or Nose); otherwise every organ is random
    pub p_food_organ: f32,
    /// Share of eyes among these food organs (the rest are noses)
    pub food_organ_eye_share: f32,
    pub p_hox: f32,
    pub neu_count_min: usize,
    pub neu_count_max: usize,
    pub sy_count_min: usize,
    pub sy_count_max: usize,
}

/// Operator weights of `mutate_genome` (config `mutationOperators`, TCK-184). One call applies one operator,
/// drawn with these relative weights; the `EVO` gene scales the insertion/deletion weights by 2h (h = 0.5: as configured).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationOperators {
    /// Letter moved by 1..=creep_max_step alphabet steps (small value change)
    pub creep: f32,
    /// Letter replaced by a random other letter
    pub point: f32,
    /// One random letter inserted
    pub insertion: f32,
    /// One letter removed
    pub deletion: f32,
    /// A whole gene (promoter..terminator) copied directly after the original
    pub gene_duplication: f32,
    /// A whole non-essential gene removed
    pub gene_deletion: f32,
    pub creep_max_step: usize,
}

/// Body trait genes `parse_genome` reads (first copy each); with `OUT` the genes every structured genome carries.
const BODY_PROMOTERS: [&str; 10] = ["COL", "SIZ", "STF", "WAV", "PUL", "STM", "TEM", "CAR", "REP", "EVO"];
/// Gene terminator as `parse_genome` scans it (degenerate: E + any Delta-class letter).
const TERMINATOR: &str = "EN";
/// Every motif the genome tools know: expressed promoters, `SYM` (legacy) and the terminator.
/// Protected from point-level mutation; never formed by accident in structured genomes.
const GENE_MOTIFS: [&str; 20] = [
    "COL", "SIZ", "STF", "WAV", "PUL", "STM", "TEM", "CAR", "REP", "EVO", "HOX",
    "EYE", "NOS", "TAC", "LUM", "NEU", "SY", "OUT", "SYM", "EN",
];
/// Lengths `mutate_genome` keeps (and `normalize_genome` compiles).
const GENOME_LENGTH_MIN: usize = 128;
const GENOME_LENGTH_MAX: usize = 512;

/// Whether the last base of `dna` completes a motif match other than `own` (start, motif).
fn stray_motif_at_end(dna: &str, own: Option<(usize, &str)>) -> bool {
    let end = dna.len();
    GENE_MOTIFS.iter().any(|&m| end >= m.len() && matches_degenerate(&dna[end - m.len()..], m) && own != Some((end - m.len(), m)))
}

/// Appends one random letter from `allowed` that completes no motif match (any clean letter if none of `allowed` is).
fn push_clean_base<R: rand::Rng + ?Sized>(dna: &mut String, allowed: &[u8], rng: &mut R) {
    for _ in 0..32 {
        dna.push(allowed[rng.gen_range(0..allowed.len())] as char);
        if !stray_motif_at_end(dna, None) {
            return;
        }
        dna.pop();
    }
    let start = rng.gen_range(0..ALPHABET.len());
    for c in allowed.iter().chain(ALPHABET.iter().cycle().skip(start).take(ALPHABET.len())) {
        dna.push(*c as char);
        if !stray_motif_at_end(dna, None) {
            return;
        }
        dna.pop();
    }
    unreachable!("some letter always completes no motif");
}

/// Appends `motif` so that it forms only its own match; adds clean spacer letters while it would complete
/// a stray match with the bases before it.
fn push_motif<R: rand::Rng + ?Sized>(dna: &mut String, motif: &str, rng: &mut R) {
    loop {
        let start = dna.len();
        let fits = motif.chars().all(|c| {
            dna.push(c);
            !stray_motif_at_end(dna, Some((start, motif)))
        });
        if fits {
            return;
        }
        dna.truncate(start);
        push_clean_base(dna, ALPHABET, rng);
    }
}

/// A random spelling `scan_genes` reads as `promoter`: each letter drawn from the letters its degenerate match accepts.
fn random_spelling<R: rand::Rng + ?Sized>(promoter: &str, rng: &mut R) -> String {
    (0..promoter.len())
        .map(|i| {
            let accepted: Vec<u8> = ALPHABET
                .iter()
                .copied()
                .filter(|&c| {
                    let mut variant = promoter.as_bytes().to_vec();
                    variant[i] = c;
                    matches_degenerate(std::str::from_utf8(&variant).unwrap_or(""), promoter)
                })
                .collect();
            accepted[rng.gen_range(0..accepted.len())] as char
        })
        .collect()
}

/// Appends a random degenerate spelling of `promoter` that is not in `used` and forms only its own match, and records
/// it in `used` (distinct spellings keep gene identities apart, TCK-179). A spacer letter changes the context when no
/// drawn spelling fits (e.g. a preceding `E` turns every `N..` spelling into a stray terminator); the canonical
/// spelling is the last resort.
fn push_promoter<R: rand::Rng + ?Sized>(dna: &mut String, promoter: &str, used: &mut Vec<String>, rng: &mut R) {
    for _ in 0..4 {
        let start = dna.len();
        for _ in 0..32 {
            let spelling = random_spelling(promoter, rng);
            if used.contains(&spelling) {
                continue;
            }
            let fits = spelling.chars().all(|c| {
                dna.push(c);
                !stray_motif_at_end(dna, Some((start, promoter)))
            });
            if fits {
                used.push(spelling);
                return;
            }
            dna.truncate(start);
        }
        push_clean_base(dna, ALPHABET, rng);
    }
    push_motif(dna, promoter, rng);
    used.push(dna[dna.len() - promoter.len()..].to_string());
}

/// Letters that give an organ gene of `kind` when they sit at the affinity locus (style at mid range).
fn affinity_letters(kind: OrganKind) -> Vec<u8> {
    ALPHABET
        .iter()
        .copied()
        .filter(|&c| OrganKind::classify(get_payload_linear_value_offset(&(c as char).to_string(), ORGAN_AFFINITY_SHIFT), 0.5) == kind)
        .collect()
}

/// Genes of a structured genome in genome order: promoter and, for a food organ, the OrganKind its payload must give.
/// All body genes and one `OUT` gene, plus organs, `HOX`, `NEU` and `SY` genes as `composition` draws them.
fn structured_gene_plan<R: rand::Rng + ?Sized>(c: &GenomeComposition, rng: &mut R) -> Vec<(&'static str, Option<OrganKind>)> {
    use rand::seq::SliceRandom;
    let count = |rng: &mut R, min: usize, max: usize| rng.gen_range(min..=max.max(min));
    let mut genes: Vec<(&str, Option<OrganKind>)> = BODY_PROMOTERS.iter().map(|&p| (p, None)).collect();
    genes.push((BRAIN_BASE_PROMOTER, None));
    for organ in 0..count(rng, c.organ_count_min, c.organ_count_max) {
        if organ == 0 && rng.gen_bool(c.p_food_organ.clamp(0.0, 1.0) as f64) {
            let eye = rng.gen_bool(c.food_organ_eye_share.clamp(0.0, 1.0) as f64);
            genes.push(if eye { ("EYE", Some(OrganKind::Eye)) } else { ("NOS", Some(OrganKind::Nose)) });
        } else {
            genes.push((ORGAN_PROMOTERS[rng.gen_range(0..ORGAN_PROMOTERS.len())], None));
        }
    }
    if rng.gen_bool(c.p_hox.clamp(0.0, 1.0) as f64) {
        genes.push(("HOX", None));
    }
    genes.extend(std::iter::repeat(("NEU", None)).take(count(rng, c.neu_count_min, c.neu_count_max)));
    genes.extend(std::iter::repeat(("SY", None)).take(count(rng, c.sy_count_min, c.sy_count_max)));
    genes.shuffle(rng);
    genes
}

/// Random genome built from valid genes (TCK-184, see `structured_gene_plan`), in random order with intergenic filler
/// towards the target length. Every gene is promoter (random degenerate spelling, so genes of one type rarely share a
/// spelling) + random payload + terminator; payloads and filler never form a promoter or terminator, so
/// `parse_genome` finds exactly the planned genes.
pub fn generate_structured_genome<R: rand::Rng + ?Sized>(composition: &GenomeComposition, rng: &mut R) -> String {
    let c = composition;
    let genes = structured_gene_plan(c, rng);
    let count = |rng: &mut R, min: usize, max: usize| rng.gen_range(min..=max.max(min));

    // scan_genes looks for the terminator within 40 letters; spacers may lengthen a payload by a few letters
    let payload_lengths: Vec<usize> = genes.iter().map(|_| count(rng, c.payload_length_min.clamp(1, 32), c.payload_length_max.min(32))).collect();
    let gene_bases: usize = genes.iter().zip(&payload_lengths).map(|((p, _), len)| p.len() + len + TERMINATOR.len()).sum();
    let filler = c.target_length.clamp(GENOME_LENGTH_MIN, GENOME_LENGTH_MAX).saturating_sub(gene_bases);
    let mut gaps = vec![0usize; genes.len() + 1];
    for _ in 0..filler {
        let gap = rng.gen_range(0..gaps.len());
        gaps[gap] += 1;
    }

    let mut dna = String::with_capacity(gene_bases + filler + 16);
    let mut used = Vec::new();
    for (g, &(promoter, kind)) in genes.iter().enumerate() {
        for _ in 0..gaps[g] {
            push_clean_base(&mut dna, ALPHABET, rng);
        }
        push_promoter(&mut dna, promoter, &mut used, rng);
        let affinity = kind.map(|k| (ORGAN_AFFINITY_SHIFT % payload_lengths[g], affinity_letters(k)));
        for i in 0..payload_lengths[g] {
            match &affinity {
                Some((locus, letters)) if *locus == i => push_clean_base(&mut dna, letters, rng),
                _ => push_clean_base(&mut dna, ALPHABET, rng),
            }
        }
        push_motif(&mut dna, TERMINATOR, rng);
    }
    for _ in 0..gaps[genes.len()] {
        push_clean_base(&mut dna, ALPHABET, rng);
    }
    while dna.len() < GENOME_LENGTH_MIN {
        push_clean_base(&mut dna, ALPHABET, rng);
    }
    dna
}

/// Positions outside the protected promoter/terminator motifs (the loci of letter-level mutations). Besides every
/// motif match, the promoter letters of every gene `parse_genome` reads are protected (a motif scan can skip a
/// promoter that overlaps an earlier match), so letter-level mutations never change a gene's spelling (`GeneRef`).
fn unprotected_indices(genome: &str) -> Vec<usize> {
    let current_length = genome.len();

    // Protect crucial promoter/terminator motifs from mutations to prevent catastrophic brain-death / synapse loss
    let mut protected = vec![false; current_length];
    for motif in &GENE_MOTIFS {
        let mut idx = 0;
        while idx <= current_length.saturating_sub(motif.len()) {
            let segment = &genome[idx..idx + motif.len()];
            if matches_degenerate(segment, motif) {
                for s in idx..idx + motif.len() {
                    if s < current_length {
                        protected[s] = true;
                    }
                }
                idx += motif.len();
            } else {
                idx += 1;
            }
        }
    }

    for promoter in EXPRESSED_PROMOTERS {
        for gene in scan_genes(genome, promoter, TERMINATOR) {
            protected[gene.start..gene.start + promoter.len()].fill(true);
        }
    }

    // Gather all unprotected indices
    (0..current_length).filter(|&i| !protected[i]).collect()
}

/// Mutation operator of `mutate_genome`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationKind {
    Creep,
    Point,
    Insertion,
    Deletion,
    GeneDuplication,
    GeneDeletion,
}

impl MutationKind {
    pub const ALL: [MutationKind; 6] = [
        MutationKind::Creep,
        MutationKind::Point,
        MutationKind::Insertion,
        MutationKind::Deletion,
        MutationKind::GeneDuplication,
        MutationKind::GeneDeletion,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MutationKind::Creep => "creep",
            MutationKind::Point => "point",
            MutationKind::Insertion => "insertion",
            MutationKind::Deletion => "deletion",
            MutationKind::GeneDuplication => "gene_duplication",
            MutationKind::GeneDeletion => "gene_deletion",
        }
    }
}

/// One applied mutation.
#[derive(Debug, Clone, PartialEq)]
pub struct Mutation {
    pub genome: String,
    pub kind: MutationKind,
    /// The changed, inserted or removed base; for gene operators the first base of the original gene
    pub index: usize,
}

/// Genes per expressed promoter (`parse_genome`'s gene table counts).
fn gene_counts(genome: &str) -> Vec<usize> {
    EXPRESSED_PROMOTERS.iter().map(|p| scan_genes(genome, p, TERMINATOR).len()).collect()
}

/// Terminated genes that overlap no other gene: (promoter index into `EXPRESSED_PROMOTERS`, start, end).
fn whole_genes(genome: &str) -> Vec<(usize, usize, usize)> {
    let all: Vec<(usize, usize, usize)> = EXPRESSED_PROMOTERS
        .iter()
        .enumerate()
        .flat_map(|(p, promoter)| scan_genes(genome, promoter, TERMINATOR).into_iter().map(move |g| (p, g.start, g.end)))
        .collect();
    let mut genes: Vec<(usize, usize, usize)> = all
        .iter()
        .copied()
        .filter(|&(_, _, e)| matches_degenerate(&genome[e - TERMINATOR.len()..e], TERMINATOR))
        .filter(|&(p, s, e)| !all.iter().any(|&(q, s2, e2)| (q, s2) != (p, s) && s2 < e && s < e2))
        .collect();
    genes.sort_by_key(|&(p, s, _)| (s, p));
    genes
}

/// `genome` with `range` replaced by `insert`, such that only promoter `p` changes its gene count, by `delta`.
/// Joins that would form a stray gene get one spacer letter; `None` if no tried spacer gives a clean result.
fn splice_gene<R: rand::Rng + ?Sized>(genome: &str, range: std::ops::Range<usize>, insert: &str, p: usize, delta: isize, rng: &mut R) -> Option<String> {
    let mut expected = gene_counts(genome);
    expected[p] = expected[p].checked_add_signed(delta)?;
    let mut spacer = String::new();
    for attempt in 0..8 {
        if attempt > 0 {
            spacer = (ALPHABET[rng.gen_range(0..26)] as char).to_string();
        }
        let candidate = format!("{}{}{}{}", &genome[..range.start], spacer, insert, &genome[range.end..]);
        if (GENOME_LENGTH_MIN..=GENOME_LENGTH_MAX).contains(&candidate.len()) && gene_counts(&candidate) == expected {
            return Some(candidate);
        }
    }
    None
}

/// Applies one mutation of `kind` at a random applicable locus; `None` if the kind is not applicable
/// (length bounds [128, 512], no unprotected locus, no suitable gene, or no clean gene splice).
fn apply_mutation<R: rand::Rng + ?Sized>(genome: &str, kind: MutationKind, ops: &MutationOperators, rng: &mut R) -> Option<(String, usize)> {
    let current_length = genome.len();
    if current_length == 0 {
        return None;
    }
    let pick_gene = |genes: &[(usize, usize, usize)], rng: &mut R| genes[rng.gen_range(0..genes.len())];
    match kind {
        MutationKind::GeneDuplication => {
            let genes = whole_genes(genome);
            if genes.is_empty() {
                return None;
            }
            let (p, start, end) = pick_gene(&genes, rng);
            splice_gene(genome, end..end, &genome[start..end], p, 1, rng).map(|g| (g, start))
        }
        MutationKind::GeneDeletion => {
            let counts = gene_counts(genome);
            let essential = |p: usize| BODY_PROMOTERS.contains(&EXPRESSED_PROMOTERS[p]) || EXPRESSED_PROMOTERS[p] == BRAIN_BASE_PROMOTER;
            let genes: Vec<_> = whole_genes(genome).into_iter().filter(|&(p, _, _)| !essential(p) || counts[p] > 1).collect();
            if genes.is_empty() {
                return None;
            }
            let (p, start, end) = pick_gene(&genes, rng);
            splice_gene(genome, start..end, "", p, -1, rng).map(|g| (g, start))
        }
        _ => {
            let unprotected = unprotected_indices(genome);
            let index = if unprotected.is_empty() { rng.gen_range(0..current_length) } else { unprotected[rng.gen_range(0..unprotected.len())] };
            let mut new_genome = String::from(genome);
            match kind {
                MutationKind::Deletion => {
                    if current_length <= GENOME_LENGTH_MIN || unprotected.is_empty() {
                        return None;
                    }
                    new_genome.remove(index);
                }
                MutationKind::Insertion => {
                    if current_length >= GENOME_LENGTH_MAX {
                        return None;
                    }
                    new_genome.insert(index, ALPHABET[rng.gen_range(0..26)] as char);
                }
                MutationKind::Point => {
                    let current_char = genome.as_bytes()[index];
                    let mut new_char = current_char;
                    while new_char == current_char {
                        new_char = ALPHABET[rng.gen_range(0..26)];
                    }
                    new_genome.replace_range(index..index + 1, &(new_char as char).to_string());
                }
                _ => {
                    // Creep: 1..=step alphabet steps (A..Z, no wrap-around), turned back at the alphabet ends
                    let value = char_to_value(genome.as_bytes()[index] as char) as isize;
                    let step = rng.gen_range(1..=ops.creep_max_step.clamp(1, 12)) as isize;
                    let up = rng.gen_bool(0.5);
                    let mut new_value = if up { value + step } else { value - step };
                    if !(0..26).contains(&new_value) {
                        new_value = if up { value - step } else { value + step };
                    }
                    new_genome.replace_range(index..index + 1, &(ALPHABET[new_value as usize] as char).to_string());
                }
            }
            Some((new_genome, index))
        }
    }
}

/// One mutation of a fixed kind at a random locus (diagnostic probes, TCK-172/184): the operators of
/// `mutate_genome` without the kind roll and without the creep fallback. `None` if the kind is not applicable.
pub fn mutate_genome_with_kind<R: rand::Rng + ?Sized>(genome: &str, kind: MutationKind, ops: &MutationOperators, rng: &mut R) -> Option<String> {
    apply_mutation(genome, kind, ops, rng).map(|(g, _)| g)
}

/// One mutation (TCK-184): the operator is drawn by the `ops` weights, the insertion/deletion weights scaled by the
/// genome's `EVO` rates (2h, so h = 0.5 keeps the configured weight). An operator that is not applicable
/// (length bounds [128, 512], no suitable gene) falls back to creep. `None` only for an empty genome.
pub fn mutate_genome<R: rand::Rng + ?Sized>(genome: &str, ops: &MutationOperators, rng: &mut R) -> Option<Mutation> {
    let (insertion_rate, deletion_rate) = read_evolution_rates(genome);
    let weights = [
        ops.creep,
        ops.point,
        ops.insertion * insertion_rate / MAX_INDEL_RATE * 2.0,
        ops.deletion * deletion_rate / MAX_INDEL_RATE * 2.0,
        ops.gene_duplication,
        ops.gene_deletion,
    ];
    let total: f32 = weights.iter().map(|w| w.max(0.0)).sum();
    let mut kind = MutationKind::Creep;
    if total > 0.0 {
        let mut roll = rng.gen_range(0.0..total);
        for (k, w) in MutationKind::ALL.into_iter().zip(weights) {
            if roll < w.max(0.0) {
                kind = k;
                break;
            }
            roll -= w.max(0.0);
        }
    }
    if let Some((genome, index)) = apply_mutation(genome, kind, ops, rng) {
        return Some(Mutation { genome, kind, index });
    }
    apply_mutation(genome, MutationKind::Creep, ops, rng).map(|(genome, index)| Mutation { genome, kind: MutationKind::Creep, index })
}

// Embedded structs for derive_ecological_metrics return
struct EcologicalMetrics {
    latin_name: String,
    visus: f32,
    olfaction: f32,
    tactility: f32,
    biolum: f32,
    diet: String,
    habitat: String,
    survival_score: f32,
    survival_analysis: String,
}

/// Procedural Nomenclature, Ecological and Survival Fitness Analyzer
fn derive_ecological_metrics(
    genome: &str,
    symmetry: &str,
    emergent_segments_count: usize,
    organelles: &[SensoryPatch],
    primary_color: HSLColor,
    stiffness: f32,
    pulse_speed: f32,
    wave_phase: f32,
    carnivory: f32,
) -> EcologicalMetrics {
    let mut visus_score = 0.0;
    let mut olfaction_score = 0.0;
    let mut tactility_score = 0.0;
    let mut biolum_score = 0.0;

    for patch in organelles {
        let band_adjusted = 0.2 + patch.bandwidth * 0.8;
        let aff = patch.spectral_affinity;
        let power = patch.scale * 22.0;

        let w_mech = (1.0 - (aff - 0.0).abs() / band_adjusted).max(0.0);
        let w_chem = (1.0 - (aff - 0.45).abs() / band_adjusted).max(0.0);
        let w_thermal = (1.0 - (aff - 0.75).abs() / band_adjusted).max(0.0);
        let w_light = (1.0 - (aff - 1.0).abs() / band_adjusted).max(0.0);

        visus_score += w_light * power;
        olfaction_score += w_chem * power;
        tactility_score += w_mech * power;
        biolum_score += w_thermal * power;
    }

    let visus = visus_score.min(100.0).round();
    let olfaction = olfaction_score.min(100.0).round();
    let tactility = tactility_score.min(100.0).round();
    let biolum = biolum_score.min(100.0).round();

    let is_predator = carnivory >= 0.55;
    let mut diet = String::from("Ancestral Filter Feeder (Detritus)");

    let mut scores = vec![
        ("light", visus),
        ("chemical", olfaction),
        ("kinetic", tactility),
        ("thermal", biolum),
    ];
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    if carnivory >= 0.60 {
        diet = String::from("Sabertooth Hunter (Carnivore)");
    } else if carnivory >= 0.40 {
        diet = String::from("Omnivore (Omnivorous)");
    } else if scores[0].1 > 15.0 {
        match scores[0].0 {
            "light" => diet = String::from("Diatom Grazer (Herbivore)"),
            "chemical" => diet = String::from("Chemotactic Filter Feeder"),
            "kinetic" => diet = String::from("Vibrational Plankton Feeder"),
            "thermal" => diet = String::from("Deep Sea Thermotroph"),
            _ => {}
        }
    }

    let mut habitat = String::from("Epipelagic (Sunlit Zone)");
    if biolum > 40.0 {
        habitat = String::from("Bathypelagic (Abyssal Zone)");
    } else if tactility > 45.0 || emergent_segments_count >= 4 {
        habitat = String::from("Hadopelagic (Benthic Trench)");
    } else if primary_color.l < 45.0 {
        habitat = String::from("Mesopelagic (Twilight Swamps)");
    }

    // Procedural Taxonomy Latin Name
    let seg_prefixes = ["Monoplasma", "Biplasma", "Triplasma", "Tetraplasma", "Pentaplasma"];
    let prefix = seg_prefixes[(emergent_segments_count.saturating_sub(1)).min(4)];
    let mid_sym = if symmetry == "vertical" { "bilateralis" } else { "tetramerum" };

    let mut suffix = String::from("lēvis");
    if is_predator {
        suffix = String::from("raptor");
    } else if !organelles.is_empty() {
        // Fixed type order breaks ties deterministically (first type in this order wins)
        let type_order = ["light", "chem", "kinetic", "thermal"];
        let mut type_counts = [0usize; 4];
        for patch in organelles {
            let aff = patch.spectral_affinity;
            let key = if aff >= 0.8 {
                "light"
            } else if (0.25..=0.65).contains(&aff) {
                "chem"
            } else if aff > 0.65 && aff < 0.8 {
                "thermal"
            } else {
                "kinetic"
            };
            if let Some(i) = type_order.iter().position(|t| *t == key) {
                type_counts[i] += 1;
            }
        }
        let mut dominant = 0;
        for i in 1..type_order.len() {
            if type_counts[i] > type_counts[dominant] {
                dominant = i;
            }
        }
        suffix = match type_order[dominant] {
            "light" => String::from("ocularium"),
            "chem" => String::from("ciliatum"),
            "kinetic" => String::from("vibrans"),
            "thermal" => String::from("thermum"),
            _ => String::from("lēvis"),
        };
    }

    // Generate unique Strain ID
    let first_loci = char_to_value(genome.chars().next().unwrap_or('A'));
    let last_loci = char_to_value(genome.chars().last().unwrap_or('A'));
    let mid_loci = char_to_value(genome.chars().nth(genome.len() / 2).unwrap_or('A'));
    
    let hash_char1 = ALPHABET[(first_loci + last_loci) % 26] as char;
    let hash_char2 = ALPHABET[(mid_loci + last_loci) % 26] as char;
    let hash_num = (first_loci * mid_loci + last_loci) % 10;
    let strain_suffix = format!("(Str. {}{}-{})", hash_char1, hash_char2, hash_num);

    let latin_name = format!("{} {} {} {}", prefix, mid_sym, suffix, strain_suffix);

    // Survival Estimation Computation
    let mut survival_score = 40.0;
    let freq_hz = pulse_speed * 1000.0;
    let waves_thrust_factor = wave_phase.sin();
    let locomotive_efficiency = stiffness * (freq_hz * 0.15) * if waves_thrust_factor > 0.1 { waves_thrust_factor } else { 0.0 };
    let loco_bonus = (locomotive_efficiency * 30.0).round().min(35.0);
    survival_score += loco_bonus;

    let max_sensory = visus.max(olfaction).max(tactility).max(biolum);
    if max_sensory < 10.0 {
        survival_score -= 25.0;
    } else if max_sensory > 55.0 {
        survival_score += 15.0;
    } else {
        survival_score += 10.0;
    }

    let metabolic_tax = organelles.len() as f32 * 4.5;
    survival_score -= metabolic_tax;

    if is_predator {
        survival_score += 15.0;
    } else {
        survival_score += 5.0;
    }

    let final_survival_score = survival_score.clamp(1.0, 100.0).round();

    let survival_analysis = if final_survival_score >= 80.0 {
        if is_predator {
            "Excellent hunting potential. Highly elastic thrust paired with rigid, destructive impact force."
        } else {
            "Outstanding chance of survival. Extremely agile, highly responsive, and energetically highly efficient."
        }
    } else if final_survival_score >= 60.0 {
        "Good. Stable life form with pronounced ecological maturity at moderate efficiency."
    } else if final_survival_score >= 40.0 {
        "Mediocre. Undefined spectral receptors or unfavorable power-to-mass ratio."
    } else if final_survival_score >= 20.0 {
        "Endangered. Poorly coupled flagellar waves, high metabolic burden."
    } else {
        "Critical. Sensorially isolated, immobile organism; easy prey for predators."
    };

    EcologicalMetrics {
        latin_name,
        visus,
        olfaction,
        tactility,
        biolum,
        diet,
        habitat,
        survival_score: final_survival_score,
        survival_analysis: String::from(survival_analysis),
    }
}

pub fn get_payload_linear_value_offset(payload: &str, shift: usize) -> f32 {
    if payload.is_empty() {
        return 0.5;
    }
    let chars: Vec<char> = payload.chars().collect();
    // Deterministically select exactly one character locus based on the shift offset
    let idx = shift % chars.len();
    let c = chars[idx];
    
    // Add shift to character value, wrapping modulo 26, to get a beautifully uniform single-character value!
    let val = (char_to_value(c) + shift) % 26;
    val as f32 / 25.0
}

/// Uppercases and pads/truncates a genome to the compiled length range [128, 512].
pub fn normalize_genome(genome: &str) -> String {
    let current_length = genome.len().clamp(128, 512);
    let mut clean_genome = genome.to_ascii_uppercase().chars().take(current_length).collect::<String>();
    while clean_genome.len() < current_length {
        clean_genome.push('A');
    }
    clean_genome
}

/// Marks open chromatin (promoter..terminator spans of known genes); returns the state and transcription logs.
fn scan_chromatin(clean_genome: &str) -> (Vec<bool>, Vec<String>) {
    let current_length = clean_genome.len();
    let mut chromatin_state = vec![false; current_length];
    let mut logs = Vec::new();

    // Scan for active specialized organ and trait promoters to highlight active gene spans on the DNA Helix
    let active_promoters = [
        "COL", "STF", "PUL", "SIZ", "WAV", "SYM", "STM", "TEM",
        "EYE", "NOS", "TAC", "LUM", "CAR", "REP", "EVO", "NEU", "SY", "OUT"
    ];
    for promoter in &active_promoters {
        let mut idx = 0;
        while idx <= current_length.saturating_sub(promoter.len()) {
            let segment_start = &clean_genome[idx..idx + promoter.len()];
            if matches_degenerate(segment_start, promoter) {
                let payload_start = idx + promoter.len();
                let mut payload_end = None;
                for j in payload_start..=current_length.saturating_sub(2) {
                    let segment_stop = &clean_genome[j..j + 2];
                    if matches_degenerate(segment_stop, "SP") || matches_degenerate(segment_stop, "EN") {
                        payload_end = Some(j);
                        break;
                    }
                }

                if let Some(end_idx) = payload_end {
                    for s in idx..(end_idx + 2) {
                        if s < current_length {
                            chromatin_state[s] = true;
                        }
                    }
                    logs.push(format!("Active Gene Found: Transcribing \"{}\" promoter block at locus {}..{} successfully completed.", promoter, idx, end_idx + 2));
                    idx = end_idx + 2;
                } else {
                    idx += 1;
                }
            } else {
                idx += 1;
            }
        }
    }
    (chromatin_state, logs)
}

/// Insertion/deletion rates exactly as `parse_genome` derives them, without compiling the full phenotype.
pub fn read_evolution_rates(genome: &str) -> (f32, f32) {
    let clean_genome = normalize_genome(genome);
    let evo_source = first_payload(&clean_genome, "EVO");
    let h_insert = get_payload_linear_value_offset(&evo_source, 0);
    let h_delete = get_payload_linear_value_offset(&evo_source, 5);
    (h_insert * MAX_INDEL_RATE, h_delete * MAX_INDEL_RATE)
}

/// Promoters whose genes `parse_genome` expresses.
const EXPRESSED_PROMOTERS: [&str; 18] = [
    "COL", "SIZ", "STF", "WAV", "PUL", "STM", "TEM", "CAR", "REP", "EVO", "HOX",
    "EYE", "NOS", "TAC", "LUM", "NEU", "SY", "OUT",
];
/// Organ genes in organ (and input slot) order.
const ORGAN_PROMOTERS: [&str; 4] = ["EYE", "NOS", "TAC", "LUM"];
/// Brain-base gene: motor bias/tau and systemic → motor weights.
const BRAIN_BASE_PROMOTER: &str = "OUT";
/// Payload locus shift of an organ gene's spectral affinity (decides its OrganKind with the expression style).
const ORGAN_AFFINITY_SHIFT: usize = 1;
/// Insertion/deletion rate at EVO h = 1 (rates are h * MAX_INDEL_RATE).
const MAX_INDEL_RATE: f32 = 0.12;
/// Synapse keys keep 53 bits so they stay exact as JavaScript numbers.
const KEY_MASK: u64 = (1 << 53) - 1;

/// Payload of the first gene of a promoter type; empty (all values h = 0.5) when the gene is missing.
fn first_payload(genome: &str, promoter: &str) -> String {
    scan_genes(genome, promoter, "EN").first().map(|g| g.payload.to_string()).unwrap_or_default()
}

/// Expressed genes of a genome: per promoter in genome order, plus all spans sorted by position (owner ids).
struct GeneTable<'a> {
    by_promoter: Vec<(&'static str, Vec<ScannedGene<'a>>)>,
    spans: Vec<GeneSpan>,
}

impl<'a> GeneTable<'a> {
    fn scan(genome: &'a str) -> Self {
        let by_promoter: Vec<(&'static str, Vec<ScannedGene<'a>>)> = EXPRESSED_PROMOTERS.iter().map(|&p| (p, scan_genes(genome, p, "EN"))).collect();
        let mut spans: Vec<GeneSpan> = by_promoter
            .iter()
            .flat_map(|(p, genes)| {
                genes.iter().map(move |g| GeneSpan {
                    gene: GeneRef { promoter: p.to_string(), spelling: genome[g.start..g.start + p.len()].to_string(), occurrence: 0 },
                    start: g.start,
                    end: g.end,
                })
            })
            .collect();
        spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.gene.promoter.cmp(&b.gene.promoter)));
        for i in 0..spans.len() {
            let same = |s: &GeneSpan| s.gene.promoter == spans[i].gene.promoter && s.gene.spelling == spans[i].gene.spelling;
            spans[i].gene.occurrence = spans[..i].iter().filter(|s| same(s)).count();
        }
        Self { by_promoter, spans }
    }

    fn genes(&self, promoter: &str) -> &[ScannedGene<'a>] {
        self.by_promoter.iter().find(|(p, _)| *p == promoter).map(|(_, g)| &g[..]).unwrap_or(&[])
    }

    fn first_payload(&self, promoter: &str) -> &'a str {
        self.genes(promoter).first().map(|g| g.payload).unwrap_or("")
    }

    /// Owner id (index into `spans`) of the gene of `promoter` that starts at `start`.
    fn owner(&self, promoter: &str, start: usize) -> usize {
        self.spans.iter().position(|s| s.start == start && s.gene.promoter == promoter).expect("scanned gene is in the gene table")
    }

    /// Stable identity of the gene with owner id `owner`.
    fn gene_ref(&self, owner: usize) -> GeneRef {
        self.spans[owner].gene.clone()
    }
}

/// An organ gene and the organs it expressed (`organelles[organs]`: one, or left + right of a HOX pair).
struct OrganGene<'a> {
    gene: GeneRef,
    /// 1-based position among the genes of its promoter type (labels only)
    number: usize,
    owner: usize,
    payload: &'a str,
    organs: std::ops::Range<usize>,
}

/// A hidden neuron decoded from its `NEU` gene.
struct HiddenGene<'a> {
    gene: GeneRef,
    owner: usize,
    payload: &'a str,
    bias: f32,
    tau: f32,
    activation: &'static str,
    depth: f32,
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Stable synapse identity from the source and target roles (independent of neuron indices and genome positions).
fn synapse_key(src: &NeuronRole, dst: &NeuronRole) -> u64 {
    let mut bytes = Vec::with_capacity(32);
    src.encode(false, &mut bytes);
    bytes.push(0xFF);
    dst.encode(false, &mut bytes);
    fnv1a(&bytes) & KEY_MASK
}

/// Initial weight of a synapse owned by `owner`: one entry of the owner's payload tape, letter A..Z mapped to [-0.5, 0.5].
/// The entry is picked by hashing both roles with the owner's own gene identity left out, so it depends neither on
/// the owner's identity nor on other genes. `None` for an empty payload (the synapse is not wired).
fn tape_weight(tape: &str, owner: &GeneRef, src: &NeuronRole, dst: &NeuronRole) -> Option<f32> {
    if tape.is_empty() {
        return None;
    }
    let mut bytes = Vec::with_capacity(32);
    src.encode(src.gene() == Some(owner), &mut bytes);
    bytes.push(0xFF);
    dst.encode(dst.gene() == Some(owner), &mut bytes);
    let slot = (fnv1a(&bytes) % tape.len() as u64) as usize;
    Some(char_to_value(tape.as_bytes()[slot] as char) as f32 / 25.0 - 0.5)
}

/// Letter index 0..=25 read like `get_payload_linear_value_offset` (discrete choices).
fn payload_index(payload: &str, shift: usize) -> usize {
    (get_payload_linear_value_offset(payload, shift) * 25.0).round() as usize
}

fn channel_name(channel: usize) -> String {
    if channel == DELTA_CHANNEL { String::from("ΔI") } else { format!("Ch {}", channel + 1) }
}

fn sensor_label(patch: &SensoryPatch, channel: usize) -> String {
    let (icon, names) = match patch.kind {
        OrganKind::Eye => ("👁️", ["UV Vision", "Blue Vision", "Green Vision", "Red Vision", "IR Vision"]),
        OrganKind::Nose => ("👃", ["Acid Smell", "Sugar Smell", "Keton Smell", "Protein Smell", "Phero Smell"]),
        OrganKind::Thermal => ("🌡️", ["Thermal UV", "Thermal Blue", "Thermal Green", "Thermal Red", "Thermal IR"]),
        OrganKind::Bristle | OrganKind::LateralLine | OrganKind::Proprioceptor => {
            ("🔊", ["Tactile Hardness", "Tactile Flow", "Tactile Temperature", "Tactile Strain", "Tactile Pain"])
        }
    };
    let name = if channel == DELTA_CHANNEL { "Change ΔI" } else { names[channel] };
    format!("{} {} ({}°)", icon, name, patch.angle.round() as i32)
}

fn thalamus_label_prefix(og: &OrganGene, organelles: &[SensoryPatch]) -> String {
    let deg = |o: usize| organelles[o].angle.round() as i32;
    let name = format!("{} #{}", og.gene.promoter, og.number);
    if og.organs.len() == 2 {
        format!("Pair {} ({}°/{}°)", name, deg(og.organs.start), deg(og.organs.start + 1))
    } else {
        format!("Organelle {} ({}°)", name, deg(og.organs.start))
    }
}

/// Brain v2 (TCK-179/182/144/153). Neuron order Inputs (sensors, systemic) → Thalamus → Hidden (by depth) → Motors,
/// so one ascending pass integrates in evaluation order (sensor → motor latency 1 tick). Targeted wiring of live
/// structure only; every synapse has a role key and exactly one owner gene:
/// - sensor → thalamus (fixed ±1) and thalamus → motor: the organ gene;
/// - thalamus/systemic → hidden and hidden → motor: the hidden neuron's `NEU` gene;
/// - systemic → motor: the `OUT` brain-base gene (not wired without one);
/// - explicit `SY` genes override or add one synapse each (first `SY` gene per pair wins).
fn compile_brain(genes: &GeneTable, organelles: &[SensoryPatch], organ_genes: &[OrganGene]) -> BrainTopology {
    let input_count = organelles.len() * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT;
    let neuron = |id: usize, neuron_type: NeuronType, label: String, role: NeuronRole, tau: f32, bias: f32, activation: Option<&str>, x: f32, y: f32| CTRNNNeuron {
        id,
        neuron_type,
        label,
        role,
        tau,
        bias,
        activation_type: activation.map(String::from),
        x: Some(x),
        y: Some(y),
    };
    let input_y = |id: usize| (id as f32 + 1.0) / (input_count as f32 + 1.0);
    let mut neurons: Vec<CTRNNNeuron> = Vec::new();

    // A. Sensors: CHANNELS_PER_ORGANELLE input slots per organ in organ order (layout of `compute_sensory_inputs`)
    for og in organ_genes {
        for patch in &organelles[og.organs.clone()] {
            for channel in 0..CHANNELS_PER_ORGANELLE {
                let id = neurons.len();
                let role = NeuronRole::Sensor { gene: og.gene.clone(), side: patch.side, channel, kind: patch.kind };
                neurons.push(neuron(id, NeuronType::Input, sensor_label(patch, channel), role, 1.0, 0.0, None, 0.0, input_y(id)));
            }
        }
    }

    // B. Systemic inputs
    for input in SystemicInput::ALL {
        let id = neurons.len();
        neurons.push(neuron(id, NeuronType::Input, input.label().to_string(), NeuronRole::Systemic { input }, 1.0, 0.0, None, 0.0, input_y(id)));
    }
    let systemic: Vec<usize> = (input_count - SYSTEMIC_BASE_INPUTS_COUNT..input_count).collect();

    // C. Thalamus relays (tau TAU_MIN: near pass-through): per organ gene and live channel a Sum node, plus a Diff node for HOX pairs
    let paired = |og: &OrganGene| og.organs.len() == 2;
    let live: Vec<[bool; CHANNELS_PER_ORGANELLE]> = organ_genes.iter().map(|og| live_channels(&organelles[og.organs.start])).collect();
    let thalamus_count: usize = organ_genes.iter().zip(&live).map(|(og, l)| l.iter().filter(|&&x| x).count() * if paired(og) { 2 } else { 1 }).sum();
    let mut thalamus: Vec<(usize, usize, ThalamusKind, usize)> = Vec::new(); // (organ gene, channel, node, neuron id)
    for (g, og) in organ_genes.iter().enumerate() {
        let nodes: &[ThalamusKind] = if paired(og) { &[ThalamusKind::Sum, ThalamusKind::Diff] } else { &[ThalamusKind::Sum] };
        let prefix = thalamus_label_prefix(og, organelles);
        for &node in nodes {
            for channel in (0..CHANNELS_PER_ORGANELLE).filter(|&c| live[g][c]) {
                let id = neurons.len();
                let (name, activation) = match node {
                    ThalamusKind::Sum => ("Sum", "sigmoid"),
                    ThalamusKind::Diff => ("Diff", "tanh"),
                };
                let label = format!("🧠 Thalamus {} {} ({})", name, prefix, channel_name(channel));
                let y = (thalamus.len() + 1) as f32 / (thalamus_count as f32 + 1.0);
                let role = NeuronRole::Thalamus { gene: og.gene.clone(), channel, node };
                neurons.push(neuron(id, NeuronType::Hidden, label, role, TAU_MIN, 0.0, Some(activation), 0.20, y));
                thalamus.push((g, channel, node, id));
            }
        }
    }
    let thalamus_ids: Vec<usize> = thalamus.iter().map(|t| t.3).collect();

    // D. Hidden neurons: one per NEU gene (first MAX_HIDDEN_NEURONS); lower depth integrates first
    let hidden: Vec<HiddenGene> = genes
        .genes("NEU")
        .iter()
        .take(MAX_HIDDEN_NEURONS)
        .map(|gene| {
            let p = gene.payload;
            let owner = genes.owner("NEU", gene.start);
            HiddenGene {
                gene: genes.gene_ref(owner),
                owner,
                payload: p,
                bias: get_payload_linear_value_offset(p, 1) * 2.0 - 1.0,
                tau: (0.2 + get_payload_linear_value_offset(p, 4) * 1.8).clamp(TAU_MIN, TAU_MAX),
                activation: match (get_payload_linear_value_offset(p, 7) * 100.0) as usize % 4 {
                    1 => "relu",
                    2 => "sigmoid",
                    3 => "sin",
                    _ => "tanh",
                },
                depth: 0.55 + get_payload_linear_value_offset(p, 10) * 0.30,
            }
        })
        .collect();
    let mut by_depth: Vec<usize> = (0..hidden.len()).collect();
    by_depth.sort_by(|&a, &b| hidden[a].depth.total_cmp(&hidden[b].depth).then(a.cmp(&b)));
    let mut hidden_ids = vec![0; hidden.len()]; // by NEU gene order
    for h in by_depth {
        let hg = &hidden[h];
        let id = neurons.len();
        hidden_ids[h] = id;
        let role = NeuronRole::Hidden { gene: hg.gene.clone() };
        neurons.push(neuron(id, NeuronType::Hidden, format!("Hidden #{}", h + 1), role, hg.tau, hg.bias, Some(hg.activation), 0.5, hg.depth));
    }

    // E. Motors: bias and tau from the OUT brain-base gene (without one: empty payload, bias 0, tau 1.1)
    let out_gene = genes.genes(BRAIN_BASE_PROMOTER).first();
    let out_payload = out_gene.map(|g| g.payload).unwrap_or("");
    let mut motor_ids = Vec::new();
    for (i, motor) in Motor::ALL.into_iter().enumerate() {
        let bias = get_payload_linear_value_offset(out_payload, i + 1) * 2.0 - 1.0;
        let tau = (0.2 + get_payload_linear_value_offset(out_payload, i + 10) * 1.8).clamp(TAU_MIN, TAU_MAX);
        let label = match motor {
            Motor::Thrust => "Thrust (Fwd/Bwd)",
            Motor::Bend => "Bending (Left/Right)",
        };
        let id = neurons.len();
        motor_ids.push(id);
        neurons.push(neuron(id, NeuronType::Output, label.to_string(), NeuronRole::Motor { motor }, tau, bias, None, 1.0, (i + 1) as f32 / 3.0));
    }

    // Synapses
    let mut synapses: Vec<CTRNNSynapse> = Vec::new();
    let wire = |synapses: &mut Vec<CTRNNSynapse>, from: usize, to: usize, weight: f32, owner: usize| {
        let (source, target) = (&neurons[from].role, &neurons[to].role);
        let key = synapse_key(source, target);
        // Sensor → thalamus reflexes (fixed ±1 sum/difference relays) do not learn (TCK-152)
        let plastic = !matches!((source, target), (NeuronRole::Sensor { .. }, NeuronRole::Thalamus { .. }));
        synapses.push(CTRNNSynapse { from_node: from, to_node: to, weight, key, owner, plastic });
    };
    let tape = |payload: &str, owner: &GeneRef, from: usize, to: usize| tape_weight(payload, owner, &neurons[from].role, &neurons[to].role);

    // 1. Reflex sensor → thalamus (fixed ±1, organ gene); a pair shares one weight set through its Sum/Diff nodes
    for &(g, channel, node, id) in &thalamus {
        let og = &organ_genes[g];
        let sensor = |o: usize| o * CHANNELS_PER_ORGANELLE + channel;
        let first = og.organs.start;
        match (node, paired(og)) {
            (ThalamusKind::Sum, false) => wire(&mut synapses, sensor(first), id, 1.0, og.owner),
            (ThalamusKind::Sum, true) => {
                wire(&mut synapses, sensor(first), id, 1.0, og.owner);
                wire(&mut synapses, sensor(first + 1), id, 1.0, og.owner);
            }
            (ThalamusKind::Diff, _) => {
                wire(&mut synapses, sensor(first), id, 1.0, og.owner);
                wire(&mut synapses, sensor(first + 1), id, -1.0, og.owner);
            }
        }
    }

    // 2. Thalamus → motors (organ gene tape)
    for &(g, _, _, id) in &thalamus {
        let og = &organ_genes[g];
        for &m in &motor_ids {
            if let Some(w) = tape(og.payload, &og.gene, id, m) {
                wire(&mut synapses, id, m, w, og.owner);
            }
        }
    }

    // 3. Into and out of hidden neurons (NEU gene tape): thalamus → hidden, systemic → hidden, hidden → motors
    for (hg, &hid) in hidden.iter().zip(&hidden_ids) {
        for &src in thalamus_ids.iter().chain(&systemic) {
            if let Some(w) = tape(hg.payload, &hg.gene, src, hid) {
                wire(&mut synapses, src, hid, w, hg.owner);
            }
        }
        for &m in &motor_ids {
            if let Some(w) = tape(hg.payload, &hg.gene, hid, m) {
                wire(&mut synapses, hid, m, w, hg.owner);
            }
        }
    }

    // 4. Systemic → motors (OUT gene tape)
    if let Some(out) = out_gene {
        let owner = genes.owner(BRAIN_BASE_PROMOTER, out.start);
        let out_ref = genes.gene_ref(owner);
        for &s in &systemic {
            for &m in &motor_ids {
                if let Some(w) = tape(out.payload, &out_ref, s, m) {
                    wire(&mut synapses, s, m, w, owner);
                }
            }
        }
    }

    // 5. Explicit SY genes address roles: (source class, index, target class, index), index modulo the class size;
    //    weight in [-2, 2]. Sources: systemic, thalamus, hidden; targets: hidden, motors (hidden in NEU gene order).
    let sources: [&[usize]; 3] = [&systemic, &thalamus_ids, &hidden_ids];
    let targets: [&[usize]; 2] = [&hidden_ids, &motor_ids];
    let mut explicit = vec![false; synapses.len()];
    for gene in genes.genes("SY").iter().take(MAX_EXPLICIT_SYNAPSES) {
        let p = gene.payload;
        let (src_class, dst_class) = (sources[payload_index(p, 1) % 3], targets[payload_index(p, 3) % 2]);
        if src_class.is_empty() || dst_class.is_empty() {
            continue;
        }
        let from = src_class[payload_index(p, 2) % src_class.len()];
        let to = dst_class[payload_index(p, 4) % dst_class.len()];
        if from == to {
            continue;
        }
        let weight = get_payload_linear_value_offset(p, 7) * 4.0 - 2.0;
        let owner = genes.owner("SY", gene.start);
        match synapses.iter().position(|s| s.from_node == from && s.to_node == to) {
            Some(i) if !explicit[i] => {
                synapses[i].weight = weight;
                synapses[i].owner = owner;
                explicit[i] = true;
            }
            Some(_) => {} // the first SY gene of a pair wins
            None => {
                wire(&mut synapses, from, to, weight, owner);
                explicit.push(true);
            }
        }
    }

    // Sorted by target for the single linear pass in execute_brain_with_learning
    synapses.sort_by_key(|s| s.to_node);
    BrainTopology { neurons, synapses }
}

/// DNA De-compiler & Phenotype De-compiler (Genotype to Epigenetic Phenotype Compiler)
pub fn parse_genome(genome: &str, antisense_input: Option<&str>, parent_methylations: Option<&[f32]>) -> CreaturePhenotype {
    let clean_genome = normalize_genome(genome);
    let current_length = clean_genome.len();

    let mut antisense_strand = match antisense_input {
        Some(anti) => anti.to_ascii_uppercase().chars().take(current_length).collect::<String>(),
        None => get_complementary_string(&clean_genome),
    };
    while antisense_strand.len() < current_length {
        antisense_strand.push('Z');
    }

    // 1. Epigenetic chromatin mapping (Dynamic Active Gene Scanning for specialized organs - TCK-116)
    let mut epigenetic_logs = Vec::new();
    
    if let Some(m) = parent_methylations {
        if m.len() == current_length {
            epigenetic_logs.push(String::from("Transgenerational epigenetic inheritance: Learned brain methylation pattern of the parent cell successfully assimilated."));
        }
    }
    let methylations = match parent_methylations {
        Some(m) if m.len() == current_length => m.to_vec(),
        _ => vec![0.0; current_length],
    };

    let (chromatin_state, gene_logs) = scan_chromatin(&clean_genome);
    epigenetic_logs.extend(gene_logs);

    // --- MODULAR GENOME LOCI LINEAR COMPILER ---
    // Every trait is read from its own gene only (TCK-179, trait locality). A missing gene reads the
    // empty payload: every value at the middle of its range (h = 0.5), never other genes or active DNA.
    let gene_table = GeneTable::scan(&clean_genome);
    let col_source = gene_table.first_payload("COL");
    let h_sym = get_payload_linear_value_offset(col_source, 11);
    let h_color1 = get_payload_linear_value_offset(col_source, 0);
    let h_color2 = get_payload_linear_value_offset(col_source, 5);

    let siz_source = gene_table.first_payload("SIZ");
    let h_radius = get_payload_linear_value_offset(siz_source, 0);
    let h_length = get_payload_linear_value_offset(siz_source, 4);
    let h_seed = get_payload_linear_value_offset(siz_source, 8);

    let stf_source = gene_table.first_payload("STF");
    let h_stiffness = get_payload_linear_value_offset(stf_source, 0);
    let h_hydraulic = get_payload_linear_value_offset(stf_source, 7);

    let wav_source = gene_table.first_payload("WAV");
    let h_curve = get_payload_linear_value_offset(wav_source, 0);
    let h_curve_freq = get_payload_linear_value_offset(wav_source, 3);
    let h_para_amp = get_payload_linear_value_offset(wav_source, 6);
    let h_para_freq = get_payload_linear_value_offset(wav_source, 9);

    let pul_source = gene_table.first_payload("PUL");
    let h_head = get_payload_linear_value_offset(pul_source, 0);
    let h_pulse = get_payload_linear_value_offset(pul_source, 2);
    let h_phase = get_payload_linear_value_offset(pul_source, 5);
    let h_wiggle = get_payload_linear_value_offset(pul_source, 9);

    let h_stomach = get_payload_linear_value_offset(gene_table.first_payload("STM"), 0);

    let tem_source = gene_table.first_payload("TEM");
    let h_thermal_c = get_payload_linear_value_offset(tem_source, 0);
    let h_thermal_w = get_payload_linear_value_offset(tem_source, 6);

    let h_carnivory = get_payload_linear_value_offset(gene_table.first_payload("CAR"), 0);

    let rep_source = gene_table.first_payload("REP");
    let h_mature = get_payload_linear_value_offset(rep_source, 0);
    let h_repro = get_payload_linear_value_offset(rep_source, 4);
    let h_split = get_payload_linear_value_offset(rep_source, 8);

    let evo_source = gene_table.first_payload("EVO");
    let h_insert = get_payload_linear_value_offset(evo_source, 0);
    let h_delete = get_payload_linear_value_offset(evo_source, 5);
    let h_fidelity = get_payload_linear_value_offset(evo_source, 10);

    // 1. Symmetry
    let symmetry = if h_sym >= 0.5 { "quad" } else { "vertical" };

    // 2. Primary & Secondary Colors
    let primary_color = HSLColor {
        h: (h_color1 * 360.0).round(),
        s: (55.0 + h_color2 * 45.0).round(),
        l: (35.0 + h_seed * 45.0).round(),
    };

    let secondary_color = HSLColor {
        h: (((h_color1 * 360.0) + 180.0) % 360.0).round(), // complementary secondary color
        s: (55.0 + h_seed * 45.0).round(),
        l: (35.0 + h_color2 * 45.0).round(),
    };

    // 3. Body Seed
    let body_seed = (h_seed * 100_000.0) as u32;

    // 4. Body Size (Mean Radius & Base Length)
    let mean_radius = (16.0f32).max((28.0f32).min(16.0 + h_radius * 12.0));
    let base_length = (90.0f32).max((200.0f32).min(90.0 + h_length * 110.0));

    // 5. Spinal Harmonics (Amplitudes & Phases)
    let mut amplitudes = vec![0.0; 4];
    for j in 0..4 {
        let offset_hash = get_payload_linear_value_offset(&wav_source, j + 15);
        amplitudes[j] = offset_hash * 0.3 - 0.15;
    }
    let mut phases = vec![0.0; 4];
    for j in 0..4 {
        let offset_hash = get_payload_linear_value_offset(&wav_source, j + 25);
        phases[j] = offset_hash * std::f32::consts::PI * 2.0;
    }

    // 6. Stiffness
    let stiffness = (0.15f32).max((1.0f32).min(0.15 + h_stiffness * 0.85));

    // 7. Curves, Parapodia, flattening head
    let spinal_curve = h_curve * 44.0 - 22.0;
    let spinal_curve_freq = 1.0 + (h_curve_freq * 3.0).floor();
    let parapodia_amp = h_para_amp * 0.45;
    let parapodia_freq = 2.0 + (h_para_freq * 3.0).floor();
    let flattening_head = h_head * 1.4 - 0.4;

    // 8. Pulse Speed, Wave Phase, Wiggle Amplitude
    let pulse_speed = 0.0015 + h_pulse * 0.0075;
    let wave_phase = h_phase * 1.6;
    let wiggle_amplitude = h_wiggle * 0.22;

    // 9. Stomach Capacity & Hydraulic Pressure
    let stomach_capacity = 50.0 + h_stomach * 450.0;
    let hydraulic_pressure = 0.2 + h_hydraulic * 0.8;

    // 10. Thermal Tolerance
    let thermal_tolerance_min = -5.0f32.max((h_thermal_c * 60.0 - h_thermal_w * 15.0).round());
    let thermal_tolerance_max = 105.0f32.min((h_thermal_c * 60.0 + h_thermal_w * 15.0).round());

    // 11. Organelles: one organ per organ gene, a mirrored pair or a midline organ with HOX (FIX-131)
    let has_hox = !gene_table.genes("HOX").is_empty();
    let mut organelles = Vec::new();
    let mut organ_genes = Vec::new();
    for promoter in ORGAN_PROMOTERS {
        for (number, gene) in gene_table.genes(promoter).iter().enumerate() {
            let payload = gene.payload;
            let spectral_affinity = get_payload_linear_value_offset(payload, ORGAN_AFFINITY_SHIFT);
            let bandwidth = 0.05 + get_payload_linear_value_offset(payload, 4) * 0.85;
            let expression_style = get_payload_linear_value_offset(payload, 7);
            let scale = 0.35 + get_payload_linear_value_offset(payload, 10) * 1.45;
            let spinal_pos = 0.05 + get_payload_linear_value_offset(payload, 13) * 0.9;
            let angle_offset = get_payload_linear_value_offset(payload, 16);
            let hue_shift = (get_payload_linear_value_offset(payload, 19) * 360.0 - 180.0).round();
            let kind = OrganKind::classify(spectral_affinity, expression_style);

            // Symmetrical Angle Space: 0.0 is straight ahead, Left is negative [-170 .. 0], Right is positive [0 .. 170]
            let base_angle = -170.0 + angle_offset * 340.0;
            let patch = |angle: f32, side: OrganSide| SensoryPatch {
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
                organelles.push(patch(base_angle, OrganSide::Single));
            } else if base_angle.abs() <= 7.5 {
                // Midline coalescence/fusion to a single unpaired central organelle
                organelles.push(patch(0.0, OrganSide::Single));
            } else {
                organelles.push(patch(-base_angle.abs(), OrganSide::Left));
                organelles.push(patch(base_angle.abs(), OrganSide::Right));
            }
            let owner = gene_table.owner(promoter, gene.start);
            organ_genes.push(OrganGene {
                gene: gene_table.gene_ref(owner),
                number: number + 1,
                owner,
                payload,
                organs: first..organelles.len(),
            });
        }
    }

    // 12. Carnivory
    let carnivory = h_carnivory;
    let is_predator = carnivory >= 0.50;

    // 13. Reproduction Metrics
    let mature_age = (300.0 + h_mature * 2400.0).round() as u32;
    let repro_threshold = 0.60 + h_repro * 0.35;
    let split_loss = 0.05 + h_split * 0.35;

    // 14. Evolutionary Drift
    let insertion_rate = h_insert * MAX_INDEL_RATE;
    let deletion_rate = h_delete * MAX_INDEL_RATE;
    let repair_fidelity = 0.15 + h_fidelity * 0.8;

    // 15. CTRNN brain (Brain v2: roles, targeted wiring, gene-owned weights)
    let brain = compile_brain(&gene_table, &organelles, &organ_genes);

    // --- RECALCULATE EMERGENCE CHAMBERS ---
    let mut peaks_count = 0;
    let mut was_rising = false;
    let mut prev_r = 0.0;

    let get_spinal_radius_at = |s: f32| -> f32 {
        let mut modulation = 0.0;
        for j in 0..4 {
            let frequency_factor = (j + 1) as f32 * std::f32::consts::PI;
            modulation += amplitudes[j] * (frequency_factor * s + phases[j]).cos();
        }
        mean_radius * (1.0 + modulation)
    };

    for j in 0..=50 {
        let s = j as f32 / 50.0;
        let r_current = get_spinal_radius_at(s);
        if j > 0 {
            let is_rising = r_current > prev_r;
            if was_rising && !is_rising {
                peaks_count += 1;
            }
            was_rising = is_rising;
        }
        prev_r = r_current;
    }
    let emergent_chambers_count = peaks_count.max(1);

    let spinal_harmonics = SpinalHarmonics {
        base_length,
        mean_radius,
        amplitudes: amplitudes.clone(),
        phases: phases.clone(),
        spinal_curve,
        spinal_curve_freq,
        parapodia_amp,
        parapodia_freq,
        flattening_head,
    };

    // 16. BMR and metabolic cost calculation
    let mut integrated_area_sum = 0.0;
    for steps in 0..=20 {
        let s_coord = steps as f32 / 20.0;
        let r_at_s = get_spinal_radius_at(s_coord);
        integrated_area_sum += r_at_s * r_at_s * std::f32::consts::PI;
    }
    let mean_area = integrated_area_sum / 21.0;
    
    let total_synapses = brain.synapses.len();
    let brain_complexity_multiplier = 1.0 + total_synapses as f32 * 0.06;

    let mut vestigial_organ_tax = 0.0;
    for patch in &organelles {
        let tax = patch.scale * patch.scale * (1.1 - patch.bandwidth) * 1.0;
        vestigial_organ_tax += tax;
    }

    let basal_metabolic_rate = ((mean_area * base_length * 0.000008 + vestigial_organ_tax)
        * (1.0 + (organelles.len() * SPECTRAL_CHANNELS) as f32 * 0.08)
        * (stiffness * 1.4)
        * brain_complexity_multiplier)
        .round();

    let eco = derive_ecological_metrics(
        &clean_genome,
        symmetry,
        emergent_chambers_count,
        &organelles,
        primary_color,
        stiffness,
        pulse_speed,
        wave_phase,
        carnivory,
    );

    // Dummy segments array representation matching TypeScript
    let mut segments = Vec::with_capacity(emergent_chambers_count);
    for _ in 0..emergent_chambers_count {
        segments.push(serde_json::json!({
            "parentIndex": -1,
            "localXOffset": 0,
            "localYOffset": 25,
            "baseRadius": mean_radius,
            "elongation": 0,
            "tilt": 0,
            "amplitudes": amplitudes
        }));
    }

    CreaturePhenotype {
        symmetry: String::from(symmetry),
        primary_color,
        secondary_color,
        body_seed,
        segments,
        spinal_harmonics,
        emergent_chambers_count,
        organelles,
        genes: gene_table.spans,
        pulse_speed,
        wave_phase,
        wiggle_amplitude,
        stiffness,
        mature_age,
        repro_threshold,
        split_loss,
        brain,
        carnivory,
        is_predator,
        latin_name: eco.latin_name,
        sensory_visus: eco.visus,
        sensory_olfaction: eco.olfaction,
        sensory_tactility: eco.tactility,
        sensory_biolum: eco.biolum,
        diet_class: eco.diet,
        preferred_habitat: eco.habitat,
        basal_metabolic_rate,
        stomach_capacity,
        thermal_tolerance_min,
        thermal_tolerance_max,
        hydraulic_pressure,
        rotational_inertia: 0.0,
        survival_expectation: eco.survival_score,
        survival_analysis: eco.survival_analysis,
        chromatin_state,
        epigenetic_logs,
        methylations,
        antisense_strand,
        repair_fidelity,
        insertion_rate,
        deletion_rate,
        genome_string: clean_genome,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_value_conversion() {
        assert_eq!(char_to_value('A'), 0);
        assert_eq!(char_to_value('Z'), 25);
        assert_eq!(char_to_value('a'), 0);
        assert_eq!(value_to_char(0.0), 'A');
        assert_eq!(value_to_char(25.9), 'Z');
    }

    #[test]
    fn test_complementary_dna() {
        assert_eq!(get_complementary_char('A'), 'Z');
        assert_eq!(get_complementary_char('Z'), 'A');
        assert_eq!(get_complementary_char('M'), 'N');
        assert_eq!(get_complementary_string("ABC"), "ZYX");
    }

    #[test]
    fn test_random_generation() {
        let length = 128;
        let dna = generate_random_genome(length, &mut crate::shared::rng::seeded(1));
        assert_eq!(dna.len(), length); // Sequence-based base_dna length is dynamic and matches requested length exactly
        for c in dna.chars() {
            assert!(ALPHABET.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_biochem_groups_and_degenerate_matching() {
        // Alpha class: A, E, I, O, U
        assert_eq!(get_bio_class('A'), 1);
        assert_eq!(get_bio_class('E'), 1);
        // Beta class: Y, W, F, H, K, R
        assert_eq!(get_bio_class('Y'), 2);
        assert_eq!(get_bio_class('W'), 2);
        // Gamma class: L, M, V, P, T
        assert_eq!(get_bio_class('L'), 3);
        // Delta class: D, N, Q, S, C
        assert_eq!(get_bio_class('N'), 4);
        assert_eq!(get_bio_class('S'), 4);

        // Degenerate matching checks (Golden-Middle: 1st exact, rest wobble)
        assert!(matches_degenerate("EYE", "EYE"));
        assert!(matches_degenerate("EYO", "EYE")); // matches exact 'E' + wobble Beta 'Y' + wobble Alpha 'O'
        assert!(matches_degenerate("EWA", "EYE")); // matches exact 'E' + wobble Beta 'W' + wobble Alpha 'A'
        assert!(!matches_degenerate("OWO", "EYE")); // different first character ('O' != 'E')
        assert!(!matches_degenerate("ABC", "EYE")); // completely different

        // Brain-related promoters ("NEU", "SY") must support full-wobble group matching!
        assert!(matches_degenerate("NEU", "NEU"));
        assert!(matches_degenerate("DAA", "NEU")); // D is Delta (like N), A is Alpha (like E), A is Alpha (like U)
        assert!(matches_degenerate("CAA", "NEU")); // C is Delta, A is Alpha, A is Alpha
        assert!(!matches_degenerate("DAA", "EYE")); // different promoter, should not match
    }

    #[test]
    fn test_extract_raw_gene_payloads() {
        // Standard matching (which is backward compatible)
        let genome = "STFAENCOLABSTPEYEAABEN";
        let stf_payloads = extract_raw_gene_payloads(genome, "STF", "EN");
        assert_eq!(stf_payloads.len(), 1);
        assert_eq!(stf_payloads[0], "A");

        let col_payloads = extract_raw_gene_payloads(genome, "COL", "STP");
        assert_eq!(col_payloads.len(), 1);
        assert_eq!(col_payloads[0], "AB");

        // Degenerate matching: "EYO" as EYE, "ES" as EN
        let degenerate_genome = "EYOAABES"; // EYO matches EYE, ES matches EN
        let eye_payloads = extract_raw_gene_payloads(degenerate_genome, "EYE", "EN");
        assert_eq!(eye_payloads.len(), 1);
        assert_eq!(eye_payloads[0], "AAB");

        // Test Point 1: Implicit end-of-genome stop fallback (no stop_motif present)
        let genome_no_stop = "EYEAA";
        let eye_payloads_fallback = extract_raw_gene_payloads(genome_no_stop, "EYE", "EN");
        assert_eq!(eye_payloads_fallback.len(), 1);
        assert_eq!(eye_payloads_fallback[0], "AA"); // Read successfully until the end of the strand!

        // Test Point 2: Isolated, non-greedy scanning (no merging of EYE...EN...EYE...EN)
        let genome_multi = "EYEAAENEYECCDEN";
        let multi_payloads = extract_raw_gene_payloads(genome_multi, "EYE", "EN");
        assert_eq!(multi_payloads.len(), 2);
        assert_eq!(multi_payloads[0], "AA");
        assert_eq!(multi_payloads[1], "CCD"); // Two distinct, isolated organs successfully translated!
    }

    fn ops() -> &'static MutationOperators {
        &crate::shared::types::AppConfig::global().mutation_operators
    }

    fn composition() -> &'static GenomeComposition {
        &crate::shared::types::AppConfig::global().genome_composition
    }

    fn promoter_counts(genome: &str) -> std::collections::BTreeMap<String, usize> {
        let mut counts = std::collections::BTreeMap::new();
        for span in parse_genome(genome, None, None).genes {
            *counts.entry(span.gene.promoter).or_insert(0) += 1;
        }
        counts
    }

    #[test]
    fn test_mutation() {
        // A 200-base genome is in the safe range for every letter-level operator
        let mut rng = crate::shared::rng::seeded(2);
        let original = generate_random_genome(200, &mut rng);
        for (kind, delta) in [(MutationKind::Creep, 0), (MutationKind::Point, 0), (MutationKind::Insertion, 1), (MutationKind::Deletion, -1)] {
            let (mutated, idx) = apply_mutation(&original, kind, ops(), &mut rng).expect("applicable at 200 bases");
            assert_ne!(mutated, original);
            assert_eq!(mutated.len() as isize, original.len() as isize + delta, "{:?}", kind);
            match kind {
                MutationKind::Deletion => assert_eq!(format!("{}{}{}", &mutated[..idx], &original[idx..idx + 1], &mutated[idx..]), original),
                MutationKind::Insertion => assert_eq!(format!("{}{}", &mutated[..idx], &mutated[idx + 1..]), original),
                _ => assert_eq!((&mutated[..idx], &mutated[idx + 1..]), (&original[..idx], &original[idx + 1..])),
            }
        }
        let mutation = mutate_genome(&original, ops(), &mut rng).expect("non-empty genome");
        assert_ne!(mutation.genome, original);

        // Letter-level operators only hit unprotected loci: in "COLABCEN" the payload ABC (3..=5)
        let genome_with_promoter = "COLABCEN";
        for kind in [MutationKind::Creep, MutationKind::Point, MutationKind::Insertion] {
            for _ in 0..100 {
                let (mutated, idx) = apply_mutation(genome_with_promoter, kind, ops(), &mut rng).expect("applicable");
                assert!((3..=5).contains(&idx), "{:?} hit protected index {}", kind, idx);
                assert!(matches_degenerate(&mutated[0..3], "COL") && matches_degenerate(&mutated[mutated.len() - 2..], "EN"), "Promoters and terminators must never be broken!");
            }
        }
    }

    #[test]
    fn test_structured_genome_parses_to_exactly_the_planned_genes() {
        let c = composition();
        let mut food_organ = 0;
        let n = 2000;
        for seed in 0..n {
            let plan = structured_gene_plan(c, &mut crate::shared::rng::seeded(seed));
            let genome = generate_structured_genome(c, &mut crate::shared::rng::seeded(seed));
            assert!((GENOME_LENGTH_MIN..=GENOME_LENGTH_MAX).contains(&genome.len()), "length {}", genome.len());

            let mut planned = std::collections::BTreeMap::new();
            for (promoter, _) in &plan {
                *planned.entry(promoter.to_string()).or_insert(0usize) += 1;
            }
            assert_eq!(promoter_counts(&genome), planned, "genome {}", genome);
            for p in BODY_PROMOTERS.iter().chain([&BRAIN_BASE_PROMOTER]) {
                assert_eq!(planned.get(*p), Some(&1), "{} missing", p);
            }
            let organs = plan.iter().filter(|(p, _)| ORGAN_PROMOTERS.contains(p)).count();
            assert!((c.organ_count_min..=c.organ_count_max).contains(&organs));

            let phenotype = parse_genome(&genome, None, None);
            // Distinct promoter spellings: every gene has occurrence 0 (stable identity, TCK-179)
            assert!(phenotype.genes.iter().all(|s| s.gene.occurrence == 0), "repeated spelling: {}", genome);
            for (promoter, kind) in plan.iter().filter_map(|(p, k)| k.map(|k| (p, k))) {
                assert!(phenotype.organelles.iter().any(|o| o.kind == kind), "{} planned as {:?}: {}", promoter, kind, genome);
            }
            if phenotype.organelles.iter().any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose)) {
                food_organ += 1;
            }
        }
        // p(food organ) = pFoodOrgan plus random organs that happen to be eyes or noses
        assert!(food_organ as f32 / n as f32 >= c.p_food_organ - 0.03, "food organ share {}", food_organ as f32 / n as f32);
    }

    #[test]
    fn test_gene_duplication_adds_exactly_one_gene_of_its_promoter() {
        let mut rng = crate::shared::rng::seeded(11);
        let mut applied = 0;
        for seed in 0..300 {
            let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
            let before = promoter_counts(&genome);
            let Some((child, start)) = apply_mutation(&genome, MutationKind::GeneDuplication, ops(), &mut rng) else { continue };
            applied += 1;
            let promoter = EXPRESSED_PROMOTERS.iter().find(|p| matches_degenerate(&genome[start..start + p.len()], p)).expect("gene start");
            let mut expected = before.clone();
            *expected.get_mut(*promoter).unwrap() += 1;
            assert_eq!(promoter_counts(&child), expected, "duplicating {} at {}", promoter, start);
        }
        assert!(applied > 250, "only {} duplications applied", applied);
    }

    #[test]
    fn test_gene_deletion_removes_one_gene_and_never_an_essential_one() {
        let mut rng = crate::shared::rng::seeded(12);
        for seed in 0..100 {
            let mut genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
            // Duplicate a few genes first so body genes have deletable second copies
            for _ in 0..3 {
                if let Some((g, _)) = apply_mutation(&genome, MutationKind::GeneDuplication, ops(), &mut rng) {
                    genome = g;
                }
            }
            while let Some((child, start)) = apply_mutation(&genome, MutationKind::GeneDeletion, ops(), &mut rng) {
                let before = promoter_counts(&genome);
                let after = promoter_counts(&child);
                let changed: Vec<_> = before.iter().filter(|(p, n)| after.get(*p).copied().unwrap_or(0) != **n).collect();
                assert_eq!(changed.len(), 1, "deletion at {} changed {:?}", start, changed);
                assert_eq!(after.get(changed[0].0).copied().unwrap_or(0) + 1, *changed[0].1);
                genome = child;
            }
            let counts = promoter_counts(&genome);
            for p in BODY_PROMOTERS.iter().chain([&BRAIN_BASE_PROMOTER]) {
                assert!(counts.get(*p).copied().unwrap_or(0) >= 1, "essential {} deleted", p);
            }
            assert!(genome.len() >= GENOME_LENGTH_MIN);
        }
    }

    #[test]
    fn test_creep_changes_a_tape_weight_by_at_most_the_step() {
        let mut rng = crate::shared::rng::seeded(13);
        let step = ops().creep_max_step;
        let max_delta = step as f32 / 25.0 + 1e-6;
        let mut checked = 0;
        for seed in 0..300 {
            let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
            let parent = parse_genome(&genome, None, None);
            for _ in 0..5 {
                let (child_genome, idx) = apply_mutation(&genome, MutationKind::Creep, ops(), &mut rng).expect("creep applies");
                let letter_delta = (char_to_value(child_genome.as_bytes()[idx] as char) as isize - char_to_value(genome.as_bytes()[idx] as char) as isize).unsigned_abs();
                assert!((1..=step).contains(&letter_delta), "letter step {}", letter_delta);

                let child = parse_genome(&child_genome, None, None);
                if child.genes != parent.genes {
                    continue; // the creep formed a promoter or terminator
                }
                let owner = parent.genes.iter().position(|g| (g.start..g.end).contains(&idx));
                let tape_owner = owner.filter(|&o| parent.genes[o].gene.promoter != "SY");
                for (p, c) in parent.brain.synapses.iter().filter(|s| Some(s.owner) == tape_owner).filter_map(|p| child.brain.synapses.iter().find(|c| c.key == p.key).map(|c| (p, c))) {
                    assert!((p.weight - c.weight).abs() <= max_delta, "weight {} -> {}", p.weight, c.weight);
                    checked += 1;
                }
            }
        }
        assert!(checked > 1000, "only {} homologous tape weights checked", checked);
    }

    #[test]
    fn test_structured_genome_and_mutation_are_deterministic() {
        let run = |seed: u64| {
            let mut rng = crate::shared::rng::seeded(seed);
            let mut genome = generate_structured_genome(composition(), &mut rng);
            let mut kinds = Vec::new();
            for _ in 0..300 {
                let m = mutate_genome(&genome, ops(), &mut rng).unwrap();
                kinds.push(m.kind);
                genome = m.genome;
            }
            (genome, kinds)
        };
        assert_eq!(run(21), run(21));
        assert_ne!(run(21).0, run(22).0);
        // Every operator occurs with the default weights
        let kinds = run(21).1;
        assert!(MutationKind::ALL.iter().all(|k| kinds.contains(k)), "{:?}", kinds);
    }

    #[test]
    fn test_progenitor_parse_genome() {
        // High quality test input representing a well-formed progenitor genome
        let progenitor_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";
        let phenotype = parse_genome(progenitor_dna, None, None);

        // Core assertions
        assert!(phenotype.genome_string.starts_with(progenitor_dna));
        assert!(phenotype.spinal_harmonics.mean_radius >= 16.0 && phenotype.spinal_harmonics.mean_radius <= 28.0);
        assert!(phenotype.basal_metabolic_rate > 0.0);
        assert!(phenotype.stiffness >= 0.15 && phenotype.stiffness <= 1.0);
        assert!(phenotype.antisense_strand.starts_with(&get_complementary_string(progenitor_dna)));

        // Brain topology assertions
        assert!(!phenotype.brain.neurons.is_empty());
        assert!(!phenotype.brain.synapses.is_empty());
        
        // Assert that we have inputs, outputs and hidden neurons
        let input_count = phenotype.brain.neurons.iter().filter(|n| n.neuron_type == NeuronType::Input).count();
        let output_count = phenotype.brain.neurons.iter().filter(|n| n.neuron_type == NeuronType::Output).count();
        let hidden_count = phenotype.brain.neurons.iter().filter(|n| n.neuron_type == NeuronType::Hidden).count();

        assert_eq!(input_count, phenotype.organelles.len() * CHANNELS_PER_ORGANELLE + SYSTEMIC_BASE_INPUTS_COUNT);
        assert_eq!(output_count, OUTPUT_MOTOR_NODES_COUNT);
        assert!(hidden_count >= 1 && hidden_count <= 35); // Includes our generic thalamus sum/diff pre-processor interneurons!

        // Verify that BMR scaling values are deterministic and finite
        assert!(phenotype.basal_metabolic_rate.is_finite());
        assert!(phenotype.stomach_capacity > 0.0);
    }

    #[test]
    fn test_dynamic_neuro_generation() {
        // DNA with 3 NEU promoters and 2 SY promoters using degenerate Option A + full wobble matching
        // Ending with Z prevents S-A padding boundary matches, and YNB prevents N-A-A matches.
        let dna = "NEUAAESNEUBBESNEUCCESYBAAESYBBBESZ";
        let phenotype = parse_genome(dna, None, None);
        
        let hidden_count = phenotype.brain.neurons.iter().filter(|n| n.neuron_type == NeuronType::Hidden).count();
        // Since we have 3 NEU blocks, we should have exactly 3 hidden neurons compiled dynamically!
        assert_eq!(hidden_count, 3);
        
        // Synapses should be dynamically compiled from "SY" payloads
        assert!(!phenotype.brain.synapses.is_empty());
    }

    #[test]
    fn test_hebbian_learning_and_epigenetics() {
        use crate::shared::brain::execute_brain_with_learning;

        let dna = "NEUAAESNEUBBESZZSYBCAESYBBBES"; // 3 hiddens, 1 SY
        let phenotype = parse_genome(dna, None, None);
        let brain = phenotype.brain;

        let mut states = vec![0.0; brain.neurons.len()];
        let mut activations = vec![0.0; brain.neurons.len()];
        let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();

        let inputs = vec![0.5, 0.5]; // 1 sensor, 1 clock

        // Step 1: Execute brain with learning active (hebbian_rate > 0.0)
        let original_weight = synapse_weights[0];
        for _ in 0..5 {
            execute_brain_with_learning(
                &brain,
                &inputs,
                &mut states,
                &mut activations,
                &mut synapse_weights,
                &mut crate::shared::brain::BrainRuntime::default(),
                0.1, // high learning rate
            );
        }

        let learned_weight = synapse_weights[0];
        // Weight should change due to Hebbian correlation!
        assert_ne!(learned_weight, original_weight);
    }

    #[test]
    fn test_targeted_wiring_and_active_pruning() {
        use crate::shared::brain::execute_brain_with_learning;

        let dna = "NEUAAESNEUBBESZZSYBCAESYBBBES"; // 3 NEU genes (one inside an SY payload), 2 SY genes, no organ, no OUT gene
        let phenotype = parse_genome(dna, None, None);
        let brain = &phenotype.brain;
        let role = |i: usize| &brain.neurons[i].role;
        let of_role = |pred: fn(&NeuronRole) -> bool| (0..brain.neurons.len()).filter(|&i| pred(role(i))).collect::<Vec<usize>>();
        let hidden = of_role(|r| matches!(r, NeuronRole::Hidden { .. }));
        let systemic = of_role(|r| matches!(r, NeuronRole::Systemic { .. }));
        let motors = of_role(|r| matches!(r, NeuronRole::Motor { .. }));
        let is_sy = |s: &CTRNNSynapse| phenotype.genes[s.owner].gene.promoter == "SY";
        assert_eq!(hidden.len(), 3);
        assert_eq!(motors.len(), OUTPUT_MOTOR_NODES_COUNT);

        // Targeted wiring (TCK-182): every systemic → hidden and hidden → motor pair exactly once; anything else
        // (no hidden → hidden, no systemic → motor without an OUT gene) only through an SY gene
        let count = |from: usize, to: usize| brain.synapses.iter().filter(|s| s.from_node == from && s.to_node == to).count();
        for &h in &hidden {
            assert!(systemic.iter().all(|&s| count(s, h) == 1));
            assert!(motors.iter().all(|&m| count(h, m) == 1));
        }
        let regular = |s: &CTRNNSynapse| matches!((role(s.from_node), role(s.to_node)), (NeuronRole::Systemic { .. }, NeuronRole::Hidden { .. }) | (NeuronRole::Hidden { .. }, NeuronRole::Motor { .. }));
        assert!(brain.synapses.iter().all(|s| regular(s) || is_sy(s)));
        assert_eq!(brain.synapses.iter().filter(|s| regular(s)).count(), hidden.len() * (SYSTEMIC_BASE_INPUTS_COUNT + OUTPUT_MOTOR_NODES_COUNT));
        let keys: std::collections::BTreeSet<u64> = brain.synapses.iter().map(|s| s.key).collect();
        assert_eq!(keys.len(), brain.synapses.len(), "synapse keys are unique");

        // Gene-tape weights start weak in [-0.5, 0.5] (never exactly 0); SY weights in [-2, 2]
        assert!(brain.synapses.iter().any(|s| is_sy(s)));
        for s in &brain.synapses {
            let limit = if is_sy(s) { 2.0 } else { 0.5 };
            assert!(s.weight != 0.0 && s.weight.abs() <= limit, "{:?}", s);
        }

        // Test Active Pruning
        let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
        let mut states = vec![0.0; brain.neurons.len()];
        let mut activations = vec![0.0; brain.neurons.len()];
        let inputs = vec![0.0, 0.0]; // Zero inputs: Oja's −η·post²·w term alone shrinks the synapses without input

        // Run 50 steps of a high learning rate to prune weak connections
        for _ in 0..50 {
            execute_brain_with_learning(
                brain,
                &inputs,
                &mut states,
                &mut activations,
                &mut synapse_weights,
                &mut crate::shared::brain::BrainRuntime::default(),
                0.5, // high learning rate to prune quickly
            );
        }

        // Weak synapses should now be exactly 0.0 (permanently pruned!)
        let mut pruned_count = 0;
        for &w in &synapse_weights {
            if w == 0.0 {
                pruned_count += 1;
            }
        }
        assert!(pruned_count > 0);
    }

    #[test]
    fn test_linear_allele_mapping() {
        // 1. Verify middle fallback on empty payload
        assert_eq!(get_payload_linear_value_offset("", 0), 0.5);
        assert_eq!(get_payload_linear_value_offset("", 10), 0.5);

        // 2. Verify bounds of different alphabet values
        let val_a = get_payload_linear_value_offset("A", 0);
        let val_z = get_payload_linear_value_offset("Z", 0);
        assert_eq!(val_a, 0.0);
        assert_eq!(val_z, 1.0);

        // 3. Verify perfect mutational continuity: transition from 'O' to 'P'
        let val_o = get_payload_linear_value_offset("O", 0);
        let val_p = get_payload_linear_value_offset("P", 0);
        // 'O' is index 14, 'P' is index 15. The difference should be exactly 1 / 25 = 0.04 (4%)!
        assert!((val_p - val_o - 0.04).abs() < 1e-6);
    }

    #[test]
    fn test_mutation_robustness_and_deadlocks() {
        use crate::shared::brain::execute_brain_with_learning;

        // 1. Generate 50 heavily mutated genomes of different lengths (fuzzing sweep)
        let mut rng = crate::shared::rng::seeded(3);
        for trial in 0..50 {
            let start_len = 128 + (trial * 7) % 380; // various lengths in [128..508]
            let mut genome = generate_random_genome(start_len, &mut rng);

            // Apply 50 random mutations, insertions, or deletions consecutively to scramble promoters and terminators
            for _ in 0..50 {
                if let Some(mutation) = mutate_genome(&genome, ops(), &mut rng) {
                    genome = mutation.genome;
                }
            }

            // A. Assert compilation robustness: must NEVER panic, crash or deadlock
            let phenotype = parse_genome(&genome, None, None);
            let brain = phenotype.brain;

            // B. Assert state boundaries are finite
            assert!(phenotype.stiffness.is_finite());
            assert!(phenotype.stiffness >= 0.15 && phenotype.stiffness <= 1.0);
            assert!(phenotype.basal_metabolic_rate.is_finite());
            assert!(phenotype.stomach_capacity > 0.0);

            // C. Assert brain execution robustness: run euler integration for 10 ticks and ensure zero NaNs/Infs
            let mut states = vec![0.0; brain.neurons.len()];
            let mut activations = vec![0.0; brain.neurons.len()];
            let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();

            let input_count = brain.neurons.iter().filter(|n| n.neuron_type == NeuronType::Input).count();
            let inputs = vec![0.5; input_count];

            for _tick in 0..10 {
                let outputs = execute_brain_with_learning(
                    &brain,
                    &inputs,
                    &mut states,
                    &mut activations,
                    &mut synapse_weights,
                    &mut crate::shared::brain::BrainRuntime::default(),
                    0.05, // active Oja learning
                );

                // Assert that all outputs, neuron states, activations and synapse weights are perfectly finite and NOT NaN
                for &out in &outputs {
                    assert!(out.is_finite(), "Output was NaN or Infinite!");
                }
                for &s in &states {
                    assert!(s.is_finite(), "Neuron state was NaN or Infinite!");
                }
                for &a in &activations {
                    assert!(a.is_finite(), "Neuron activation was NaN or Infinite!");
                }
                for &w in &synapse_weights {
                    assert!(w.is_finite(), "Synapse weight was NaN or Infinite!");
                }
            }
        }
    }

    #[test]
    fn test_hox_bilateral_symmetry() {
        // A. Without HOX gene (Asymmetric primitive)
        // This genome contains exactly one EYE promoter and no HOX promoter.
        let no_hox_dna = "EYEABCDEFEN";
        let pheno_no_hox = parse_genome(no_hox_dna, None, None);
        // It must compile exactly 1 organelle asymmetrically.
        assert_eq!(pheno_no_hox.organelles.len(), 1);
        let angle = pheno_no_hox.organelles[0].angle;
        // Verify angle range is primitive [-170 to 170 deg]
        assert!(angle >= -170.0 && angle <= 170.0);

        // B. With HOX gene - Lateral Organelle (Paired/Mirrored)
        // Let's craft a genome with both HOX and EYE promoters.
        // We set index 0 of the eye payload to 'K' to yield -170.0 deg exactly.
        let hox_paired_dna = "HOXABCDEZENEYEKAAAAAAAEN"; 
        let pheno_paired = parse_genome(hox_paired_dna, None, None);
        // It should contain exactly 2 organelles due to bilateral symmetry (Left + Right mirror)
        assert_eq!(pheno_paired.organelles.len(), 2);
        assert_eq!(pheno_paired.organelles[0].angle, -170.0);
        assert_eq!(pheno_paired.organelles[1].angle, 170.0); // mirrored to +170.0 degrees!

        // C. With HOX gene - Midline Coalescence (Unpaired/Snapped)
        // Let's craft a genome with both HOX and EYE promoters.
        // We place 'X' at index 16 of the eye payload to yield 6.8 deg, snapping to 0.0 deg.
        let hox_midline_dna = "HOXABCDEZENEYEAAAAAAAAAAAAAAAAXAEN"; 
        let pheno_midline = parse_genome(hox_midline_dna, None, None);
        // Coalescence/Fusion check: It must snap to exactly 0.0 degrees (straight ahead) and NOT duplicate!
        assert_eq!(pheno_midline.organelles.len(), 1);
        assert_eq!(pheno_midline.organelles[0].angle, 0.0);

        // D. Open-end fallback test case (No trailing "EN" terminator for EYE promoter)
        // This genome has HOX active, and an EYE promoter but no trailing "EN" terminator.
        // It must fall back to reading exactly 15 characters of payload ("AKAAAAAAAAAAAAA").
        // Index 16 % 15 = 1. Since index 1 of the payload is 'K', the angle evaluates to -170.0 deg.
        let hox_open_dna = "HOXABCDEZENEYEAKAAAAAAAAAAAAA"; // No trailing "EN"!
        let pheno_open = parse_genome(hox_open_dna, None, None);
        // Bilateral symmetry must still work on the 15-char fallback payload!
        assert_eq!(pheno_open.organelles.len(), 2);
        assert_eq!(pheno_open.organelles[0].angle, -170.0);
        assert_eq!(pheno_open.organelles[1].angle, 170.0); // mirrored to +170.0 deg!
    }

    /// Deterministic genome set (no rand) covering random strands plus organ/HOX/brain promoters.
    fn golden_genomes() -> Vec<String> {
        let mut state: u64 = 0x2545F4914F6CDD1D;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let motifs = ["EYE", "NOS", "TAC", "LUM", "HOX", "NEU", "SY", "EN", "SP", "COL", "STF", "PUL", "SIZ", "WAV", "REP", "EVO", "CAR"];
        (0..300)
            .map(|_| {
                let len = 128 + (next() % 300) as usize;
                let mut g = String::with_capacity(len + 64);
                while g.len() < len {
                    if next() % 4 == 0 {
                        g.push_str(motifs[(next() % motifs.len() as u64) as usize]);
                    } else {
                        g.push(ALPHABET[(next() % 26) as usize] as char);
                    }
                }
                g
            })
            .collect()
    }

    /// Body traits with the promoter of the gene they are read from (derived values such as BMR are left out).
    fn owned_traits(p: &CreaturePhenotype) -> Vec<(&'static str, f32)> {
        let sh = &p.spinal_harmonics;
        let mut traits = vec![
            ("COL", (p.symmetry == "quad") as u8 as f32),
            ("COL", p.primary_color.h),
            ("COL", p.primary_color.s),
            ("SIZ", p.primary_color.l),
            ("COL", p.secondary_color.h),
            ("SIZ", p.secondary_color.s),
            ("COL", p.secondary_color.l),
            ("SIZ", p.body_seed as f32),
            ("SIZ", sh.mean_radius),
            ("SIZ", sh.base_length),
            ("STF", p.stiffness),
            ("STF", p.hydraulic_pressure),
            ("WAV", sh.spinal_curve),
            ("WAV", sh.spinal_curve_freq),
            ("WAV", sh.parapodia_amp),
            ("WAV", sh.parapodia_freq),
            ("PUL", sh.flattening_head),
            ("PUL", p.pulse_speed),
            ("PUL", p.wave_phase),
            ("PUL", p.wiggle_amplitude),
            ("STM", p.stomach_capacity),
            ("TEM", p.thermal_tolerance_min),
            ("TEM", p.thermal_tolerance_max),
            ("CAR", p.carnivory),
            ("REP", p.mature_age as f32),
            ("REP", p.repro_threshold),
            ("REP", p.split_loss),
            ("EVO", p.insertion_rate),
            ("EVO", p.deletion_rate),
            ("EVO", p.repair_fidelity),
        ];
        traits.extend(sh.amplitudes.iter().chain(&sh.phases).map(|&v| ("WAV", v)));
        traits
    }

    /// Synapse as compared across genomes: weight, owner span and the genes of both endpoints.
    #[derive(Debug, PartialEq)]
    struct Edge {
        weight: f32,
        owner: GeneSpan,
        endpoints: [Option<GeneRef>; 2],
    }

    fn brain_by_key(p: &CreaturePhenotype) -> std::collections::BTreeMap<u64, Edge> {
        let gene_of = |i: usize| p.brain.neurons[i].role.gene().cloned();
        let map: std::collections::BTreeMap<u64, Edge> = p
            .brain
            .synapses
            .iter()
            .map(|s| (s.key, Edge { weight: s.weight, owner: p.genes[s.owner].clone(), endpoints: [gene_of(s.from_node), gene_of(s.to_node)] }))
            .collect();
        assert_eq!(map.len(), p.brain.synapses.len(), "synapse keys are unique");
        map
    }

    fn gene_ref(p: &CreaturePhenotype, g: usize) -> GeneRef {
        p.genes[g].gene.clone()
    }

    /// The genome with the base at `pos` replaced by another letter.
    fn point_mutant(genome: &str, pos: usize, shift: usize) -> String {
        let mut bytes = genome.as_bytes().to_vec();
        bytes[pos] = ALPHABET[(char_to_value(bytes[pos] as char) + shift) % 26];
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn test_parse_genome_never_panics_on_random_ascii() {
        use rand::Rng;
        let mut rng = crate::shared::rng::seeded(4);
        for _ in 0..400 {
            let len = rng.gen_range(0..=1024);
            let genome: String = (0..len).map(|_| rng.gen_range(0u8..128) as char).collect();
            let p = parse_genome(&genome, None, None);
            let n = p.brain.neurons.len();
            assert!(p.brain.synapses.iter().all(|s| s.from_node < n && s.to_node < n && s.owner < p.genes.len() && s.weight.is_finite()));
        }
    }

    #[test]
    fn test_mutate_genome_keeps_length_bounds() {
        let mut rng = crate::shared::rng::seeded(5);
        let starts = [generate_random_genome(128, &mut rng), generate_random_genome(512, &mut rng), generate_structured_genome(composition(), &mut rng)];
        for mut genome in starts {
            for _ in 0..10_000 {
                genome = mutate_genome(&genome, ops(), &mut rng).expect("non-empty genome").genome;
                assert!((128..=512).contains(&genome.len()), "length {}", genome.len());
            }
        }
    }

    #[test]
    fn test_point_mutation_outside_genes_changes_no_weight_or_trait() {
        let mut tested = 0;
        for g in golden_genomes() {
            let parent_genome = normalize_genome(&g);
            let parent = parse_genome(&parent_genome, None, None);
            let covered = |i: usize| parent.genes.iter().any(|s| (s.start..s.end).contains(&i));
            for pos in (0..parent_genome.len()).filter(|&i| !covered(i)).step_by(2) {
                let child = parse_genome(&point_mutant(&parent_genome, pos, 11), None, None);
                if child.genes != parent.genes {
                    continue; // the mutation created or re-terminated a gene
                }
                tested += 1;
                assert_eq!(child.brain, parent.brain);
                assert_eq!(child.organelles, parent.organelles);
                assert_eq!(owned_traits(&child), owned_traits(&parent));
            }
        }
        assert!(tested > 1000, "{} mutants tested", tested);
    }

    #[test]
    fn test_point_mutation_inside_a_gene_changes_only_that_gene() {
        let mut tested = 0;
        for g in golden_genomes().into_iter().step_by(30) {
            let parent_genome = normalize_genome(&g);
            let parent = parse_genome(&parent_genome, None, None);
            let parent_brain = brain_by_key(&parent);
            for span in &parent.genes {
                for pos in (span.start + span.gene.promoter.len()..span.end).step_by(3) {
                    let child = parse_genome(&point_mutant(&parent_genome, pos, 5), None, None);
                    if child.genes != parent.genes {
                        continue; // the mutation broke the terminator or created a gene
                    }
                    tested += 1;
                    // Genes of different promoters may overlap: the base belongs to every gene that covers it
                    let hit: Vec<(usize, GeneRef)> = (0..parent.genes.len())
                        .filter(|&g| (parent.genes[g].start..parent.genes[g].end).contains(&pos))
                        .map(|g| (g, gene_ref(&parent, g)))
                        .collect();
                    let is_hit = |r: &GeneRef| hit.iter().any(|(_, h)| h == r);

                    // Synapses: a changed, added or removed synapse is owned by a hit gene or touches one of its neurons.
                    // Exception: SY-owned synapses (SY addresses modulo the class size, TCK-168).
                    let child_brain = brain_by_key(&child);
                    let attributable = |e: Option<&Edge>| {
                        e.is_some_and(|e| hit.iter().any(|(g, r)| e.owner == parent.genes[*g] || e.endpoints.contains(&Some(r.clone()))) || e.owner.gene.promoter == "SY")
                    };
                    for key in parent_brain.keys().chain(child_brain.keys()) {
                        let (p, c) = (parent_brain.get(key), child_brain.get(key));
                        assert!(p == c || attributable(p) || attributable(c), "genes {:?} pos {} changed synapse {:?} -> {:?}", hit, pos, p, c);
                    }

                    // Neurons of other genes keep their parameters (motors belong to the first OUT gene)
                    let expressed = |promoter: &str| hit.iter().any(|(g, _)| parent.genes.iter().position(|s| s.gene.promoter == promoter) == Some(*g));
                    for n in &parent.brain.neurons {
                        let own = n.role.gene().is_some_and(|r| is_hit(r)) || (expressed(BRAIN_BASE_PROMOTER) && matches!(n.role, NeuronRole::Motor { .. }));
                        if !own {
                            let m = child.brain.neurons.iter().find(|m| m.role == n.role).expect("neuron of another gene kept");
                            assert_eq!((m.tau, m.bias, &m.activation_type), (n.tau, n.bias, &n.activation_type), "{:?}", n.role);
                        }
                    }

                    // Traits: only those of a hit gene that is the expressed (first) gene of its type
                    for ((promoter, pv), (_, cv)) in owned_traits(&parent).into_iter().zip(owned_traits(&child)) {
                        if !expressed(promoter) {
                            assert_eq!(pv, cv, "genes {:?} changed a {} trait", hit, promoter);
                        }
                    }
                    let others = |p: &CreaturePhenotype| p.organelles.iter().filter(|o| !hit.iter().any(|(g, _)| parent.genes[*g].start == o.gene_start_index)).cloned().collect::<Vec<_>>();
                    assert_eq!(others(&child), others(&parent));
                }
            }
        }
        assert!(tested > 1000, "{} mutants tested", tested);
    }

    #[test]
    fn test_adding_an_organ_gene_keeps_unrelated_synapses() {
        // Payloads without vowels and Delta letters, so they contain no promoter or terminator (eye, nose, tactile, eye)
        let organ_genes = ["EYEBTKLMRVXZGHJWFPYBTKLEN", "NOSBKLMRVXZGHJWFPYBTKLMEN", "TACBBTKLMRVXZGHJWFPYBTKEN", "LUMZXTKLMRVBGHJWFPYBTKLEN"];
        let mut tested = 0;
        for g in golden_genomes() {
            // A trailing Z spacer (no motif) in the parent: its unterminated genes end there, not in the new gene
            let parent_genome = format!("{}{}", normalize_genome(&g), "Z".repeat(30));
            let parent = parse_genome(&parent_genome, None, None);
            let parent_brain = brain_by_key(&parent);
            for organ in organ_genes {
                if parent_genome.len() + organ.len() > 512 {
                    continue;
                }
                let child = parse_genome(&format!("{}{}", parent_genome, organ), None, None);
                // Appended as the last gene of its type: all parent genes unchanged, exactly one new gene
                if child.genes.len() != parent.genes.len() + 1 || !parent.genes.iter().all(|s| child.genes.contains(s)) {
                    continue;
                }
                tested += 1;
                let occurrence = parent.genes.iter().filter(|s| s.gene.promoter == organ[..3] && s.gene.spelling == organ[..3]).count();
                let new_gene = GeneRef { promoter: organ[..3].to_string(), spelling: organ[..3].to_string(), occurrence };
                let child_brain = brain_by_key(&child);
                for (key, p) in &parent_brain {
                    match child_brain.get(key) {
                        Some(c) => assert!(p == c || p.owner.gene.promoter == "SY" || c.owner.gene.promoter == "SY", "{:?} -> {:?}", p, c),
                        None => assert_eq!(p.owner.gene.promoter, "SY", "parent synapse lost: {:?}", p),
                    }
                }
                for (key, c) in &child_brain {
                    assert!(parent_brain.contains_key(key) || c.endpoints.contains(&Some(new_gene.clone())) || c.owner.gene.promoter == "SY", "{:?}", c);
                }
            }
        }
        assert!(tested > 300, "{} genomes tested", tested);
    }

    #[test]
    fn test_letter_mutations_never_change_a_promoter_spelling() {
        let mut rng = crate::shared::rng::seeded(14);
        let mut checked = 0;
        for genome in golden_genomes().into_iter().map(|g| normalize_genome(&g)).chain((0..100).map(|seed| generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed)))) {
            let promoters: Vec<std::ops::Range<usize>> = parse_genome(&genome, None, None).genes.iter().map(|s| s.start..s.start + s.gene.promoter.len()).collect();
            for kind in [MutationKind::Creep, MutationKind::Point, MutationKind::Insertion, MutationKind::Deletion] {
                for _ in 0..20 {
                    let Some((_, idx)) = apply_mutation(&genome, kind, ops(), &mut rng) else { continue };
                    assert!(!promoters.iter().any(|p| p.contains(&idx)), "{:?} at {} hit a promoter: {}", kind, idx, genome);
                    checked += 1;
                }
            }
        }
        assert!(checked > 20_000, "{} mutations checked", checked);
    }

    #[test]
    fn test_gene_duplication_and_deletion_keep_unrelated_synapses() {
        // Synapses that neither belong to nor touch the duplicated/deleted gene keep key and weight; a duplication keeps
        // every parent synapse (the original keeps its identity, the copy is a new gene). Exceptions: SY-owned synapses
        // (SY addresses modulo the class size, TCK-168) and HOX (pairs or unpairs every organ).
        let mut tested = [0; 2];
        for (k, kind) in [MutationKind::GeneDuplication, MutationKind::GeneDeletion].into_iter().enumerate() {
            for seed in 0..300 {
                let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
                let Some((child_genome, start)) = apply_mutation(&genome, kind, ops(), &mut crate::shared::rng::seeded(seed + 7_000)) else { continue };
                let parent = parse_genome(&genome, None, None);
                let child = parse_genome(&child_genome, None, None);
                let g = parent.genes.iter().position(|s| s.start == start).expect("mutated gene");
                if parent.genes[g].gene.promoter == "HOX" {
                    continue;
                }
                tested[k] += 1;
                let r = gene_ref(&parent, g);
                let new_genes: Vec<GeneRef> = child.genes.iter().map(|s| s.gene.clone()).filter(|c| !parent.genes.iter().any(|p| &p.gene == c)).collect();
                assert_eq!(new_genes.len(), k ^ 1, "{:?} on {:?} gave new genes {:?}", kind, r, new_genes);
                let (parent_brain, child_brain) = (brain_by_key(&parent), brain_by_key(&child));
                let is_sy = |e: &Edge| e.owner.gene.promoter == "SY";
                for (key, p) in parent_brain.iter().filter(|(_, e)| !is_sy(e)) {
                    let touches = p.owner == parent.genes[g] || p.endpoints.contains(&Some(r.clone()));
                    if kind == MutationKind::GeneDeletion && touches {
                        continue;
                    }
                    match child_brain.get(key) {
                        Some(c) => assert!(c.weight == p.weight || is_sy(c), "{:?} on {:?} changed {:?} -> {:?}", kind, r, p, c),
                        None => panic!("{:?} on {:?} lost {:?}", kind, r, p),
                    }
                }
                for (key, c) in &child_brain {
                    let touches_new = new_genes.iter().any(|n| c.endpoints.contains(&Some(n.clone())) || &c.owner.gene == n);
                    assert!(parent_brain.contains_key(key) || touches_new || is_sy(c), "{:?} on {:?} added {:?}", kind, r, c);
                }
            }
        }
        assert!(tested.iter().all(|&t| t > 250), "tested {:?}", tested);
    }

    /// Key-shift measurement (TCK-179, `cargo test --release --lib measure_gene_operator_key_shift -- --ignored --nocapture`):
    /// one gene duplication or deletion on each of 1000 structured genomes; for the parent synapses that do not touch
    /// the mutated gene (`SY`-owned excluded) the share kept by key, the share with the identical weight and the weight
    /// correlation. "shifting": the genome has a later gene of the mutated gene's promoter type. Excluded: HOX genes
/// (pairing touches every organ) and synapses an `SY` gene owns in the child.
    #[test]
    #[ignore]
    fn measure_gene_operator_key_shift() {
        for kind in [MutationKind::GeneDuplication, MutationKind::GeneDeletion] {
            // (cases, unrelated parent synapses, kept, identical weight, weight pairs) for non-shifting / shifting
            let mut tally: [(usize, usize, usize, usize, Vec<(f32, f32)>); 2] = Default::default();
            let mut hox = 0;
            let mut same_spelling = 0;
            for seed in 0..1000 {
                let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
                let Some((child_genome, start)) = apply_mutation(&genome, kind, ops(), &mut crate::shared::rng::seeded(seed + 1_000_000)) else { continue };
                let parent = parse_genome(&genome, None, None);
                let child = parse_genome(&child_genome, None, None);
                let g = parent.genes.iter().position(|s| s.start == start).expect("mutated gene");
                if parent.genes[g].gene.promoter == "HOX" {
                    hox += 1; // HOX pairs or unpairs every organ: touches all organ synapses
                    continue;
                }
                let r = gene_ref(&parent, g);
                let shifting = parent.genes[g + 1..].iter().any(|s| s.gene.promoter == parent.genes[g].gene.promoter);
                same_spelling += parent.genes[g + 1..].iter().any(|s| s.gene.promoter == r.promoter && s.gene.spelling == r.spelling) as usize;
                let (parent_brain, child_brain) = (brain_by_key(&parent), brain_by_key(&child));
                let t = &mut tally[shifting as usize];
                t.0 += 1;
                for (key, p) in parent_brain.iter().filter(|(_, e)| e.owner.gene.promoter != "SY" && e.owner != parent.genes[g] && !e.endpoints.contains(&Some(r.clone()))) {
                    let c = child_brain.get(key);
                    if c.is_some_and(|c| c.owner.gene.promoter == "SY") {
                        continue; // an SY gene re-addressed by the changed class sizes (TCK-168)
                    }
                    t.1 += 1;
                    if let Some(c) = c {
                        t.2 += 1;
                        t.3 += (c.weight == p.weight) as usize;
                        t.4.push((p.weight, c.weight));
                    }
                }
            }
            for (label, t) in ["non-shifting", "shifting"].iter().zip(&tally) {
                println!(
                    "{:?} {}: cases {}, unrelated synapses {}, kept {:.4}, identical weight {:.4}, r {:.4}",
                    kind,
                    label,
                    t.0,
                    t.1,
                    t.2 as f32 / t.1.max(1) as f32,
                    t.3 as f32 / t.1.max(1) as f32,
                    crate::biology::probes::pearson(&t.4).unwrap_or(f32::NAN)
                );
            }
            println!("{:?} HOX cases (excluded): {}; cases with a later gene of the same spelling: {}", kind, hox, same_spelling);
        }
    }

    #[test]
    fn test_parse_genome_golden_master() {
        let mut combined = String::new();
        for g in golden_genomes() {
            let mut pheno = parse_genome(&g, None, None);
            pheno.latin_name.clear(); // covered by test_latin_name_is_deterministic
            combined.push_str(&serde_json::to_string(&pheno).unwrap());
        }
        let fingerprint = fnv1a(combined.as_bytes());
        println!("GOLDEN_FINGERPRINT={:#018x}", fingerprint);
        assert_eq!(fingerprint, GOLDEN_FINGERPRINT);
    }

    #[test]
    fn test_read_evolution_rates_matches_parse_genome() {
        for g in golden_genomes() {
            let pheno = parse_genome(&g, None, None);
            assert_eq!(read_evolution_rates(&g), (pheno.insertion_rate, pheno.deletion_rate));
        }
    }

    #[test]
    fn test_latin_name_is_deterministic() {
        for g in golden_genomes() {
            let first = parse_genome(&g, None, None).latin_name;
            for _ in 0..4 {
                assert_eq!(parse_genome(&g, None, None).latin_name, first, "genome {}", g);
            }
        }
    }

    // Fingerprint of parse_genome output over golden_genomes(); Brain v2 model change (P3 slice A: TCK-179/182/144/153/149;
    // slice C: gene identity by promoter spelling + occurrence, 0x0c404a33923c8dd9 -> 0x65f701a6937c1e71;
    // P4 TCK-152: `plastic` flag per synapse, 0x65f701a6937c1e71 -> 0xe0a2bfd1c7310306, unchanged without the flag)
    const GOLDEN_FINGERPRINT: u64 = 0xe0a2bfd1c7310306;

}
