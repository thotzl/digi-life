//! Tests of the phenotype compiler: golden master, trait locality, brain wiring, HOX pairing.

use super::body::read_evolution_rates;
use super::*;
use crate::biology::brain::compile::fnv1a;
use crate::biology::brain::*;
use crate::biology::genome::mutate::apply_mutation;
use crate::biology::genome::promoters::{self, GeneRole};
use crate::biology::genome::tests::{composition, ops};
use crate::biology::genome::*;

/// Traits the phenotype no longer carries (dead payload removed in TCK-199) but the golden master still hashes, so
/// the fingerprint stays comparable: decoded here from the genome exactly as the compiler did.
struct RetiredTraits {
    hydraulic_pressure: f32,
    repair_fidelity: f32,
}

fn retired_traits(p: &CreaturePhenotype) -> RetiredTraits {
    let genes = GeneTable::scan(&p.genome_string);
    RetiredTraits {
        hydraulic_pressure: 0.2 + get_payload_linear_value_offset(genes.first_payload("STF").as_str(), 7) * 0.8,
        repair_fidelity: 0.15 + get_payload_linear_value_offset(genes.first_payload("EVO").as_str(), 10) * 0.8,
    }
}

/// The golden genome motifs are frozen test data (changing them changes the fingerprint); they must still cover
/// every expressed promoter and the terminator.
#[test]
fn test_golden_genomes_cover_every_promoter() {
    let all: String = golden_genomes().concat();
    for motif in PROMOTERS.iter().map(|s| s.motif).chain([TERMINATOR]) {
        assert!(all.contains(motif), "{} missing from golden_genomes", motif);
    }
}

#[test]
fn test_hox_bilateral_symmetry() {
    // A. Without HOX gene (Asymmetric primitive)
    // This genome contains exactly one EYE promoter and no HOX promoter.
    let no_hox_dna = "EYEABCDEFEN";
    let pheno_no_hox = parse_genome(no_hox_dna, None, None, &crate::config::test_config().brain);
    // It must compile exactly 1 organelle asymmetrically.
    assert_eq!(pheno_no_hox.organelles.len(), 1);
    let angle = pheno_no_hox.organelles[0].angle;
    // Verify angle range is primitive [-170 to 170 deg]
    assert!(angle >= -170.0 && angle <= 170.0);

    // B. With HOX gene - Lateral Organelle (Paired/Mirrored)
    // Let's craft a genome with both HOX and EYE promoters.
    // We set index 0 of the eye payload to 'K' to yield -170.0 deg exactly.
    let hox_paired_dna = "HOXABCDEZENEYEKAAAAAAAEN"; 
    let pheno_paired = parse_genome(hox_paired_dna, None, None, &crate::config::test_config().brain);
    // It should contain exactly 2 organelles due to bilateral symmetry (Left + Right mirror)
    assert_eq!(pheno_paired.organelles.len(), 2);
    assert_eq!(pheno_paired.organelles[0].angle, -170.0);
    assert_eq!(pheno_paired.organelles[1].angle, 170.0); // mirrored to +170.0 degrees!

    // C. With HOX gene - Midline Coalescence (Unpaired/Snapped)
    // Let's craft a genome with both HOX and EYE promoters.
    // We place 'X' at index 16 of the eye payload to yield 6.8 deg, snapping to 0.0 deg.
    let hox_midline_dna = "HOXABCDEZENEYEAAAAAAAAAAAAAAAAXAEN"; 
    let pheno_midline = parse_genome(hox_midline_dna, None, None, &crate::config::test_config().brain);
    // Coalescence/Fusion check: It must snap to exactly 0.0 degrees (straight ahead) and NOT duplicate!
    assert_eq!(pheno_midline.organelles.len(), 1);
    assert_eq!(pheno_midline.organelles[0].angle, 0.0);

    // D. Open-end fallback test case (No trailing "EN" terminator for EYE promoter)
    // This genome has HOX active, and an EYE promoter but no trailing "EN" terminator.
    // It must fall back to reading exactly 15 characters of payload ("AKAAAAAAAAAAAAA").
    // Index 16 % 15 = 1. Since index 1 of the payload is 'K', the angle evaluates to -170.0 deg.
    let hox_open_dna = "HOXABCDEZENEYEAKAAAAAAAAAAAAA"; // No trailing "EN"!
    let pheno_open = parse_genome(hox_open_dna, None, None, &crate::config::test_config().brain);
    // Bilateral symmetry must still work on the 15-char fallback payload!
    assert_eq!(pheno_open.organelles.len(), 2);
    assert_eq!(pheno_open.organelles[0].angle, -170.0);
    assert_eq!(pheno_open.organelles[1].angle, 170.0); // mirrored to +170.0 deg!
}

/// Deterministic genome set (no rand) covering random strands plus organ/HOX/brain promoters.
pub(crate) fn golden_genomes() -> Vec<String> {
    let mut state: u64 = 0x2545F4914F6CDD1D;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let motifs = ["EYE", "NOS", "TAC", "LUM", "HOX", "NEU", "SY", "OUT", "EN", "SP", "COL", "STF", "PUL", "SIZ", "WAV", "STM", "TEM", "REP", "EVO", "CAR"];
    (0..300)
        .map(|_| {
            let len = 128 + (next() % 300) as usize;
            let mut g = String::with_capacity(len + 64);
            while g.len() < len {
                if next() % 4 == 0 {
                    g.push_str(motifs[(next() % motifs.len() as u64) as usize]);
                } else {
                    g.push(ALPHABET[(next() % 26) as usize] as char);
                }
            }
            g
        })
        .collect()
}

/// Body traits with the promoter of the gene they are read from (derived values such as BMR are left out).
fn owned_traits(p: &CreaturePhenotype) -> Vec<(&'static str, f32)> {
    let sh = &p.spinal_harmonics;
    let retired = retired_traits(p);
    let mut traits = vec![
        ("COL", (p.symmetry == Symmetry::Quad) as u8 as f32),
        ("COL", p.primary_color.h),
        ("COL", p.primary_color.s),
        ("SIZ", p.primary_color.l),
        ("COL", p.secondary_color.h),
        ("SIZ", p.secondary_color.s),
        ("COL", p.secondary_color.l),
        ("SIZ", p.body_seed as f32),
        ("SIZ", sh.mean_radius),
        ("SIZ", sh.base_length),
        ("STF", p.stiffness),
        ("STF", retired.hydraulic_pressure),
        ("WAV", sh.spinal_curve),
        ("WAV", sh.spinal_curve_freq),
        ("WAV", sh.parapodia_amp),
        ("WAV", sh.parapodia_freq),
        ("PUL", sh.flattening_head),
        ("PUL", p.pulse_speed),
        ("PUL", p.wave_phase),
        ("PUL", p.wiggle_amplitude),
        ("STM", p.stomach_capacity),
        ("TEM", p.thermal_tolerance_min),
        ("TEM", p.thermal_tolerance_max),
        ("CAR", p.carnivory),
        ("REP", p.mature_age as f32),
        ("REP", p.repro_threshold),
        ("REP", p.split_loss),
        ("EVO", p.insertion_rate),
        ("EVO", p.deletion_rate),
        ("EVO", retired.repair_fidelity),
    ];
    traits.extend(sh.amplitudes.iter().chain(&sh.phases).map(|&v| ("WAV", v)));
    traits
}

/// Synapse as compared across genomes: weight, owner span and the genes of both endpoints.
#[derive(Debug, PartialEq)]
struct Edge {
    weight: f32,
    owner: GeneSpan,
    endpoints: [Option<GeneRef>; 2],
}

fn brain_by_key(p: &CreaturePhenotype) -> std::collections::BTreeMap<u64, Edge> {
    let gene_of = |i: usize| p.brain.neurons[i].role.gene().cloned();
    let map: std::collections::BTreeMap<u64, Edge> = p
        .brain
        .synapses
        .iter()
        .map(|s| (s.key, Edge { weight: s.weight, owner: p.genes[s.owner].clone(), endpoints: [gene_of(s.from_node), gene_of(s.to_node)] }))
        .collect();
    assert_eq!(map.len(), p.brain.synapses.len(), "synapse keys are unique");
    map
}

fn gene_ref(p: &CreaturePhenotype, g: usize) -> GeneRef {
    p.genes[g].gene.clone()
}

/// The genome with the base at `pos` replaced by another letter.
fn point_mutant(genome: &str, pos: usize, shift: usize) -> String {
    let mut bytes = genome.as_bytes().to_vec();
    bytes[pos] = ALPHABET[(char_to_value(bytes[pos] as char) + shift) % 26];
    String::from_utf8(bytes).unwrap()
}

#[test]
fn test_parse_genome_never_panics_on_random_ascii() {
    use rand::Rng;
    let mut rng = crate::shared::rng::seeded(4);
    for _ in 0..400 {
        let len = rng.gen_range(0..=1024);
        let genome: String = (0..len).map(|_| rng.gen_range(0u8..128) as char).collect();
        let p = parse_genome(&genome, None, None, &crate::config::test_config().brain);
        let n = p.brain.neurons.len();
        assert!(p.brain.synapses.iter().all(|s| s.from_node < n && s.to_node < n && s.owner < p.genes.len() && s.weight.is_finite()));
    }
}

#[test]
fn test_point_mutation_outside_genes_changes_no_weight_or_trait() {
    let mut tested = 0;
    for g in golden_genomes() {
        let parent_genome = normalize_genome(&g);
        let parent = parse_genome(&parent_genome, None, None, &crate::config::test_config().brain);
        let covered = |i: usize| parent.genes.iter().any(|s| (s.start..s.end).contains(&i));
        for pos in (0..parent_genome.len()).filter(|&i| !covered(i)).step_by(2) {
            let child = parse_genome(&point_mutant(&parent_genome, pos, 11), None, None, &crate::config::test_config().brain);
            if child.genes != parent.genes {
                continue; // the mutation created or re-terminated a gene
            }
            tested += 1;
            assert_eq!(child.brain, parent.brain);
            assert_eq!(child.organelles, parent.organelles);
            assert_eq!(owned_traits(&child), owned_traits(&parent));
        }
    }
    assert!(tested > 1000, "{} mutants tested", tested);
}

#[test]
fn test_point_mutation_inside_a_gene_changes_only_that_gene() {
    let mut tested = 0;
    for g in golden_genomes().into_iter().step_by(30) {
        let parent_genome = normalize_genome(&g);
        let parent = parse_genome(&parent_genome, None, None, &crate::config::test_config().brain);
        let parent_brain = brain_by_key(&parent);
        for span in &parent.genes {
            for pos in (span.start + span.gene.promoter.len()..span.end).step_by(3) {
                let child = parse_genome(&point_mutant(&parent_genome, pos, 5), None, None, &crate::config::test_config().brain);
                if child.genes != parent.genes {
                    continue; // the mutation broke the terminator or created a gene
                }
                tested += 1;
                // Genes of different promoters may overlap: the base belongs to every gene that covers it
                let hit: Vec<(usize, GeneRef)> = (0..parent.genes.len())
                    .filter(|&g| (parent.genes[g].start..parent.genes[g].end).contains(&pos))
                    .map(|g| (g, gene_ref(&parent, g)))
                    .collect();
                let is_hit = |r: &GeneRef| hit.iter().any(|(_, h)| h == r);

                // Synapses: a changed, added or removed synapse is owned by a hit gene or touches one of its neurons.
                // Exception: SY-owned synapses (SY addresses modulo the class size, TCK-168).
                let child_brain = brain_by_key(&child);
                let attributable = |e: Option<&Edge>| {
                    e.is_some_and(|e| hit.iter().any(|(g, r)| e.owner == parent.genes[*g] || e.endpoints.contains(&Some(r.clone()))) || e.owner.gene.promoter == "SY")
                };
                for key in parent_brain.keys().chain(child_brain.keys()) {
                    let (p, c) = (parent_brain.get(key), child_brain.get(key));
                    assert!(p == c || attributable(p) || attributable(c), "genes {:?} pos {} changed synapse {:?} -> {:?}", hit, pos, p, c);
                }

                // Neurons of other genes keep their parameters (motors belong to the first OUT gene)
                let expressed = |promoter: &str| hit.iter().any(|(g, _)| parent.genes.iter().position(|s| s.gene.promoter == promoter) == Some(*g));
                for n in &parent.brain.neurons {
                    let own = n.role.gene().is_some_and(|r| is_hit(r)) || (expressed(promoters::motif_of(GeneRole::BrainBase)) && matches!(n.role, NeuronRole::Motor { .. }));
                    if !own {
                        let m = child.brain.neurons.iter().find(|m| m.role == n.role).expect("neuron of another gene kept");
                        assert_eq!((m.tau, m.bias, &m.activation_type), (n.tau, n.bias, &n.activation_type), "{:?}", n.role);
                    }
                }

                // Traits: only those of a hit gene that is the expressed (first) gene of its type
                for ((promoter, pv), (_, cv)) in owned_traits(&parent).into_iter().zip(owned_traits(&child)) {
                    if !expressed(promoter) {
                        assert_eq!(pv, cv, "genes {:?} changed a {} trait", hit, promoter);
                    }
                }
                let others = |p: &CreaturePhenotype| p.organelles.iter().filter(|o| !hit.iter().any(|(g, _)| parent.genes[*g].start == o.gene_start_index)).cloned().collect::<Vec<_>>();
                assert_eq!(others(&child), others(&parent));
            }
        }
    }
    assert!(tested > 1000, "{} mutants tested", tested);
}

#[test]
fn test_adding_an_organ_gene_keeps_unrelated_synapses() {
    // Payloads without vowels and Delta letters, so they contain no promoter or terminator (eye, nose, tactile, eye)
    let organ_genes = ["EYEBTKLMRVXZGHJWFPYBTKLEN", "NOSBKLMRVXZGHJWFPYBTKLMEN", "TACBBTKLMRVXZGHJWFPYBTKEN", "LUMZXTKLMRVBGHJWFPYBTKLEN"];
    let mut tested = 0;
    for g in golden_genomes() {
        // A trailing Z spacer (no motif) in the parent: its unterminated genes end there, not in the new gene
        let parent_genome = format!("{}{}", normalize_genome(&g), "Z".repeat(30));
        let parent = parse_genome(&parent_genome, None, None, &crate::config::test_config().brain);
        let parent_brain = brain_by_key(&parent);
        for organ in organ_genes {
            if parent_genome.len() + organ.len() > 512 {
                continue;
            }
            let child = parse_genome(&format!("{}{}", parent_genome, organ), None, None, &crate::config::test_config().brain);
            // Appended as the last gene of its type: all parent genes unchanged, exactly one new gene
            if child.genes.len() != parent.genes.len() + 1 || !parent.genes.iter().all(|s| child.genes.contains(s)) {
                continue;
            }
            tested += 1;
            let occurrence = parent.genes.iter().filter(|s| s.gene.promoter == organ[..3] && s.gene.spelling == organ[..3]).count();
            let new_gene = GeneRef { promoter: organ[..3].to_string(), spelling: organ[..3].to_string(), occurrence };
            let child_brain = brain_by_key(&child);
            for (key, p) in &parent_brain {
                match child_brain.get(key) {
                    Some(c) => assert!(p == c || p.owner.gene.promoter == "SY" || c.owner.gene.promoter == "SY", "{:?} -> {:?}", p, c),
                    None => assert_eq!(p.owner.gene.promoter, "SY", "parent synapse lost: {:?}", p),
                }
            }
            for (key, c) in &child_brain {
                assert!(parent_brain.contains_key(key) || c.endpoints.contains(&Some(new_gene.clone())) || c.owner.gene.promoter == "SY", "{:?}", c);
            }
        }
    }
    assert!(tested > 300, "{} genomes tested", tested);
}

#[test]
fn test_gene_duplication_and_deletion_keep_unrelated_synapses() {
    // Synapses that neither belong to nor touch the duplicated/deleted gene keep key and weight; a duplication keeps
    // every parent synapse (the original keeps its identity, the copy is a new gene). Exceptions: SY-owned synapses
    // (SY addresses modulo the class size, TCK-168) and HOX (pairs or unpairs every organ).
    let mut tested = [0; 2];
    for (k, kind) in [MutationKind::GeneDuplication, MutationKind::GeneDeletion].into_iter().enumerate() {
        for seed in 0..300 {
            let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
            let Some((child_genome, start)) = apply_mutation(&genome, kind, ops(), None, &mut crate::shared::rng::seeded(seed + 7_000)) else { continue };
            let parent = parse_genome(&genome, None, None, &crate::config::test_config().brain);
            let child = parse_genome(&child_genome, None, None, &crate::config::test_config().brain);
            let g = parent.genes.iter().position(|s| s.start == start).expect("mutated gene");
            if parent.genes[g].gene.promoter == "HOX" {
                continue;
            }
            tested[k] += 1;
            let r = gene_ref(&parent, g);
            let new_genes: Vec<GeneRef> = child.genes.iter().map(|s| s.gene.clone()).filter(|c| !parent.genes.iter().any(|p| &p.gene == c)).collect();
            assert_eq!(new_genes.len(), k ^ 1, "{:?} on {:?} gave new genes {:?}", kind, r, new_genes);
            let (parent_brain, child_brain) = (brain_by_key(&parent), brain_by_key(&child));
            let is_sy = |e: &Edge| e.owner.gene.promoter == "SY";
            for (key, p) in parent_brain.iter().filter(|(_, e)| !is_sy(e)) {
                let touches = p.owner == parent.genes[g] || p.endpoints.contains(&Some(r.clone()));
                if kind == MutationKind::GeneDeletion && touches {
                    continue;
                }
                match child_brain.get(key) {
                    Some(c) => assert!(c.weight == p.weight || is_sy(c), "{:?} on {:?} changed {:?} -> {:?}", kind, r, p, c),
                    None => panic!("{:?} on {:?} lost {:?}", kind, r, p),
                }
            }
            for (key, c) in &child_brain {
                let touches_new = new_genes.iter().any(|n| c.endpoints.contains(&Some(n.clone())) || &c.owner.gene == n);
                assert!(parent_brain.contains_key(key) || touches_new || is_sy(c), "{:?} on {:?} added {:?}", kind, r, c);
            }
        }
    }
    assert!(tested.iter().all(|&t| t > 250), "tested {:?}", tested);
}

/// Key-shift measurement (TCK-179, `cargo test --release --lib measure_gene_operator_key_shift -- --ignored --nocapture`):
/// one gene duplication or deletion on each of 1000 structured genomes; for the parent synapses that do not touch
/// the mutated gene (`SY`-owned excluded) the share kept by key, the share with the identical weight and the weight
/// correlation. "shifting": the genome has a later gene of the mutated gene's promoter type. Excluded: HOX genes
/// (pairing touches every organ) and synapses an `SY` gene owns in the child.
#[test]
#[ignore]
fn measure_gene_operator_key_shift() {
    for kind in [MutationKind::GeneDuplication, MutationKind::GeneDeletion] {
        // (cases, unrelated parent synapses, kept, identical weight, weight pairs) for non-shifting / shifting
        let mut tally: [(usize, usize, usize, usize, Vec<(f32, f32)>); 2] = Default::default();
        let mut hox = 0;
        let mut same_spelling = 0;
        for seed in 0..1000 {
            let genome = generate_structured_genome(composition(), &mut crate::shared::rng::seeded(seed));
            let Some((child_genome, start)) = apply_mutation(&genome, kind, ops(), None, &mut crate::shared::rng::seeded(seed + 1_000_000)) else { continue };
            let parent = parse_genome(&genome, None, None, &crate::config::test_config().brain);
            let child = parse_genome(&child_genome, None, None, &crate::config::test_config().brain);
            let g = parent.genes.iter().position(|s| s.start == start).expect("mutated gene");
            if parent.genes[g].gene.promoter == "HOX" {
                hox += 1; // HOX pairs or unpairs every organ: touches all organ synapses
                continue;
            }
            let r = gene_ref(&parent, g);
            let shifting = parent.genes[g + 1..].iter().any(|s| s.gene.promoter == parent.genes[g].gene.promoter);
            same_spelling += parent.genes[g + 1..].iter().any(|s| s.gene.promoter == r.promoter && s.gene.spelling == r.spelling) as usize;
            let (parent_brain, child_brain) = (brain_by_key(&parent), brain_by_key(&child));
            let t = &mut tally[shifting as usize];
            t.0 += 1;
            for (key, p) in parent_brain.iter().filter(|(_, e)| e.owner.gene.promoter != "SY" && e.owner != parent.genes[g] && !e.endpoints.contains(&Some(r.clone()))) {
                let c = child_brain.get(key);
                if c.is_some_and(|c| c.owner.gene.promoter == "SY") {
                    continue; // an SY gene re-addressed by the changed class sizes (TCK-168)
                }
                t.1 += 1;
                if let Some(c) = c {
                    t.2 += 1;
                    t.3 += (c.weight == p.weight) as usize;
                    t.4.push((p.weight, c.weight));
                }
            }
        }
        for (label, t) in ["non-shifting", "shifting"].iter().zip(&tally) {
            println!(
                "{:?} {}: cases {}, unrelated synapses {}, kept {:.4}, identical weight {:.4}, r {:.4}",
                kind,
                label,
                t.0,
                t.1,
                t.2 as f32 / t.1.max(1) as f32,
                t.3 as f32 / t.1.max(1) as f32,
                crate::shared::stats::pearson(&t.4).unwrap_or(f32::NAN)
            );
        }
        println!("{:?} HOX cases (excluded): {}; cases with a later gene of the same spelling: {}", kind, hox, same_spelling);
    }
}

/// Canonical bytes of the model fields of a phenotype (TCK-197): body traits, initial methylations, organs
/// (kind, side, sensing/placement params), neurons in order (role, tau, bias, activation) and synapses (key, weight,
/// owner gene, plastic). Display strings, derived labels and dead payload are left out, so moving or dropping them
/// does not change the golden master.
fn model_fields(p: &CreaturePhenotype, out: &mut Vec<u8>) {
    let f = |v: f32, out: &mut Vec<u8>| out.extend_from_slice(&v.to_bits().to_le_bytes());
    let gene = |g: &GeneRef, out: &mut Vec<u8>| {
        out.extend_from_slice(g.promoter.as_bytes());
        out.extend_from_slice(g.spelling.as_bytes());
        out.extend_from_slice(&(g.occurrence as u32).to_le_bytes());
    };
    let len = |n: usize, out: &mut Vec<u8>| out.extend_from_slice(&(n as u32).to_le_bytes());
    let traits = owned_traits(p);
    len(traits.len(), out);
    for (promoter, v) in traits {
        out.extend_from_slice(promoter.as_bytes());
        f(v, out);
    }
    len(p.methylations.len(), out);
    p.methylations.iter().for_each(|&v| f(v, out));
    len(p.organelles.len(), out);
    for o in &p.organelles {
        out.extend_from_slice(&[o.kind as u8, o.side as u8]);
        for v in [o.spectral_affinity, o.bandwidth, o.expression_style, o.scale, o.spinal_pos, o.angle] {
            f(v, out);
        }
    }
    len(p.brain.neurons.len(), out);
    for n in &p.brain.neurons {
        n.role.encode(false, out);
        f(n.tau, out);
        f(n.bias, out);
        out.extend_from_slice(n.activation_type.map_or("-", |a| a.label()).as_bytes());
        out.push(0);
    }
    len(p.brain.synapses.len(), out);
    for s in &p.brain.synapses {
        out.extend_from_slice(&s.key.to_le_bytes());
        f(s.weight, out);
        gene(&p.genes[s.owner].gene, out);
        out.push(s.plastic as u8);
    }
}

#[test]
fn test_parse_genome_golden_master() {
    let mut combined = Vec::new();
    for g in golden_genomes() {
        model_fields(&parse_genome(&g, None, None, &crate::config::test_config().brain), &mut combined);
    }
    let fingerprint = fnv1a(&combined);
    println!("GOLDEN_FINGERPRINT={:#018x}", fingerprint);
    assert_eq!(fingerprint, GOLDEN_FINGERPRINT);
}

#[test]
fn test_read_evolution_rates_matches_parse_genome() {
    for g in golden_genomes() {
        let pheno = parse_genome(&g, None, None, &crate::config::test_config().brain);
        assert_eq!(read_evolution_rates(&g), (pheno.insertion_rate, pheno.deletion_rate));
    }
}

#[test]
fn test_latin_name_is_deterministic() {
    for g in golden_genomes() {
        let first = parse_genome(&g, None, None, &crate::config::test_config().brain).latin_name;
        for _ in 0..4 {
            assert_eq!(parse_genome(&g, None, None, &crate::config::test_config().brain).latin_name, first, "genome {}", g);
        }
    }
}

// Fingerprint of parse_genome output over golden_genomes(); Brain v2 model change (P3 slice A: TCK-179/182/144/153/149;
// slice C: gene identity by promoter spelling + occurrence, 0x0c404a33923c8dd9 -> 0x65f701a6937c1e71;
// P4 TCK-152: `plastic` flag per synapse, 0x65f701a6937c1e71 -> 0xe0a2bfd1c7310306, unchanged without the flag;
// P6 TCK-151: `basalMetabolicRate` removed from the phenotype (cost computed by `biology::metabolism`),
// 0xe0a2bfd1c7310306 -> 0x9d206dfd8124f808; TCK-190 tanh thalamus sum nodes -> 0x0ab34073297ace74;
// TCK-197: hash of the model fields only (`model_fields`) instead of the full serde output, and the motifs OUT, STM,
// TEM added to `golden_genomes`, 0x0ab34073297ace74 -> 0x6b78c31a4d19b0e8, no model change)
const GOLDEN_FINGERPRINT: u64 = 0x6b78c31a4d19b0e8;
