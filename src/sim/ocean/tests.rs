//! Headless Ocean tests (TCK-156): possible since the Ocean step no longer needs a Tauri window or a database.

use super::*;
use crate::biology::genome::WILDTYPE_GENOME;
use crate::shared::dto::FoodSpore;

fn config() -> &'static AppConfig {
    crate::config::test_config()
}

fn founder(sim: &mut OceanSim, x: f32, y: f32) -> usize {
    let mut out = OceanStep::default();
    sim.add_founder(
        Founder::Genome {
            genome: WILDTYPE_GENOME,
            methylations: None,
            synapse_weights: None,
        },
        &mut derive(1, &[]),
        &mut out,
    );
    let agent = sim.creatures.last_mut().unwrap();
    agent.px = x;
    agent.py = y;
    agent.vx = 0.0;
    agent.vy = 0.0;
    sim.creatures.len() - 1
}

fn spore(id: u32, type_id: u32, x: f32, y: f32) -> FoodSpore {
    FoodSpore {
        id,
        type_id,
        x,
        y,
        amount: 15.0,
        vx: 0.0,
        vy: 0.0,
    }
}

fn snapshot_json(sim: &OceanSim) -> String {
    serde_json::to_string(&sim.snapshot()).unwrap()
}

#[test]
fn test_creature_eats_a_spore_in_reach_and_the_spore_respawns() {
    let mut sim = OceanSim::empty(config(), 1);
    let idx = founder(&mut sim, 5000.0, 6000.0);
    sim.spores.spores = vec![spore(1, 1, 5000.0, 6000.0), spore(2, 1, 9000.0, 9000.0)];
    let step = sim.step();
    assert_eq!(
        step.events.first(),
        Some(&OceanEvent::Ate { x: 5000.0, y: 6000.0 }),
        "{:?}",
        step.events
    );
    assert!(sim.creatures[idx].has_eaten, "eating counts for mitosis");
    let eaten = &sim.spores.spores[0];
    assert!(
        (eaten.x, eaten.y) != (5000.0, 6000.0),
        "the eaten spore respawned elsewhere"
    );
    assert_eq!(sim.creatures[idx].age, 1);
}

#[test]
fn test_starved_creature_dies_and_decomposes_into_meat_near_its_corpse() {
    let mut sim = OceanSim::empty(config(), 2);
    let idx = founder(&mut sim, 3000.0, 9000.0);
    sim.creatures[idx].energy = 1e-6;
    sim.spores.spores = (0..20).map(|i| spore(i, 1, 15000.0, 1000.0 + i as f32)).collect();
    let id = sim.creatures[idx].id;
    let step = sim.step();
    assert!(
        step.events
            .iter()
            .any(|e| matches!(e, OceanEvent::Died { id: dead, cause: DeathCause::Starved, .. } if *dead == id)),
        "{:?}",
        step.events
    );
    assert!(sim.creatures.is_empty());
    let meat: Vec<&FoodSpore> = sim.spores.spores.iter().filter(|s| s.type_id == 2).collect();
    assert!(
        !meat.is_empty()
            && meat
                .iter()
                .all(|s| (s.x - 3000.0).abs() <= 16.0 && (s.y - 9000.0).abs() <= 16.0)
    );
    // The species census reports the extinction of the only species
    assert!(step.events.iter().any(|e| matches!(e, OceanEvent::Extinct { .. })));
    assert!(step.species.iter().any(|c| matches!(c, SpeciesChange::Extinct { .. })));
}

#[test]
fn test_bite_damage_kills_a_weak_victim_as_hunted() {
    let mut sim = OceanSim::empty(config(), 3);
    let attacker = founder(&mut sim, 4000.0, 6000.0);
    let victim = founder(&mut sim, 4002.0, 6000.0);
    sim.creatures[attacker].phenotype.carnivory = 1.0;
    sim.creatures[attacker].energy = 50.0;
    sim.creatures[victim].species_id = "prey".into();
    sim.creatures[victim].phenotype.carnivory = 0.0;
    sim.creatures[victim].energy = 1.0;
    let victim_id = sim.creatures[victim].id;
    let step = sim.step();
    assert!(
        step.events
            .iter()
            .any(|e| matches!(e, OceanEvent::Bite { victim, .. } if *victim == victim_id)),
        "{:?}",
        step.events
    );
    assert!(
        step.events
            .iter()
            .any(|e| matches!(e, OceanEvent::Died { id, cause: DeathCause::Hunted, .. } if *id == victim_id))
    );
    assert_eq!(sim.creatures.len(), 1);
    assert!(sim.creatures[0].has_eaten, "a bite counts as eating");
}

#[test]
fn test_restock_refills_from_the_catalogue_and_records_species() {
    let mut sim = OceanSim::empty(config(), 4);
    let catalogue = vec![CatalogueFounder {
        name: "Saved".into(),
        genome: WILDTYPE_GENOME.into(),
        methylations: vec![],
        synapse_weights: vec![],
    }];
    let step = sim.tick(|| catalogue.clone());
    let target = config().ocean.target_population;
    assert_eq!((sim.creatures.len(), step.born.len()), (target, target));
    assert!(
        step.events
            .iter()
            .all(|e| matches!(e, OceanEvent::Restocked { catalogue_name: Some(n), .. } if n == "Saved"))
    );
    assert_eq!(
        step.species
            .iter()
            .filter(|c| matches!(c, SpeciesChange::Discovered { .. }))
            .count(),
        1,
        "one genome, one species"
    );
    // Empty catalogue: structured random founders, each a new species
    let mut sim = OceanSim::empty(config(), 4);
    let step = sim.tick(Vec::new);
    assert!(step.events.iter().all(|e| matches!(
        e,
        OceanEvent::Restocked {
            catalogue_name: None,
            ..
        }
    )));
    assert!(step.species.len() > 1);
    // At the target population the catalogue is not read
    sim.tick(|| panic!("catalogue read without a missing creature"));
}

#[test]
fn test_same_seed_same_ocean_and_snapshot_continues_identically() {
    let run = |seed: u64, ticks: usize| {
        let (mut sim, _) = OceanSim::fresh(config(), seed);
        for _ in 0..ticks {
            sim.tick(Vec::new);
        }
        sim
    };
    let a = run(7, 60);
    assert_eq!(snapshot_json(&a), snapshot_json(&run(7, 60)));
    assert_ne!(snapshot_json(&a), snapshot_json(&run(8, 60)));

    // Snapshot after 30 ticks, restored and continued == 60 ticks in one go
    let mut restored = OceanSim::restore(config(), run(7, 30).snapshot(), 0);
    for _ in 0..30 {
        restored.tick(Vec::new);
    }
    assert_eq!(snapshot_json(&restored), snapshot_json(&a));
}

#[test]
fn test_fresh_reset_and_release_rules() {
    let (mut sim, out) = OceanSim::fresh(config(), 5);
    assert_eq!(sim.creatures.len(), FRESH_FOUNDERS);
    assert_eq!(sim.spores.spores.len(), config().ocean.food_spore_count);
    assert_eq!(sim.spores.nutrient_centers.len(), spore_field::NUTRIENT_CENTER_COUNT);
    assert!(
        out.born.is_empty(),
        "founders of a fresh Ocean reach the UI with INIT_STATE"
    );
    assert!(!out.species.is_empty());

    let next_spore = sim.spores.next_id;
    let out = sim.reset(99, &[]);
    assert_eq!(
        (sim.creatures.len(), sim.seed(), sim.tick_count()),
        (RESET_FOUNDERS, 99, 0)
    );
    assert_eq!(sim.creatures[0].id, 1, "creature ids restart");
    assert_eq!(sim.spores.spores[0].id, next_spore, "spore ids continue");
    assert_eq!(out.species.len(), sim.ledger.entries().len());

    let released = sim.release(Founder::Random).unwrap();
    assert_eq!(released.born[0].id, sim.creatures.last().unwrap().id);
    while sim.creatures.len() < config().ocean.max_creatures {
        sim.release(Founder::Random).unwrap();
    }
    assert_eq!(
        sim.release(Founder::Random).unwrap_err(),
        SpawnRejected::PopulationCap(config().ocean.max_creatures)
    );
}

#[test]
fn test_snapshot_of_an_earlier_version_restores_with_new_centres() {
    let agent = OceanSim::fresh(config(), 6).0.creatures[0].clone();
    let old = serde_json::json!({ "creatures": [agent], "foodPellets": [], "highestGeneration": 3, "nextCreatureId": 9, "nextSporeId": 1300 });
    let snapshot: OceanSnapshot = serde_json::from_value(old).unwrap();
    assert_eq!(
        (snapshot.seed, snapshot.tick, snapshot.nutrient_centers.len()),
        (None, 0, 0)
    );
    let sim = OceanSim::restore(config(), snapshot, 42);
    assert_eq!(
        (
            sim.seed(),
            sim.highest_generation(),
            sim.next_creature_id,
            sim.spores.next_id
        ),
        (42, 3, 9, 1300)
    );
    assert_eq!(sim.spores.nutrient_centers.len(), spore_field::NUTRIENT_CENTER_COUNT);
    assert_eq!(
        sim.snapshot().nutrient_centers,
        sim.spores.nutrient_centers,
        "centres are persisted now"
    );
}

/// Makes creature `idx` ready to split this tick: mature (after the step's age + 1), fed and with a full stomach.
fn ready_to_split(sim: &mut OceanSim, idx: usize) {
    let rule = MitosisRule::from_ocean(&config().ocean);
    let agent = &mut sim.creatures[idx];
    agent.age = agent.phenotype.mature_age.max(rule.mature_age_floor);
    agent.has_eaten = true;
    agent.energy = agent.phenotype.stomach_capacity;
}

#[test]
fn test_mitosis_splits_a_ready_parent_and_adds_the_child_to_the_population() {
    let rule = MitosisRule::from_ocean(&config().ocean);
    let mut sim = OceanSim::empty(config(), 9);
    let idx = founder(&mut sim, 7000.0, 5000.0);
    ready_to_split(&mut sim, idx);
    let parent = sim.creatures[idx].clone();
    assert!(
        parent.phenotype.repro_threshold.max(rule.stomach_threshold_floor) < 0.99,
        "the step's drain must not drop it below the threshold"
    );
    let next_id = sim.next_creature_id;

    let step = sim.step();
    let after = &sim.creatures[idx];
    let mitosis = step.events.iter().find_map(|e| match e {
        OceanEvent::Mitosis {
            parent: p,
            child_name,
            generation,
            mutated,
        } => Some((*p, child_name.clone(), *generation, *mutated)),
        _ => None,
    });
    let (parent_id, child_name, generation, mutated) =
        mitosis.unwrap_or_else(|| panic!("no mitosis: {:?}", step.events));
    assert_eq!((parent_id, generation), (parent.id, parent.generation + 1));
    assert!(!after.has_eaten, "the split resets the fed flag");
    assert!(
        after.energy < parent.energy * rule.split_loss_ratio + 1e-3,
        "the split costs energy"
    );

    // The child joins the population with the next id at the parent's place and the parent's energy, and is reported
    // once for the UI (TCK-143)
    let [child] = &step.born[..] else {
        panic!("one child: {:?}", step.born)
    };
    assert_eq!(
        (child.id, child.generation, &child.phenotype.latin_name),
        (next_id, generation, &child_name)
    );
    assert_eq!((child.px, child.py, child.energy), (after.px, after.py, after.energy));
    assert_eq!(sim.creatures.len(), 2, "the child is simulated");
    assert_eq!(sim.creatures[1].id, child.id);
    let (pv, cv) = ((after.vx, after.vy), (child.vx, child.vy));
    assert!(
        pv.0 * cv.0 + pv.1 * cv.1 < 0.0,
        "parent and child drift apart: {:?} {:?}",
        pv,
        cv
    );
    assert_eq!(
        (sim.next_creature_id, sim.highest_generation()),
        (next_id + 1, generation)
    );
    // A new species is recorded only for a mutated child, under the parent's name
    for change in &step.species {
        if let SpeciesChange::Discovered { parent_name, .. } = change {
            assert!(
                mutated && parent_name.as_deref() == Some(parent.phenotype.latin_name.as_str()),
                "{:?}",
                change
            );
        }
    }

    // At `ocean.maxCreatures` a ready creature does not split
    let mut sim = OceanSim::empty(config(), 9);
    for i in 0..config().ocean.max_creatures {
        founder(&mut sim, 500.0 + 300.0 * i as f32, 9000.0);
    }
    ready_to_split(&mut sim, 0);
    let step = sim.step();
    assert!(
        !step.events.iter().any(|e| matches!(e, OceanEvent::Mitosis { .. })) && step.born.is_empty(),
        "population cap"
    );
    assert!(sim.creatures[0].has_eaten, "no split happened");

    // One place below the cap and two ready parents: only the first splits (the cap counts this tick's children)
    let mut sim = OceanSim::empty(config(), 9);
    for i in 0..config().ocean.max_creatures - 1 {
        founder(&mut sim, 500.0 + 300.0 * i as f32, 9000.0);
    }
    ready_to_split(&mut sim, 0);
    ready_to_split(&mut sim, 1);
    let step = sim.step();
    assert_eq!(step.born.len(), 1, "{:?}", step.events);
    assert_eq!(sim.creatures.len(), config().ocean.max_creatures);
    assert!(sim.creatures[1].has_eaten, "the second parent did not split");
}

/// One mitosis of a ready wildtype parent under `seed`; returns the Ocean after the split tick, the parent's index and
/// the step.
fn split_once(seed: u64) -> (OceanSim, usize, OceanStep) {
    let mut sim = OceanSim::empty(config(), seed);
    let idx = founder(&mut sim, 7000.0, 5000.0);
    ready_to_split(&mut sim, idx);
    let step = sim.step();
    assert!(
        step.events.iter().any(|e| matches!(e, OceanEvent::Mitosis { .. })),
        "no mitosis: {:?}",
        step.events
    );
    (sim, idx, step)
}

#[test]
fn test_mitosis_child_is_simulated_and_its_species_is_no_phantom() {
    let (mut seen_clone, mut seen_new_species) = (false, false);
    for seed in 0..64 {
        let (mut sim, idx, step) = split_once(seed);
        let parent = sim.creatures[idx].clone();
        let [born] = &step.born[..] else {
            panic!("one child for the UI: {:?}", step.born)
        };
        // RV-B01: the child is part of the population and simulated from the next tick
        assert_eq!(sim.creatures.len(), 2, "seed {}: the child joins the population", seed);
        let child = sim.creatures.iter().find(|c| c.id == born.id).expect("child simulated");
        assert_ne!(child.id, parent.id);
        assert_eq!(child.age, 0, "born this tick");
        let discovered = step.species.iter().any(
            |c| matches!(c, SpeciesChange::Discovered { latin_name, .. } if *latin_name == born.phenotype.latin_name),
        );
        if born.species_id == parent.species_id {
            seen_clone = true;
            assert!(!discovered, "seed {}: a clone joins the parent's species", seed);
        } else if discovered {
            seen_new_species = true;
        }
        let next = sim.step();
        let child = sim.creatures.iter().find(|c| c.id == born.id);
        assert_eq!(
            child.map(|c| c.age),
            Some(1),
            "seed {}: the child ages in the next tick",
            seed
        );
        // RV-D03: no species is flagged extinct while its only member lives
        for change in step.species.iter().chain(&next.species) {
            assert!(
                !matches!(change, SpeciesChange::Extinct { .. }),
                "seed {}: phantom extinction {:?}",
                seed,
                change
            );
        }
        if discovered {
            let entry = sim.ledger.entries().iter().find(|e| e.id == born.species_id);
            assert!(
                entry.is_some_and(|e| e.alive),
                "seed {}: the new species is alive",
                seed
            );
        }
    }
    assert!(seen_clone && seen_new_species, "both cases covered");
}
