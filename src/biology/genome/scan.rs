//! Gene scanning: promoter .. terminator spans of one promoter type.

use super::alphabet::matches_degenerate;

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
