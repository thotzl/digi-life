//! Regression tests of the R5 model corrections (TCK-202). Each test failed before its correction.

use crate::biology::creature_factory::{SpawnOptions, spawn_creature};
use crate::biology::diet::DietClass;
use crate::biology::genome::WILDTYPE_GENOME;
use crate::biology::genome::payload::Payload;
use crate::biology::organs::OrganKind;
use crate::biology::phenotype::parse_genome;
use crate::config::{AppConfig, test_config};
use crate::shared::dto::{CreatureAgent, FoodSpore};
use crate::shared::rng::derive;
use crate::sim::ocean::{OceanEvent, OceanSim};
use crate::sim::systems::sense::sense;
use crate::sim::{CreatureStep, TickCtx, World, WorldSettings};
use crate::trainer::chamber::Chamber;
use crate::trainer::room::registry;

fn agent(genome: &str, px: f32, py: f32) -> CreatureAgent {
    spawn_creature(
        genome,
        SpawnOptions {
            id: 1,
            px,
            py,
            heading_angle: 0.0,
            generation: 1,
            methylations: None,
            synapse_weights: None,
        },
        test_config(),
    )
}

fn food(id: u32, type_id: u32, x: f32, y: f32) -> FoodSpore {
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

/// Correction 1: The thermal tolerance minimum is floored at -5 °C, not negated.
#[test]
fn test_thermal_tolerance_minimum_is_floored_at_minus_five() {
    let payload = "ZZZZZZZZZZ";
    let genome = format!("TEM{}EN{}", payload, WILDTYPE_GENOME);
    let p = parse_genome(&genome, None, None, &test_config().brain);
    let (c, w) = (Payload(payload).h(0), Payload(payload).h(6));
    let expected_min = (c * 60.0 - w * 15.0).round().max(-5.0);
    assert_eq!(
        p.thermal_tolerance_min, expected_min,
        "tolerance {:?}",
        p.thermal_tolerance_min
    );
    assert!(p.thermal_tolerance_min < p.thermal_tolerance_max);
}

/// Correction 2: Trainer and Ocean eat with one radius rule from config.
#[test]
fn test_trainer_eats_with_the_config_radius() {
    let config = test_config();
    let room = registry::build("standard", config).unwrap();
    let settings = room.world_settings(config);
    let mut a = agent(WILDTYPE_GENOME, 500.0, 500.0);
    let r = a.phenotype.spinal_harmonics.mean_radius;
    let radius = r * 1.5 + 4.0;
    let mut foods = vec![food(1, 1, 500.0 + radius - 0.5, 500.0)];
    let mut step = CreatureStep::default();
    let mut world = World::single(&mut a, &mut foods, 1000.0, &settings);
    crate::sim::systems::eat::eat(&mut world, 0, &TickCtx::new(config, 1), &mut step);
    assert_eq!(
        step.events.len(),
        1,
        "food at {} px (radius {}) not eaten",
        radius - 0.5,
        radius
    );
}

/// Correction 3: Food drifts once per tick before the creatures move, in every world (pushed from the tick-start
/// position).
#[test]
fn test_trainer_food_drifts_before_the_creature_moves() {
    let config = test_config();
    let room = registry::build("standard", config).unwrap();
    let settings = room.world_settings(config);
    let mut a = agent(WILDTYPE_GENOME, 500.0, 500.0);
    a.vx = 20.0;
    let r = a.phenotype.spinal_harmonics.mean_radius;
    // Behind the creature, inside its push radius (r + 8) at the tick start, out of reach after the move
    let spore = food(1, 1, 500.0 - (r + 4.0), 500.0);
    let mut chamber = Chamber::new(room.as_ref(), vec![a]);
    chamber.foods = vec![spore.clone()];
    chamber.step(&settings, &[], config);
    assert!(
        chamber.foods[0].x < spore.x,
        "spore not pushed from the tick-start position: {:?}",
        chamber.foods[0]
    );
}

/// Correction 4: Every world's clock input uses the platform-independent `fmath::sin` (before:
/// `assert_eq!(settings.clock, ClockVariant::Fmath)` on the Ocean settings failed with `left: Std`; the variant field
/// is gone).
#[test]
fn test_ocean_clock_uses_fmath() {
    let config = test_config();
    let settings = WorldSettings::ocean(&config.ocean, Vec::new());
    let mut a = agent(WILDTYPE_GENOME, 500.0, 500.0);
    a.phenotype.organelles.clear();
    // Without organs the systemic inputs start at slot 0, the clock first (`SystemicInput::ALL`)
    let clock_slot = 0;
    for age in [0u32, 17, 123, 4567] {
        a.age = age;
        let mut foods = vec![];
        let mut step = CreatureStep::default();
        sense(&mut World::single(&mut a, &mut foods, 2000.0, &settings), 0, &mut step);
        assert_eq!(
            step.inputs[clock_slot],
            0.5 + 0.5 * crate::shared::fmath::sin(age as f32 * 0.1)
        );
    }
}

/// Correction 5: Only limb organs drive locomotion: an eye with a high expression style is no limb.
#[test]
fn test_eye_with_high_style_is_no_limb() {
    let mut p = parse_genome(WILDTYPE_GENOME, None, None, &test_config().brain);
    let mut eye = p.organelles[0].clone();
    eye.kind = OrganKind::Eye;
    eye.expression_style = 0.95;
    p.organelles = vec![eye];
    let with_eye = crate::shared::physics::locomotion_drive(&p);
    p.organelles.clear();
    assert_eq!(with_eye, crate::shared::physics::locomotion_drive(&p));
}

/// Correction 6: One diet classification: predator flag, "raptor" name, biting and photosynthesis follow `DietClass`.
#[test]
fn test_carnivory_thresholds_follow_the_diet_class() {
    let config = test_config();
    for k in 0..26u8 {
        let letter = (b'A' + k) as char;
        let payload: String = std::iter::repeat_n(letter, 10).collect();
        let genome = format!("CAR{}EN{}", payload, WILDTYPE_GENOME);
        let p = parse_genome(&genome, None, None, &config.brain);
        let class = DietClass::from_carnivory(p.carnivory);
        assert_eq!(
            p.is_predator,
            class == DietClass::Carnivore,
            "carnivory {}",
            p.carnivory
        );
        assert_eq!(
            p.latin_name.contains("raptor"),
            class == DietClass::Carnivore,
            "carnivory {} name {}",
            p.carnivory,
            p.latin_name
        );
        // Before: `p.carnivory >= ocean.bitingCarnivoryThreshold` (0.35) bit at 0.36, a herbivore
        let mut a = agent(&genome, 0.0, 0.0);
        a.phenotype.carnivory = p.carnivory;
        assert_eq!(
            crate::sim::systems::bite::BiteRule::is_biter(&a),
            class != DietClass::Herbivore,
            "carnivory {}",
            p.carnivory
        );
    }
}

/// Correction 7: One temperature model: the sensed water temperature follows the depth, as the thermal stress does.
#[test]
fn test_sensed_temperature_follows_depth() {
    let config = test_config();
    let settings = WorldSettings::ocean(&config.ocean, Vec::new());
    let read = |x: f32, y: f32| {
        let mut a = agent(WILDTYPE_GENOME, x, y);
        let mut proprioceptor = a.phenotype.organelles[0].clone();
        proprioceptor.kind = OrganKind::Proprioceptor;
        a.phenotype.organelles = vec![proprioceptor];
        let mut foods = vec![];
        let mut step = CreatureStep::default();
        let mut world = World::single(&mut a, &mut foods, 2000.0, &settings);
        sense(&mut world, 0, &mut step);
        step.inputs[2]
    };
    assert_eq!(
        read(1000.0, 1000.0),
        read(1900.0, 1000.0),
        "same depth, same temperature"
    );
    assert!(read(1000.0, 200.0) > read(1000.0, 1800.0), "warm surface, cold bottom");
}

/// Correction 8: Lateral line: only flow arriving head-on at the organ's axis registers (intended, documented in
/// TCK-202; this test passed before as well: no behaviour change).
#[test]
fn test_lateral_line_registers_head_on_flow_only() {
    let config = test_config();
    let settings = WorldSettings::ocean(&config.ocean, Vec::new());
    let read = |vx: f32| {
        let mut a = agent(WILDTYPE_GENOME, 1000.0, 1000.0);
        let mut organ = a.phenotype.organelles[0].clone();
        organ.kind = OrganKind::LateralLine;
        organ.angle = 0.0;
        a.phenotype.organelles = vec![organ];
        a.heading_angle = 0.0;
        a.vx = vx;
        a.omega_rot = 0.0;
        let mut foods = vec![];
        let mut step = CreatureStep::default();
        sense(&mut World::single(&mut a, &mut foods, 2000.0, &settings), 0, &mut step);
        step.inputs[1]
    };
    // Swimming forward (vx > 0 at heading 0) meets the water head-on: flow -vx along the axis is negative
    assert_eq!(read(0.5), 0.0);
    assert!(read(-0.5) > 0.0);
}

fn ocean_pair(seed: u64, victim_energy: f32) -> (OceanSim, u32) {
    let config = test_config();
    let mut sim = OceanSim::empty(config, seed);
    for (x, carnivory, species) in [(4000.0, 1.0, "hunter"), (4002.0, 0.0, "prey")] {
        let mut a = agent(WILDTYPE_GENOME, x, 6000.0);
        a.id = sim.creatures().len() as u32 + 1;
        a.phenotype.carnivory = carnivory;
        a.species_id = species.into();
        a.energy = 50.0;
        sim.creatures_mut().push(a);
    }
    sim.creatures_mut()[1].energy = victim_energy;
    sim.spores_mut().spores = vec![food(1, 1, 15000.0, 1000.0)];
    (sim, 2)
}

/// Correction 9a: One death threshold: a bitten creature with energy left after the damage survives.
#[test]
fn test_bitten_creature_with_energy_left_survives() {
    let config: &AppConfig = test_config();
    let (mut sim, victim) = ocean_pair(11, config.metabolism.biting_energy_damage + 0.08);
    let step = sim.step();
    assert!(
        step.events.iter().any(|e| matches!(e, OceanEvent::Bite { .. })),
        "{:?}",
        step.events
    );
    assert!(
        sim.creatures().iter().any(|c| c.id == victim),
        "victim died with energy left: {:?}",
        step.events
    );
}

/// Correction 9b: A creature that dies in its own step this tick cannot be bitten for energy.
#[test]
fn test_creature_dying_this_tick_is_not_bitten() {
    let (mut sim, victim) = ocean_pair(12, 1e-6);
    let step = sim.step();
    assert!(
        !step
            .events
            .iter()
            .any(|e| matches!(e, OceanEvent::Bite { victim: v, .. } if *v == victim)),
        "{:?}",
        step.events
    );
}

/// Correction 10: The hall of fame holds every genome once.
#[test]
fn test_hall_of_fame_has_no_duplicate_genomes() {
    use crate::trainer::breeder::{Breeder, GenomeRecord};
    let record = |genome: &str, fitness: f32| GenomeRecord {
        genome: genome.into(),
        name: genome.into(),
        fitness,
        generation: 1,
        methylations: vec![],
        synapse_weights: vec![],
    };
    let breeder = Breeder::new(vec![], vec![record("A", 1.0), record("A", 0.9), record("B", 0.5)], 10);
    let genomes: Vec<&str> = breeder.hof().iter().map(|r| r.genome.as_str()).collect();
    assert_eq!(genomes, vec!["A", "B"]);
}

/// Correction 11: One lifespan source: no config cap besides the genome's mature age.
#[test]
fn test_lifespan_has_one_source() {
    let disk: serde_json::Value = serde_json::from_str(include_str!("../../config.json")).unwrap();
    assert!(
        disk["ocean"].get("creatureMaxAgeTicks").is_none(),
        "dead lifespan cap in config.json"
    );
}

/// Correction 13: A letter mutation never lands inside a read gene's promoter, also when every letter lies in some
/// motif match (no unprotected locus; such genomes arise in long mutation walks through degenerate motif matching).
#[test]
fn test_mutation_without_unprotected_locus_keeps_promoters() {
    use crate::biology::genome::mutate::{MutationKind, apply_mutation, unprotected_indices};
    use crate::biology::genome::table::GeneTable;
    // Reached by `mutate_genome` from a random 128-letter genome (seed 5, step 683)
    let genome = "ESVWQRSPVCIVDRSIECQKSHDOOLNHUXESRDUECKSFTDHNODFNEDWDFENAAWUVQYLETILQWDWCHSWMDUENCRESLHQWCYCKECKEDKSLTANFDYQIUESYCKESLHCYEDHCRNEDFQF";
    assert!(unprotected_indices(genome).is_empty());
    let promoters: Vec<(usize, usize)> = GeneTable::scan(genome)
        .spans
        .iter()
        .map(|s| (s.start, s.start + s.gene.promoter.len()))
        .collect();
    assert!(!promoters.is_empty());
    let ops = test_config().genome.mutation_operators.clone();
    for kind in [MutationKind::Insertion, MutationKind::Point, MutationKind::Creep] {
        for seed in 0..200 {
            let Some((_, index)) = apply_mutation(genome, kind, &ops, None, &mut derive(seed, &[])) else {
                continue;
            };
            let inside = promoters.iter().any(|&(start, end)| match kind {
                // An insertion at `index` lands between letters `index - 1` and `index`
                MutationKind::Insertion => start < index && index < end,
                _ => (start..end).contains(&index),
            });
            assert!(!inside, "{:?} at {} inside a promoter {:?}", kind, index, promoters);
        }
    }
}

/// Correction 14: Ocean spores carry the config food energy (`metabolism.foodSporeEnergy`), not a literal.
#[test]
fn test_spore_amount_comes_from_config() {
    let mut config = test_config().clone();
    config.metabolism.food_spore_energy = 21.0;
    let (sim, _) = OceanSim::fresh(&config, 5);
    assert!(sim.spores().spores.iter().all(|s| s.amount == 21.0));
}

/// Correction 14: Wall-contact cooldown and spawn margin of the trainer rooms are config keys.
#[test]
fn test_trainer_room_literals_are_config_keys() {
    let disk: serde_json::Value = serde_json::from_str(include_str!("../../config.json")).unwrap();
    for key in ["trainerWallCollisionCooldown", "trainerSpawnMargin"] {
        assert!(
            disk["trainer"].get(key).is_some(),
            "config.json trainer.{} missing",
            key
        );
    }
}
