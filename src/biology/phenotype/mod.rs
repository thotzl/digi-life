//! Phenotype compiler: genome -> `CreaturePhenotype` (body traits, organs, brain, name).
//! Structure: `wiki/biology/genome-compiler.md`.

pub mod body;
pub mod naming;

#[cfg(test)]
pub(crate) mod tests;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::biology::brain::compile::compile_brain;
use crate::biology::brain::BrainTopology;
use crate::biology::genome::alphabet::{get_complementary_string, normalize_genome};
use crate::biology::genome::table::{GeneSpan, GeneTable};
use crate::biology::organs::{decode_organs, Organ};
use crate::shared::brain::BrainConfig;

/// Body symmetry (TCK-155: typed instead of a string).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
#[ts(export)]
pub enum Symmetry {
    #[default]
    Vertical,
    Quad,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HSLColor {
    pub h: f32, // 0 - 360
    pub s: f32, // 0 - 100
    pub l: f32, // 0 - 100
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CreaturePhenotype {
    pub symmetry: Symmetry,
    #[serde(rename = "primaryColor")]
    #[ts(rename = "primaryColor")]
    pub primary_color: HSLColor,
    #[serde(rename = "secondaryColor")]
    #[ts(rename = "secondaryColor")]
    pub secondary_color: HSLColor,
    #[serde(rename = "bodySeed")]
    #[ts(rename = "bodySeed")]
    pub body_seed: u32,
    #[serde(rename = "spinalHarmonics")]
    #[ts(rename = "spinalHarmonics")]
    pub spinal_harmonics: SpinalHarmonics,
    #[serde(rename = "emergentChambersCount")]
    #[ts(rename = "emergentChambersCount")]
    pub emergent_chambers_count: usize,
    pub organelles: Vec<Organ>,
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

    #[serde(rename = "stomachCapacity")]
    #[ts(rename = "stomachCapacity")]
    pub stomach_capacity: f32,
    #[serde(rename = "thermalToleranceMin")]
    #[ts(rename = "thermalToleranceMin")]
    pub thermal_tolerance_min: f32,
    #[serde(rename = "thermalToleranceMax")]
    #[ts(rename = "thermalToleranceMax")]
    pub thermal_tolerance_max: f32,

    pub methylations: Vec<f32>,

    #[serde(rename = "antisenseStrand")]
    #[ts(rename = "antisenseStrand")]
    pub antisense_strand: String,
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

/// Genotype -> phenotype. Every trait is read from its own gene only (TCK-179, trait locality); a missing gene reads
/// the empty payload (every value at the middle of its range, h = 0.5), never other genes or active DNA.
pub fn parse_genome(genome: &str, antisense_input: Option<&str>, parent_methylations: Option<&[f32]>, brain_config: &BrainConfig) -> CreaturePhenotype {
    let clean_genome = normalize_genome(genome);
    let current_length = clean_genome.len();

    let mut antisense_strand = match antisense_input {
        Some(anti) => anti.to_ascii_uppercase().chars().take(current_length).collect::<String>(),
        None => get_complementary_string(&clean_genome),
    };
    while antisense_strand.len() < current_length {
        antisense_strand.push('Z');
    }

    // Lamarckian methylations of the parent cell when they fit the genome
    let methylations = match parent_methylations {
        Some(m) if m.len() == current_length => m.to_vec(),
        _ => vec![0.0; current_length],
    };

    let gene_table = GeneTable::scan(&clean_genome);
    let body = body::decode_body(&gene_table);
    let (organelles, organ_genes) = decode_organs(&gene_table);
    let brain = compile_brain(&gene_table, &organelles, &organ_genes, brain_config);

    let spinal_harmonics = SpinalHarmonics {
        base_length: body.base_length,
        mean_radius: body.mean_radius,
        amplitudes: body.amplitudes,
        phases: body.phases,
        spinal_curve: body.spinal_curve,
        spinal_curve_freq: body.spinal_curve_freq,
        parapodia_amp: body.parapodia_amp,
        parapodia_freq: body.parapodia_freq,
        flattening_head: body.flattening_head,
    };
    let emergent_chambers_count = body::chamber_count(&spinal_harmonics);
    let latin_name = naming::latin_name(&clean_genome, body.symmetry, emergent_chambers_count, &organelles, body.carnivory);

    CreaturePhenotype {
        symmetry: body.symmetry,
        primary_color: body.primary_color,
        secondary_color: body.secondary_color,
        body_seed: body.body_seed,
        spinal_harmonics,
        emergent_chambers_count,
        organelles,
        genes: gene_table.spans,
        pulse_speed: body.pulse_speed,
        wave_phase: body.wave_phase,
        wiggle_amplitude: body.wiggle_amplitude,
        stiffness: body.stiffness,
        mature_age: body.mature_age,
        repro_threshold: body.repro_threshold,
        split_loss: body.split_loss,
        brain,
        carnivory: body.carnivory,
        is_predator: body.is_predator,
        latin_name,
        stomach_capacity: body.stomach_capacity,
        thermal_tolerance_min: body.thermal_tolerance_min,
        thermal_tolerance_max: body.thermal_tolerance_max,
        methylations,
        antisense_strand,
        insertion_rate: body.insertion_rate,
        deletion_rate: body.deletion_rate,
        genome_string: clean_genome,
    }
}
