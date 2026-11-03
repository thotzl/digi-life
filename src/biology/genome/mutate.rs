//! Mutation operators (TCK-184) and mutation channels (TCK-181).

use serde::{Deserialize, Serialize};

use super::alphabet::{ALPHABET, GENOME_LENGTH_MAX, GENOME_LENGTH_MIN, char_to_value, matches_degenerate};
use super::promoters::{self, GENE_MOTIFS, GeneChannel, PROMOTERS, TERMINATOR, promoter_channel};
use super::scan::scan_genes;
use super::table::GeneTable;
use crate::biology::phenotype::body::{MAX_INDEL_RATE, read_evolution_rates};

/// Operator weights of `mutate_genome` (config `mutationOperators`, TCK-184). One call applies one operator,
/// drawn with these relative weights; the `EVO` gene scales the insertion/deletion weights by 2h (h = 0.5: as configured).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// Positions outside the protected promoter/terminator motifs (the loci of letter-level mutations). Besides every
/// motif match, the promoter letters of every gene `parse_genome` reads are protected (a motif scan can skip a
/// promoter that overlaps an earlier match), so letter-level mutations never change a gene's spelling (`GeneRef`).
pub(crate) fn unprotected_indices(genome: &str) -> Vec<usize> {
    let current_length = genome.len();

    // Protect crucial promoter/terminator motifs from mutations to prevent catastrophic brain-death / synapse loss
    let mut protected = vec![false; current_length];
    for motif in GENE_MOTIFS.iter() {
        let mut idx = 0;
        while idx <= current_length.saturating_sub(motif.len()) {
            let segment = &genome[idx..idx + motif.len()];
            if matches_degenerate(segment, motif) {
                for p in protected.iter_mut().skip(idx).take(motif.len()) {
                    *p = true;
                }
                idx += motif.len();
            } else {
                idx += 1;
            }
        }
    }

    for promoter in PROMOTERS.iter().map(|p| p.motif) {
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
    PROMOTERS
        .iter()
        .map(|p| scan_genes(genome, p.motif, TERMINATOR).len())
        .collect()
}

/// Terminated genes that overlap no other gene: (promoter index into `PROMOTERS`, start, end).
fn whole_genes(genome: &str) -> Vec<(usize, usize, usize)> {
    let all: Vec<(usize, usize, usize)> = PROMOTERS
        .iter()
        .enumerate()
        .flat_map(|(p, promoter)| {
            scan_genes(genome, promoter.motif, TERMINATOR)
                .into_iter()
                .map(move |g| (p, g.start, g.end))
        })
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
fn splice_gene<R: rand::Rng + ?Sized>(
    genome: &str,
    range: std::ops::Range<usize>,
    insert: &str,
    p: usize,
    delta: isize,
    rng: &mut R,
) -> Option<String> {
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
/// With `scope` (TCK-181) only loci of that channel are candidates: gene operators pick among the channel's genes,
/// letter operators among unprotected positions inside a gene of the channel and of no other channel (`None` if there
/// is none). Without `scope` the candidates and RNG draws are unchanged.
pub(crate) fn apply_mutation<R: rand::Rng + ?Sized>(
    genome: &str,
    kind: MutationKind,
    ops: &MutationOperators,
    scope: Option<GeneChannel>,
    rng: &mut R,
) -> Option<(String, usize)> {
    let current_length = genome.len();
    if current_length == 0 {
        return None;
    }
    let pick_gene = |genes: &[(usize, usize, usize)], rng: &mut R| genes[rng.gen_range(0..genes.len())];
    let in_scope = |p: usize| scope.is_none_or(|c| PROMOTERS[p].channel == c);
    match kind {
        MutationKind::GeneDuplication => {
            let genes: Vec<_> = whole_genes(genome)
                .into_iter()
                .filter(|&(p, _, _)| in_scope(p))
                .collect();
            if genes.is_empty() {
                return None;
            }
            let (p, start, end) = pick_gene(&genes, rng);
            splice_gene(genome, end..end, &genome[start..end], p, 1, rng).map(|g| (g, start))
        }
        MutationKind::GeneDeletion => {
            let counts = gene_counts(genome);
            let essential = |p: usize| PROMOTERS[p].essential;
            let genes: Vec<_> = whole_genes(genome)
                .into_iter()
                .filter(|&(p, _, _)| (!essential(p) || counts[p] > 1) && in_scope(p))
                .collect();
            if genes.is_empty() {
                return None;
            }
            let (p, start, end) = pick_gene(&genes, rng);
            splice_gene(genome, start..end, "", p, -1, rng).map(|g| (g, start))
        }
        _ => {
            let mut unprotected = unprotected_indices(genome);
            if let Some(channel) = scope {
                let spans = GeneTable::scan(genome).spans;
                unprotected.retain(|&i| {
                    let mut hits = spans.iter().filter(|s| s.start <= i && i < s.end).peekable();
                    hits.peek().is_some() && hits.all(|s| promoter_channel(&s.gene.promoter) == Some(channel))
                });
                if unprotected.is_empty() {
                    return None;
                }
            }
            let index = if unprotected.is_empty() {
                rng.gen_range(0..current_length)
            } else {
                unprotected[rng.gen_range(0..unprotected.len())]
            };
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
pub fn mutate_genome_with_kind<R: rand::Rng + ?Sized>(
    genome: &str,
    kind: MutationKind,
    ops: &MutationOperators,
    rng: &mut R,
) -> Option<String> {
    apply_mutation(genome, kind, ops, None, rng).map(|(g, _)| g)
}

/// One mutation (TCK-184): the operator is drawn by the `ops` weights, the insertion/deletion weights scaled by the
/// genome's `EVO` rates (2h, so h = 0.5 keeps the configured weight). An operator that is not applicable
/// (length bounds [128, 512], no suitable gene) falls back to creep. `None` only for an empty genome.
pub fn mutate_genome<R: rand::Rng + ?Sized>(genome: &str, ops: &MutationOperators, rng: &mut R) -> Option<Mutation> {
    mutate_genome_scoped(genome, ops, None, rng)
}

/// Attempts of `mutate_genome_in_channel` before it gives up (each attempt is one scoped `mutate_genome` call).
const CHANNEL_MUTATION_ATTEMPTS: usize = 8;

/// One mutation that touches only genes of `channel` (TCK-181, mutation channel mode "separate"): `mutate_genome`
/// with the loci restricted to the channel's genes (`apply_mutation` scope). A result that still touches the other
/// channel (`classify_mutation`, e.g. a letter change that forms a new promoter of the other channel) is redrawn, at
/// most `CHANNEL_MUTATION_ATTEMPTS` times; `None` if no attempt stays in the channel or no operator is applicable.
pub fn mutate_genome_in_channel<R: rand::Rng + ?Sized>(
    genome: &str,
    ops: &MutationOperators,
    channel: GeneChannel,
    rng: &mut R,
) -> Option<Mutation> {
    for _ in 0..CHANNEL_MUTATION_ATTEMPTS {
        let mutation = mutate_genome_scoped(genome, ops, Some(channel), rng)?;
        if classify_mutation(genome, &mutation).only(channel) {
            return Some(mutation);
        }
    }
    None
}

fn mutate_genome_scoped<R: rand::Rng + ?Sized>(
    genome: &str,
    ops: &MutationOperators,
    scope: Option<GeneChannel>,
    rng: &mut R,
) -> Option<Mutation> {
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
    if let Some((genome, index)) = apply_mutation(genome, kind, ops, scope, rng) {
        return Some(Mutation { genome, kind, index });
    }
    apply_mutation(genome, MutationKind::Creep, ops, scope, rng).map(|(genome, index)| Mutation {
        genome,
        kind: MutationKind::Creep,
        index,
    })
}

/// Channel set of a mutated child (union over its operator calls, TCK-181).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MutationChannel {
    Body,
    Brain,
    Both,
    /// Only loci outside every gene (junk/spacer), or no applicable operator
    Neutral,
}

/// Gene channels touched by one or more mutations (`classify_mutation`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MutationTouch {
    pub body: bool,
    pub brain: bool,
    /// A body touch hit an organ gene (`EYE`/`NOS`/`TAC`/`LUM`), which also carries thalamus -> motor weights
    pub organ: bool,
}

impl MutationTouch {
    pub fn union(self, other: Self) -> Self {
        Self {
            body: self.body || other.body,
            brain: self.brain || other.brain,
            organ: self.organ || other.organ,
        }
    }

    pub fn channel(self) -> MutationChannel {
        match (self.body, self.brain) {
            (true, true) => MutationChannel::Both,
            (true, false) => MutationChannel::Body,
            (false, true) => MutationChannel::Brain,
            (false, false) => MutationChannel::Neutral,
        }
    }

    /// Touches `channel` and not the other one.
    pub fn only(self, channel: GeneChannel) -> bool {
        match channel {
            GeneChannel::Body => self.body && !self.brain,
            GeneChannel::Brain => self.brain && !self.body,
        }
    }

    fn mark(&mut self, promoter: &str) {
        match promoter_channel(promoter) {
            Some(GeneChannel::Body) => {
                self.body = true;
                self.organ |= promoters::is_organ(promoter);
            }
            Some(GeneChannel::Brain) => self.brain = true,
            None => {}
        }
    }
}

/// Gene channels a mutation of `parent` touched (TCK-181), from the compiler's gene table (`GeneTable`, the spans
/// `parse_genome` reads): (1) every parent gene whose span (promoter .. terminator) contains the mutation locus
/// `mutation.index` (letter operators: the changed/inserted/removed base; gene operators: the original gene's first
/// base); (2) every gene identity (`GeneRef`) present in only one of parent and child (a gene formed, destroyed,
/// duplicated or renumbered by the mutation). A locus outside every gene with an unchanged gene set touches nothing
/// (neutral). Silent copies count by their promoter (a second copy of a body trait gene, `NEU`/`SY` beyond the
/// compiler's caps, `HOX` payload letters). Deterministic, draws no random numbers.
pub fn classify_mutation(parent: &str, mutation: &Mutation) -> MutationTouch {
    let before = GeneTable::scan(parent).spans;
    let after = GeneTable::scan(&mutation.genome).spans;
    let mut touch = MutationTouch::default();
    for span in before
        .iter()
        .filter(|s| s.start <= mutation.index && mutation.index < s.end)
    {
        touch.mark(&span.gene.promoter);
    }
    for span in before.iter().filter(|b| !after.iter().any(|a| a.gene == b.gene)) {
        touch.mark(&span.gene.promoter);
    }
    for span in after.iter().filter(|a| !before.iter().any(|b| b.gene == a.gene)) {
        touch.mark(&span.gene.promoter);
    }
    touch
}
