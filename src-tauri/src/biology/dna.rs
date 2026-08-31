use serde::{Deserialize, Serialize};
use rand::Rng;
use ts_rs::TS;

pub const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

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
    pub tau: f32,  // time constant [0.5 to 5.0]
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
    let len = classic_promoter.len();
    if segment.len() != len {
        return false;
    }
    
    // For brain-related promoters ("NEU", "SY"), use full biochemical group matching (full wobble)
    if classic_promoter == "NEU" || classic_promoter == "SY" {
        return segment.chars().zip(classic_promoter.chars()).all(|(s, p)| {
            get_bio_class(s) == get_bio_class(p)
        });
    }

    if len <= 1 {
        let s_char = segment.chars().next().unwrap();
        let p_char = classic_promoter.chars().next().unwrap();
        return get_bio_class(s_char) == get_bio_class(p_char);
    }

    let s_chars: Vec<char> = segment.chars().collect();
    let p_chars: Vec<char> = classic_promoter.chars().collect();

    // Golden-Middle matching: First character must match EXACTLY
    if s_chars[0] != p_chars[0] {
        return false;
    }

    // Remaining characters match degenerately (wobble)
    for i in 1..len {
        if get_bio_class(s_chars[i]) != get_bio_class(p_chars[i]) {
            return false;
        }
    }

    true
}

pub fn extract_raw_gene_payloads(
    genome: &str,
    start_motif: &str,
    stop_motif: &str
) -> Vec<String> {
    let mut payloads = Vec::new();

    if start_motif.is_empty() || stop_motif.is_empty() || genome.len() < start_motif.len() + stop_motif.len() {
        return payloads;
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
                let payload_slice = &genome[payload_start..end_idx];
                payloads.push(payload_slice.to_string());
                idx = end_idx + stop_motif.len();
            } else {
                // Point 1 Fallback: If no stop motif found within 40 characters, read a standard 15-char payload
                // and resume scanning after it, instead of swallowing the remaining genome and breaking!
                let end_idx = (payload_start + 15).min(genome.len());
                let payload_slice = &genome[payload_start..end_idx];
                if !payload_slice.is_empty() {
                    payloads.push(payload_slice.to_string());
                }
                idx = end_idx;
            }
        } else {
            idx += 1;
        }
    }
    payloads
}

pub fn generate_random_genome(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut dna = String::with_capacity(length);
    for _ in 0..length {
        let idx = rng.gen_range(0..26);
        dna.push(ALPHABET[idx] as char);
    }
    dna
}

pub fn mutate_genome(genome: &str) -> Option<(String, usize, char, char)> {
    if genome.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let current_length = genome.len();

    // Protect crucial promoter/terminator motifs from mutations to prevent catastrophic brain-death / synapse loss
    let protected_motifs = [
        "COL", "STF", "PUL", "SIZ", "WAV", "SYM", "STM", "TEM",
        "EYE", "NOS", "TAC", "LUM", "CAR", "REP", "EVO", "NEU", "SY", "OUT", "EN"
    ];

    let mut protected = vec![false; current_length];
    for motif in &protected_motifs {
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

    // Gather all unprotected indices
    let mut unprotected_indices = Vec::new();
    for i in 0..current_length {
        if !protected[i] {
            unprotected_indices.push(i);
        }
    }

    // Parse current genome to fetch dynamic insertion/deletion rates determined by phenotype
    let pheno = parse_genome(genome, None, None);
    let insertion_rate = pheno.insertion_rate; // 0.0 .. 0.12
    let deletion_rate = pheno.deletion_rate;   // 0.0 .. 0.12

    let roll = rng.gen_range(0.0..1.0);

    // 1. Try Deletion mutation (scaled by phenotype rate)
    if roll < deletion_rate * 0.5 && current_length > 128 && !unprotected_indices.is_empty() {
        let idx_in_list = rng.gen_range(0..unprotected_indices.len());
        let index = unprotected_indices[idx_in_list];
        let mut new_genome = String::from(genome);
        let deleted_char = new_genome.remove(index);
        return Some((new_genome, index, '-', deleted_char));
    }

    // 2. Try Insertion mutation (scaled by phenotype rate)
    if roll < (deletion_rate * 0.5) + (insertion_rate * 0.5) && current_length < 512 {
        let index = if !unprotected_indices.is_empty() {
            let idx_in_list = rng.gen_range(0..unprotected_indices.len());
            unprotected_indices[idx_in_list]
        } else {
            rng.gen_range(0..current_length)
        };
        let idx = rng.gen_range(0..26);
        let inserted_char = ALPHABET[idx] as char;
        let mut new_genome = String::from(genome);
        new_genome.insert(index, inserted_char);
        return Some((new_genome, index, '+', inserted_char));
    }

    // 3. Fallback: Point Mutation (Substitution)
    let index = if !unprotected_indices.is_empty() {
        let idx_in_list = rng.gen_range(0..unprotected_indices.len());
        unprotected_indices[idx_in_list]
    } else {
        rng.gen_range(0..current_length)
    };

    let current_char = genome.chars().nth(index)?;
    let mut new_char = current_char;
    while new_char == current_char {
        let idx = rng.gen_range(0..26);
        new_char = ALPHABET[idx] as char;
    }

    let mut new_genome = String::from(genome);
    new_genome.replace_range(index..index + 1, &new_char.to_string());

    Some((new_genome, index, current_char, new_char))
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

    if carnivory >= 0.65 {
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
        let mut dominant_type = std::collections::HashMap::new();
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
            *dominant_type.entry(key).or_insert(0) += 1;
        }
        let mut sorted_types: Vec<_> = dominant_type.into_iter().collect();
        sorted_types.sort_by(|a, b| b.1.cmp(&a.1));
        if !sorted_types.is_empty() {
            suffix = match sorted_types[0].0 {
                "light" => String::from("ocularium"),
                "chem" => String::from("ciliatum"),
                "kinetic" => String::from("vibrans"),
                "thermal" => String::from("thermum"),
                _ => String::from("lēvis"),
            };
        }
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

/// DNA De-compiler & Phenotype De-compiler (Genotype to Epigenetic Phenotype Compiler)
pub fn parse_genome(genome: &str, antisense_input: Option<&str>, parent_methylations: Option<&[f32]>) -> CreaturePhenotype {
    let current_length = genome.len().clamp(128, 512);
    let mut clean_genome = genome.to_ascii_uppercase().chars().take(current_length).collect::<String>();
    while clean_genome.len() < current_length {
        clean_genome.push('A');
    }

    let mut antisense_strand = match antisense_input {
        Some(anti) => anti.to_ascii_uppercase().chars().take(current_length).collect::<String>(),
        None => get_complementary_string(&clean_genome),
    };
    while antisense_strand.len() < current_length {
        antisense_strand.push('Z');
    }

    // 1. Epigenetic chromatin mapping (Dynamic Active Gene Scanning for specialized organs - TCK-116)
    let mut chromatin_state = vec![false; current_length];
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
                    epigenetic_logs.push(format!("Active Gene Found: Transcribing \"{}\" promoter block at locus {}..{} successfully completed.", promoter, idx, end_idx + 2));
                    idx = end_idx + 2;
                } else {
                    idx += 1;
                }
            } else {
                idx += 1;
            }
        }
    }

    // Create the active_dna string by filtering clean_genome to keep ONLY open chromatin positions
    let mut active_dna = clean_genome.chars().enumerate()
        .filter(|(i, _)| chromatin_state[*i])
        .map(|(_, c)| c)
        .collect::<String>();
    
    // Fallback if active_dna is empty, keeping a minimal active genome
    if active_dna.is_empty() {
        active_dna = String::from("A");
    }

    // --- MODULAR GENOME LOCI LINEAR COMPILER ---
    // Every basic trait is determined locally by parsing and linearly scaling its specific gene payload,
    // falling back to active_dna if the specific promoter is absent. This decouples
    // genes and restores realistic transgenerational family similarity!
    let col_payloads = extract_raw_gene_payloads(&clean_genome, "COL", "EN");
    let col_source = col_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_sym = get_payload_linear_value_offset(&col_source, 11);
    let h_color1 = get_payload_linear_value_offset(&col_source, 0);
    let h_color2 = get_payload_linear_value_offset(&col_source, 5);

    let siz_payloads = extract_raw_gene_payloads(&clean_genome, "SIZ", "EN");
    let siz_source = siz_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_radius = get_payload_linear_value_offset(&siz_source, 0);
    let h_length = get_payload_linear_value_offset(&siz_source, 4);
    let h_seed = get_payload_linear_value_offset(&siz_source, 8);

    let stf_payloads = extract_raw_gene_payloads(&clean_genome, "STF", "EN");
    let stf_source = stf_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_stiffness = get_payload_linear_value_offset(&stf_source, 0);
    let h_hydraulic = get_payload_linear_value_offset(&stf_source, 7);

    let wav_payloads = extract_raw_gene_payloads(&clean_genome, "WAV", "EN");
    let wav_source = wav_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_curve = get_payload_linear_value_offset(&wav_source, 0);
    let h_curve_freq = get_payload_linear_value_offset(&wav_source, 3);
    let h_para_amp = get_payload_linear_value_offset(&wav_source, 6);
    let h_para_freq = get_payload_linear_value_offset(&wav_source, 9);

    let pul_payloads = extract_raw_gene_payloads(&clean_genome, "PUL", "EN");
    let pul_source = pul_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_head = get_payload_linear_value_offset(&pul_source, 0);
    let h_pulse = get_payload_linear_value_offset(&pul_source, 2);
    let h_phase = get_payload_linear_value_offset(&pul_source, 5);
    let h_wiggle = get_payload_linear_value_offset(&pul_source, 9);

    let stm_payloads = extract_raw_gene_payloads(&clean_genome, "STM", "EN");
    let stm_source = stm_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_stomach = get_payload_linear_value_offset(&stm_source, 0);

    let tem_payloads = extract_raw_gene_payloads(&clean_genome, "TEM", "EN");
    let tem_source = tem_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_thermal_c = get_payload_linear_value_offset(&tem_source, 0);
    let h_thermal_w = get_payload_linear_value_offset(&tem_source, 6);

    let car_payloads = extract_raw_gene_payloads(&clean_genome, "CAR", "EN");
    let car_source = car_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_carnivory = get_payload_linear_value_offset(&car_source, 0);

    let rep_payloads = extract_raw_gene_payloads(&clean_genome, "REP", "EN");
    let rep_source = rep_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_mature = get_payload_linear_value_offset(&rep_source, 0);
    let h_repro = get_payload_linear_value_offset(&rep_source, 4);
    let h_split = get_payload_linear_value_offset(&rep_source, 8);

    let evo_payloads = extract_raw_gene_payloads(&clean_genome, "EVO", "EN");
    let evo_source = evo_payloads.get(0).cloned().unwrap_or_else(|| active_dna.clone());
    let h_insert = get_payload_linear_value_offset(&evo_source, 0);
    let h_delete = get_payload_linear_value_offset(&evo_source, 5);
    let h_fidelity = get_payload_linear_value_offset(&evo_source, 10);

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

    // 11. Organelles (Sensory Patches via robust dynamic promoter scanning)
    let mut organelles = Vec::new();

    let eye_payloads = extract_raw_gene_payloads(&clean_genome, "EYE", "EN");
    for payload in &eye_payloads {
        organelles.push(SensoryPatch {
            spectral_affinity: get_payload_linear_value_offset(payload, 1),
            bandwidth: 0.05 + get_payload_linear_value_offset(payload, 4) * 0.85,
            expression_style: get_payload_linear_value_offset(payload, 7),
            scale: 0.35 + get_payload_linear_value_offset(payload, 10) * 1.45,
            spinal_pos: 0.05 + get_payload_linear_value_offset(payload, 13) * 0.9,
            angle: 10.0 + get_payload_linear_value_offset(payload, 16) * 160.0,
            hue_shift: (get_payload_linear_value_offset(payload, 19) * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    let nos_payloads = extract_raw_gene_payloads(&clean_genome, "NOS", "EN");
    for payload in &nos_payloads {
        organelles.push(SensoryPatch {
            spectral_affinity: get_payload_linear_value_offset(payload, 1),
            bandwidth: 0.05 + get_payload_linear_value_offset(payload, 4) * 0.85,
            expression_style: get_payload_linear_value_offset(payload, 7),
            scale: 0.35 + get_payload_linear_value_offset(payload, 10) * 1.45,
            spinal_pos: 0.05 + get_payload_linear_value_offset(payload, 13) * 0.9,
            angle: 10.0 + get_payload_linear_value_offset(payload, 16) * 160.0,
            hue_shift: (get_payload_linear_value_offset(payload, 19) * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    let tac_payloads = extract_raw_gene_payloads(&clean_genome, "TAC", "EN");
    for payload in &tac_payloads {
        organelles.push(SensoryPatch {
            spectral_affinity: get_payload_linear_value_offset(payload, 1),
            bandwidth: 0.05 + get_payload_linear_value_offset(payload, 4) * 0.85,
            expression_style: get_payload_linear_value_offset(payload, 7),
            scale: 0.35 + get_payload_linear_value_offset(payload, 10) * 1.45,
            spinal_pos: 0.05 + get_payload_linear_value_offset(payload, 13) * 0.9,
            angle: 10.0 + get_payload_linear_value_offset(payload, 16) * 160.0,
            hue_shift: (get_payload_linear_value_offset(payload, 19) * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    let lum_payloads = extract_raw_gene_payloads(&clean_genome, "LUM", "EN");
    for payload in &lum_payloads {
        organelles.push(SensoryPatch {
            spectral_affinity: get_payload_linear_value_offset(payload, 1),
            bandwidth: 0.05 + get_payload_linear_value_offset(payload, 4) * 0.85,
            expression_style: get_payload_linear_value_offset(payload, 7),
            scale: 0.35 + get_payload_linear_value_offset(payload, 10) * 1.45,
            spinal_pos: 0.05 + get_payload_linear_value_offset(payload, 13) * 0.9,
            angle: 10.0 + get_payload_linear_value_offset(payload, 16) * 160.0,
            hue_shift: (get_payload_linear_value_offset(payload, 19) * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    // 12. Carnivory
    let carnivory = h_carnivory;
    let is_predator = carnivory >= 0.55;

    // 13. Reproduction Metrics
    let mature_age = (300.0 + h_mature * 2400.0).round() as u32;
    let repro_threshold = 0.60 + h_repro * 0.35;
    let split_loss = 0.05 + h_split * 0.35;

    // 14. Evolutionary Drift
    let insertion_rate = h_insert * 0.12;
    let deletion_rate = h_delete * 0.12;
    let repair_fidelity = 0.15 + h_fidelity * 0.8;

    // 15. CTRNN Brain (TCK-122: 5-Channel Multispectral Sensory Inputs)
    let k_count = organelles.len() * 5;

    // Hidden neurons (Always exactly 4 hidden neurons for rich baseline wiring depth!)
    let h_count = 4;
    let total_nodes = k_count + 1 + 4 + h_count;

    let mut neurons = Vec::with_capacity(total_nodes);

    // A. Inputs (5 overlapping receptive fields/cones per organelle)
    for i in 0..organelles.len() {
        let patch = &organelles[i];
        let deg = patch.angle.round() as i32;
        let aff = patch.spectral_affinity;

        let channel_labels = if aff >= 0.80 {
            vec![
                format!("👁️ UV Vision ({}°)", deg),
                format!("👁️ Blue Vision ({}°)", deg),
                format!("👁️ Green Vision ({}°)", deg),
                format!("👁️ Red Vision ({}°)", deg),
                format!("👁️ IR Vision ({}°)", deg),
            ]
        } else if aff >= 0.25 && aff <= 0.65 {
            vec![
                format!("👃 Acid Smell ({}°)", deg),
                format!("👃 Sugar Smell ({}°)", deg),
                format!("👃 Keton Smell ({}°)", deg),
                format!("👃 Protein Smell ({}°)", deg),
                format!("👃 Phero Smell ({}°)", deg),
            ]
        } else {
            vec![
                format!("🔊 Tactile Hardness ({}°)", deg),
                format!("🔊 Tactile Strömung ({}°)", deg),
                format!("🔊 Tactile Temperatur ({}°)", deg),
                format!("🔊 Tactile Dehnung ({}°)", deg),
                format!("🔊 Tactile Schmerz ({}°)", deg),
            ]
        };

        for c in 0..5 {
            let id = i * 5 + c;
            neurons.push(CTRNNNeuron {
                id,
                neuron_type: NeuronType::Input,
                label: channel_labels[c].clone(),
                tau: 1.0,
                bias: 0.0,
                activation_type: None,
                x: Some(0.1),
                y: Some(0.1 + (id as f32 / k_count.max(1) as f32) * 0.8),
            });
        }
    }

    neurons.push(CTRNNNeuron {
        id: k_count,
        neuron_type: NeuronType::Input,
        label: String::from("⌛ Hunger Clock"),
        tau: 1.0,
        bias: 0.0,
        activation_type: None,
        x: Some(0.1),
        y: Some(0.9),
    });

    // B. Outputs (Thrust, Bending, Biolum, Reserved)
    let out_labels = ["Thrust (Fwd/Bwd)", "Bending (Left/Right)", "Biolum Flash", "Reserved"];
    for i in 0..4 {
        // Derive biases and taus dynamically from the brain params active_dna hashes linearly
        let bias_hash = get_payload_linear_value_offset(&active_dna, i + 1);
        let tau_hash = get_payload_linear_value_offset(&active_dna, i + 10);
        let bias = bias_hash * 2.0 - 1.0;
        let tau = (0.2 + tau_hash * 1.8).clamp(0.1, 2.5);

        neurons.push(CTRNNNeuron {
            id: k_count + 1 + i,
            neuron_type: NeuronType::Output,
            label: String::from(out_labels[i]),
            tau,
            bias,
            activation_type: None,
            x: Some(0.9),
            y: Some(0.2 + (i as f32 / 3.0) * 0.6),
        });
    }

    // C. Hidden Neurons (dynamically compiled from "NEU" payloads or fallbacks!)
    let neu_payloads = extract_raw_gene_payloads(&clean_genome, "NEU", "EN");
    let mut h_count = neu_payloads.len();
    
    // Fallback if no NEU promoters exist
    let fallback_h_count = if h_count == 0 {
        h_count = 2; // baseline of 2 hidden neurons
        true
    } else {
        false
    };
    let h_count = h_count.clamp(1, 12);

    for i in 0..h_count {
        let (bias, tau, activation_type, depth) = if !fallback_h_count && i < neu_payloads.len() {
            let payload = &neu_payloads[i];
            let bias_hash = get_payload_linear_value_offset(payload, 1);
            let tau_hash = get_payload_linear_value_offset(payload, 4);
            let act_hash = get_payload_linear_value_offset(payload, 7);
            let depth_hash = get_payload_linear_value_offset(payload, 10);

            let bias = bias_hash * 2.0 - 1.0;
            let tau = (0.2 + tau_hash * 1.8).clamp(0.1, 2.5);
            let activation_type = match (act_hash * 100.0) as usize % 4 {
                1 => String::from("relu"),
                2 => String::from("sigmoid"),
                3 => String::from("sin"),
                _ => String::from("tanh"),
            };
            let depth = 0.15 + depth_hash * 0.7; // maps depth between 0.15 and 0.85
            (bias, tau, activation_type, depth)
        } else {
            // Fallback default values
            let bias_hash = get_payload_linear_value_offset(&active_dna, i + 2);
            let tau_hash = get_payload_linear_value_offset(&active_dna, i + 12);
            let act_hash = get_payload_linear_value_offset(&active_dna, i + 22);

            let bias = bias_hash * 2.0 - 1.0;
            let tau = (0.2 + tau_hash * 1.8).clamp(0.1, 2.5);
            let activation_type = match (act_hash * 100.0) as usize % 4 {
                1 => String::from("relu"),
                2 => String::from("sigmoid"),
                3 => String::from("sin"),
                _ => String::from("tanh"),
            };
            let depth = 0.15 + (i as f32 / (h_count.saturating_sub(1)) as f32) * 0.7;
            (bias, tau, activation_type, depth)
        };

        neurons.push(CTRNNNeuron {
            id: k_count + 5 + i,
            neuron_type: NeuronType::Hidden,
            label: format!("Hidden #{}", i + 1),
            tau,
            bias,
            activation_type: Some(activation_type),
            x: Some(0.5),
            y: Some(depth),
        });
    }

    // D. Synapses (Exuberant Fully-Connected Brain!)
    let mut synapses = Vec::with_capacity(120);
    let syn_payloads = extract_raw_gene_payloads(&clean_genome, "SY", "EN");

    // Establish valid sources and destinations arrays for robust wiring!
    let mut sources = Vec::new();
    for s in 0..=k_count { sources.push(s); }
    for h in 0..h_count { sources.push(k_count + 5 + h); }

    let mut destinations = Vec::new();
    for o in 0..4 { destinations.push(k_count + 1 + o); }
    for h in 0..h_count { destinations.push(k_count + 5 + h); }

    // Parse all explicit "SY" promoter genes into a lookup list
    let mut explicit_synapses = Vec::new();
    for (idx, payload) in syn_payloads.iter().enumerate().take(30) {
        let h_from = get_payload_linear_value_offset(payload, 1);
        let h_to = get_payload_linear_value_offset(payload, 4);
        let h_weight = get_payload_linear_value_offset(payload, 7);

        let from_node = sources[(h_from * 1000.0) as usize % sources.len()];
        let to_node = destinations[(h_to * 1000.0) as usize % destinations.len()];
        let weight = h_weight * 4.0 - 2.0;

        explicit_synapses.push((from_node, to_node, weight));
    }

    // Build a fully-connected graph (Exuberance at birth)
    for &from_node in &sources {
        for &to_node in &destinations {
            // Avoid direct self-loops to prevent trivial infinite positive feedback loops
            if from_node == to_node {
                continue;
            }

            // Check if there is an explicit, strong "SY" gene for this connection
            let weight = if let Some(explicit) = explicit_synapses.iter().find(|s| s.0 == from_node && s.1 == to_node) {
                explicit.2 // use strong, genetically specialized weight!
            } else {
                // Initialize as a weak, random exploratory synapse with enough active power to drive Hebbian learning
                let h_weight = get_payload_linear_value_offset(&active_dna, from_node + to_node * 13);
                h_weight * 1.0 - 0.5 // exploratory weight in [-0.5 .. 0.5]
            };

            synapses.push(CTRNNSynapse {
                from_node,
                to_node,
                weight,
            });
        }
    }

    let brain = BrainTopology { neurons, synapses };

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
        * (1.0 + k_count as f32 * 0.08)
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
        let dna = generate_random_genome(length);
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

    #[test]
    fn test_mutation() {
        // Let's use a 200-char random genome which is in the safe range for substitution, deletion, and insertion
        let original = generate_random_genome(200);
        if let Some((mutated, idx, old, new)) = mutate_genome(&original) {
            assert!(mutated.len() == original.len() || mutated.len() == original.len() + 1 || mutated.len() == original.len() - 1);
            assert_ne!(mutated, original);
            if mutated.len() == original.len() {
                // Substitution
                assert_eq!(original.chars().nth(idx).unwrap(), old);
                assert_eq!(mutated.chars().nth(idx).unwrap(), new);
            } else if mutated.len() == original.len() + 1 {
                // Insertion
                assert_eq!(old, '+');
                assert_eq!(mutated.chars().nth(idx).unwrap(), new);
            } else {
                // Deletion
                assert_eq!(old, '-');
                assert_eq!(original.chars().nth(idx).unwrap(), new);
            }
        } else {
            panic!("Mutation failed");
        }

        // Verify promoter-safe conservation with degenerate matching
        let genome_with_promoter = "COLABCEN"; // "COL" (promoter), "ABC" (payload), "EN" (terminator)
        for _ in 0..100 {
            if let Some((mutated, idx, _, _)) = mutate_genome(genome_with_promoter) {
                // The mutation index should only target unprotected regions.
                // In "COLABCEN", COL is protected, EN is protected. ABC is at index 3..=5.
                // But wait! If mutate_genome rolls an insertion, mutated.len() can become 9, adding a char at 3..=5.
                // If it rolls a deletion, mutated.len() can become 7, removing a char from 3..=5.
                // If it rolls substitution, length remains 8.
                // In all cases, the index should be within 3..=5 (of original) and the promoter/terminator must remain unbroken.
                assert!(idx >= 3 && idx <= 5, "Mutation indices must only target unprotected payload regions! Found index: {}", idx);
                assert!(matches_degenerate(&mutated[0..3], "COL") && matches_degenerate(&mutated[mutated.len()-2..], "EN"), "Promoters and terminators must never be broken!");
            }
        }
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

        assert_eq!(input_count, phenotype.organelles.len() * 5 + 1); // 5 Channels per Organelle + Hunger clock
        assert_eq!(output_count, 4); // Thrust, Bending, Biolum Flash, Reserved
        assert!(hidden_count >= 1 && hidden_count <= 10);

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
                0.1, // high learning rate
                0.0,
                0.001, // small decay
            );
        }

        let learned_weight = synapse_weights[0];
        // Weight should change due to Hebbian correlation!
        assert_ne!(learned_weight, original_weight);
    }

    #[test]
    fn test_synaptic_exuberance_and_active_pruning() {
        use crate::shared::brain::execute_brain_with_learning;

        let dna = "NEUAAESNEUBBESZZSYBCAESYBBBES"; // 3 hiddens, 1 SY
        let phenotype = parse_genome(dna, None, None);
        let brain = phenotype.brain;

        // Verify Exuberance: 1 input (0 organelles + 1 clock), 3 hiddens, 4 outputs
        // Sources: 4. Destinations: 7.
        // Total synapses (excluding direct self-loops): 4 * 7 - 3 = 25!
        assert_eq!(brain.synapses.len(), 25);

        // Verify that exploratory synapses start very weak, while explicit "SY" start strong
        let mut exploratory_count = 0;
        let mut specialized_count = 0;
        for syn in &brain.synapses {
            if syn.weight.abs() < 0.1 {
                exploratory_count += 1;
            } else {
                specialized_count += 1;
            }
        }
        assert!(exploratory_count > 0);
        assert!(specialized_count > 0);

        // Test Active Pruning
        let mut synapse_weights = brain.synapses.iter().map(|s| s.weight).collect::<Vec<f32>>();
        let mut states = vec![0.0; brain.neurons.len()];
        let mut activations = vec![0.0; brain.neurons.len()];
        let inputs = vec![0.0, 0.0]; // Zero inputs to trigger pure forgetting decay

        // Run 50 steps of forgetting decay to prune weak connections
        for _ in 0..50 {
            execute_brain_with_learning(
                &brain,
                &inputs,
                &mut states,
                &mut activations,
                &mut synapse_weights,
                0.0001, // tiny learning rate to keep the learning block active!
                0.0,
                0.1, // high forgetting rate to prune quickly
            );
        }

        // Weak exploratory synapses should now be exactly 0.0 (permanently pruned!)
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
        for trial in 0..50 {
            let start_len = 128 + (trial * 7) % 380; // various lengths in [128..508]
            let mut genome = generate_random_genome(start_len);

            // Apply 50 random mutations, insertions, or deletions consecutively to scramble promoters and terminators
            for _ in 0..50 {
                if let Some((mutated, _, _, _)) = mutate_genome(&genome) {
                    genome = mutated;
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

            let k_count = brain.neurons.iter().filter(|n| n.neuron_type == NeuronType::Input).count() - 1;
            let inputs = vec![0.5; k_count + 1];

            for _tick in 0..10 {
                let outputs = execute_brain_with_learning(
                    &brain,
                    &inputs,
                    &mut states,
                    &mut activations,
                    &mut synapse_weights,
                    0.05, // active Hebbian learning
                    0.01, // active stiffness decay
                    0.02, // active forgetting decay
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
}
