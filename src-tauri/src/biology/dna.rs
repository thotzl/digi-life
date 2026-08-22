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

pub fn extract_raw_gene_signals(
    genome: &str,
    start_motif: &str,
    stop_motif: &str
) -> Vec<f32> {
    let mut signals = Vec::new();
    let genome_bytes = genome.as_bytes();
    let start_bytes = start_motif.as_bytes();
    let stop_bytes = stop_motif.as_bytes();

    if start_bytes.is_empty() || stop_bytes.is_empty() || genome.len() < start_motif.len() + stop_motif.len() {
        return signals;
    }

    let mut idx = 0;
    while idx <= genome.len().saturating_sub(start_motif.len()) {
        if &genome_bytes[idx..idx + start_motif.len()] == start_bytes {
            let payload_start = idx + start_motif.len();
            let mut payload_end = None;
            for j in payload_start..=genome.len().saturating_sub(stop_motif.len()) {
                if &genome_bytes[j..j + stop_motif.len()] == stop_bytes {
                    payload_end = Some(j);
                    break;
                }
            }

            if let Some(end_idx) = payload_end {
                let payload_slice = &genome[payload_start..end_idx];
                for c in payload_slice.chars() {
                    signals.push(char_to_value(c) as f32 / 25.0);
                }
                idx = end_idx + stop_motif.len();
            } else {
                idx += 1;
            }
        } else {
            idx += 1;
        }
    }
    signals
}

pub fn generate_random_genome(_length: usize) -> String {
    let mut rng = rand::thread_rng();
    let base_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLEN";
    let mut mutated = base_dna.to_string();

    // Apply 15 to 35 sequential mutations to randomize traits deeply while preserving promoter-terminator structures
    let num_mutations = rng.gen_range(15..=35);
    for _ in 0..num_mutations {
        if let Some((mut_dna, _, _, _)) = mutate_genome(&mutated) {
            mutated = mut_dna;
        }
    }
    mutated
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
        "EYE", "NOS", "TAC", "LUM", "CAR", "REP", "EVO", "NEU", "SYN", "OUT", "EN"
    ];

    let mut protected = vec![false; current_length];
    for motif in &protected_motifs {
        let motif_bytes = motif.as_bytes();
        let genome_bytes = genome.as_bytes();
        let mut idx = 0;
        while idx <= current_length.saturating_sub(motif.len()) {
            if &genome_bytes[idx..idx + motif.len()] == motif_bytes {
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

    // Choose index to mutate
    let index = if !unprotected_indices.is_empty() {
        let idx_in_list = rng.gen_range(0..unprotected_indices.len());
        unprotected_indices[idx_in_list]
    } else {
        // Fallback if everything is protected
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

/// DNA De-compiler & Phenotype De-compiler (Genotype to Epigenetic Phenotype Compiler)
pub fn parse_genome(genome: &str, antisense_input: Option<&str>, parent_methylations: Option<&[f32]>) -> CreaturePhenotype {
    let current_length = genome.len().clamp(128, 384);
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

    // 1. Epigenetic chromatin mapping (Dynamic Active Gene Scanning - TCK-116)
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

    // Scan for all active gene promoter regions and map them to chromatin_state and epigenetic logs!
    let active_promoters = [
        "SYM", "COL", "SED", "SIZ", "WAV", "STF", "CRV", "PUL", "STM", "TEM",
        "EYE", "NOS", "TAC", "LUM", "CAR", "REP", "EVO", "NEU", "SYN", "OUT"
    ];
    let genome_bytes = clean_genome.as_bytes();
    for promoter in &active_promoters {
        let promoter_bytes = promoter.as_bytes();
        let mut idx = 0;
        while idx <= current_length.saturating_sub(promoter.len()) {
            if &genome_bytes[idx..idx + promoter.len()] == promoter_bytes {
                let payload_start = idx + promoter.len();
                let mut payload_end = None;
                for j in payload_start..=current_length.saturating_sub(2) {
                    let c_a = genome_bytes[j];
                    let c_b = genome_bytes[j + 1];
                    if (c_a == b'S' && c_b == b'P') || (c_a == b'E' && c_b == b'N') {
                        payload_end = Some(j);
                        break;
                    }
                }

                if let Some(end_idx) = payload_end {
                    // Mark the entire transcribing gene span as active/open (true) on chromatin!
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

    // --- DECOUPLED SEQUENCE-BASED GENOME COMPILER (TCK-116) ---

    // 1. Symmetry
    let sym_signals = extract_raw_gene_signals(&clean_genome, "SYM", "EN");
    let symmetry = if sym_signals.len() >= 1 && sym_signals[0] >= 0.5 { "quad" } else { "vertical" };

    // 2. Primary & Secondary Colors
    let col_signals = extract_raw_gene_signals(&clean_genome, "COL", "EN");
    let primary_color = if col_signals.len() >= 3 {
        HSLColor {
            h: (col_signals[0] * 360.0).round(),
            s: (55.0 + col_signals[1] * 45.0).round(),
            l: (35.0 + col_signals[2] * 45.0).round(),
        }
    } else {
        HSLColor { h: 130.0, s: 75.0, l: 45.0 } // Default vibrant green
    };

    let secondary_color = if col_signals.len() >= 6 {
        HSLColor {
            h: (col_signals[3] * 360.0).round(),
            s: (55.0 + col_signals[4] * 45.0).round(),
            l: (35.0 + col_signals[5] * 45.0).round(),
        }
    } else if col_signals.len() >= 3 {
        HSLColor {
            h: ((col_signals[0] * 360.0 + 180.0) % 360.0).round(),
            s: (55.0 + col_signals[1] * 45.0).round(),
            l: (35.0 + col_signals[2] * 45.0).round(),
        }
    } else {
        HSLColor { h: 310.0, s: 75.0, l: 45.0 } // Default vibrant purple
    };

    // 3. Body Seed
    let sed_signals = extract_raw_gene_signals(&clean_genome, "SED", "EN");
    let body_seed = if sed_signals.len() >= 1 { (sed_signals[0] * 100_000.0) as u32 } else { 4293 };

    // 4. Body Size (Mean Radius & Base Length)
    let size_signals = extract_raw_gene_signals(&clean_genome, "SIZ", "EN");
    let mean_radius = if size_signals.len() >= 1 { (16.0f32).max((28.0f32).min(16.0 + size_signals[0] * 12.0)) } else { 22.0 };
    let base_length = if size_signals.len() >= 2 { (90.0f32).max((200.0f32).min(90.0 + size_signals[1] * 110.0)) } else { 145.0 };

    // 5. Spinal Harmonics (Amplitudes & Phases)
    let wav_signals = extract_raw_gene_signals(&clean_genome, "WAV", "EN");
    let mut amplitudes = vec![0.0; 4];
    for j in 0..4 {
        if wav_signals.len() > j {
            amplitudes[j] = wav_signals[j] * 0.3 - 0.15;
        }
    }
    let mut phases = vec![0.0; 4];
    for j in 0..4 {
        if wav_signals.len() > 4 + j {
            phases[j] = wav_signals[4 + j] * std::f32::consts::PI * 2.0;
        }
    }

    // 6. Stiffness
    let stf_signals = extract_raw_gene_signals(&clean_genome, "STF", "EN");
    let stiffness = if !stf_signals.is_empty() { (0.15f32).max((1.0f32).min(0.15 + stf_signals[0] * 0.85)) } else { 0.5 };

    // 7. Curves, Parapodia, etc.
    let crv_signals = extract_raw_gene_signals(&clean_genome, "CRV", "EN");
    let spinal_curve = if crv_signals.len() >= 1 { crv_signals[0] * 44.0 - 22.0 } else { 0.0 };
    let spinal_curve_freq = if crv_signals.len() >= 2 { 1.0 + (crv_signals[1] * 3.0).floor() } else { 2.0 };
    let parapodia_amp = if crv_signals.len() >= 3 { crv_signals[2] * 0.45 } else { 0.2 };
    let parapodia_freq = if crv_signals.len() >= 4 { 2.0 + (crv_signals[3] * 3.0).floor() } else { 3.0 };
    let flattening_head = if crv_signals.len() >= 5 { crv_signals[4] * 1.4 - 0.4 } else { 0.1 };

    // 8. Pulse Speed, Wave Phase, Wiggle Amplitude
    let pul_signals = extract_raw_gene_signals(&clean_genome, "PUL", "EN");
    let pulse_speed = if pul_signals.len() >= 1 { 0.0015 + pul_signals[0] * 0.0075 } else { 0.005 };
    let wave_phase = if pul_signals.len() >= 2 { pul_signals[1] * 1.6 } else { 0.8 };
    let wiggle_amplitude = if pul_signals.len() >= 3 { pul_signals[2] * 0.22 } else { 0.11 };

    // 9. Stomach Capacity & Hydraulic Pressure
    let stm_signals = extract_raw_gene_signals(&clean_genome, "STM", "EN");
    let stomach_capacity = if stm_signals.len() >= 1 { 50.0 + stm_signals[0] * 450.0 } else { 150.0 };
    let hydraulic_pressure = if stm_signals.len() >= 2 { 0.2 + stm_signals[1] * 0.8 } else { 0.6 };

    // 10. Thermal Tolerance
    let tem_signals = extract_raw_gene_signals(&clean_genome, "TEM", "EN");
    let thermal_center = if tem_signals.len() >= 1 { 10.0 + tem_signals[0] * 60.0 } else { 40.0 };
    let thermal_width = if tem_signals.len() >= 2 { 10.0 + tem_signals[1] * 30.0 } else { 20.0 };
    let thermal_tolerance_min = -5.0f32.max((thermal_center - thermal_width / 2.0).round());
    let thermal_tolerance_max = 105.0f32.min((thermal_center + thermal_width / 2.0).round());

    // 11. Organelles (Sensory Patches)
    let mut organelles = Vec::new();

    let eye_signals = extract_raw_gene_signals(&clean_genome, "EYE", "EN");
    for chunk in eye_signals.chunks_exact(7) {
        organelles.push(SensoryPatch {
            spectral_affinity: 0.85 + chunk[0] * 0.15,
            bandwidth: 0.1 + chunk[1] * 0.4,
            expression_style: chunk[2],
            scale: 0.35 + chunk[3] * 1.45,
            spinal_pos: 0.05 + chunk[4] * 0.9,
            angle: 10.0 + chunk[5] * 160.0,
            hue_shift: (chunk[6] * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    let nos_signals = extract_raw_gene_signals(&clean_genome, "NOS", "EN");
    for chunk in nos_signals.chunks_exact(7) {
        organelles.push(SensoryPatch {
            spectral_affinity: 0.35 + chunk[0] * 0.3,
            bandwidth: 0.2 + chunk[1] * 0.6,
            expression_style: chunk[2],
            scale: 0.35 + chunk[3] * 1.45,
            spinal_pos: 0.05 + chunk[4] * 0.9,
            angle: 10.0 + chunk[5] * 160.0,
            hue_shift: (chunk[6] * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    let tac_signals = extract_raw_gene_signals(&clean_genome, "TAC", "EN");
    for chunk in tac_signals.chunks_exact(7) {
        organelles.push(SensoryPatch {
            spectral_affinity: 0.05 + chunk[0] * 0.15,
            bandwidth: 0.3 + chunk[1] * 0.7,
            expression_style: chunk[2],
            scale: 0.35 + chunk[3] * 1.45,
            spinal_pos: 0.05 + chunk[4] * 0.9,
            angle: 10.0 + chunk[5] * 160.0,
            hue_shift: (chunk[6] * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    let lum_signals = extract_raw_gene_signals(&clean_genome, "LUM", "EN");
    for chunk in lum_signals.chunks_exact(7) {
        organelles.push(SensoryPatch {
            spectral_affinity: chunk[0],
            bandwidth: chunk[1],
            expression_style: chunk[2],
            scale: 0.35 + chunk[3] * 1.45,
            spinal_pos: 0.05 + chunk[4] * 0.9,
            angle: 10.0 + chunk[5] * 160.0,
            hue_shift: (chunk[6] * 360.0 - 180.0).round(),
            gene_start_index: 0,
            gene_end_index: 0,
        });
    }

    // 12. Carnivory
    let car_signals = extract_raw_gene_signals(&clean_genome, "CAR", "EN");
    let carnivory = if !car_signals.is_empty() { car_signals[0] } else { 0.2 };
    let is_predator = carnivory >= 0.55;

    // 13. Reproduction Metrics
    let rep_signals = extract_raw_gene_signals(&clean_genome, "REP", "EN");
    let mature_age = if rep_signals.len() >= 1 { (300.0 + rep_signals[0] * 2400.0).round() as u32 } else { 1200 };
    let repro_threshold = if rep_signals.len() >= 2 { 0.60 + rep_signals[1] * 0.35 } else { 0.75 };
    let split_loss = if rep_signals.len() >= 3 { 0.05 + rep_signals[2] * 0.35 } else { 0.15 };

    // 14. Evolutionary Drift
    let evo_signals = extract_raw_gene_signals(&clean_genome, "EVO", "EN");
    let insertion_rate = if evo_signals.len() >= 1 { evo_signals[0] * 0.12 } else { 0.05 };
    let deletion_rate = if evo_signals.len() >= 2 { evo_signals[1] * 0.12 } else { 0.05 };
    let repair_fidelity = if evo_signals.len() >= 3 { 0.15 + evo_signals[2] * 0.8 } else { 0.85 };

    // 15. CTRNN Brain
    let k_count = organelles.len();

    // Hidden neurons
    let neu_signals = extract_raw_gene_signals(&clean_genome, "NEU", "EN");
    let mut hidden_neurons = Vec::new();
    for chunk in neu_signals.chunks_exact(3) {
        let bias = chunk[0] * 2.0 - 1.0;
        let tau = (0.2 + chunk[1] * 1.8).clamp(0.1, 2.5);
        let act_val = (chunk[2] * 100.0) as usize;
        let activation_type = match act_val % 4 {
            1 => String::from("relu"),
            2 => String::from("sigmoid"),
            3 => String::from("sin"),
            _ => String::from("tanh"),
        };
        hidden_neurons.push((bias, tau, activation_type));
    }
    let h_count = hidden_neurons.len().clamp(2, 10);
    let total_nodes = k_count + 1 + 4 + h_count;

    let mut neurons = Vec::with_capacity(total_nodes);

    // A. Inputs
    for i in 0..k_count {
        let patch = &organelles[i];
        let deg = patch.angle.round() as i32;
        let label = if patch.spectral_affinity >= 0.8 {
            format!("👁️ Vision ({}°)", deg)
        } else if (0.25..=0.65).contains(&patch.spectral_affinity) {
            format!("👃 Smell ({}°)", deg)
        } else if patch.spectral_affinity < 0.25 {
            format!("🔊 Tactile ({}°)", deg)
        } else {
            format!("Receptor ({}°)", deg)
        };

        neurons.push(CTRNNNeuron {
            id: i,
            neuron_type: NeuronType::Input,
            label,
            tau: 1.0,
            bias: 0.0,
            activation_type: None,
            x: Some(0.1),
            y: Some(0.1 + (i as f32 / k_count.max(1) as f32) * 0.8),
        });
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
    let out_signals = extract_raw_gene_signals(&clean_genome, "OUT", "EN");
    for i in 0..4 {
        let bias = if out_signals.len() > i * 2 { out_signals[i * 2] * 2.0 - 1.0 } else { 0.0 };
        let tau = if out_signals.len() > i * 2 + 1 { (0.2 + out_signals[i * 2 + 1] * 1.8).clamp(0.1, 2.5) } else { 1.0 };

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

    // C. Hidden Neurons
    for i in 0..h_count {
        let (bias, tau, activation_type) = if i < hidden_neurons.len() {
            hidden_neurons[i].clone()
        } else {
            (0.0, 1.0, String::from("tanh"))
        };

        neurons.push(CTRNNNeuron {
            id: k_count + 5 + i,
            neuron_type: NeuronType::Hidden,
            label: format!("Hidden #{}", i + 1),
            tau,
            bias,
            activation_type: Some(activation_type),
            x: Some(0.5),
            y: Some(0.15 + (i as f32 / (h_count.saturating_sub(1)) as f32) * 0.7),
        });
    }

    // D. Synapses
    let mut synapses = Vec::new();
    let syn_signals = extract_raw_gene_signals(&clean_genome, "SYN", "EN");
    for chunk in syn_signals.chunks_exact(4) {
        let raw_from = (chunk[0] * 1000.0) as usize;
        let raw_to = (chunk[1] * 1000.0) as usize;
        let weight = chunk[2] * 4.0 - 2.0;

        let from_node = raw_from % total_nodes;
        let to_node = (raw_to % (4 + h_count)) + (k_count + 1);

        if !synapses.iter().any(|syn: &CTRNNSynapse| syn.from_node == from_node && syn.to_node == to_node) {
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
        let dna = generate_random_genome(128);
        assert_eq!(dna.len(), 107); // Sequence-based base_dna length is 107
        for c in dna.chars() {
            assert!(ALPHABET.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_extract_raw_gene_signals() {
        let genome = "STFAENCOLABSTPEYEAABEN";
        let stf_signals = extract_raw_gene_signals(genome, "STF", "EN");
        assert_eq!(stf_signals.len(), 1); // "A" -> 0.0
        assert_eq!(stf_signals[0], 0.0);

        let col_signals = extract_raw_gene_signals(genome, "COL", "STP");
        assert_eq!(col_signals.len(), 2); // "AB" -> [0/25, 1/25]
        assert_eq!(col_signals[0], 0.0);
        assert_eq!(col_signals[1], 1.0 / 25.0);
    }

    #[test]
    fn test_mutation() {
        let original = "AAAA";
        if let Some((mutated, idx, old, new)) = mutate_genome(original) {
            assert_eq!(mutated.len(), original.len());
            assert_ne!(mutated, original);
            assert_eq!(old, 'A');
            assert_eq!(mutated.chars().nth(idx).unwrap(), new);
        } else {
            panic!("Mutation failed");
        }

        // Verify promoter-safe conservation
        let genome_with_promoter = "COLABCEN"; // "COL" (promoter), "ABC" (payload), "EN" (terminator)
        for _ in 0..100 {
            if let Some((mutated, idx, _, _)) = mutate_genome(genome_with_promoter) {
                // The mutation index should only ever land in the unprotected payload "ABC" (indices 3, 4, or 5)
                assert!(idx >= 3 && idx <= 5, "Mutation indices must only target unprotected payload regions! Found index: {}", idx);
                assert!(mutated.starts_with("COL") && mutated.ends_with("EN"), "Promoters and terminators must never be broken!");
            }
        }
    }

    #[test]
    fn test_progenitor_parse_genome() {
        // High quality test input representing a well-formed progenitor genome
        let progenitor_dna = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLEN";
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

        assert_eq!(input_count, phenotype.organelles.len() + 1); // Organelles + Hunger clock
        assert_eq!(output_count, 4); // Thrust, Bending, Biolum Flash, Reserved
        assert!(hidden_count >= 2 && hidden_count <= 10);

        // Verify that BMR scaling values are deterministic and finite
        assert!(phenotype.basal_metabolic_rate.is_finite());
        assert!(phenotype.stomach_capacity > 0.0);
    }
}
