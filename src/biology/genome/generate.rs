//! Structured random genomes (TCK-184): valid genes in random order with intergenic filler.

use serde::{Deserialize, Serialize};

use super::alphabet::{matches_degenerate, ALPHABET, GENOME_LENGTH_MAX, GENOME_LENGTH_MIN};
use super::payload::get_payload_linear_value_offset;
use super::promoters::{self, GeneRole, GENE_MOTIFS, TERMINATOR};
use crate::biology::organs::{OrganKind, AFFINITY_LOCUS};

/// Composition of `generate_structured_genome` (config `genomeComposition`, TCK-184).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        .filter(|&c| OrganKind::classify(get_payload_linear_value_offset(&(c as char).to_string(), AFFINITY_LOCUS), 0.5) == kind)
        .collect()
}

/// Genes of a structured genome in genome order: promoter and, for a food organ, the OrganKind its payload must give.
/// All body genes and one `OUT` gene, plus organs, `HOX`, `NEU` and `SY` genes as `composition` draws them.
pub(crate) fn structured_gene_plan<R: rand::Rng + ?Sized>(c: &GenomeComposition, rng: &mut R) -> Vec<(&'static str, Option<OrganKind>)> {
    use rand::seq::SliceRandom;
    let count = |rng: &mut R, min: usize, max: usize| rng.gen_range(min..=max.max(min));
    let organ_promoters: Vec<&'static str> = promoters::with_role(GeneRole::Organ).collect();
    let mut genes: Vec<(&str, Option<OrganKind>)> = promoters::with_role(GeneRole::Body).map(|p| (p, None)).collect();
    genes.push((promoters::motif_of(GeneRole::BrainBase), None));
    for organ in 0..count(rng, c.organ_count_min, c.organ_count_max) {
        if organ == 0 && rng.gen_bool(c.p_food_organ.clamp(0.0, 1.0) as f64) {
            let eye = rng.gen_bool(c.food_organ_eye_share.clamp(0.0, 1.0) as f64);
            genes.push(if eye { ("EYE", Some(OrganKind::Eye)) } else { ("NOS", Some(OrganKind::Nose)) });
        } else {
            genes.push((organ_promoters[rng.gen_range(0..organ_promoters.len())], None));
        }
    }
    if rng.gen_bool(c.p_hox.clamp(0.0, 1.0) as f64) {
        genes.push((promoters::motif_of(GeneRole::Hox), None));
    }
    genes.extend(std::iter::repeat((promoters::motif_of(GeneRole::Hidden), None)).take(count(rng, c.neu_count_min, c.neu_count_max)));
    genes.extend(std::iter::repeat((promoters::motif_of(GeneRole::Synapse), None)).take(count(rng, c.sy_count_min, c.sy_count_max)));
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
        let affinity = kind.map(|k| (AFFINITY_LOCUS % payload_lengths[g], affinity_letters(k)));
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
