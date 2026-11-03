//! Tests of generation, mutation and mutation channels.

use super::generate::structured_gene_plan;
use super::mutate::{apply_mutation, unprotected_indices};
use super::promoters::{self, GeneRole};
use super::*;
use crate::biology::organs::OrganKind;
use crate::biology::phenotype::parse_genome;
use crate::biology::phenotype::tests::golden_genomes;

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
    let length = 128;
    let dna = generate_random_genome(length, &mut crate::shared::rng::seeded(1));
    assert_eq!(dna.len(), length); // Sequence-based base_dna length is dynamic and matches requested length exactly
    for c in dna.chars() {
        assert!(ALPHABET.contains(&(c as u8)));
    }
}

#[test]
fn test_biochem_groups_and_degenerate_matching() {
    // Alpha class: A, E, I, O, U
    assert_eq!(get_bio_class('A'), 1);
    assert_eq!(get_bio_class('E'), 1);
    // Beta class: Y, W, F, H, K, R
    assert_eq!(get_bio_class('Y'), 2);
    assert_eq!(get_bio_class('W'), 2);
    // Gamma class: L, M, V, P, T
    assert_eq!(get_bio_class('L'), 3);
    // Delta class: D, N, Q, S, C
    assert_eq!(get_bio_class('N'), 4);
    assert_eq!(get_bio_class('S'), 4);

    // Degenerate matching checks (Golden-Middle: 1st exact, rest wobble)
    assert!(matches_degenerate("EYE", "EYE"));
    assert!(matches_degenerate("EYO", "EYE")); // matches exact 'E' + wobble Beta 'Y' + wobble Alpha 'O'
    assert!(matches_degenerate("EWA", "EYE")); // matches exact 'E' + wobble Beta 'W' + wobble Alpha 'A'
    assert!(!matches_degenerate("OWO", "EYE")); // different first character ('O' != 'E')
    assert!(!matches_degenerate("ABC", "EYE")); // completely different

    // Brain-related promoters ("NEU", "SY") must support full-wobble group matching!
    assert!(matches_degenerate("NEU", "NEU"));
    assert!(matches_degenerate("DAA", "NEU")); // D is Delta (like N), A is Alpha (like E), A is Alpha (like U)
    assert!(matches_degenerate("CAA", "NEU")); // C is Delta, A is Alpha, A is Alpha
    assert!(!matches_degenerate("DAA", "EYE")); // different promoter, should not match
}

#[test]
fn test_extract_raw_gene_payloads() {
    // Standard matching (which is backward compatible)
    let genome = "STFAENCOLABSTPEYEAABEN";
    let stf_payloads = extract_raw_gene_payloads(genome, "STF", "EN");
    assert_eq!(stf_payloads.len(), 1);
    assert_eq!(stf_payloads[0], "A");

    let col_payloads = extract_raw_gene_payloads(genome, "COL", "STP");
    assert_eq!(col_payloads.len(), 1);
    assert_eq!(col_payloads[0], "AB");

    // Degenerate matching: "EYO" as EYE, "ES" as EN
    let degenerate_genome = "EYOAABES"; // EYO matches EYE, ES matches EN
    let eye_payloads = extract_raw_gene_payloads(degenerate_genome, "EYE", "EN");
    assert_eq!(eye_payloads.len(), 1);
    assert_eq!(eye_payloads[0], "AAB");

    // Test Point 1: Implicit end-of-genome stop fallback (no stop_motif present)
    let genome_no_stop = "EYEAA";
    let eye_payloads_fallback = extract_raw_gene_payloads(genome_no_stop, "EYE", "EN");
    assert_eq!(eye_payloads_fallback.len(), 1);
    assert_eq!(eye_payloads_fallback[0], "AA"); // Read successfully until the end of the strand!

    // Test Point 2: Isolated, non-greedy scanning (no merging of EYE...EN...EYE...EN)
    let genome_multi = "EYEAAENEYECCDEN";
    let multi_payloads = extract_raw_gene_payloads(genome_multi, "EYE", "EN");
    assert_eq!(multi_payloads.len(), 2);
    assert_eq!(multi_payloads[0], "AA");
    assert_eq!(multi_payloads[1], "CCD"); // Two distinct, isolated organs successfully translated!
}

pub(crate) fn ops() -> &'static MutationOperators {
    &crate::config::test_config().genome.mutation_operators
}

pub(crate) fn composition() -> &'static GenomeComposition {
    &crate::config::test_config().genome.genome_composition
}

fn promoter_counts(genome: &str) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for span in parse_genome(genome, None, None, &crate::config::test_config().brain).genes {
        *counts.entry(span.gene.promoter).or_insert(0) += 1;
    }
    counts
}

#[test]
fn test_mutation() {
    // A 200-base genome is in the safe range for every letter-level operator
    let mut rng = crate::shared::rng::seeded(2);
    let original = generate_random_genome(200, &mut rng);
    for (kind, delta) in [
        (MutationKind::Creep, 0),
        (MutationKind::Point, 0),
        (MutationKind::Insertion, 1),
        (MutationKind::Deletion, -1),
    ] {
        let (mutated, idx) = apply_mutation(&original, kind, ops(), None, &mut rng).expect("applicable at 200 bases");
        assert_ne!(mutated, original);
        assert_eq!(mutated.len() as isize, original.len() as isize + delta, "{:?}", kind);
        match kind {
            MutationKind::Deletion => assert_eq!(
                format!("{}{}{}", &mutated[..idx], &original[idx..idx + 1], &mutated[idx..]),
                original
            ),
            MutationKind::Insertion => assert_eq!(format!("{}{}", &mutated[..idx], &mutated[idx + 1..]), original),
            _ => assert_eq!(
                (&mutated[..idx], &mutated[idx + 1..]),
                (&original[..idx], &original[idx + 1..])
            ),
        }
    }
    let mutation = mutate_genome(&original, ops(), &mut rng).expect("non-empty genome");
    assert_ne!(mutation.genome, original);

    // Letter-level operators only hit unprotected loci: in "COLABCEN" the payload ABC (3..=5)
    let genome_with_promoter = "COLABCEN";
    for kind in [MutationKind::Creep, MutationKind::Point, MutationKind::Insertion] {
        for _ in 0..100 {
            let (mutated, idx) = apply_mutation(genome_with_promoter, kind, ops(), None, &mut rng).expect("applicable");
            assert!((3..=5).contains(&idx), "{:?} hit protected index {}", kind, idx);
            assert!(
                matches_degenerate(&mutated[0..3], "COL") && matches_degenerate(&mutated[mutated.len() - 2..], "EN"),
                "Promoters and terminators must never be broken!"
            );
        }
    }
}

#[test]
fn test_structured_genome_parses_to_exactly_the_planned_genes() {
    let c = composition();
    let mut food_organ = 0;
    let n = 2000;
    for seed in 0..n {
        let plan = structured_gene_plan(c, &mut crate::shared::rng::seeded(seed));
        let genome = generate_structured_genome(c, &mut crate::shared::rng::seeded(seed));
        assert!(
            (GENOME_LENGTH_MIN..=GENOME_LENGTH_MAX).contains(&genome.len()),
            "length {}",
            genome.len()
        );

        let mut planned = std::collections::BTreeMap::new();
        for (promoter, _) in &plan {
            *planned.entry(promoter.to_string()).or_insert(0usize) += 1;
        }
        assert_eq!(promoter_counts(&genome), planned, "genome {}", genome);
        for p in PROMOTERS.iter().filter(|s| s.essential).map(|s| s.motif) {
            assert_eq!(planned.get(p), Some(&1), "{} missing", p);
        }
        let organs = plan.iter().filter(|(p, _)| promoters::is_organ(p)).count();
        assert!((c.organ_count_min..=c.organ_count_max).contains(&organs));

        let phenotype = parse_genome(&genome, None, None, &crate::config::test_config().brain);
        // Distinct promoter spellings: every gene has occurrence 0 (stable identity, TCK-179)
        assert!(
            phenotype.genes.iter().all(|s| s.gene.occurrence == 0),
            "repeated spelling: {}",
            genome
        );
        for (promoter, kind) in plan.iter().filter_map(|(p, k)| k.map(|k| (p, k))) {
            assert!(
                phenotype.organelles.iter().any(|o| o.kind == kind),
                "{} planned as {:?}: {}",
                promoter,
                kind,
                genome
            );
        }
        if phenotype
            .organelles
            .iter()
            .any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose))
        {
            food_organ += 1;
        }
    }
    // p(food organ) = pFoodOrgan plus random organs that happen to be eyes or noses
    assert!(
        food_organ as f32 / n as f32 >= c.p_food_organ - 0.03,
        "food organ share {}",
        food_organ as f32 / n as f32
    );
}

#[test]
fn test_gene_duplication_adds_exactly_one_gene_of_its_promoter() {
    let mut rng = crate::shared::rng::seeded(11);
    let mut applied = 0;
    for seed in 0..300 {
        let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
        let before = promoter_counts(&genome);
        let Some((child, start)) = apply_mutation(&genome, MutationKind::GeneDuplication, ops(), None, &mut rng) else {
            continue;
        };
        applied += 1;
        let promoter = PROMOTERS
            .iter()
            .map(|s| s.motif)
            .find(|p| matches_degenerate(&genome[start..start + p.len()], p))
            .expect("gene start");
        let mut expected = before.clone();
        *expected.get_mut(promoter).unwrap() += 1;
        assert_eq!(
            promoter_counts(&child),
            expected,
            "duplicating {} at {}",
            promoter,
            start
        );
    }
    assert!(applied > 250, "only {} duplications applied", applied);
}

#[test]
fn test_gene_deletion_removes_one_gene_and_never_an_essential_one() {
    let mut rng = crate::shared::rng::seeded(12);
    for seed in 0..100 {
        let mut genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
        // Duplicate a few genes first so body genes have deletable second copies
        for _ in 0..3 {
            if let Some((g, _)) = apply_mutation(&genome, MutationKind::GeneDuplication, ops(), None, &mut rng) {
                genome = g;
            }
        }
        while let Some((child, start)) = apply_mutation(&genome, MutationKind::GeneDeletion, ops(), None, &mut rng) {
            let before = promoter_counts(&genome);
            let after = promoter_counts(&child);
            let changed: Vec<_> = before
                .iter()
                .filter(|(p, n)| after.get(*p).copied().unwrap_or(0) != **n)
                .collect();
            assert_eq!(changed.len(), 1, "deletion at {} changed {:?}", start, changed);
            assert_eq!(after.get(changed[0].0).copied().unwrap_or(0) + 1, *changed[0].1);
            genome = child;
        }
        let counts = promoter_counts(&genome);
        for p in PROMOTERS.iter().filter(|s| s.essential).map(|s| s.motif) {
            assert!(counts.get(p).copied().unwrap_or(0) >= 1, "essential {} deleted", p);
        }
        assert!(genome.len() >= GENOME_LENGTH_MIN);
    }
}

#[test]
fn test_creep_changes_a_tape_weight_by_at_most_the_step() {
    let mut rng = crate::shared::rng::seeded(13);
    let step = ops().creep_max_step;
    let max_delta = step as f32 / 25.0 + 1e-6;
    let mut checked = 0;
    for seed in 0..300 {
        let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
        let parent = parse_genome(&genome, None, None, &crate::config::test_config().brain);
        for _ in 0..5 {
            let (child_genome, idx) =
                apply_mutation(&genome, MutationKind::Creep, ops(), None, &mut rng).expect("creep applies");
            let letter_delta = (char_to_value(child_genome.as_bytes()[idx] as char) as isize
                - char_to_value(genome.as_bytes()[idx] as char) as isize)
                .unsigned_abs();
            assert!((1..=step).contains(&letter_delta), "letter step {}", letter_delta);

            let child = parse_genome(&child_genome, None, None, &crate::config::test_config().brain);
            if child.genes != parent.genes {
                continue; // the creep formed a promoter or terminator
            }
            let owner = parent.genes.iter().position(|g| (g.start..g.end).contains(&idx));
            let tape_owner = owner.filter(|&o| parent.genes[o].gene.promoter != "SY");
            for (p, c) in parent
                .brain
                .synapses
                .iter()
                .filter(|s| Some(s.owner) == tape_owner)
                .filter_map(|p| child.brain.synapses.iter().find(|c| c.key == p.key).map(|c| (p, c)))
            {
                assert!(
                    (p.weight - c.weight).abs() <= max_delta,
                    "weight {} -> {}",
                    p.weight,
                    c.weight
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 1000, "only {} homologous tape weights checked", checked);
}

#[test]
fn test_structured_genome_and_mutation_are_deterministic() {
    let run = |seed: u64| {
        let mut rng = crate::shared::rng::seeded(seed);
        let mut genome = generate_structured_genome(composition(), &mut rng);
        let mut kinds = Vec::new();
        for _ in 0..300 {
            let m = mutate_genome(&genome, ops(), &mut rng).unwrap();
            kinds.push(m.kind);
            genome = m.genome;
        }
        (genome, kinds)
    };
    assert_eq!(run(21), run(21));
    assert_ne!(run(21).0, run(22).0);
    // Every operator occurs with the default weights
    let kinds = run(21).1;
    assert!(MutationKind::ALL.iter().all(|k| kinds.contains(k)), "{:?}", kinds);
}

#[test]
fn test_mutate_genome_keeps_length_bounds() {
    let mut rng = crate::shared::rng::seeded(5);
    let starts = [
        generate_random_genome(128, &mut rng),
        generate_random_genome(512, &mut rng),
        generate_structured_genome(composition(), &mut rng),
    ];
    for mut genome in starts {
        for _ in 0..10_000 {
            genome = mutate_genome(&genome, ops(), &mut rng)
                .expect("non-empty genome")
                .genome;
            assert!((128..=512).contains(&genome.len()), "length {}", genome.len());
        }
    }
}

#[test]
fn test_letter_mutations_never_change_a_promoter_spelling() {
    let mut rng = crate::shared::rng::seeded(14);
    let mut checked = 0;
    for genome in golden_genomes()
        .into_iter()
        .map(|g| normalize_genome(&g))
        .chain((0..100).map(|seed| generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed))))
    {
        let promoters: Vec<std::ops::Range<usize>> =
            parse_genome(&genome, None, None, &crate::config::test_config().brain)
                .genes
                .iter()
                .map(|s| s.start..s.start + s.gene.promoter.len())
                .collect();
        for kind in [
            MutationKind::Creep,
            MutationKind::Point,
            MutationKind::Insertion,
            MutationKind::Deletion,
        ] {
            for _ in 0..20 {
                let Some((_, idx)) = apply_mutation(&genome, kind, ops(), None, &mut rng) else {
                    continue;
                };
                assert!(
                    !promoters.iter().any(|p| p.contains(&idx)),
                    "{:?} at {} hit a promoter: {}",
                    kind,
                    idx,
                    genome
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 20_000, "{} mutations checked", checked);
}

/// A copy of `genome` with the letter at `index` changed such that the gene table keeps every gene identity.
fn letter_change(genome: &str, index: usize) -> Mutation {
    let refs = |g: &str| GeneTable::scan(g).spans.into_iter().map(|s| s.gene).collect::<Vec<_>>();
    let original = refs(genome);
    for &c in ALPHABET.iter().filter(|&&c| c != genome.as_bytes()[index]) {
        let mut child = genome.to_string();
        child.replace_range(index..index + 1, &(c as char).to_string());
        if refs(&child) == original {
            return Mutation {
                genome: child,
                kind: MutationKind::Point,
                index,
            };
        }
    }
    panic!("no clean letter change at {}", index);
}

#[test]
fn test_classify_mutation_by_gene_channel() {
    let mut checked = [0usize; 4];
    for seed in 0..40 {
        let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(4100 + seed));
        let spans = GeneTable::scan(&genome).spans;
        let alone = |s: &GeneSpan| spans.iter().filter(|o| o.start < s.end && s.start < o.end).count() == 1;
        // A payload letter of a gene that overlaps no other gene
        let payload_letter = |promoter: &str| {
            spans
                .iter()
                .find(|s| s.gene.promoter == promoter && alone(s))
                .map(|s| s.start + promoter.len())
        };
        for (promoter, body, brain) in [
            ("NEU", false, true),
            ("OUT", false, true),
            ("COL", true, false),
            ("SIZ", true, false),
        ] {
            let Some(index) = payload_letter(promoter) else {
                continue;
            };
            let touch = classify_mutation(&genome, &letter_change(&genome, index));
            assert_eq!(
                (touch.body, touch.brain, touch.organ),
                (body, brain, false),
                "{} letter at {}: {}",
                promoter,
                index,
                genome
            );
            checked[brain as usize] += 1;
        }
        if let Some(index) = promoters::with_role(GeneRole::Organ).find_map(|p| payload_letter(p)) {
            let touch = classify_mutation(&genome, &letter_change(&genome, index));
            assert_eq!(touch.channel(), MutationChannel::Body);
            assert!(touch.organ);
        }
        // A filler letter outside every gene
        if let Some(index) = unprotected_indices(&genome)
            .into_iter()
            .find(|&i| !spans.iter().any(|s| s.start <= i && i < s.end))
        {
            let mutation = letter_change(&genome, index);
            assert_eq!(
                classify_mutation(&genome, &mutation).channel(),
                MutationChannel::Neutral
            );
            checked[2] += 1;
        }
        // Gene duplication of a brain gene: the new copy is a new identity of a brain promoter
        if let Some((child, start)) = apply_mutation(
            &genome,
            MutationKind::GeneDuplication,
            ops(),
            Some(GeneChannel::Brain),
            &mut crate::shared::rng::seeded(seed),
        ) {
            let mutation = Mutation {
                genome: child,
                kind: MutationKind::GeneDuplication,
                index: start,
            };
            assert_eq!(classify_mutation(&genome, &mutation).channel(), MutationChannel::Brain);
            checked[3] += 1;
        }
    }
    assert!(checked.iter().all(|&c| c > 0), "every case exercised: {:?}", checked);
}

#[test]
fn test_mutate_genome_in_channel_touches_only_its_channel() {
    let mut applied = 0;
    for seed in 0..150u64 {
        let mut genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
        for (step, channel) in [
            GeneChannel::Body,
            GeneChannel::Brain,
            GeneChannel::Body,
            GeneChannel::Brain,
        ]
        .into_iter()
        .enumerate()
        {
            let mut rng = crate::shared::rng::seeded(seed * 10 + step as u64);
            let Some(mutation) = mutate_genome_in_channel(&genome, ops(), channel, &mut rng) else {
                continue;
            };
            let touch = classify_mutation(&genome, &mutation);
            assert!(
                touch.only(channel),
                "{:?} mutation ({:?} at {}) touched {:?}",
                channel,
                mutation.kind,
                mutation.index,
                touch
            );
            applied += 1;
            genome = mutation.genome;
        }
    }
    assert!(applied > 550, "{} of 600 channel mutations applied", applied);
}
