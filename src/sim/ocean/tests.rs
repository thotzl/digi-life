//! Headless Ocean tests (TCK-156): possible since the Ocean step no longer needs a Tauri window or a database.

use super::*;
use crate::biology::genome::WILDTYPE_GENOME;
use crate::shared::dto::FoodSpore;

fn config() -> &'static AppConfig {
    crate::config::test_config()
}

fn founder(sim: &mut OceanSim, x: f32, y: f32) -> usize {
    let mut out = OceanStep::default();
    sim.add_founder(Founder::Genome { genome: WILDTYPE_GENOME, methylations: None, synapse_weights: None }, &mut derive(1, &[]), &mut out);
    let agent = sim.creatures.last_mut().unwrap();
    agent.px = x;
    agent.py = y;
    agent.vx = 0.0;
    agent.vy = 0.0;
    sim.creatures.len() - 1
}

fn spore(id: u32, type_id: u32, x: f32, y: f32) -> FoodSpore {
    FoodSpore { id, type_id, x, y, amount: 15.0, vx: 0.0, vy: 0.0 }
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
    assert_eq!(step.events.first(), Some(&OceanEvent::Ate { x: 5000.0, y: 6000.0 }), "{:?}", step.events);
    assert!(sim.creatures[idx].has_eaten, "eating counts for mitosis");
    let eaten = &sim.spores.spores[0];
    assert!((eaten.x, eaten.y) != (5000.0, 6000.0), "the eaten spore respawned elsewhere");
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
    assert!(step.events.iter().any(|e| matches!(e, OceanEvent::Died { id: dead, hunted: false, .. } if *dead == id)), "{:?}", step.events);
    assert!(sim.creatures.is_empty());
    let meat: Vec<&FoodSpore> = sim.spores.spores.iter().filter(|s| s.type_id == 2).collect();
    assert!(!meat.is_empty() && meat.iter().all(|s| (s.x - 3000.0).abs() <= 16.0 && (s.y - 9000.0).abs() <= 16.0));
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
    assert!(step.events.iter().any(|e| matches!(e, OceanEvent::Bite { victim, .. } if *victim == victim_id)), "{:?}", step.events);
    assert!(step.events.iter().any(|e| matches!(e, OceanEvent::Died { id, hunted: true, .. } if *id == victim_id)));
    assert_eq!(sim.creatures.len(), 1);
    assert!(sim.creatures[0].has_eaten, "a bite counts as eating");
}

#[test]
fn test_restock_refills_from_the_catalogue_and_records_species() {
    let mut sim = OceanSim::empty(config(), 4);
    let catalogue = vec![CatalogueFounder { name: "Saved".into(), genome: WILDTYPE_GENOME.into(), methylations: vec![], synapse_weights: vec![] }];
    let step = sim.tick(|| catalogue.clone());
    let target = config().ocean.target_population;
    assert_eq!((sim.creatures.len(), step.born.len()), (target, target));
    assert!(step.events.iter().all(|e| matches!(e, OceanEvent::Restocked { catalogue_name: Some(n), .. } if n == "Saved")));
    assert_eq!(step.species.iter().filter(|c| matches!(c, SpeciesChange::Discovered { .. })).count(), 1, "one genome, one species");
    // Empty catalogue: structured random founders, each a new species
    let mut sim = OceanSim::empty(config(), 4);
    let step = sim.tick(Vec::new);
    assert!(step.events.iter().all(|e| matches!(e, OceanEvent::Restocked { catalogue_name: None, .. })));
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
    assert!(out.born.is_empty(), "founders of a fresh Ocean reach the UI with INIT_STATE");
    assert!(!out.species.is_empty());

    let next_spore = sim.spores.next_id;
    let out = sim.reset(99, &[]);
    assert_eq!((sim.creatures.len(), sim.seed(), sim.tick_count()), (RESET_FOUNDERS, 99, 0));
    assert_eq!(sim.creatures[0].id, 1, "creature ids restart");
    assert_eq!(sim.spores.spores[0].id, next_spore, "spore ids continue");
    assert_eq!(out.species.len(), sim.ledger.entries().len());

    let released = sim.release(Founder::Random).unwrap();
    assert_eq!(released.born[0].id, sim.creatures.last().unwrap().id);
    while sim.creatures.len() < config().ocean.max_creatures {
        sim.release(Founder::Random).unwrap();
    }
    assert_eq!(sim.release(Founder::Random).unwrap_err(), SpawnRejected::PopulationCap(config().ocean.max_creatures));
}

#[test]
fn test_snapshot_of_an_earlier_version_restores_with_new_centres() {
    let agent = OceanSim::fresh(config(), 6).0.creatures[0].clone();
    let old = serde_json::json!({ "creatures": [agent], "foodPellets": [], "highestGeneration": 3, "nextCreatureId": 9, "nextSporeId": 1300 });
    let snapshot: OceanSnapshot = serde_json::from_value(old).unwrap();
    assert_eq!((snapshot.seed, snapshot.tick, snapshot.nutrient_centers.len()), (None, 0, 0));
    let sim = OceanSim::restore(config(), snapshot, 42);
    assert_eq!((sim.seed(), sim.highest_generation(), sim.next_creature_id, sim.spores.next_id), (42, 3, 9, 1300));
    assert_eq!(sim.spores.nutrient_centers.len(), spore_field::NUTRIENT_CENTER_COUNT);
    assert_eq!(sim.snapshot().nutrient_centers, sim.spores.nutrient_centers, "centres are persisted now");
}
