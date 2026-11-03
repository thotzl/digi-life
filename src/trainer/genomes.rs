//! Genomes of new sandboxes: mutation of elites/hof/mutants, random genomes and the minimal criterion (moved from
//! `trainer_engine`, TCK-201). Used by the `Breeder`.

use rand::Rng;

use crate::biology::genome::{
    GeneChannel, GenomeComposition, MutationTouch, classify_mutation, generate_structured_genome, mutate_genome,
    mutate_genome_in_channel,
};
use crate::biology::organs::OrganKind;
use crate::biology::phenotype::parse_genome;
use crate::biology::trainer_config::MutationChannelMode;
use crate::config::AppConfig;
use crate::shared::rng::{SimRng, seeded};

use super::room::registry;
use super::sandbox::{SandboxOrigin, TrainerSandbox};

/// Number of mutation operator calls for one child: uniform in `[ops_min, max(ops_max, ops_min)]` (TCK-174;
/// config `mutationOpsMin` / `mutationOpsMax`, replaces `round(len * mutation_rate)`).
pub fn draw_mutation_ops(ops_min: u32, ops_max: u32, rng: &mut SimRng) -> u32 {
    rng.gen_range(ops_min..=ops_max.max(ops_min))
}

/// Genome of a new sandbox: mutants and hall-of-fame copies get `draw_mutation_ops(ops_min, ops_max)` calls of
/// `mutate_genome` (operator mix `mutationOperators`), random immigrants a fresh structured genome (config
/// `genomeComposition`; `random_genome` with the minimal criterion), elites an unmodified copy. Hof entries are mutated
/// since TCK-174 (before: unmodified copies).
pub fn breed_genome(
    parent_genome: &str,
    origin: SandboxOrigin,
    ops_min: u32,
    ops_max: u32,
    minimal_criterion: bool,
    rng: &mut SimRng,
    config: &AppConfig,
) -> String {
    breed_genome_traced(
        parent_genome,
        origin,
        ops_min,
        ops_max,
        minimal_criterion,
        MutationChannelMode::Mixed,
        rng,
        config,
    )
    .0
}

/// Probability that a child in mutation channel mode "separate" mutates its body genes (otherwise its brain genes).
pub const SEPARATE_BODY_SHARE: f64 = 0.5;

/// `breed_genome` with the gene channels its mutations touched (TCK-181; `genome::classify_mutation` per operator call,
/// unioned over the child's calls): `Some` for mutated origins (mutant, hof), `None` otherwise. Mode `Mixed` draws
/// exactly what `breed_genome` draws (the classification uses no RNG). Mode `Separate` first draws the child's
/// channel (`SEPARATE_BODY_SHARE`) from `rng`, then restricts every call to it (`genome::mutate_genome_in_channel`).
#[allow(clippy::too_many_arguments)]
pub fn breed_genome_traced(
    parent_genome: &str,
    origin: SandboxOrigin,
    ops_min: u32,
    ops_max: u32,
    minimal_criterion: bool,
    mode: MutationChannelMode,
    rng: &mut SimRng,
    config: &AppConfig,
) -> (String, Option<MutationTouch>) {
    let mut genome = parent_genome.to_string();
    match origin {
        SandboxOrigin::Mutant | SandboxOrigin::Hof => {
            let channel = match mode {
                MutationChannelMode::Mixed => None,
                MutationChannelMode::Separate => Some(if rng.gen_bool(SEPARATE_BODY_SHARE) {
                    GeneChannel::Body
                } else {
                    GeneChannel::Brain
                }),
            };
            let mut touch = MutationTouch::default();
            for _ in 0..draw_mutation_ops(ops_min, ops_max, rng) {
                let mutation = match channel {
                    None => mutate_genome(&genome, &config.genome.mutation_operators, rng),
                    Some(c) => mutate_genome_in_channel(&genome, &config.genome.mutation_operators, c, rng),
                };
                if let Some(mutation) = mutation {
                    touch = touch.union(classify_mutation(&genome, &mutation));
                    genome = mutation.genome;
                }
            }
            return (genome, Some(touch));
        }
        SandboxOrigin::Random => genome = random_genome(minimal_criterion, rng, config),
        SandboxOrigin::Elite => {}
    }
    (genome, None)
}

/// Random trainer genome (generation 1 and immigrants). Without the minimal criterion: `generate_structured_genome`
/// with config `genomeComposition`. With it (RV-S09, config `trainerMinimalCriterion`): the same generator with a
/// food organ gene in every genome (`pFoodOrgan` = 1.0), redrawn up to `maxRedraws` times until
/// `meets_minimal_criterion`; if no candidate passes, the last one is kept. All draws come from `rng`, so the result
/// is deterministic.
pub fn random_genome(minimal_criterion: bool, rng: &mut SimRng, config: &AppConfig) -> String {
    if !minimal_criterion {
        return generate_structured_genome(&config.genome.genome_composition, rng);
    }
    let criterion = &config.trainer.trainer_minimal_criterion;
    let composition = GenomeComposition {
        p_food_organ: 1.0,
        ..config.genome.genome_composition.clone()
    };
    let mut genome = generate_structured_genome(&composition, rng);
    for _ in 0..criterion.max_redraws {
        if meets_minimal_criterion(&genome, config) {
            break;
        }
        genome = generate_structured_genome(&composition, rng);
    }
    genome
}

/// Fixed seed of the mobility check's standard-room layout (independent of the run seed).
const MOBILITY_CHECK_SEED: u64 = 0x4D43;
/// Room of the mobility check.
const MOBILITY_CHECK_ROOM: &str = "standard";

/// Minimal criterion (RV-S09): the phenotype has at least one food-sensing organ (eye or nose, as `food_organ_share`)
/// and the creature moves: in a fixed standard-room trial (centre start, `MOBILITY_CHECK_SEED` layout) its own brain
/// moves it at least `mobilityMinDistance` px within `mobilityTicks` ticks.
pub fn meets_minimal_criterion(genome: &str, config: &AppConfig) -> bool {
    let criterion = &config.trainer.trainer_minimal_criterion;
    let phenotype = parse_genome(genome, None, None, &config.brain);
    if !phenotype
        .organelles
        .iter()
        .any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose))
    {
        return false;
    }
    let room = registry::build(MOBILITY_CHECK_ROOM, config).expect("mobility check room is registered");
    let mut sb = TrainerSandbox::spawn(
        1,
        genome,
        1,
        SandboxOrigin::Random,
        room,
        None,
        None,
        &mut seeded(MOBILITY_CHECK_SEED),
        config,
    );
    for _ in 0..criterion.mobility_ticks {
        sb.step(config);
        if sb.trial().distance_traveled >= criterion.mobility_min_distance {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::genome::{WILDTYPE_GENOME, generate_random_genome};
    use crate::shared::rng::derive;

    #[test]
    fn test_mutation_ops_drawn_within_bounds() {
        let mut rng = seeded(11);
        let mut seen = [0u32; 4];
        for _ in 0..600 {
            let k = draw_mutation_ops(1, 3, &mut rng);
            assert!((1..=3).contains(&k), "{} ops", k);
            seen[k as usize] += 1;
        }
        assert!(seen[1..].iter().all(|&c| c > 100), "{:?}", seen);
        // max below min collapses to min
        assert_eq!(draw_mutation_ops(2, 1, &mut rng), 2);
    }

    #[test]
    fn test_breed_genome_mutates_hof_and_mutants_but_not_elites() {
        let mut changed = [0; 3];
        for seed in 0..20 {
            for (i, origin) in [SandboxOrigin::Hof, SandboxOrigin::Mutant, SandboxOrigin::Elite]
                .into_iter()
                .enumerate()
            {
                changed[i] += (breed_genome(
                    WILDTYPE_GENOME,
                    origin,
                    1,
                    3,
                    false,
                    &mut seeded(seed),
                    crate::config::test_config(),
                ) != WILDTYPE_GENOME) as usize;
            }
        }
        assert!(changed[0] >= 15 && changed[1] >= 15, "{:?}", changed);
        assert_eq!(changed[2], 0);
        // Hof and mutant follow the same rule: same stream, same child
        assert_eq!(
            breed_genome(
                WILDTYPE_GENOME,
                SandboxOrigin::Hof,
                1,
                3,
                false,
                &mut seeded(3),
                crate::config::test_config()
            ),
            breed_genome(
                WILDTYPE_GENOME,
                SandboxOrigin::Mutant,
                1,
                3,
                false,
                &mut seeded(3),
                crate::config::test_config()
            )
        );
    }

    #[test]
    fn test_minimal_criterion_yields_food_organ_and_mobility() {
        let config = crate::config::test_config();
        let mut passed = 0;
        for seed in 0..40 {
            let genome = random_genome(true, &mut seeded(seed), config);
            // Deterministic: same stream, same genome
            assert_eq!(genome, random_genome(true, &mut seeded(seed), config));
            let phenotype = parse_genome(&genome, None, None, &config.brain);
            assert!(
                phenotype
                    .organelles
                    .iter()
                    .any(|o| matches!(o.kind, OrganKind::Eye | OrganKind::Nose)),
                "seed {} without food organ",
                seed
            );
            passed += meets_minimal_criterion(&genome, config) as usize;
        }
        assert_eq!(passed, 40, "every minimal-criterion genome moves");
        // Off: the plain structured generator (same stream as before TCK-175)
        assert_eq!(
            random_genome(false, &mut seeded(3), config),
            generate_structured_genome(&config.genome.genome_composition, &mut seeded(3))
        );
        assert_eq!(
            breed_genome(
                WILDTYPE_GENOME,
                SandboxOrigin::Random,
                1,
                3,
                true,
                &mut seeded(5),
                config
            ),
            random_genome(true, &mut seeded(5), config)
        );
    }

    #[test]
    fn test_minimal_criterion_rejects_organless_genomes() {
        // A genome without organs fails the criterion (random letters rarely form an organ gene)
        let organless = (0..400)
            .map(|i| generate_random_genome(256, &mut derive(20, &[i])))
            .find(|g| {
                parse_genome(g, None, None, &crate::config::test_config().brain)
                    .organelles
                    .is_empty()
            })
            .expect("random letters contain organless genomes");
        assert!(!meets_minimal_criterion(&organless, crate::config::test_config()));
    }

    #[test]
    fn test_breed_genome_traced_mixed_matches_breed_genome_and_separate_stays_in_one_channel() {
        use crate::biology::genome::MutationChannel;
        let mut channels = std::collections::BTreeMap::new();
        for seed in 0..80 {
            let parent = random_genome(false, &mut seeded(900 + seed), crate::config::test_config());
            let (mixed, touch) = breed_genome_traced(
                &parent,
                SandboxOrigin::Mutant,
                1,
                3,
                false,
                MutationChannelMode::Mixed,
                &mut seeded(seed),
                crate::config::test_config(),
            );
            assert_eq!(
                mixed,
                breed_genome(
                    &parent,
                    SandboxOrigin::Mutant,
                    1,
                    3,
                    false,
                    &mut seeded(seed),
                    crate::config::test_config()
                ),
                "mixed draws exactly like breed_genome"
            );
            assert!(touch.is_some());
            let (_, touch) = breed_genome_traced(
                &parent,
                SandboxOrigin::Mutant,
                1,
                3,
                false,
                MutationChannelMode::Separate,
                &mut seeded(seed),
                crate::config::test_config(),
            );
            let channel = touch.expect("mutated origin").channel();
            assert_ne!(channel, MutationChannel::Both, "separate child touched both channels");
            *channels.entry(format!("{:?}", channel)).or_insert(0) += 1;
        }
        assert!(
            channels.get("Body").copied().unwrap_or(0) > 20 && channels.get("Brain").copied().unwrap_or(0) > 20,
            "{:?}",
            channels
        );
        // Unmutated origins carry no channel
        assert_eq!(
            breed_genome_traced(
                WILDTYPE_GENOME,
                SandboxOrigin::Elite,
                1,
                3,
                false,
                MutationChannelMode::Separate,
                &mut seeded(1),
                crate::config::test_config()
            )
            .1,
            None
        );
    }
}
