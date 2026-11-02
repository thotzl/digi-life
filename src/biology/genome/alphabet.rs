//! Genome alphabet: letters, biochemical classes, degenerate motif matching, complementary strand, normalisation.

use super::promoters;

pub const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Lengths `mutate_genome` keeps (and `normalize_genome` compiles).
pub const GENOME_LENGTH_MIN: usize = 128;
pub const GENOME_LENGTH_MAX: usize = 512;

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

    // Full-wobble promoters (`Wobble::Full`: "NEU", "SY") and one-letter motifs use full biochemical group matching
    if promoters::full_wobble(classic_promoter) {
        return s.iter().zip(p).all(same_class);
    }

    // Golden-Middle matching: first character exact, the remaining characters degenerate (wobble)
    s[0] == p[0] && s[1..].iter().zip(&p[1..]).all(same_class)
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

/// Uppercases and pads/truncates a genome to the compiled length range [128, 512].
pub fn normalize_genome(genome: &str) -> String {
    let current_length = genome.len().clamp(GENOME_LENGTH_MIN, GENOME_LENGTH_MAX);
    let mut clean_genome = genome.to_ascii_uppercase().chars().take(current_length).collect::<String>();
    while clean_genome.len() < current_length {
        clean_genome.push('A');
    }
    clean_genome
}
