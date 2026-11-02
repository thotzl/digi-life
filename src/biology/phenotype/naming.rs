//! Procedural Latin name: chamber prefix, symmetry, dominant organ kind (or predator) and a strain id from the genome.

use super::Symmetry;
use crate::biology::genome::alphabet::{char_to_value, ALPHABET};
use crate::biology::organs::Organ;

/// Carnivory from which the name says "raptor" (display only; `is_predator` uses 0.50).
// TCK-202: naming and `is_predator` use different predator thresholds
const RAPTOR_CARNIVORY: f32 = 0.55;

pub fn latin_name(genome: &str, symmetry: Symmetry, chambers: usize, organs: &[Organ], carnivory: f32) -> String {
    let seg_prefixes = ["Monoplasma", "Biplasma", "Triplasma", "Tetraplasma", "Pentaplasma"];
    let prefix = seg_prefixes[(chambers.saturating_sub(1)).min(4)];
    let mid_sym = match symmetry {
        Symmetry::Vertical => "bilateralis",
        Symmetry::Quad => "tetramerum",
    };

    let suffix = if carnivory >= RAPTOR_CARNIVORY {
        "raptor"
    } else {
        dominant_suffix(organs).unwrap_or("lēvis")
    };

    // Strain id from the first, middle and last base
    let first_loci = char_to_value(genome.chars().next().unwrap_or('A'));
    let last_loci = char_to_value(genome.chars().last().unwrap_or('A'));
    let mid_loci = char_to_value(genome.chars().nth(genome.len() / 2).unwrap_or('A'));
    let hash_char1 = ALPHABET[(first_loci + last_loci) % 26] as char;
    let hash_char2 = ALPHABET[(mid_loci + last_loci) % 26] as char;
    let hash_num = (first_loci * mid_loci + last_loci) % 10;

    format!("{} {} {} (Str. {}{}-{})", prefix, mid_sym, suffix, hash_char1, hash_char2, hash_num)
}

/// Latin suffix of the most frequent suffix group of the organs (`OrganSpec::latin_suffix`); ties go to the lower
/// `latin_rank`. `None` without organs.
fn dominant_suffix(organs: &[Organ]) -> Option<&'static str> {
    let mut groups: Vec<(u8, &'static str, usize)> = Vec::new();
    for organ in organs {
        let spec = organ.kind.spec();
        match groups.iter_mut().find(|g| g.0 == spec.latin_rank) {
            Some(g) => g.2 += 1,
            None => groups.push((spec.latin_rank, spec.latin_suffix, 1)),
        }
    }
    groups.sort_by_key(|g| g.0);
    let mut dominant = groups.first()?;
    for g in &groups[1..] {
        if g.2 > dominant.2 {
            dominant = g;
        }
    }
    Some(dominant.1)
}
