//! Headless engine tests (TCK-156): `Engine` / `run_loop` with `VecSink` and `MemoryDb`, no Tauri window.

use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use super::*;
use crate::persistence::{MemoryDb, NewCatalogueEntry, OceanSnapshotRepo, TrainerGenome};
use crate::server::ipc::EngineEvent;
use crate::server::ocean_ipc::{OceanAction, OceanUiEvent};
use crate::server::trainer_ipc::{TrainerAction, TrainerHyperparamsUpdate, TrainerUiEvent};
use crate::sim::ocean::{OceanSnapshot, FRESH_FOUNDERS, RESET_FOUNDERS};

fn config() -> AppConfig {
    crate::config::test_config().clone()
}

fn engine_on(store: MemoryDb, sink: &mut VecSink, now: Instant) -> Engine {
    Engine::new(config(), Box::new(store), SnapshotWriter::Inline, None, sink, now)
}

fn ocean(action: OceanAction) -> EngineInput {
    EngineInput::Action(ClientAction::Ocean(action))
}

fn trainer(action: TrainerAction) -> EngineInput {
    EngineInput::Action(ClientAction::Trainer(action))
}

fn set_mode(mode: SimMode) -> EngineInput {
    EngineInput::Action(ClientAction::Engine(EngineAction::SetMode { mode }))
}

fn client_ready() -> EngineInput {
    EngineInput::Action(ClientAction::Engine(EngineAction::ClientReady))
}

fn logs(events: &[UiEvent]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            UiEvent::Engine(EngineEvent::LogEvent { message, .. }) => Some(message.as_str()),
            _ => None,
        })
        .collect()
}

fn trainer_states(events: &[UiEvent]) -> Vec<&crate::server::trainer_ipc::TrainerState> {
    events
        .iter()
        .filter_map(|e| match e {
            UiEvent::Trainer(TrainerUiEvent::TrainerStateChanged(state)) => Some(state),
            _ => None,
        })
        .collect()
}

#[test]
fn test_fresh_engine_seeds_the_ocean_and_its_species() {
    let mut sink = VecSink::new();
    let engine = engine_on(MemoryDb::new(), &mut sink, Instant::now());
    let events = sink.take();
    assert!(matches!(events[..], [UiEvent::Engine(EngineEvent::DatabaseChanged)]), "{:?}", events);
    assert_eq!(engine.ocean().sim().creatures().len(), FRESH_FOUNDERS);
    let species = engine.store().list_species().unwrap();
    assert!(!species.is_empty() && species.len() <= FRESH_FOUNDERS);
    assert_eq!(engine.mode(), SimMode::Ocean);
}

#[test]
fn test_client_ready_sends_the_world_size_from_config_and_keeps_the_mode() {
    let mut sink = VecSink::new();
    let now = Instant::now();
    let mut engine = engine_on(MemoryDb::new(), &mut sink, now);
    engine.handle(set_mode(SimMode::Trainer), &mut sink, now);
    sink.take();
    engine.handle(client_ready(), &mut sink, now);
    let events = sink.take();
    let Some(UiEvent::Ocean(OceanUiEvent::InitState(init))) = events.first() else { panic!("{:?}", events) };
    assert_eq!((init.world.width, init.world.height), (config().ocean.world_width, config().ocean.world_height));
    assert_eq!(init.creatures.len(), FRESH_FOUNDERS);
    assert_eq!(engine.mode(), SimMode::Trainer, "invariant 1: CLIENT_READY never changes the mode");
}

#[test]
fn test_mode_switch_steps_only_the_active_mode_and_keeps_the_trainer_on_reentry() {
    let mut sink = VecSink::new();
    let start = Instant::now();
    let mut engine = engine_on(MemoryDb::new(), &mut sink, start);
    engine.handle(ocean(OceanAction::ToggleSimulation { running: true }), &mut sink, start);
    engine.frame(&mut sink, start);
    assert_eq!(engine.ocean().sim().tick_count(), 1);

    engine.handle(set_mode(SimMode::Trainer), &mut sink, start);
    engine.handle(trainer(TrainerAction::StartTraining), &mut sink, start);
    sink.take();
    let later = start + Duration::from_millis(50);
    for _ in 0..3 {
        engine.frame(&mut sink, later);
    }
    let events = sink.take();
    assert_eq!(engine.ocean().sim().tick_count(), 1, "the Ocean is frozen in trainer mode");
    assert_eq!(engine.trainer().sim().epoch_ticks, 3);
    assert!(events.iter().any(|e| matches!(e, UiEvent::Trainer(TrainerUiEvent::TrainerTelemetryTick(_)))));
    assert!(!events.iter().any(|e| matches!(e, UiEvent::Ocean(OceanUiEvent::TelemetryTick(_)))));

    // Back to the Ocean and into the trainer again: no rebuild from the store, the generation continues
    engine.handle(set_mode(SimMode::Ocean), &mut sink, later);
    engine.frame(&mut sink, later);
    assert_eq!(engine.ocean().sim().tick_count(), 2);
    engine.handle(set_mode(SimMode::Trainer), &mut sink, later);
    assert_eq!(engine.trainer().sim().epoch_ticks, 3);
    assert_eq!(trainer_states(&sink.take()).len(), 1, "every entry answers with the trainer state");

    // A command changed the run's stored genomes: the next entry rebuilds
    engine.store().add_trainer_genome(trainer_mode::DEFAULT_RUN_ID, &TrainerGenome { name: "X".into(), genome: crate::biology::genome::WILDTYPE_GENOME.into(), fitness: 1.0, generation: 1, methylations: vec![], synapse_weights: vec![] }).unwrap();
    engine.handle(EngineInput::TrainerStoreChanged { run_id: trainer_mode::DEFAULT_RUN_ID.into() }, &mut sink, later);
    engine.handle(set_mode(SimMode::Trainer), &mut sink, later);
    assert_eq!(engine.trainer().sim().epoch_ticks, 0, "rebuilt from the store");
}

#[test]
fn test_warp_is_clamped_and_echoed_for_ocean_and_trainer() {
    let mut sink = VecSink::new();
    let now = Instant::now();
    let mut engine = engine_on(MemoryDb::new(), &mut sink, now);
    sink.take();
    engine.handle(ocean(OceanAction::SetSimulationSpeed { speed: 9999 }), &mut sink, now);
    engine.handle(trainer(TrainerAction::UpdateTrainerHyperparams(TrainerHyperparamsUpdate { warp_speed: Some(9999), ..Default::default() })), &mut sink, now);
    let events = sink.take();
    assert!(matches!(events[0], UiEvent::Ocean(OceanUiEvent::SimState { running: false, speed: 35 })), "{:?}", events[0]);
    assert_eq!(trainer_states(&events)[0].warp_speed, MAX_STEPS_PER_FRAME);
    assert_eq!(engine.ocean().warp(), MAX_STEPS_PER_FRAME);
    engine.handle(ocean(OceanAction::ToggleSimulation { running: true }), &mut sink, now);
    engine.frame(&mut sink, now);
    assert_eq!(engine.ocean().sim().tick_count(), MAX_STEPS_PER_FRAME as u64);
}

#[test]
fn test_reset_repopulates_from_the_catalogue_and_clears_newborns_and_snapshot() {
    let store = MemoryDb::new();
    let entry = NewCatalogueEntry {
        id: "c1".into(),
        name: "Saved".into(),
        genome: crate::biology::genome::WILDTYPE_GENOME.into(),
        source: "Ocean Substrate".into(),
        fitness: 1.0,
        carnivory: 0.1,
        methylations: vec![],
        synapse_weights: vec![],
    };
    crate::persistence::CatalogueRepo::save_catalogue_entry(&store, &entry).unwrap();
    let mut sink = VecSink::new();
    let now = Instant::now();
    let mut engine = engine_on(store, &mut sink, now);
    engine.handle(ocean(OceanAction::InjectUrzelle), &mut sink, now);
    engine.store().save_ocean_snapshot("{}").unwrap();
    sink.take();

    engine.handle(ocean(OceanAction::ResetEvolution), &mut sink, now);
    let events = sink.take();
    let Some(UiEvent::Ocean(OceanUiEvent::InitState(init))) = events.iter().find(|e| matches!(e, UiEvent::Ocean(OceanUiEvent::InitState(_)))) else { panic!() };
    assert_eq!(init.creatures.len(), RESET_FOUNDERS);
    assert!(logs(&events).iter().any(|l| l.contains("clones from Catalogue")));
    assert_eq!(engine.store().ocean_snapshot().unwrap(), None);
    assert_eq!(engine.store().list_species().unwrap().len(), 1, "25 clones of one catalogue genome");

    // Invariant 3: the injected founder from before the reset is not re-sent
    engine.frame(&mut sink, now + Duration::from_millis(50));
    let tick = sink.take().into_iter().find_map(|e| match e {
        UiEvent::Ocean(OceanUiEvent::TelemetryTick(t)) => Some(t),
        _ => None,
    });
    assert!(tick.expect("telemetry").new_creatures.is_empty());
}

#[test]
fn test_running_ocean_writes_species_changes_and_snapshot_restores_the_same_ocean() {
    let mut sink = VecSink::new();
    let start = Instant::now();
    let mut engine = engine_on(MemoryDb::new(), &mut sink, start);
    engine.handle(ocean(OceanAction::ToggleSimulation { running: true }), &mut sink, start);
    engine.handle(ocean(OceanAction::SetSimulationSpeed { speed: 35 }), &mut sink, start);
    for _ in 0..6 {
        engine.frame(&mut sink, start);
    }
    // Restock to the target population went through the ledger into the store
    assert_eq!(engine.ocean().sim().creatures().len(), config().ocean.target_population);
    let stored = engine.store().list_species().unwrap();
    // Stored by genome, or ignored because its latin name is already stored (the store's unique name, as before)
    let creatures = engine.ocean().sim().creatures();
    assert!(creatures.iter().all(|c| stored.iter().any(|s| s.id == c.species_id || s.latin_name == c.phenotype.latin_name)), "every living species is stored");
    assert!(creatures.iter().any(|c| stored.iter().any(|s| s.id == c.species_id)));

    assert_eq!(engine.store().ocean_snapshot().unwrap(), None);
    engine.frame(&mut sink, start + Duration::from_secs(11));
    let json = engine.store().ocean_snapshot().unwrap().expect("snapshot after 10 s");
    let snapshot: OceanSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(snapshot.tick, engine.ocean().sim().tick_count());

    let restored_store = MemoryDb::new();
    restored_store.save_ocean_snapshot(&json).unwrap();
    let restored = engine_on(restored_store, &mut sink, start);
    let (a, b) = (engine.ocean().sim(), restored.ocean().sim());
    assert_eq!((a.creatures().len(), a.seed(), a.tick_count()), (b.creatures().len(), b.seed(), b.tick_count()));
    assert_eq!(a.spores().nutrient_centers, b.spores().nutrient_centers, "nutrient centres survive a restart");
}

#[test]
fn test_population_cap_rejects_releases_with_a_log_line() {
    let mut sink = VecSink::new();
    let now = Instant::now();
    let mut engine = engine_on(MemoryDb::new(), &mut sink, now);
    for _ in 0..config().ocean.max_creatures {
        engine.handle(ocean(OceanAction::InjectUrzelle), &mut sink, now);
    }
    assert_eq!(engine.ocean().sim().creatures().len(), config().ocean.max_creatures);
    assert!(logs(&sink.take()).last().unwrap().starts_with("Population cap reached"));
}

#[test]
fn test_sink_coalesces_database_changed_per_flush() {
    let mut sink = VecSink::new();
    sink.emit(UiEvent::database_changed());
    sink.emit(UiEvent::log("a", LogKind::System));
    sink.emit(UiEvent::database_changed());
    sink.flush();
    sink.emit(UiEvent::database_changed());
    let events = sink.take();
    assert_eq!(events.len(), 3, "{:?}", events);
    assert!(events[0].is_database_changed() && !events[1].is_database_changed() && events[2].is_database_changed());
}

#[test]
fn test_unopenable_store_falls_back_to_memory_and_reports_on_handshake() {
    // Before TCK-156 the engine thread called `init_db(..).expect(..)`: a failure killed the thread silently
    let (store, error) = open_store(Path::new("no/such/dir/pixel_life.db"));
    let error = error.expect("open fails");
    let mut sink = VecSink::new();
    let now = Instant::now();
    let mut engine = Engine::new(config(), store, SnapshotWriter::Inline, Some(error), &mut sink, now);
    engine.handle(client_ready(), &mut sink, now);
    assert!(logs(&sink.take()).iter().any(|l| l.starts_with("Database unavailable")));
    assert_eq!(engine.ocean().sim().creatures().len(), FRESH_FOUNDERS);
}

#[test]
fn test_run_loop_processes_inputs_in_order_and_stops_without_senders() {
    let mut sink = VecSink::new();
    let mut engine = engine_on(MemoryDb::new(), &mut sink, Instant::now());
    let (tx, rx) = channel();
    tx.send(client_ready()).unwrap();
    tx.send(set_mode(SimMode::Trainer)).unwrap();
    drop(tx);
    run_loop(&mut engine, &rx, &mut sink);
    let events = sink.take();
    let init = events.iter().position(|e| matches!(e, UiEvent::Ocean(OceanUiEvent::InitState(_)))).expect("INIT_STATE");
    let state = events.iter().position(|e| matches!(e, UiEvent::Trainer(TrainerUiEvent::TrainerStateChanged(_)))).expect("trainer state");
    assert!(init < state);
    assert_eq!(engine.mode(), SimMode::Trainer);
}
