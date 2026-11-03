//! The genome: alphabet, promoter registry, gene scanning and table, payload reader, random generation and
//! mutation. The phenotype compiler (`biology::phenotype::parse_genome`) reads genomes through these modules.
//! Structure: `wiki/biology/genome-compiler.md`.

pub mod alphabet;
pub mod generate;
pub mod mutate;
pub mod payload;
pub mod promoters;
pub mod scan;
pub mod table;

#[cfg(test)]
pub(crate) mod tests;

use serde::{Deserialize, Serialize};

pub use alphabet::{
    ALPHABET, GENOME_LENGTH_MAX, GENOME_LENGTH_MIN, char_to_value, generate_random_genome, get_bio_class,
    get_complementary_char, get_complementary_string, matches_degenerate, normalize_genome, value_to_char,
};
pub use generate::{GenomeComposition, generate_structured_genome};
pub use mutate::{
    Mutation, MutationChannel, MutationKind, MutationOperators, MutationTouch, classify_mutation, mutate_genome,
    mutate_genome_in_channel, mutate_genome_with_kind,
};
pub use payload::{Payload, get_payload_linear_value_offset};
pub use promoters::{GeneChannel, GeneRole, PROMOTERS, PromoterSpec, TERMINATOR, promoter_channel};
pub use scan::{ScannedGene, extract_raw_gene_payloads, scan_genes};
pub use table::{GeneRef, GeneSpan, GeneTable};

/// Wildtype progenitor genome (trainer slot plan placeholder for random immigrants, tests).
pub const WILDTYPE_GENOME: &str = "COLOOOENSTFZENPULKKKENSIZMLENWAVABCDEFGHENSYMAENSTMHLENEYEABCDEFGENNOSHIJKLMNENNEUABCDEFENSYNABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHEN";

/// Config section `genome`: random genome composition, mutation operators and Lamarckian inheritance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenomeConfig {
    /// Probability per plastic synapse with a homologue that a child takes the parent's learned offset (TCK-152;
    /// `creature_factory::inherit_learned_weights`, Ocean mitosis and trainer breeding)
    pub lamarckian_assimilation_chance: f32,
    /// Random genomes of trainer generation 1, immigrants and Ocean founders (TCK-184)
    pub genome_composition: GenomeComposition,
    /// Operator weights of `mutate_genome` (TCK-184)
    pub mutation_operators: MutationOperators,
}

/// Writes a generated data file next to the ts-rs bindings (`bindings/<name>`); called by the registry export tests.
#[cfg(test)]
pub(crate) fn write_binding(name: &str, content: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("bindings")
        .join(name);
    if std::fs::read_to_string(&path).ok().as_deref() != Some(content) {
        std::fs::write(&path, content).expect("write binding");
    }
}
