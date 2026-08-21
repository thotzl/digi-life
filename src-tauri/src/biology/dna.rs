use serde::{Deserialize, Serialize};
use rand::Rng;

pub const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HSLColor {
    pub h: f32, // 0 - 360
    pub s: f32, // 0 - 100
    pub l: f32, // 0 - 100
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensoryPatch {
    #[serde(rename = "spectralAffinity")]
    pub spectral_affinity: f32,
    pub bandwidth: f32,
    #[serde(rename = "expressionStyle")]
    pub expression_style: f32,
    pub scale: f32,
    #[serde(rename = "spinalPos")]
    pub spinal_pos: f32,
    pub angle: f32,
    #[serde(rename = "hueShift")]
    pub hue_shift: f32,
    #[serde(rename = "geneStartIndex")]
    pub gene_start_index: usize,
    #[serde(rename = "geneEndIndex")]
    pub gene_end_index: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpinalHarmonics {
    #[serde(rename = "baseLength")]
    pub base_length: f32,
    #[serde(rename = "meanRadius")]
    pub mean_radius: f32,
    pub amplitudes: Vec<f32>,
    pub phases: Vec<f32>,
    #[serde(rename = "spinalCurve")]
    pub spinal_curve: f32,
    #[serde(rename = "spinalCurveFreq")]
    pub spinal_curve_freq: f32,
    #[serde(rename = "parapodiaAmp")]
    pub parapodia_amp: f32,
    #[serde(rename = "parapodiaFreq")]
    pub parapodia_freq: f32,
    #[serde(rename = "flatteningHead")]
    pub flattening_head: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CTRNNSynapse {
    #[serde(rename = "fromNode")]
    pub from_node: usize,
    #[serde(rename = "toNode")]
    pub to_node: usize,
    pub weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NeuronType {
    Input,
    Hidden,
    Output,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CTRNNNeuron {
    pub id: usize,
    #[serde(rename = "type")]
    pub neuron_type: NeuronType,
    pub label: String,
    pub tau: f32,  // time constant [0.5 to 5.0]
    pub bias: f32, // neural bias [-1.0 to 1.0]
    #[serde(rename = "activationType")]
    pub activation_type: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrainTopology {
    pub neurons: Vec<CTRNNNeuron>,
    pub synapses: Vec<CTRNNSynapse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreaturePhenotype {
    pub symmetry: String, // "vertical" | "quad"
    #[serde(rename = "primaryColor")]
    pub primary_color: HSLColor,
    #[serde(rename = "secondaryColor")]
    pub secondary_color: HSLColor,
    #[serde(rename = "bodySeed")]
    pub body_seed: u32,
    pub segments: Vec<serde_json::Value>,
    #[serde(rename = "spinalHarmonics")]
    pub spinal_harmonics: SpinalHarmonics,
    #[serde(rename = "emergentChambersCount")]
    pub emergent_chambers_count: usize,
    pub organelles: Vec<SensoryPatch>,
    #[serde(rename = "pulseSpeed")]
    pub pulse_speed: f32,
    #[serde(rename = "wavePhase")]
    pub wave_phase: f32,
    #[serde(rename = "wiggleAmplitude")]
    pub wiggle_amplitude: f32,
    pub stiffness: f32,

    #[serde(rename = "matureAge")]
    pub mature_age: u32,
    #[serde(rename = "reproThreshold")]
    pub repro_threshold: f32,
    #[serde(rename = "splitLoss")]
    pub split_loss: f32,

    pub brain: BrainTopology,
    pub carnivory: f32,
    #[serde(rename = "isPredator")]
    pub is_predator: bool,

    #[serde(rename = "latinName")]
    pub latin_name: String,
    #[serde(rename = "sensoryVisus")]
    pub sensory_visus: f32,
    #[serde(rename = "sensoryOlfaction")]
    pub sensory_olfaction: f32,
    #[serde(rename = "sensoryTactility")]
    pub sensory_tactility: f32,
    #[serde(rename = "sensoryBiolum")]
    pub sensory_biolum: f32,
    #[serde(rename = "dietClass")]
    pub diet_class: String,
    #[serde(rename = "preferredHabitat")]
    pub preferred_habitat: String,

    #[serde(rename = "basalMetabolicRate")]
    pub basal_metabolic_rate: f32,
    #[serde(rename = "stomachCapacity")]
    pub stomach_capacity: f32,
    #[serde(rename = "thermalToleranceMin")]
    pub thermal_tolerance_min: f32,
    #[serde(rename = "thermalToleranceMax")]
    pub thermal_tolerance_max: f32,
    #[serde(rename = "hydraulicPressure")]
    pub hydraulic_pressure: f32,
    #[serde(rename = "rotationalInertia")]
    pub rotational_inertia: f32,

    #[serde(rename = "survivalExpectation")]
    pub survival_expectation: f32,
    #[serde(rename = "survivalAnalysis")]
    pub survival_analysis: String,

    #[serde(rename = "chromatinState")]
    pub chromatin_state: Vec<bool>,
    #[serde(rename = "epigeneticLogs")]
    pub epigenetic_logs: Vec<String>,
    pub methylations: Vec<f32>,

    #[serde(rename = "antisenseStrand")]
    pub antisense_strand: String,
    #[serde(rename = "repairFidelity")]
    pub repair_fidelity: f32,
    #[serde(rename = "insertionRate")]
    pub insertion_rate: f32,
    #[serde(rename = "deletionRate")]
    pub deletion_rate: f32,

    #[serde(rename = "genomeString")]
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
    let index = rng.gen_range(0..genome.len());
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

    let char_vals: Vec<usize> = clean_genome.chars().map(char_to_value).collect();
    let get_val = |idx: usize| -> usize { char_vals[idx] };

    // Primary evolutionary drift metrics
    let insertion_rate = (get_val(9) as f32 / 25.0) * 0.12;
    let deletion_rate = (get_val(10) as f32 / 25.0) * 0.12;
    let repair_fidelity = 0.15 + (get_val(11) as f32 / 25.0) * 0.8;

    let mut antisense_strand = match antisense_input {
        Some(anti) => anti.to_ascii_uppercase().chars().take(current_length).collect::<String>(),
        None => get_complementary_string(&clean_genome),
    };
    while antisense_strand.len() < current_length {
        antisense_strand.push('Z');
    }

    // 1. Epigenetic chromatin mapping (Embryology cascade waves)
    let mut chromatin_state = vec![false; current_length];
    for idx in 0..16 {
        chromatin_state[idx] = true;
    }

    for idx in 16..(current_length.saturating_sub(2)) {
        let char_a = clean_genome.as_bytes()[idx];
        let char_b = clean_genome.as_bytes()[idx + 1];
        if (char_a == b'S' && char_b == b'T') || (char_a == b'G' && char_b == b'O') {
            chromatin_state[idx] = true;
            chromatin_state[idx + 1] = true;
            if idx + 2 < current_length { chromatin_state[idx + 2] = true; }
            if idx + 3 < current_length { chromatin_state[idx + 3] = true; }
        }
    }

    let mut epigenetic_logs = Vec::new();
    let mut methylations = match parent_methylations {
        Some(m) if m.len() == current_length => {
            epigenetic_logs.push(String::from("Transgenerational epigenetic inheritance: Learned brain methylation pattern of the parent cell successfully assimilated."));
            m.to_vec()
        }
        _ => vec![0.0; current_length],
    };

    let get_methylated_val = |idx: usize, genome_chars: &[usize], meths: &[f32]| -> usize {
        let raw = genome_chars[idx] as i32;
        let meth = meths[idx] as i32;
        ((raw + meth).rem_euclid(26)) as usize
    };

    // Execute 3 embryological cascade hox waves
    for wave in 1..=3 {
        let mut wave_loci_modified = 0;
        let mut wave_methylations = 0;

        let mut idx = 16;
        while idx < current_length.saturating_sub(2) {
            if chromatin_state[idx] && chromatin_state[idx + 1] {
                let char_a = clean_genome.as_bytes()[idx];
                let char_b = clean_genome.as_bytes()[idx + 1];
                let is_hox = (char_a == b'E' && char_b == b'P') || (char_a == b'H' && char_b == b'X');

                if is_hox {
                    let hox_start = idx;
                    let payload_start = idx + 2;
                    let mut payload_end = None;

                    for j in payload_start..(current_length.saturating_sub(1)) {
                        let c_a = clean_genome.as_bytes()[j];
                        let c_b = clean_genome.as_bytes()[j + 1];
                        if (c_a == b'S' && c_b == b'P') || (c_a == b'E' && c_b == b'N') {
                            payload_end = Some(j);
                            break;
                        }
                    }

                    let end_idx = payload_end.unwrap_or(current_length);
                    let payload_length = end_idx.saturating_sub(payload_start);

                    if payload_length >= 3 {
                        let target_char_idx = get_methylated_val(payload_start, &char_vals, &methylations);
                        let target_letter = ALPHABET[target_char_idx];
                        let action_val = get_methylated_val(payload_start + 1, &char_vals, &methylations);
                        let power_radius = 5.0 + get_methylated_val(payload_start + 2, &char_vals, &methylations) as f32 * 1.5;

                        for target_idx in 16..current_length {
                            if clean_genome.as_bytes()[target_idx] == target_letter {
                                let distance = (target_idx as f32 - hox_start as f32).abs();
                                if distance <= power_radius {
                                    if action_val < 9 {
                                        let start_slot = (target_idx.saturating_sub(3)).max(16);
                                        let end_slot = (target_idx + 3).min(current_length - 1);
                                        for s in start_slot..=end_slot {
                                            if !chromatin_state[s] {
                                                chromatin_state[s] = true;
                                                wave_loci_modified += 1;
                                            }
                                        }
                                    } else if action_val < 18 {
                                        let start_slot = (target_idx.saturating_sub(3)).max(16);
                                        let end_slot = (target_idx + 3).min(current_length - 1);
                                        for s in start_slot..=end_slot {
                                            if chromatin_state[s] {
                                                chromatin_state[s] = false;
                                                wave_loci_modified += 1;
                                            }
                                        }
                                    } else {
                                        let shift_direction = if action_val % 2 == 0 { 5.0 } else { -5.0 };
                                        methylations[target_idx] += shift_direction;
                                        wave_methylations += 1;
                                    }
                                }
                            }
                        }
                    }
                    idx = (end_idx + 2).min(current_length);
                } else {
                    idx += 1;
                }
            } else {
                idx += 1;
            }
        }
        if wave_loci_modified > 0 || wave_methylations > 0 {
            epigenetic_logs.push(format!("Wave {}: Hox networks active. {} chromatin loops formed, {} methylations completed.", wave, wave_loci_modified, wave_methylations));
        }
    }

    // Loci 0-15 are hard-locked against somatic methylations during embryogenesis
    for idx in 0..16 {
        chromatin_state[idx] = true;
        methylations[idx] = 0.0;
    }

    // Epigenetic regulatory network parameters
    let m1_size_regulator = 0.55 + (get_methylated_val(2, &char_vals, &methylations) as f32 / 25.0) * 0.9;
    let m2_muscle_strength = 0.5 + (get_methylated_val(1, &char_vals, &methylations) as f32 / 25.0) * 1.0;
    let m3_sensory_acuity = 0.3 + (get_methylated_val(3, &char_vals, &methylations) as f32 / 25.0) * 1.2;
    let m4_neural_tau = 0.4 + (get_methylated_val(5, &char_vals, &methylations) as f32 / 25.0) * 1.2;
    let m5_asymmetry_level = 0.15 + (get_methylated_val(6, &char_vals, &methylations) as f32 / 25.0) * 1.35;

    let symmetry = if get_methylated_val(0, &char_vals, &methylations) < 13 { "vertical" } else { "quad" };

    let primary_color = HSLColor {
        h: ((get_methylated_val(1, &char_vals, &methylations) as f32 / 25.0) * 360.0).round(),
        s: (55.0 + (get_methylated_val(2, &char_vals, &methylations) as f32 / 25.0) * 45.0).round(),
        l: (35.0 + (get_methylated_val(3, &char_vals, &methylations) as f32 / 25.0) * 45.0).round(),
    };

    let secondary_color = HSLColor {
        h: ((get_methylated_val(4, &char_vals, &methylations) as f32 / 25.0) * 360.0).round(),
        s: (55.0 + (get_methylated_val(5, &char_vals, &methylations) as f32 / 25.0) * 45.0).round(),
        l: (35.0 + (get_methylated_val(6, &char_vals, &methylations) as f32 / 25.0) * 45.0).round(),
    };

    let body_seed = get_methylated_val(7, &char_vals, &methylations) as u32 * 4293 + get_methylated_val(8, &char_vals, &methylations) as u32 * 117;

    let mean_radius = (16.0f32).max((28.0f32).min((18.0 + (get_methylated_val(7, &char_vals, &methylations) as f32 / 25.0) * 18.0) * m1_size_regulator));
    let base_length = (90.0f32).max((200.0f32).min((90.0 + (get_methylated_val(8, &char_vals, &methylations) as f32 / 25.0) * 110.0) * m1_size_regulator));

    let mut amplitudes = Vec::with_capacity(4);
    for j in 0..4 {
        amplitudes.push((get_methylated_val(9 + j, &char_vals, &methylations) as f32 / 25.0) * 0.3 - 0.15);
    }

    let mut phases = Vec::with_capacity(4);
    for j in 0..4 {
        phases.push((get_methylated_val((3 + j) % 16, &char_vals, &methylations) as f32 / 25.0) * std::f32::consts::PI * 2.0);
    }

    let raw_stiffness = 0.15 + (get_methylated_val(12, &char_vals, &methylations) as f32 / 25.0) * 0.85;
    let stiffness = (0.15f32).max((1.0f32).min(raw_stiffness * m2_muscle_strength));

    let raw_spinal_curve = (get_methylated_val(15, &char_vals, &methylations) as f32 / 25.0) * 44.0 - 22.0;
    let spinal_curve = (-25.0f32).max((25.0f32).min(raw_spinal_curve * m5_asymmetry_level * (1.0 - stiffness * 0.25)));

    let spinal_curve_freq = 1.0 + (get_methylated_val(14, &char_vals, &methylations) / 12) as f32;
    let parapodia_amp = (get_methylated_val(13, &char_vals, &methylations) as f32 / 25.0) * 0.45;
    let parapodia_freq = 2.0 + (get_methylated_val(2, &char_vals, &methylations) / 12) as f32;
    let flattening_head = (get_methylated_val(1, &char_vals, &methylations) as f32 / 25.0) * 1.0 - 0.4;

    let pulse_speed = 0.0015 + (get_methylated_val(13, &char_vals, &methylations) as f32 / 25.0) * 0.0075;
    let wave_phase = (get_methylated_val(14, &char_vals, &methylations) as f32 / 25.0) * 1.6;
    let wiggle_amplitude = (get_methylated_val(15, &char_vals, &methylations) as f32 / 25.0) * 0.22;

    let stomach_capacity = 50.0 + get_methylated_val(13, &char_vals, &methylations) as f32 * 18.0;
    let hydraulic_pressure = 0.2 + (get_methylated_val(15, &char_vals, &methylations) as f32 / 25.0) * 0.8;

    let thermal_center = 10.0 + (get_methylated_val(5, &char_vals, &methylations) as f32 / 25.0) * 60.0;
    let thermal_width = 10.0 + (get_methylated_val(6, &char_vals, &methylations) as f32 / 25.0) * 30.0;
    let thermal_tolerance_min = -5.0f32.max((thermal_center - thermal_width / 2.0).round());
    let thermal_tolerance_max = 105.0f32.min((thermal_center + thermal_width / 2.0).round());

    // Calculate spinal peaks count
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

    // 2. Active organelles (sensory patches) de-compilation
    let mut organelles = Vec::new();
    let mut scan_idx = 16;

    while scan_idx < current_length.saturating_sub(1) {
        if chromatin_state[scan_idx] && chromatin_state[scan_idx + 1] {
            let char_a = clean_genome.as_bytes()[scan_idx];
            let char_b = clean_genome.as_bytes()[scan_idx + 1];
            let is_start = (char_a == b'S' && char_b == b'T') || (char_a == b'G' && char_b == b'O');

            if is_start {
                let gene_start = scan_idx;
                let payload_start = scan_idx + 2;
                let mut payload_end = None;
                let mut stop_found_at = None;

                for j in payload_start..(current_length.saturating_sub(1)) {
                    let c_a = clean_genome.as_bytes()[j];
                    let c_b = clean_genome.as_bytes()[j + 1];
                    let is_stop = (c_a == b'S' && c_b == b'P') || (c_a == b'E' && c_b == b'N');

                    if is_stop {
                        payload_end = Some(j);
                        stop_found_at = Some(j);
                        break;
                    }
                }

                let p_end = payload_end.unwrap_or(current_length);
                let p_stop = stop_found_at.unwrap_or(current_length - 1);
                let payload_length = p_end.saturating_sub(payload_start);

                if payload_length > 0 {
                    let get_methylated_payload_val = |offset: usize, fallback: usize| -> usize {
                        let j_locus = payload_start + offset;
                        if j_locus < p_end {
                            get_methylated_val(j_locus, &char_vals, &methylations)
                        } else {
                            fallback
                        }
                    };

                    let spectral_affinity = get_methylated_payload_val(0, 0) as f32 / 25.0;
                    let bandwidth = get_methylated_payload_val(1, 12) as f32 / 25.0;
                    let expression_style = get_methylated_payload_val(2, 10) as f32 / 25.0;
                    let scale = (0.35f32).max((1.8f32).min((0.35 + (get_methylated_payload_val(3, 12) as f32 / 25.0) * 1.5) * m3_sensory_acuity));
                    let spinal_pos = 0.05 + (get_methylated_payload_val(4, 0) as f32 / 25.0) * 0.9;
                    let angle = 10.0 + (get_methylated_payload_val(5, 12) as f32 / 25.0) * 160.0;
                    let hue_shift = ((get_methylated_payload_val(6, 5) as f32 / 25.0) * 360.0 - 180.0).round();

                    organelles.push(SensoryPatch {
                        spectral_affinity,
                        bandwidth,
                        expression_style,
                        scale,
                        spinal_pos,
                        angle,
                        hue_shift,
                        gene_start_index: gene_start,
                        gene_end_index: (current_length - 1).min(p_stop + 1),
                    });
                }
                scan_idx = (current_length).min(p_stop + 2);
            } else {
                scan_idx += 1;
            }
        } else {
            scan_idx += 1;
        }
    }

    let mut visus_score = 0.0;
    for patch in &organelles {
        let band_adjusted = 0.2 + patch.bandwidth * 0.8;
        let aff = patch.spectral_affinity;
        let w_light = (1.0 - (aff - 1.0).abs() / band_adjusted).max(0.0);
        visus_score += w_light * patch.scale * 22.0;
    }
    let _visus = visus_score.min(100.0).round();

    // Predator / Prey classification & Epistatic metabolic trade-off regulation
    let raw_carnivory = (get_methylated_val(11, &char_vals, &methylations) as f32 / 25.0).clamp(0.0, 1.0);
    let carnivory = (raw_carnivory * (1.0 - (stiffness * 0.25) * (1.0 - m3_sensory_acuity).max(0.0))).clamp(0.0, 1.0);
    let is_predator = carnivory >= 0.55;

    let mature_age = (300.0 + (get_methylated_val(14, &char_vals, &methylations) as f32 / 25.0) * 2400.0).round() as u32;
    let repro_threshold = 0.60 + (get_methylated_val(13, &char_vals, &methylations) as f32 / 25.0) * 0.35;
    let split_loss = 0.05 + (get_methylated_val(15, &char_vals, &methylations) as f32 / 25.0) * 0.35;

    // 3. CTRNN directed graph compilation (Brain)
    let k_count = organelles.len();
    let h_count = 2 + (get_methylated_val(16, &char_vals, &methylations) % 7);
    let total_nodes = k_count + 1 + 4 + h_count;

    let mut neurons = Vec::with_capacity(total_nodes);

    // A. Inputs (Sensors + Hunger)
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

    // B. Outputs
    let out_labels = ["Thrust (Fwd/Bwd)", "Bending (Left/Right)", "Biolum Flash", "Reserved"];
    for i in 0..4 {
        let bias_val = get_methylated_val((17 + i) % current_length, &char_vals, &methylations);
        let bias = (bias_val as f32 / 25.0) * 2.0 - 1.0;

        let tau_val = get_methylated_val((18 + i) % current_length, &char_vals, &methylations);
        let raw_tau = 0.2 + (tau_val as f32 / 25.0) * 1.8;
        let tau = (raw_tau * m4_neural_tau).clamp(0.1, 2.5);

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
        let bias_val = get_methylated_val((19 + i) % current_length, &char_vals, &methylations);
        let bias = (bias_val as f32 / 25.0) * 2.0 - 1.0;

        let tau_val = get_methylated_val((20 + i) % current_length, &char_vals, &methylations);
        let raw_tau = 0.2 + (tau_val as f32 / 25.0) * 1.8;
        let tau = (raw_tau * m4_neural_tau).clamp(0.1, 2.5);

        let act_val = get_methylated_val((21 + i) % current_length, &char_vals, &methylations);
        let activation_type = match act_val % 4 {
            1 => String::from("relu"),
            2 => String::from("sigmoid"),
            3 => String::from("sin"),
            _ => String::from("tanh"),
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

    // D. Synapses (NEAT-like topology)
    let mut synapses = Vec::new();
    let mut dna_pointer = 21;
    let synapses_count = (current_length - 21) / 4;

    for _ in 0..synapses_count {
        let raw_from = get_methylated_val(dna_pointer % current_length, &char_vals, &methylations);
        let raw_to = get_methylated_val((dna_pointer + 1) % current_length, &char_vals, &methylations);
        let raw_w = get_methylated_val((dna_pointer + 2) % current_length, &char_vals, &methylations);
        let raw_unused = get_methylated_val((dna_pointer + 3) % current_length, &char_vals, &methylations);
        dna_pointer += 4;

        let combined_from = if total_nodes > 26 { raw_from + raw_unused * 26 } else { raw_from };
        let from_node = combined_from % total_nodes;
        let to_node = (raw_to % (4 + h_count)) + (k_count + 1);
        let weight = (raw_w as f32 / 25.0) * 4.0 - 2.0;

        if !synapses.iter().any(|syn: &CTRNNSynapse| syn.from_node == from_node && syn.to_node == to_node) {
            synapses.push(CTRNNSynapse {
                from_node,
                to_node,
                weight,
            });
        }
    }

    let brain = BrainTopology { neurons, synapses };

    // 4. BMR and metabolic cost calculation
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
        let length = 100;
        let dna = generate_random_genome(length);
        assert_eq!(dna.len(), length);
        for c in dna.chars() {
            assert!(ALPHABET.contains(&(c as u8)));
        }
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
    }

    #[test]
    fn test_progenitor_parse_genome() {
        // High quality test input representing a well-formed progenitor genome
        let progenitor_dna = "HJKLABCDPQRS1234EFGHTRUSTANDBENDPROGENITORALIFEWELLFORMEDMEMBRANEFOURIERSEGMENTSHARMONICSWAVEPHASEPULSESTIFFNESS";
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
        assert!(hidden_count >= 2 && hidden_count <= 8);

        // Verify that BMR scaling values are deterministic and finite
        assert!(phenotype.basal_metabolic_rate.is_finite());
        assert!(phenotype.stomach_capacity > 0.0);
    }
}
