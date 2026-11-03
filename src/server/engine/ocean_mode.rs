//! `OceanMode` (TCK-156): the Ocean as an engine mode. Owns the IO-free `OceanSim` plus the UI side (run state, warp,
//! selection, newborn queue, throttles) and maps each `OceanStep` onto UI events and the species store.

use std::time::Duration;

use crate::persistence::{LogDbError, NewSpecies, SpeciesStatus};
use crate::server::ipc::{LogKind, UiEvent};
use crate::server::ocean_ipc::{
    OceanAction, OceanInitState, OceanRules, OceanTelemetryTick, OceanUiEvent, SelectedBrain,
};
use crate::shared::dto::{CreatureAgent, TelemetryCreature};
use crate::shared::time;
use crate::sim::ocean::{
    CatalogueFounder, DeathCause, Founder, LedgerEntry, OceanEvent, OceanSim, OceanStep, RESET_FOUNDERS, SpawnRejected,
    SpeciesChange, ocean_seed,
};

use super::mode::{Interval, Mode, ModeCtx, clamp_warp};
use super::store::{EngineStore, SnapshotWriter, read_snapshot};

/// Live Ocean frame rate (25 Hz).
pub const TELEMETRY_PERIOD: Duration = Duration::from_millis(40);
/// All food spores (2 Hz).
pub const FOOD_PERIOD: Duration = Duration::from_millis(500);
/// Ocean snapshot and species retention.
pub const SNAPSHOT_PERIOD: Duration = Duration::from_secs(10);

pub struct OceanMode {
    sim: OceanSim,
    running: bool,
    /// Steps per frame, clamped to `MAX_STEPS_PER_FRAME`
    warp: usize,
    selected: Option<u32>,
    /// Creatures the UI has not received yet; moved into the next `TELEMETRY_TICK` (cleared on reset, invariant 3)
    newborns: Vec<CreatureAgent>,
    /// Species changes of this frame's steps, written once per frame
    pending_species: Vec<SpeciesChange>,
    telemetry: Interval,
    food: Interval,
    snapshot: Interval,
    snapshots: SnapshotWriter,
}

impl OceanMode {
    /// The Ocean from the stored snapshot, or a fresh one (seed from `core.seed`); the species ledger is loaded from
    /// the store after the fresh founders are written.
    pub fn load(ctx: &mut ModeCtx, snapshots: SnapshotWriter) -> Self {
        let seed = ocean_seed(&ctx.config.core.seed, None);
        let sim = match read_snapshot(ctx.store) {
            Some(snapshot) => {
                log::info!("restored {} creatures from the database", snapshot.creatures.len());
                OceanSim::restore(ctx.config, snapshot, seed)
            }
            None => {
                let (sim, out) = OceanSim::fresh(ctx.config, seed);
                apply_species(ctx.store, &out.species);
                sim
            }
        };
        let mut mode = Self {
            sim,
            running: false,
            warp: 1,
            selected: None,
            newborns: Vec::new(),
            pending_species: Vec::new(),
            telemetry: Interval::new(TELEMETRY_PERIOD, ctx.now),
            food: Interval::new(FOOD_PERIOD, ctx.now),
            snapshot: Interval::new(SNAPSHOT_PERIOD, ctx.now),
            snapshots,
        };
        mode.reload_ledger(ctx.store);
        mode
    }

    pub fn sim(&self) -> &OceanSim {
        &self.sim
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn warp(&self) -> usize {
        self.warp
    }

    /// `CLIENT_READY`: the full state, a roster refresh and the handshake line.
    pub fn on_client_ready(&mut self, ctx: &mut ModeCtx) {
        log::info!(
            "client handshake, synchronizing {} creatures and {} spores",
            self.sim.creatures().len(),
            self.sim.spores().spores.len()
        );
        ctx.emit(self.init_state(ctx));
        ctx.emit(UiEvent::database_changed());
        ctx.emit(UiEvent::log(
            "Handshake successful. Ecosystem synchronized.",
            LogKind::System,
        ));
    }

    /// `INIT_STATE` (on `CLIENT_READY` and after a reset); the world size is `world.width` / `world.height`.
    fn init_state(&self, ctx: &ModeCtx) -> OceanUiEvent {
        OceanUiEvent::InitState(OceanInitState {
            highest_generation: self.sim.highest_generation(),
            running: self.running,
            seed: ctx.config.core.seed.clone(),
            world: self.sim.world().clone(),
            rules: OceanRules {
                spore_energy: ctx.config.metabolism.food_spore_energy,
            },
            food_pellets: self.sim.spores().spores.clone(),
            creatures: self.sim.creatures().to_vec(),
        })
    }

    fn sim_state(&self) -> OceanUiEvent {
        OceanUiEvent::SimState {
            running: self.running,
            speed: self.warp as u32,
        }
    }

    fn reload_ledger(&mut self, store: &dyn EngineStore) {
        match store.list_species() {
            Ok(records) => self.sim.ledger_mut().load(records.into_iter().map(|r| LedgerEntry {
                id: r.id,
                latin_name: r.latin_name,
                alive: r.status == SpeciesStatus::Alive,
                peak: r.peak_population,
            })),
            Err(e) => log::error!("reading species_records failed: {}", e),
        }
    }

    /// Events and log lines of a step, in order; `DATABASE_CHANGED` if the species changed.
    fn emit_step(&mut self, out: OceanStep, ctx: &mut ModeCtx) {
        for event in out.events {
            // A bite is the shockwave ring plus its log line
            if let OceanEvent::Bite {
                attacker, victim, x, y, ..
            } = event
            {
                ctx.emit(OceanUiEvent::BiteEvent {
                    attacker_id: attacker,
                    victim_id: victim,
                    x,
                    y,
                });
            }
            ctx.emit(ui_event(event));
        }
        if !out.species.is_empty() {
            ctx.emit(UiEvent::database_changed());
        }
        self.newborns.extend(out.born);
    }

    fn release(&mut self, founder: Founder, ctx: &mut ModeCtx) -> Option<CreatureAgent> {
        match self.sim.release(founder) {
            Ok(out) => {
                apply_species(ctx.store, &out.species);
                let agent = out.born[0].clone();
                self.newborns.extend(out.born);
                ctx.emit(UiEvent::database_changed());
                Some(agent)
            }
            Err(SpawnRejected::PopulationCap(cap)) => {
                ctx.emit(UiEvent::log(
                    format!("Population cap reached ({} creatures), spawn rejected.", cap),
                    LogKind::System,
                ));
                None
            }
        }
    }

    fn reset(&mut self, ctx: &mut ModeCtx) {
        log::info!("evolution reset");
        ctx.store.clear_species().log_db_err("clearing species_records");
        self.snapshots.clear(ctx.store);
        self.newborns.clear(); // invariant 3: no stale newborns after a reset
        let catalogue = catalogue(ctx.store);
        // A reset starts a new Ocean: fresh seed (clock nonce), stored with the next snapshot
        let out = self
            .sim
            .reset(ocean_seed(&ctx.config.core.seed, Some(time::now_nanos())), &catalogue);
        apply_species(ctx.store, &out.species);
        let message = if catalogue.is_empty() {
            format!(
                "Evolution reset. Biosphere repopulated with {} structured random founder cells.",
                RESET_FOUNDERS
            )
        } else {
            format!(
                "Evolution reset. Biosphere repopulated with {} clones from Catalogue.",
                RESET_FOUNDERS
            )
        };
        ctx.emit(UiEvent::database_changed());
        ctx.emit(UiEvent::log(message, LogKind::System));
        ctx.emit(self.init_state(ctx));
    }
}

impl Mode for OceanMode {
    type Action = OceanAction;

    fn on_enter(&mut self, _ctx: &mut ModeCtx) {
        log::info!("Ocean mode active");
    }

    fn on_exit(&mut self, _ctx: &mut ModeCtx) {
        log::info!("leaving the Ocean");
    }

    fn handle(&mut self, action: OceanAction, ctx: &mut ModeCtx) {
        match action {
            OceanAction::ToggleSimulation { running } => {
                self.running = running;
                ctx.emit(self.sim_state());
            }
            OceanAction::InjectUrzelle => {
                if let Some(agent) = self.release(Founder::Random, ctx) {
                    ctx.emit(UiEvent::log(
                        format!(
                            "Released wildtype organism: Strain #{} '{}'.",
                            agent.id, agent.phenotype.latin_name
                        ),
                        LogKind::Mutation,
                    ));
                }
            }
            OceanAction::SpawnClone(clone) => {
                let load_learned = clone.load_learned_synapses.unwrap_or(false);
                let methylations = clone.methylations.unwrap_or_default();
                let founder = Founder::Genome {
                    genome: &clone.genome,
                    methylations: if methylations.is_empty() {
                        None
                    } else {
                        Some(&methylations[..])
                    },
                    synapse_weights: if load_learned {
                        clone.synapse_weights.clone()
                    } else {
                        None
                    },
                };
                if let Some(agent) = self.release(founder, ctx) {
                    ctx.emit(UiEvent::log(
                        format!(
                            "Released catalogue specimen: Strain #{} '{}' (Pre-trained: {}).",
                            agent.id, agent.phenotype.latin_name, load_learned
                        ),
                        LogKind::System,
                    ));
                }
            }
            OceanAction::ResetEvolution => self.reset(ctx),
            OceanAction::SelectAgent { id } => self.selected = id,
            OceanAction::SetSimulationSpeed { speed } => {
                self.warp = clamp_warp(speed as usize);
                ctx.emit(self.sim_state());
            }
        }
    }

    fn step(&mut self, ctx: &mut ModeCtx) {
        if self.running {
            for _ in 0..self.warp {
                let out = self.sim.tick(|| catalogue(ctx.store));
                self.pending_species.extend(out.species.iter().cloned());
                self.emit_step(out, ctx);
            }
            apply_species(ctx.store, &self.pending_species);
            self.pending_species.clear();
        }
        // Every 10 s: snapshot (written off the frame thread by the app's writer) and species retention
        if self.snapshot.due(ctx.now) {
            if !self.sim.creatures().is_empty() {
                self.snapshots.save(self.sim.snapshot(), ctx.store);
            }
            match ctx
                .store
                .prune_extinct_species(ctx.config.ocean.species_record_retention)
            {
                Ok(0) => {}
                Ok(_) => self.reload_ledger(ctx.store),
                Err(e) => log::error!("species retention failed: {}", e),
            }
        }
    }

    fn emit_telemetry(&mut self, ctx: &mut ModeCtx) {
        if self.telemetry.due(ctx.now) {
            let creatures = self.sim.creatures();
            let selected_brain = self.selected.and_then(|id| {
                creatures.iter().find(|c| c.id == id).map(|c| SelectedBrain {
                    id,
                    activations: c.neuron_activations.clone(),
                })
            });
            ctx.emit(OceanUiEvent::TelemetryTick(OceanTelemetryTick {
                highest_generation: self.sim.highest_generation(),
                creatures: creatures.iter().map(TelemetryCreature::from).collect(),
                // Newborns are sent once: the queue moves into the event
                new_creatures: std::mem::take(&mut self.newborns),
                selected_brain,
            }));
        }
        if self.food.due(ctx.now) {
            ctx.emit(OceanUiEvent::FoodTick {
                food_pellets: self.sim.spores().spores.clone(),
            });
        }
    }
}

/// The catalogue as Ocean founders (read errors are logged; the Ocean then spawns structured random founders).
fn catalogue(store: &dyn EngineStore) -> Vec<CatalogueFounder> {
    store
        .list_catalogue()
        .unwrap_or_else(|e| {
            log::error!("reading the catalogue failed: {}", e);
            Vec::new()
        })
        .into_iter()
        .map(|e| CatalogueFounder {
            name: e.name,
            genome: e.genome,
            methylations: e.methylations,
            synapse_weights: e.synapse_weights,
        })
        .collect()
}

/// Writes species changes to the store in order (failures are logged).
pub fn apply_species(store: &dyn EngineStore, changes: &[SpeciesChange]) {
    for change in changes {
        match change {
            SpeciesChange::Discovered {
                genome,
                latin_name,
                parent_name,
                generation,
                carnivory,
            } => store
                .insert_species(&NewSpecies {
                    genome: genome.clone(),
                    latin_name: latin_name.clone(),
                    parent_name: parent_name.clone(),
                    generation: *generation as i64,
                    carnivory: *carnivory as f64,
                })
                .log_db_err("inserting a species"),
            SpeciesChange::Peak { id, peak } => store.set_species_peak(id, *peak).log_db_err("updating a species peak"),
            SpeciesChange::Revived { id } => store.mark_species_alive(id).log_db_err("marking a species alive"),
            SpeciesChange::Extinct { id, .. } => store.mark_species_extinct(id).log_db_err("marking a species extinct"),
        }
    }
}

/// Log message and type of a death: "hunted down" only for a creature that took bite damage this tick, otherwise
/// the starvation/senescence message (metabolism and thermal stress also end at ≤ 0.1 energy).
pub fn death_log(id: u32, latin_name: &str, hunted: bool) -> (String, LogKind) {
    if hunted {
        (
            format!("✝️ Specimen #{} '{}' was hunted down by predators.", id, latin_name),
            LogKind::Mutation,
        )
    } else {
        (
            format!(
                "💀 Species '{}' (Strain: #{}) deceased due to starvation/senescence.",
                latin_name, id
            ),
            LogKind::System,
        )
    }
}

/// The UI event of an Ocean event (for a bite: its log line; the ring is sent before it).
fn ui_event(event: OceanEvent) -> UiEvent {
    match event {
        OceanEvent::Ate { x, y } => OceanUiEvent::EatEvent { x, y }.into(),
        OceanEvent::Bite {
            attacker,
            attacker_name,
            victim,
            damage,
            gain,
            ..
        } => UiEvent::log(
            format!(
                "⚡ [BITE ATTACK] {} #{} bites #{}! (-{:.0}nJ damage, +{:.0}nJ gain)",
                attacker_name, attacker, victim, damage, gain
            ),
            LogKind::Mutation,
        ),
        OceanEvent::Died { id, latin_name, cause } => {
            let (message, kind) = death_log(id, &latin_name, cause == DeathCause::Hunted);
            UiEvent::log(message, kind)
        }
        OceanEvent::Mitosis {
            parent,
            child_name,
            generation,
            mutated: true,
        } => UiEvent::log(
            format!(
                "🧬 Mitosis Mutation! Specimen #{} birthed new species '{}' (Gen. {}).",
                parent, child_name, generation
            ),
            LogKind::Mutation,
        ),
        OceanEvent::Mitosis {
            parent, mutated: false, ..
        } => UiEvent::log(
            format!("🌱 Mitosis Clone! Specimen #{} duplicated itself successfully.", parent),
            LogKind::Repair,
        ),
        OceanEvent::Extinct { latin_name } => UiEvent::log(
            format!(
                "✝️ Extinction Event! Species '{}' has gone completely extinct. The only relics remain in the silent fossil archive.",
                latin_name
            ),
            LogKind::Mutation,
        ),
        OceanEvent::Restocked { catalogue_name, .. } => {
            let source = match catalogue_name {
                Some(name) => format!("📚 Catalog Clonal '{}'", name),
                None => "🌱 Random Founder".to_string(),
            };
            UiEvent::log(format!("Restocked population: Spawned {}.", source), LogKind::System)
        }
    }
}
