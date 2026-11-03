use super::*;
use crate::biology::brain::CTRNNSynapse;
use crate::biology::creature_factory::inherit_learned_weights;
use crate::biology::genome::{MutationChannel, WILDTYPE_GENOME};
use crate::biology::phenotype::parse_genome;
use crate::trainer::curriculum::tests::plain_curriculum;

fn small_params() -> TrainerParams {
    TrainerParams {
        population: 8,
        ..TrainerParams::from_config(crate::config::test_config())
    }
}

fn room(id: &str) -> Arc<dyn Room> {
    registry::build(id, crate::config::test_config()).unwrap()
}

/// Values that make two rooms the same task (the id is not compared: a registry room and the level room at its
/// level are the same room).
fn room_values(r: &dyn Room) -> (u32, u32, u32, u32) {
    let ws = r.world_settings(crate::config::test_config());
    (
        r.chamber_size().to_bits(),
        r.episode_ticks(),
        r.level().to_bits(),
        ws.metabolic_scale.to_bits(),
    )
}

fn level_room(level: f32) -> (u32, u32, u32, u32) {
    room_values(&LevelRoom::at(level, crate::config::test_config()))
}

/// Runs until `generations` generations completed; returns the stats and elite genomes per generation.
fn run(sim: &mut TrainerSim, generations: usize) -> Vec<(GenerationStats, Vec<String>)> {
    let mut out = Vec::new();
    while out.len() < generations {
        if let Some(TrainerEvent::GenerationCompleted { stats, elites, .. }) = sim.step(false) {
            out.push((stats, elites.into_iter().map(|e| e.genome).collect()));
        }
    }
    out
}

fn genomes(sim: &TrainerSim) -> Vec<String> {
    sim.sandboxes.iter().map(|s| s.agent().genome.clone()).collect()
}

#[test]
fn test_same_seed_same_generations_different_seed_differs() {
    let a = run(
        &mut TrainerSim::new(small_params(), 42, 200, crate::config::test_config()),
        3,
    );
    let b = run(
        &mut TrainerSim::new(small_params(), 42, 200, crate::config::test_config()),
        3,
    );
    assert_eq!(a, b);
    assert_eq!(a.iter().map(|(s, _)| s.generation).collect::<Vec<_>>(), vec![1, 2, 3]);

    let c = run(
        &mut TrainerSim::new(small_params(), 43, 200, crate::config::test_config()),
        1,
    );
    assert_ne!(a[0], c[0]);
}

#[test]
fn test_restore_continues_with_given_parents() {
    let params = TrainerParams {
        multi_trial: false,
        ..small_params()
    };
    let mut original = TrainerSim::new(params.clone(), 9, 200, crate::config::test_config());
    run(&mut original, 1);
    assert_eq!(original.generation, 2);

    let restored = TrainerSim::restore(
        params,
        9,
        2,
        original.parents().to_vec(),
        original.hof().to_vec(),
        200,
        crate::config::test_config(),
    );
    assert_eq!(restored.generation, 2);
    assert_eq!(genomes(&restored), genomes(&original));
    // Elite slots carry the parents unmodified
    assert_eq!(restored.sandboxes[0].origin_type, SandboxOrigin::Elite);
    assert_eq!(restored.sandboxes[0].agent().genome, original.parents()[0].genome);
    assert!(restored.sandboxes.iter().all(|s| s.agent().generation == 2));
}

fn lamarck_parent() -> GenomeRecord {
    let len = WILDTYPE_GENOME.len();
    let synapses = parse_genome(WILDTYPE_GENOME, None, None, &crate::config::test_config().brain)
        .brain
        .synapses
        .len();
    GenomeRecord {
        genome: WILDTYPE_GENOME.to_string(),
        name: "parent".to_string(),
        fitness: 100.0,
        generation: 1,
        methylations: vec![0.3; len],
        synapse_weights: vec![0.25; synapses],
    }
}

#[test]
fn test_lamarck_fallback_mutants_and_phenotype_matches_genome() {
    // hof_rate > elite_ratio with a single parent and empty hof: elite and hof slots fall back to
    // mutants of parent 0 (used to underflow the Lamarck source index)
    let params = TrainerParams {
        population: 8,
        elite_ratio: 0.25,
        hof_rate: 0.5,
        inflow_rate: 0.0,
        lamarckian: true,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let parent = lamarck_parent();
    let sim = TrainerSim::restore(
        params,
        5,
        2,
        vec![parent.clone()],
        Vec::new(),
        200,
        crate::config::test_config(),
    );

    assert_eq!(sim.sandboxes[0].origin_type, SandboxOrigin::Elite);
    assert!(
        sim.sandboxes[1..]
            .iter()
            .all(|s| s.origin_type == SandboxOrigin::Mutant)
    );
    assert!(sim.sandboxes.iter().any(|s| s.agent().genome != parent.genome));

    for sb in &sim.sandboxes {
        let expected = parse_genome(
            &sb.agent().genome,
            None,
            Some(&parent.methylations),
            &crate::config::test_config().brain,
        );
        assert_eq!(
            sb.agent().phenotype,
            expected,
            "sandbox {} phenotype does not match its genome",
            sb.id
        );
        assert_eq!(sb.agent().antisense, sb.agent().phenotype.antisense_strand);
        assert_eq!(sb.agent().neuron_states.len(), sb.agent().phenotype.brain.neurons.len());
    }
    // The unmutated elite inherits each plastic learned weight with probability lamarckianAssimilationChance;
    // the rest, and every fixed reflex, keep the genome weight (TCK-152)
    let elite = sim.sandboxes[0].agent();
    let synapses = &elite.phenotype.brain.synapses;
    let inherited = synapses
        .iter()
        .zip(&elite.synapse_weights)
        .filter(|(s, w)| **w != s.weight)
        .count();
    for ((s, &w), &learned) in synapses.iter().zip(&elite.synapse_weights).zip(&parent.synapse_weights) {
        assert!(w == s.weight || (s.plastic && w == learned), "synapse {:?}: {}", s, w);
    }
    assert!(
        inherited > 0 && inherited < synapses.iter().filter(|s| s.plastic).count(),
        "{} inherited",
        inherited
    );
}

/// Structured parent genome with learned weights = genome weight + a distinct offset per synapse.
fn learned_parent(seed: u64) -> (GenomeRecord, Vec<CTRNNSynapse>, Vec<f32>) {
    let config = crate::config::test_config();
    let genome = crate::biology::genome::generate_structured_genome(
        &config.genome.genome_composition,
        &mut crate::shared::rng::seeded(seed),
    );
    let synapses = parse_genome(&genome, None, None, &config.brain).brain.synapses;
    let offsets: Vec<f32> = (0..synapses.len()).map(|i| 0.01 * (i + 1) as f32).collect();
    let learned = synapses.iter().zip(&offsets).map(|(s, o)| s.weight + o).collect();
    let record = GenomeRecord {
        genome,
        name: "parent".to_string(),
        fitness: 100.0,
        generation: 1,
        methylations: Vec::new(),
        synapse_weights: learned,
    };
    (record, synapses, offsets)
}

/// Learned offset per key that `weights` carry over the genome weights of `genome`.
fn offsets_by_key(genome: &str, weights: &[f32]) -> std::collections::BTreeMap<u64, f32> {
    parse_genome(genome, None, None, &crate::config::test_config().brain)
        .brain
        .synapses
        .iter()
        .zip(weights)
        .map(|(s, w)| (s.key, w - s.weight))
        .collect()
}

#[test]
fn test_lamarck_offsets_follow_synapse_keys_across_gene_duplication() {
    use crate::biology::genome::{MutationKind, mutate_genome_with_kind};
    let ops = &crate::config::test_config().genome.mutation_operators;
    let mut checked = 0;
    for seed in 0..40 {
        let (parent, synapses, offsets) = learned_parent(seed);
        let Some(child_genome) = mutate_genome_with_kind(
            &parent.genome,
            MutationKind::GeneDuplication,
            ops,
            &mut crate::shared::rng::seeded(seed),
        ) else {
            continue;
        };
        let child = parse_genome(&child_genome, None, None, &crate::config::test_config().brain)
            .brain
            .synapses;
        let inherited = inherit_learned_weights(
            &synapses,
            &parent.synapse_weights,
            &child,
            1.0,
            &mut crate::shared::rng::seeded(seed),
        );
        let child_offsets = offsets_by_key(&child_genome, &inherited);
        // The duplication shifts synapse indices, but every plastic parent offset lands on its own key; fixed reflexes
        // are never inherited
        for (s, o) in synapses.iter().zip(&offsets) {
            let c = child_offsets
                .get(&s.key)
                .expect("parent synapse kept by the duplication");
            let expected = if s.plastic { *o } else { 0.0 };
            assert!(
                (c - expected).abs() < 1e-5,
                "key {} offset {} -> {}",
                s.key,
                expected,
                c
            );
            checked += 1;
        }
        // The copy's new synapses start at their genome weight
        assert!(
            child_offsets
                .iter()
                .filter(|(k, _)| !synapses.iter().any(|s| s.key == **k))
                .all(|(_, &o)| o == 0.0)
        );
    }
    assert!(checked > 1000, "{} offsets checked", checked);
}

#[test]
fn test_lamarck_breeding_matches_learned_offsets_by_key() {
    let params = TrainerParams {
        population: 8,
        elite_ratio: 0.25,
        hof_rate: 0.0,
        inflow_rate: 0.0,
        lamarckian: true,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let (parent, synapses, offsets) = learned_parent(3);
    let sim = TrainerSim::restore(
        params,
        5,
        2,
        vec![parent.clone()],
        Vec::new(),
        200,
        crate::config::test_config(),
    );
    // Plastic parent offsets only; each homologous child synapse takes the full offset or none (TCK-152)
    let parent_offsets: std::collections::BTreeMap<u64, f32> = synapses
        .iter()
        .zip(&offsets)
        .filter(|(s, _)| s.plastic)
        .map(|(s, o)| (s.key, *o))
        .collect();
    let (mut matched, mut taken) = (0, 0);
    for sb in &sim.sandboxes {
        for (key, o) in offsets_by_key(&sb.agent().genome, &sb.agent().synapse_weights) {
            let expected = parent_offsets.get(&key).copied().unwrap_or(0.0);
            assert!(
                o == 0.0 || (o - expected).abs() < 1e-5,
                "sandbox {} key {}: offset {} expected {} or 0",
                sb.id,
                key,
                o,
                expected
            );
            matched += parent_offsets.contains_key(&key) as usize;
            taken += (o != 0.0) as usize;
        }
    }
    let share = taken as f32 / matched as f32;
    let chance = crate::config::test_config().genome.lamarckian_assimilation_chance;
    assert!(matched > 200, "{} homologous synapses", matched);
    assert!(
        (share - chance).abs() < 0.08,
        "inherited share {} vs chance {}",
        share,
        chance
    );
}

#[test]
fn test_phenotype_matches_genome_after_breeding() {
    let params = TrainerParams {
        multi_trial: false,
        lamarckian: true,
        ..small_params()
    };
    let mut sim = TrainerSim::new(params, 11, 200, crate::config::test_config());
    run(&mut sim, 1);
    let parents = sim.parents().to_vec();

    for sb in &sim.sandboxes {
        // Lamarckian sources are the parents (elites and mutants); hof and random get no methylations
        let methylations = match sb.origin_type {
            SandboxOrigin::Elite | SandboxOrigin::Mutant => parents.iter().map(|p| p.methylations.clone()).find(|m| {
                parse_genome(&sb.agent().genome, None, Some(m), &crate::config::test_config().brain)
                    == sb.agent().phenotype
            }),
            _ => None,
        };
        let expected = parse_genome(
            &sb.agent().genome,
            None,
            methylations.as_deref(),
            &crate::config::test_config().brain,
        );
        assert_eq!(
            sb.agent().phenotype,
            expected,
            "sandbox {} phenotype does not match its genome",
            sb.id
        );
    }
}

#[test]
fn test_assign_creature_rebuilds_agent() {
    let mut sim = TrainerSim::new(
        TrainerParams {
            population: 2,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        3,
        200,
        crate::config::test_config(),
    );
    let (px, heading) = (sim.sandboxes[1].agent().px, sim.sandboxes[1].agent().heading_angle);
    let assigned = sim.assign_creature(2, WILDTYPE_GENOME, None, None).unwrap();
    assert_eq!(assigned.sandbox_id, 2);
    let agent = assigned.agent;
    assert_eq!(
        agent.phenotype,
        parse_genome(WILDTYPE_GENOME, None, None, &crate::config::test_config().brain)
    );
    assert_eq!(agent.antisense, agent.phenotype.antisense_strand);
    assert_eq!(agent.generation, 1);
    // At a trial start the new creature takes the start pose of the trial's shared layout
    assert_eq!((agent.px, agent.heading_angle), (px, heading));
    assert!(sim.assign_creature(99, WILDTYPE_GENOME, None, None).is_none());
}

/// TCK-201 fix: before it, the assigned creature kept the old trial record (approach episode, rotation, coverage),
/// restarted the sandbox's trial clock mid-trial, and its slot kept the previous creature's trial fitness and lineage.
#[test]
fn test_assign_creature_starts_the_slot_fresh() {
    let params = TrainerParams {
        population: 4,
        trials: 2,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut sim = TrainerSim::restore(
        params,
        41,
        2,
        vec![lamarck_parent()],
        Vec::new(),
        200,
        crate::config::test_config(),
    );
    let epoch = sim.room().episode_ticks();
    for _ in 0..epoch + 40 {
        assert!(sim.step(false).is_none());
    }
    let slot = 1;
    let id = sim.sandboxes[slot].id;
    assert_eq!(
        sim.breeder.slot_lineage(slot).unwrap().0,
        Some(0),
        "slot 1 is a mutant of parent 0"
    );
    assert_eq!(sim.acc_fitness[slot].len(), 1);
    sim.sandboxes[slot].trial_mut().sight_distance = Some(100.0);
    sim.sandboxes[slot].trial_mut().approach_min_distance = 50.0;
    sim.assign_creature(id, WILDTYPE_GENOME, None, None).unwrap();
    let sb = &sim.sandboxes[slot];
    let (parent, lineage) = sim.breeder.slot_lineage(slot).unwrap();
    let state = (
        sb.trial().sight_distance,
        sb.trial().approach_min_distance,
        sb.trial().cumulative_rotation,
        sb.trial().coverage_mask,
        sb.chamber.tick == sim.epoch_ticks,
        sim.acc_fitness[slot].len(),
        parent,
        lineage.own.is_none(),
    );
    assert_eq!(
        state,
        (None, f32::MAX, 0.0, 0, true, 0, None, true),
        "stale trial record, trial clock, trial fitness or lineage after assign"
    );
    assert!(sim.trial_records.iter().all(|r| r.slot != slot));
    // The trial still ends for every sandbox together, and the generation completes
    assert!(sim.sandboxes.iter().all(|sb| sb.chamber.tick == sim.epoch_ticks));
    let mut completed = None;
    while completed.is_none() {
        completed = sim.step(false);
    }
}

#[test]
fn test_parallel_step_equals_sequential() {
    let params = TrainerParams {
        population: 12,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut seq = TrainerSim::new(params.clone(), 77, 200, crate::config::test_config());
    let mut par = TrainerSim::new(params, 77, 200, crate::config::test_config());
    let (mut seq_events, mut par_events) = (Vec::new(), Vec::new());
    while seq_events.len() < 2 {
        seq_events.extend(seq.step(false));
        par_events.extend(par.step(true));
    }
    assert_eq!(seq_events, par_events);
    assert_eq!(genomes(&seq), genomes(&par));
    let state = |sim: &TrainerSim| {
        sim.sandboxes
            .iter()
            .map(|s| {
                (
                    s.agent().px.to_bits(),
                    s.agent().py.to_bits(),
                    s.current_fitness.to_bits(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(state(&seq), state(&par));
}

#[test]
fn test_parallel_breeding_equals_sequential() {
    let params = TrainerParams {
        population: 20,
        multi_trial: false,
        lamarckian: true,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut sim = TrainerSim::new(params, 21, 200, crate::config::test_config());
    run(&mut sim, 1);
    let snapshot = |sim: &TrainerSim| serde_json::to_string(&sim.sandboxes).unwrap();
    let sequential = snapshot(&sim);
    assert!(sim.sandboxes.iter().any(|s| s.origin_type == SandboxOrigin::Mutant));

    sim.breed(true);
    assert_eq!(snapshot(&sim), sequential);
}

/// Start state of a sandbox's trial: heading, food positions, respawn seed (bit patterns).
fn trial_start(sb: &TrainerSandbox) -> (u32, Vec<(u32, u32)>, u64) {
    (
        sb.agent().heading_angle.to_bits(),
        sb.chamber
            .foods
            .iter()
            .map(|f| (f.x.to_bits(), f.y.to_bits()))
            .collect(),
        sb.chamber.respawn_seed,
    )
}

#[test]
fn test_layout_identical_across_sandboxes_and_trials_differ() {
    for scenario in [Scenario::Standard, Scenario::Exploration] {
        let mut sim = TrainerSim::new(
            TrainerParams {
                population: 6,
                scenario,
                ..TrainerParams::from_config(crate::config::test_config())
            },
            31,
            200,
            crate::config::test_config(),
        );
        let first = trial_start(&sim.sandboxes[0]);
        assert!(
            sim.sandboxes.iter().all(|sb| trial_start(sb) == first),
            "{:?} trial 0 differs between sandboxes",
            scenario
        );

        // Next trial: a new shared layout, again identical for all sandboxes
        let epoch = room(scenario.mode()).episode_ticks();
        for _ in 0..epoch {
            assert!(sim.step(false).is_none());
        }
        let second = trial_start(&sim.sandboxes[0]);
        assert_ne!(second, first);
        assert!(
            sim.sandboxes
                .iter()
                .all(|sb| trial_start(sb) == second && sb.agent().age == 0)
        );
    }
}

/// Runs one generation; at the end of every trial all sandboxes are marked as (not) having caught food.
fn forced_generation(sim: &mut TrainerSim, caught: bool) -> GenerationStats {
    loop {
        let epoch = sim.room().episode_ticks();
        for _ in 0..epoch - 1 {
            assert!(sim.step(false).is_none());
        }
        sim.epoch_ticks = epoch;
        sim.sandboxes.iter_mut().for_each(|sb| {
            sb.trial_mut().finished = caught;
            sb.trial_mut().finish_tick = caught.then_some(1);
        });
        if let Some(TrainerEvent::GenerationCompleted { stats, .. }) = sim.end_epoch(false) {
            return stats;
        }
    }
}

#[test]
fn test_layout_persists_across_generations_until_solved() {
    let params = TrainerParams {
        population: 6,
        trials: 2,
        layout_policy: LayoutPolicy::Hold,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut sim = TrainerSim::new(params, 32, 200, crate::config::test_config());
    let layout0 = trial_start(&sim.sandboxes[0]);

    // Not solved: the next generation is evaluated on the same layout
    let stats = forced_generation(&mut sim, false);
    assert_eq!((stats.layout_id, sim.layout_id()), (0, 0));
    assert_eq!(trial_start(&sim.sandboxes[0]), layout0);
    // Every elite caught food in every trial: the layout advances
    let stats = forced_generation(&mut sim, true);
    assert_eq!((stats.layout_id, sim.layout_id()), (0, 1));
    let layout1 = trial_start(&sim.sandboxes[0]);
    assert_ne!(layout1, layout0);
    assert!(sim.sandboxes.iter().all(|sb| trial_start(sb) == layout1));
}

#[test]
fn test_one_fifth_rule_adapts_ops_within_bounds() {
    let limit = crate::config::test_config().trainer.mutation_ops_limit;
    let params = TrainerParams {
        population: 10,
        one_fifth_rule: true,
        multi_trial: false,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut sim = TrainerSim::new(params, 33, 200, crate::config::test_config());
    let mut k_before = sim.params.mutation_ops_max;
    for _ in 0..4 {
        let (stats, _) = run(&mut sim, 1).remove(0);
        assert_eq!(stats.mutation_ops_max, k_before);
        let k = sim.params.mutation_ops_max;
        assert!((1..=limit).contains(&k));
        let expected = match stats.child_success_share {
            Some(share) if share > 0.2 => k_before + 1,
            Some(share) if share < 0.2 => k_before.saturating_sub(1),
            _ => k_before,
        };
        assert_eq!(k, expected.clamp(1, limit));
        k_before = k;
    }
    // Without the rule the ops bound stays fixed
    let mut fixed = TrainerSim::new(
        TrainerParams {
            population: 10,
            one_fifth_rule: false,
            multi_trial: false,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        33,
        200,
        crate::config::test_config(),
    );
    let k0 = fixed.params.mutation_ops_max;
    run(&mut fixed, 3);
    assert_eq!(fixed.params.mutation_ops_max, k0);
}

#[test]
fn test_hof_slots_are_mutated() {
    let params = TrainerParams {
        population: 10,
        elite_ratio: 0.1,
        hof_rate: 0.5,
        inflow_rate: 0.0,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let parent = lamarck_parent();
    // Distinct genomes: the hall of fame keeps every genome once (TCK-202)
    let hof: Vec<GenomeRecord> = (0..5)
        .map(|i| GenomeRecord {
            fitness: 50.0 - i as f32,
            genome: format!("{}{}", parent.genome, "A".repeat(i)),
            ..parent.clone()
        })
        .collect();
    let sim = TrainerSim::restore(
        params,
        34,
        2,
        vec![parent.clone()],
        hof,
        200,
        crate::config::test_config(),
    );
    let hof_slots: Vec<_> = sim
        .sandboxes
        .iter()
        .filter(|s| s.origin_type == SandboxOrigin::Hof)
        .collect();
    assert_eq!(hof_slots.len(), 5);
    assert!(
        hof_slots.iter().filter(|s| s.agent().genome != parent.genome).count() >= 4,
        "hof slots must be mutated"
    );
    assert_eq!(sim.sandboxes[0].agent().genome, parent.genome, "elite stays unmodified");
}

#[test]
fn test_avg_fitness_divides_by_the_sandbox_count() {
    let mut sim = TrainerSim::new(
        TrainerParams {
            population: 6,
            multi_trial: false,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        35,
        200,
        crate::config::test_config(),
    );
    // A population setting that no longer matches the sandboxes (e.g. changed without a rebuild)
    sim.params.population = 60;
    loop {
        if let Some(TrainerEvent::GenerationCompleted { avg_fitness, stats, .. }) = sim.step(false) {
            assert!(
                (avg_fitness - stats.mean_fitness).abs() <= 1e-3 * stats.mean_fitness.abs().max(1.0),
                "avg {} vs mean {}",
                avg_fitness,
                stats.mean_fitness
            );
            break;
        }
    }
}

#[test]
fn test_median_aggregation_is_deterministic_and_parallel_safe() {
    let params = TrainerParams {
        population: 8,
        trials: 3,
        trial_aggregation: TrialAggregation::Median,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut seq = TrainerSim::new(params.clone(), 36, 200, crate::config::test_config());
    let mut par = TrainerSim::new(params, 36, 200, crate::config::test_config());
    let (mut a, mut b) = (Vec::new(), Vec::new());
    while a.is_empty() {
        a.extend(seq.step(false));
        b.extend(par.step(true));
    }
    assert_eq!(a, b);
}

#[test]
fn test_live_telemetry_fitness_equals_epoch_end_fitness() {
    // UI path (fitness every tick) and runner path (epoch end only) use the same function: same events
    let params = TrainerParams {
        population: 8,
        trials: 2,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut live = TrainerSim::new(params.clone(), 37, 200, crate::config::test_config());
    let mut headless = TrainerSim::new(params, 37, 200, crate::config::test_config());
    headless.track_live_fitness = false;
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let mut expected_next: Option<Vec<f32>> = None;
    while a.is_empty() {
        a.extend(live.step(false));
        b.extend(headless.step(false));
        // The telemetry value of a tick is the room's score of the state the previous tick left
        if let Some(expected) = expected_next.take() {
            assert_eq!(
                live.sandboxes.iter().map(|sb| sb.current_fitness).collect::<Vec<_>>(),
                expected
            );
        }
        if a.is_empty() && live.epoch_ticks.is_multiple_of(50) {
            expected_next = Some(live.sandboxes.iter().map(|sb| sb.score()).collect());
        }
    }
    assert_eq!(a, b);
}

#[test]
fn test_generation_fitness_is_the_trial_mean_without_gate() {
    let params = TrainerParams {
        population: 6,
        trials: 3,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let epoch = room("standard").episode_ticks();
    let mut sim = TrainerSim::new(params, 38, 200, crate::config::test_config());
    let weights = crate::config::test_config().trainer.trainer_fitness.clone();
    let mut per_trial: Vec<Vec<f32>> = vec![Vec::new(); 6];
    loop {
        for _ in 0..epoch - 1 {
            assert!(sim.step(false).is_none());
        }
        for (i, sb) in sim.sandboxes.iter().enumerate() {
            // The last tick is stepped below; record the fitness the sim will compute at its end
            let mut probe = sb.clone();
            probe.step(crate::config::test_config());
            per_trial[i].push(crate::trainer::room::calculate_fitness(probe.trial(), epoch, &weights));
        }
        if let Some(TrainerEvent::GenerationCompleted { stats, .. }) = sim.step(false) {
            let mut expected: Vec<f32> = per_trial
                .iter()
                .map(|t| t.iter().sum::<f32>() / t.len() as f32)
                .collect();
            expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert!(
                (stats.best_fitness - expected[5]).abs() < 1e-5,
                "best {} vs {}",
                stats.best_fitness,
                expected[5]
            );
            assert!((stats.mean_fitness - expected.iter().sum::<f32>() / 6.0).abs() < 1e-5);
            break;
        }
    }
}

#[test]
fn test_minimal_criterion_population_is_deterministic_and_parallel_safe() {
    let params = TrainerParams {
        population: 10,
        minimal_criterion: true,
        multi_trial: false,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let seq = TrainerSim::new(params.clone(), 39, 200, crate::config::test_config());
    let mut par = TrainerSim::new(params.clone(), 39, 200, crate::config::test_config());
    par.breed(true);
    assert_eq!(genomes(&seq), genomes(&par));
    let config = crate::config::test_config();
    assert!(
        seq.sandboxes
            .iter()
            .all(|sb| crate::trainer::genomes::meets_minimal_criterion(&sb.agent().genome, config))
    );
    // Off: other genomes from the same seed (the plain generator)
    let plain = TrainerSim::new(
        TrainerParams {
            minimal_criterion: false,
            ..params
        },
        39,
        200,
        crate::config::test_config(),
    );
    assert_ne!(genomes(&seq), genomes(&plain));
}

#[test]
fn test_mutation_channel_stats_count_mutants_and_separate_is_parallel_safe() {
    for mode in [MutationChannelMode::Mixed, MutationChannelMode::Separate] {
        let params = TrainerParams {
            population: 12,
            multi_trial: false,
            mutation_channels: mode,
            ..TrainerParams::from_config(crate::config::test_config())
        };
        let mut seq = TrainerSim::new(params.clone(), 55, 200, crate::config::test_config());
        let mut par = TrainerSim::new(params, 55, 200, crate::config::test_config());
        let (mut seq_events, mut par_events) = (Vec::new(), Vec::new());
        while seq_events.len() < 3 {
            seq_events.extend(seq.step(false));
            par_events.extend(par.step(true));
        }
        assert_eq!(seq_events, par_events, "{:?}", mode);
        for event in &seq_events {
            let TrainerEvent::GenerationCompleted {
                generation,
                elites,
                stats,
                ..
            } = event;
            let c = &stats.mutation_channels;
            let counted = c.body.mutants + c.brain.mutants + c.both.mutants + c.neutral.mutants;
            assert_eq!(stats.elite_last_mutation.len(), elites.len());
            if *generation == 1 {
                assert_eq!(counted, 0, "generation 1 has no mutants");
                assert!(stats.elite_last_mutation.iter().all(Option::is_none));
            } else {
                assert!(counted > 0, "{:?} generation {}", mode, generation);
            }
            if mode == MutationChannelMode::Separate {
                assert_eq!(c.both.mutants, 0, "separate: no child touches both channels");
                assert!(
                    stats
                        .elite_last_mutation
                        .iter()
                        .flatten()
                        .all(|&ch| ch != MutationChannel::Both)
                );
            }
        }
    }
}

// ---------------------------------------------------------------- curriculum (TCK-176)

fn curriculum_params(population: usize, level: f32, level_auto: bool) -> TrainerParams {
    TrainerParams {
        population,
        scenario: Scenario::Curriculum,
        curriculum: plain_curriculum(),
        level,
        level_auto,
        ..TrainerParams::from_config(crate::config::test_config())
    }
}

/// Events with the training mode blanked (the only field that names the mode).
fn without_mode(events: Vec<TrainerEvent>) -> Vec<TrainerEvent> {
    events
        .into_iter()
        .map(
            |TrainerEvent::GenerationCompleted {
                 generation,
                 best_fitness,
                 avg_fitness,
                 best_genome,
                 elites,
                 mut stats,
             }| {
                stats.scenario = TrainingMode::Room("standard");
                TrainerEvent::GenerationCompleted {
                    generation,
                    best_fitness,
                    avg_fitness,
                    best_genome,
                    elites,
                    stats,
                }
            },
        )
        .collect()
}

fn events(sim: &mut TrainerSim, generations: usize, parallel: bool) -> Vec<TrainerEvent> {
    let mut out = Vec::new();
    while out.len() < generations {
        out.extend(sim.step(parallel));
    }
    out
}

#[test]
fn test_curriculum_levels_zero_and_one_reproduce_the_fixed_rooms() {
    for (level, fixed) in [(0.0, Scenario::Standard), (1.0, Scenario::Exploration)] {
        let mut fixed_room = TrainerSim::new(
            TrainerParams {
                population: 4,
                scenario: fixed,
                ..TrainerParams::from_config(crate::config::test_config())
            },
            61,
            200,
            crate::config::test_config(),
        );
        let mut curriculum = TrainerSim::new(
            curriculum_params(4, level, false),
            61,
            200,
            crate::config::test_config(),
        );
        assert_eq!(
            room_values(curriculum.room()),
            room_values(room(fixed.mode()).as_ref()),
            "level {} is the {:?} room",
            level,
            fixed
        );
        assert!(
            curriculum
                .sandboxes
                .iter()
                .zip(&fixed_room.sandboxes)
                .all(|(a, b)| trial_start(a) == trial_start(b)),
            "same layouts"
        );
        let (a, b) = (events(&mut fixed_room, 2, false), events(&mut curriculum, 2, false));
        assert_eq!(
            without_mode(a),
            without_mode(b),
            "level {} differs from {:?}",
            level,
            fixed
        );
    }
}

#[test]
fn test_curriculum_gate_moves_the_level_of_the_next_generation() {
    let mut sim = TrainerSim::new(
        TrainerParams {
            trials: 1,
            ..curriculum_params(4, 0.0, true)
        },
        62,
        200,
        crate::config::test_config(),
    );
    let stats = forced_generation(&mut sim, true);
    assert_eq!((stats.level, stats.level_move), (0.0, Some(LevelMove::Up)));
    assert_eq!(sim.params.level, 0.1);
    assert!(
        sim.sandboxes
            .iter()
            .all(|sb| room_values(sb.room.as_ref()) == level_room(0.1)),
        "next generation runs at the new level"
    );
    assert_eq!(forced_generation(&mut sim, true).level, 0.1);
    let stats = forced_generation(&mut sim, false);
    assert_eq!((stats.level, stats.level_move), (0.2, Some(LevelMove::Down)));
    assert_eq!(room_values(sim.room()), level_room(0.1));
    // Manual level: gate off, level fixed; curriculum mode again turns it back on
    sim.set_level(0.7);
    assert_eq!(
        (sim.params.level, sim.params.level_auto, room_values(sim.room())),
        (0.7, false, level_room(0.7))
    );
    let stats = forced_generation(&mut sim, true);
    assert_eq!((stats.level, stats.level_move, sim.params.level), (0.7, None, 0.7));
    sim.set_scenario(Scenario::Curriculum);
    assert!(sim.params.level_auto);
    assert_eq!(forced_generation(&mut sim, true).level_move, Some(LevelMove::Up));
    assert_eq!(sim.params.level, 0.8);
    // Fixed rooms have no gate
    let mut fixed = TrainerSim::new(
        TrainerParams {
            population: 4,
            trials: 1,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        62,
        200,
        crate::config::test_config(),
    );
    assert_eq!(forced_generation(&mut fixed, true).level_move, None);
}

#[test]
fn test_room_switch_waits_for_the_generation_boundary() {
    let mut sim = TrainerSim::new(
        TrainerParams {
            population: 4,
            trials: 2,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        63,
        200,
        crate::config::test_config(),
    );
    let standard = room_values(room("standard").as_ref());
    for _ in 0..10 {
        sim.step(false);
    }
    sim.set_scenario(Scenario::Exploration);
    assert_eq!(sim.pending_scenario(), Some(Scenario::Exploration));
    assert_eq!(
        (sim.params.scenario, room_values(sim.room())),
        (Scenario::Standard, standard)
    );
    // The rest of the generation (both trials) still runs in the old room
    let mut completed = None;
    while completed.is_none() {
        assert!(sim.sandboxes.iter().all(|sb| room_values(sb.room.as_ref()) == standard));
        completed = sim.step(false);
    }
    let Some(TrainerEvent::GenerationCompleted { stats, .. }) = completed else {
        unreachable!()
    };
    assert_eq!(stats.scenario, TrainingMode::Room("standard"));
    let exploration = room("exploration");
    assert_eq!(
        (
            sim.params.scenario,
            sim.room().id(),
            sim.pending_scenario(),
            sim.epoch_ticks
        ),
        (Scenario::Exploration, "exploration", None, 0)
    );
    assert!(sim.sandboxes.iter().all(|sb| sb.room.id() == "exploration"
        && sb.chamber.tick == 0
        && sb.agent().px == exploration.chamber_size() / 2.0));
}

#[test]
fn test_room_switch_at_a_generation_start_applies_at_once() {
    // A fresh generation is at its boundary: switching equals starting in the new room
    let mut switched = TrainerSim::new(
        TrainerParams {
            population: 4,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        64,
        200,
        crate::config::test_config(),
    );
    switched.set_scenario(Scenario::Exploration);
    assert_eq!(switched.pending_scenario(), None);
    let mut direct = TrainerSim::new(
        TrainerParams {
            population: 4,
            scenario: Scenario::Exploration,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        64,
        200,
        crate::config::test_config(),
    );
    assert!(
        switched
            .sandboxes
            .iter()
            .zip(&direct.sandboxes)
            .all(|(a, b)| trial_start(a) == trial_start(b)
                && room_values(a.room.as_ref()) == room_values(b.room.as_ref()))
    );
    assert_eq!(events(&mut switched, 1, false), events(&mut direct, 1, false));
}

#[test]
fn test_set_mode_selects_any_registry_room_and_the_ui_mode_follows() {
    let mut sim = TrainerSim::new(
        TrainerParams {
            population: 4,
            scenario: Scenario::Curriculum,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        66,
        200,
        crate::config::test_config(),
    );
    sim.set_mode(TrainingMode::parse("exploration").unwrap());
    assert_eq!(
        (sim.params.scenario, sim.params.room, sim.params.mode()),
        (Scenario::Exploration, None, TrainingMode::Room("exploration"))
    );
    assert_eq!(sim.room().id(), "exploration");
    // A room without a UI mode is carried in `room` (simulated with a registry id under another UI mode)
    let mut params = TrainerParams::from_config(crate::config::test_config());
    params.room = Some("exploration");
    assert_eq!(params.mode(), TrainingMode::Room("exploration"));
    params.set_mode(TrainingMode::Curriculum);
    assert_eq!((params.scenario, params.room), (Scenario::Curriculum, None));
}

#[test]
fn test_layout_rotates_every_generation_with_rotate_policy() {
    let params = TrainerParams {
        population: 6,
        trials: 2,
        layout_policy: LayoutPolicy::Rotate,
        ..TrainerParams::from_config(crate::config::test_config())
    };
    let mut sim = TrainerSim::new(params, 21, 200, crate::config::test_config());
    for expected in 1..=3u64 {
        while !matches!(sim.step(false), Some(TrainerEvent::GenerationCompleted { .. })) {}
        assert_eq!(sim.layout_id(), expected, "a new layout after every generation");
    }
}

#[test]
fn test_rehearsal_trials_run_at_the_rehearsal_level_and_skip_the_gate() {
    let c = CurriculumParams {
        rehearsal_share: 1.0 / 3.0,
        rehearsal_level: 0.0,
        ..plain_curriculum()
    };
    let params = TrainerParams {
        curriculum: c,
        trials: 3,
        ..curriculum_params(4, 0.5, false)
    };
    let mut sim = TrainerSim::new(params.clone(), 65, 200, crate::config::test_config());
    let rehearsal = rehearsal_trials(65, 0, 3, c.rehearsal_share);
    assert_eq!(rehearsal.len(), 1);
    let mut levels = Vec::new();
    let stats = loop {
        if sim.epoch_ticks == 0 {
            levels.push(sim.room().level());
            assert!(
                sim.sandboxes
                    .iter()
                    .all(|sb| room_values(sb.room.as_ref()) == room_values(sim.room()))
            );
        }
        if let Some(TrainerEvent::GenerationCompleted { stats, .. }) = sim.step(false) {
            break stats;
        }
    };
    let expected: Vec<f32> = (0..3).map(|t| if rehearsal.contains(&t) { 0.0 } else { 0.5 }).collect();
    assert_eq!(levels, expected);
    assert_eq!((stats.level, stats.rehearsal_trials), (0.5, 1));
    assert!(stats.rehearsal_success_rate.is_some());
    // Same run, parallel: identical
    let mut par = TrainerSim::new(params, 65, 200, crate::config::test_config());
    let mut seq = TrainerSim::new(sim.params.clone(), 65, 200, crate::config::test_config());
    assert_eq!(events(&mut par, 2, true), events(&mut seq, 2, false));
}

#[test]
fn test_metabolic_ramp_scales_curriculum_rooms_only() {
    let config = crate::config::test_config();
    let scale = |r: &dyn Room| r.world_settings(config).metabolic_scale / config.metabolism.trainer_metabolic_scale;
    let c = CurriculumParams {
        metabolic_ramp_start: 0.0,
        ..plain_curriculum()
    };
    let sim = TrainerSim::new(
        TrainerParams {
            curriculum: c,
            ..curriculum_params(2, 0.4, false)
        },
        66,
        200,
        config,
    );
    assert!((scale(sim.room()) - 0.4).abs() < 1e-6);
    assert!(
        sim.sandboxes
            .iter()
            .all(|sb| scale(sb.room.as_ref()) == scale(sim.room()))
    );
    let fixed = TrainerSim::new(
        TrainerParams {
            population: 2,
            curriculum: c,
            ..TrainerParams::from_config(config)
        },
        66,
        200,
        config,
    );
    assert_eq!(scale(fixed.room()), 1.0);
}

#[test]
fn test_evaluate_population_is_deterministic_and_leaves_the_sim_unchanged() {
    let mut sim = TrainerSim::new(
        TrainerParams {
            population: 6,
            trials: 2,
            ..TrainerParams::from_config(crate::config::test_config())
        },
        67,
        200,
        crate::config::test_config(),
    );
    run(&mut sim, 1);
    let before = serde_json::to_string(&sim.sandboxes).unwrap();
    let standard = room("standard");
    let a = sim.evaluate_population(&standard, None, false);
    assert_eq!(a, sim.evaluate_population(&standard, None, true));
    assert_eq!(serde_json::to_string(&sim.sandboxes).unwrap(), before);
    assert_eq!((a.trials, a.level), (2, 0.0));
    assert!((0.0..=1.0).contains(&a.success_rate));
    assert!(a.elite_success_rate.is_some(), "generation 2 has elite slots");
    assert_eq!(sim.evaluate_population(&standard, Some(4), false).trials, 4);
}

// ---------------------------------------------------------------- run state (TCK-191)

#[test]
fn test_restore_with_run_state_continues_layout_level_mode_and_mutation_ops() {
    let params = TrainerParams {
        layout_policy: LayoutPolicy::Rotate,
        mutation_ops_max: 5,
        trials: 2,
        ..curriculum_params(6, 0.3, true)
    };
    let mut original = TrainerSim::new(params.clone(), 71, 200, crate::config::test_config());
    run(&mut original, 2);
    let state = TrainerRunState::of(&original);
    assert_eq!(state.layout_id, 2, "rotate: a new layout after every generation");

    // A restore from other params (app start: config defaults, other mode and ops) with the saved state applied
    let mut restored_params = TrainerParams {
        scenario: Scenario::Standard,
        level: 0.0,
        level_auto: false,
        mutation_ops_max: 2,
        ..params.clone()
    };
    state.apply(&mut restored_params, crate::config::test_config());
    let restored = TrainerSim::restore_at_layout(
        restored_params,
        71,
        original.generation,
        state.layout_id,
        original.parents().to_vec(),
        original.hof().to_vec(),
        200,
        crate::config::test_config(),
    );
    assert_eq!(TrainerRunState::of(&restored), state);
    assert_eq!(
        (restored.params.scenario, restored.params.level_auto),
        (Scenario::Curriculum, true)
    );
    assert_eq!(
        (room_values(restored.room()), &restored.rehearsal),
        (room_values(original.room()), &original.rehearsal)
    );
    assert_eq!(genomes(&restored), genomes(&original));
    assert!(
        restored
            .sandboxes
            .iter()
            .zip(&original.sandboxes)
            .all(|(a, b)| a.agent().px == b.agent().px && a.agent().py == b.agent().py)
    );

    // No saved state: today's restore on layout 0
    let fresh = TrainerSim::restore(
        params,
        71,
        original.generation,
        original.parents().to_vec(),
        original.hof().to_vec(),
        200,
        crate::config::test_config(),
    );
    assert_eq!(fresh.layout_id(), 0);
}

#[test]
fn test_run_state_apply_clamps_mutation_ops_and_snaps_level() {
    let state = TrainerRunState {
        scenario: Scenario::Exploration,
        level: 0.47,
        level_auto: false,
        layout_id: 9,
        mutation_ops_max: 10_000,
        population: Some(42),
        seed: Some(7),
    };
    let mut params = TrainerParams::from_config(crate::config::test_config());
    params.room = Some("standard");
    state.apply(&mut params, crate::config::test_config());
    let limit = crate::config::test_config()
        .trainer
        .mutation_ops_limit
        .max(params.mutation_ops_min.max(1));
    assert_eq!(
        (
            params.scenario,
            params.level,
            params.level_auto,
            params.mutation_ops_max
        ),
        (Scenario::Exploration, snap_level(0.47), false, limit)
    );
    assert_eq!(params.population, 42, "the stored population is restored (TCK-192)");
    assert_eq!(
        params.mode(),
        TrainingMode::Room("exploration"),
        "the stored UI mode replaces a headless room"
    );
}

/// TCK-194: without a nonce the run seed is the one used since TCK-173; a reset nonce gives a new generation 1.
#[test]
fn test_run_seed_nonce_draws_a_new_generation_one() {
    assert_eq!(run_seed("BASE", "run", None), seed_from_str("BASE:run"));
    let genomes = |seed: u64| {
        TrainerSim::new(small_params(), seed, 200, crate::config::test_config())
            .sandboxes
            .iter()
            .map(|sb| sb.agent().genome.clone())
            .collect::<Vec<_>>()
    };
    let replay = genomes(run_seed("BASE", "run", None));
    assert_eq!(
        genomes(run_seed("BASE", "run", None)),
        replay,
        "same seed: identical run"
    );
    let reset_a = genomes(run_seed("BASE", "run", Some(1)));
    let reset_b = genomes(run_seed("BASE", "run", Some(2)));
    assert_ne!(reset_a, replay);
    assert_ne!(reset_a, reset_b);
    assert_eq!(
        TrainerRunState::of(&TrainerSim::new(small_params(), 99, 200, crate::config::test_config())).seed,
        Some(99)
    );
}
