//! Gene table: every expressed gene of a genome with its stable identity (`GeneRef`) and span.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::payload::Payload;
use super::promoters::{PROMOTERS, TERMINATOR};
use super::scan::{ScannedGene, scan_genes};

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

/// Payload of the first gene of `promoter` without building the whole table; empty when the gene is missing.
pub fn first_payload<'a>(genome: &'a str, promoter: &str) -> Payload<'a> {
    Payload(
        scan_genes(genome, promoter, TERMINATOR)
            .first()
            .map(|g| g.payload)
            .unwrap_or(""),
    )
}

/// Expressed genes of a genome: per promoter in genome order, plus all spans sorted by position (owner ids).
pub struct GeneTable<'a> {
    by_promoter: Vec<(&'static str, Vec<ScannedGene<'a>>)>,
    pub spans: Vec<GeneSpan>,
}

impl<'a> GeneTable<'a> {
    pub fn scan(genome: &'a str) -> Self {
        let by_promoter: Vec<(&'static str, Vec<ScannedGene<'a>>)> = PROMOTERS
            .iter()
            .map(|p| (p.motif, scan_genes(genome, p.motif, TERMINATOR)))
            .collect();
        let mut spans: Vec<GeneSpan> = by_promoter
            .iter()
            .flat_map(|(p, genes)| {
                genes.iter().map(move |g| GeneSpan {
                    gene: GeneRef {
                        promoter: p.to_string(),
                        spelling: genome[g.start..g.start + p.len()].to_string(),
                        occurrence: 0,
                    },
                    start: g.start,
                    end: g.end,
                })
            })
            .collect();
        spans.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| a.gene.promoter.cmp(&b.gene.promoter))
        });
        for i in 0..spans.len() {
            let same =
                |s: &GeneSpan| s.gene.promoter == spans[i].gene.promoter && s.gene.spelling == spans[i].gene.spelling;
            spans[i].gene.occurrence = spans[..i].iter().filter(|s| same(s)).count();
        }
        Self { by_promoter, spans }
    }

    pub fn genes(&self, promoter: &str) -> &[ScannedGene<'a>] {
        self.by_promoter
            .iter()
            .find(|(p, _)| *p == promoter)
            .map(|(_, g)| &g[..])
            .unwrap_or(&[])
    }

    /// Payload of the first gene of `promoter`; empty (h = 0.5 everywhere) when the gene is missing.
    pub fn first_payload(&self, promoter: &str) -> Payload<'a> {
        Payload(self.genes(promoter).first().map(|g| g.payload).unwrap_or(""))
    }

    /// Owner id (index into `spans`) of the gene of `promoter` that starts at `start`.
    pub fn owner(&self, promoter: &str, start: usize) -> usize {
        self.spans
            .iter()
            .position(|s| s.start == start && s.gene.promoter == promoter)
            .expect("scanned gene is in the gene table")
    }

    /// Stable identity of the gene with owner id `owner`.
    pub fn gene_ref(&self, owner: usize) -> GeneRef {
        self.spans[owner].gene.clone()
    }
}
