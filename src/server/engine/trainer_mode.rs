//! `TrainerMode` (TCK-156): the trainer as an engine mode. A thin wrapper over the IO-free `TrainerSim` (population,
//! epochs, breeding; `trainer::sim`): UI state (run state, warp, display count, selected sandbox, run id),
//! persistence of elites and run state through `trainer_store`, telemetry.

use std::time::{Duration, Instant};

use crate::biology::trainer_store;
use crate::config::AppConfig;
use crate::persistence::LogDbError;
use crate::server::ipc::UiEvent;
use crate::server::trainer_ipc::{
    TrainerAction, TrainerGenerationCompleted, TrainerState, TrainerTelemetryTick, TrainerUiEvent,
};
use crate::shared::time;
use crate::trainer::breeder::GenomeRecord;
use crate::trainer::mode::Scenario;
use crate::trainer::sim::{TrainerEvent, TrainerParams, TrainerSim, run_seed};

use super::mode::{Interval, Mode, ModeCtx, clamp_warp};
use super::store::EngineStore;

/// Trainer telemetry rate (25 Hz).
pub const TELEMETRY_PERIOD: Duration = Duration::from_millis(40);
/// Run id of a fresh engine.
pub const DEFAULT_RUN_ID: &str = "default_run";
/// Displayed sandboxes of a fresh engine.
pub const DEFAULT_DISPLAY_COUNT: usize = 16;

pub struct TrainerMode {
    sim: TrainerSim,
    running: bool,
    /// Steps per frame, clamped to `MAX_STEPS_PER_FRAME`
    warp: usize,
    /// Only echoed back to the UI
    headless: bool,
    run_id: String,
    selected_sandbox: Option<u32>,
    /// Displayed sandboxes (UI grid tiles and telemetry); the population is `sim.params.population` (TCK-192). Only
    /// the telemetry reads it, the simulation never does.
    display_count: usize,
    retention: usize,
    /// Whether `sim` holds the run state of `run_id` (false until the first restore loads it)
    run_state_loaded: bool,
    /// The run's stored genomes changed outside the engine (a command); the next entry rebuilds from the store
    store_changed: bool,
    telemetry: Interval,
}

impl TrainerMode {
    pub fn new(config: &AppConfig, now: Instant) -> Self {
        let retention = config.trainer.trainer_genome_retention_per_run;
        Self {
            sim: TrainerSim::new(
                TrainerParams::from_config(config),
                run_seed(&config.core.seed, DEFAULT_RUN_ID, None),
                retention,
                config,
            ),
            running: false,
            warp: 1,
            headless: false,
            run_id: DEFAULT_RUN_ID.to_string(),
            selected_sandbox: Some(1),
            display_count: DEFAULT_DISPLAY_COUNT,
            retention,
            run_state_loaded: false,
            store_changed: false,
            telemetry: Interval::new(TELEMETRY_PERIOD, now),
        }
    }

    pub fn sim(&self) -> &TrainerSim {
        &self.sim
    }

    pub fn running(&self) -> bool {
        self.running
    }

    /// A command changed the stored genomes of `run_id` (catalogue → training, history cleared): the next entry into
    /// the trainer rebuilds the population from the store.
    pub fn store_changed(&mut self, run_id: &str) {
        if run_id == self.run_id {
            self.store_changed = true;
        }
    }

    /// The settings as the UI shows them (TRAINER_STATE_CHANGED / TRAINER_RESET_COMPLETED).
    fn state(&self) -> TrainerState {
        let params = &self.sim.params;
        TrainerState {
            is_running: self.running,
            population: params.population,
            display_count: self.display_count.min(params.population),
            warp_speed: self.warp,
            elite_ratio: params.elite_ratio,
            mutation_rate: params.mutation_ops_max,
            mutation_ops_min: params.mutation_ops_min.max(1),
            mutation_ops_limit: self.sim.config().trainer.mutation_ops_limit,
            inflow_rate: params.inflow_rate,
            hof_rate: params.hof_rate,
            multi_trial: params.multi_trial,
            is_headless: self.headless,
            run_id: self.run_id.clone(),
            lamarckian: params.lamarckian,
            training_mode: params.scenario,
            level: params.scenario.fixed_level().unwrap_or(params.level),
            level_auto: params.level_auto,
            pending_mode: self.sim.pending_scenario(),
            generation: self.sim.generation,
        }
    }

    fn emit_state(&self, ctx: &mut ModeCtx) {
        ctx.emit(TrainerUiEvent::TrainerStateChanged(self.state()));
    }

    /// Continues the run from the store: active parents + hall of fame; the generation counter continues from the
    /// newest parent. Run state (TCK-191): with `load_run_state` (the trainer does not hold this run's state yet: app
    /// start, run switch) the saved mode, level, gate, layout and mutation ops of the run replace the current ones
    /// (no row: current params, layout 0); otherwise the current params and layout continue, so unsaved UI changes
    /// since the last generation end are kept. The run seed follows the same rule: stored seed (else the run's
    /// default seed) on load, the current trainer's seed otherwise (TCK-194).
    fn restore(&mut self, generation: u32, store: &dyn EngineStore, config: &AppConfig) {
        let load_run_state = !self.run_state_loaded;
        let default_seed = run_seed(&config.core.seed, &self.run_id, None);
        let mut params = self.sim.params.clone();
        let (layout_id, seed) = if load_run_state {
            trainer_store::load_run_state(store, &self.run_id).map_or((0, default_seed), |state| {
                state.apply(&mut params, config);
                (state.layout_id, state.seed.unwrap_or(default_seed))
            })
        } else {
            (self.sim.layout_id(), self.sim.seed())
        };
        let parents = trainer_store::load_genomes(store, &self.run_id, true, params.population);
        let hof = trainer_store::load_genomes(store, &self.run_id, false, self.retention);
        let generation = parents.iter().map(|p| p.generation).fold(generation, u32::max);
        self.sim = TrainerSim::restore_at_layout(
            params,
            seed,
            generation,
            layout_id,
            parents,
            hof,
            self.retention,
            config,
        );
        self.run_state_loaded = true;
        self.store_changed = false;
    }

    fn reset(&mut self, ctx: &mut ModeCtx) {
        self.running = false;
        // Delete the saved genomes and run state of the current run (hard reset)
        ctx.store
            .delete_trainer_run(&self.run_id)
            .log_db_err("deleting the trainer run");
        // A fresh run restarts the automatic curriculum at its start level (a manual level is kept)
        let mut params = self.sim.params.clone();
        if params.level_auto {
            params.level = params.curriculum.start_level;
        }
        // A reset starts a new run: fresh seed (clock nonce), stored at once so a restart before the first generation
        // end continues this run (TCK-194)
        let seed = run_seed(&ctx.config.core.seed, &self.run_id, Some(time::now_nanos()));
        self.sim = TrainerSim::new(params, seed, self.retention, ctx.config);
        trainer_store::save_run_state(ctx.store, &self.run_id, &self.sim);
        self.run_state_loaded = true;
        self.store_changed = false;
        ctx.emit(TrainerUiEvent::TrainerResetCompleted(TrainerState {
            generation: 1,
            ..self.state()
        }));
    }

    /// Generation end: UI events, elites and run state to the store.
    fn on_event(&mut self, event: TrainerEvent, ctx: &mut ModeCtx) {
        match event {
            TrainerEvent::GenerationCompleted {
                generation,
                best_fitness,
                avg_fitness,
                best_genome,
                elites,
                stats,
            } => {
                log::info!(
                    "generation {} completed, best fitness {:.2}, mean {:.2}",
                    generation,
                    best_fitness,
                    avg_fitness
                );
                let params = &self.sim.params;
                ctx.emit(TrainerUiEvent::TrainerGenerationCompleted(TrainerGenerationCompleted {
                    generation,
                    best_fitness,
                    avg_fitness,
                    best_genome,
                    mutation_ops_max: params.mutation_ops_max,
                    training_mode: params.scenario,
                    next_level: params.scenario.fixed_level().unwrap_or(params.level),
                    stats: (&stats).into(),
                }));
                // Settings after the generation boundary (applied room switch, gate level, adapted mutation ops)
                self.emit_state(ctx);
                trainer_store::save_elites(ctx.store, &self.run_id, generation, &elites, self.retention);
                // Run state after the boundary (TCK-191); a switch requested later stays pending until its generation ends
                trainer_store::save_run_state(ctx.store, &self.run_id, &self.sim);
                ctx.emit(UiEvent::database_changed());
            }
        }
    }
}

impl Mode for TrainerMode {
    type Action = TrainerAction;

    /// Rebuilds the population from the store only when the run state is not loaded yet or a command changed the
    /// run's genomes (before TCK-156 every navigation rebuilt it and restarted the running generation).
    fn on_enter(&mut self, ctx: &mut ModeCtx) {
        if !self.run_state_loaded || self.store_changed {
            self.restore(self.sim.generation, ctx.store, ctx.config);
            log::info!(
                "rebuilt sandbox grid with {} chambers for run '{}', restored Gen {}",
                self.sim.params.population,
                self.run_id,
                self.sim.generation
            );
        }
        self.emit_state(ctx);
    }

    fn on_exit(&mut self, _ctx: &mut ModeCtx) {
        log::info!("leaving trainer mode, resuming the Ocean");
    }

    fn handle(&mut self, action: TrainerAction, ctx: &mut ModeCtx) {
        match action {
            // Room switches take effect at the next generation (at once at a generation start; TCK-176)
            TrainerAction::SetTrainerMode { mode: scenario } => {
                let sim = &mut self.sim;
                if scenario != sim.params.scenario
                    || sim.pending_scenario().is_some()
                    || (scenario == Scenario::Curriculum && !sim.params.level_auto)
                {
                    sim.set_scenario(scenario);
                    let size = sim.room().chamber_size();
                    log::info!(
                        "training mode {:?} requested (pending: {:?}, current chamber {}x{})",
                        scenario,
                        sim.pending_scenario(),
                        size,
                        size
                    );
                }
                self.emit_state(ctx);
            }
            TrainerAction::SetTrainerLevel { level } => {
                self.sim.set_level(level);
                log::info!(
                    "manual level {:.2} requested (pending: {:?})",
                    level,
                    self.sim.pending_scenario()
                );
                self.emit_state(ctx);
            }
            TrainerAction::StartTraining => {
                log::info!("training started (warp {})", self.warp);
                self.running = true;
                self.emit_state(ctx);
            }
            TrainerAction::PauseTraining => {
                log::info!("training paused");
                self.running = false;
                self.emit_state(ctx);
            }
            TrainerAction::TrainerReset => self.reset(ctx),
            TrainerAction::SelectTrainerSandbox { id } => self.selected_sandbox = id,
            TrainerAction::AssignSandboxCreature(assign) => {
                let methylations = assign.methylations.unwrap_or_default();
                let methylations = if methylations.is_empty() {
                    None
                } else {
                    Some(&methylations[..])
                };
                // Learned weights only on request; the factory falls back to the genome weights on a length mismatch
                let synapses = if assign.load_learned_synapses.unwrap_or(false) {
                    Some(assign.synapse_weights.unwrap_or_default())
                } else {
                    None
                };
                let generation = self.sim.generation;
                if let Some(sb) = self
                    .sim
                    .assign_creature(assign.sandbox_id, &assign.genome, methylations, synapses)
                {
                    // Also save this assigned creature as an active parent of this run
                    let record = GenomeRecord {
                        genome: sb.agent.genome.clone(),
                        name: sb.agent.phenotype.latin_name.clone(),
                        fitness: 0.0,
                        generation,
                        methylations: sb.agent.phenotype.methylations.clone(),
                        synapse_weights: sb.agent.synapse_weights.clone(),
                    };
                    trainer_store::save_assigned(ctx.store, &self.run_id, generation, assign.sandbox_id, &record);
                }
                ctx.emit(UiEvent::database_changed());
            }
            TrainerAction::UpdateTrainerHyperparams(update) => {
                let mut grid_changed = false;
                let mut generation_base = self.sim.generation;
                let p = &mut self.sim.params;
                // Population (TCK-192): a change restores the population from the store
                if let Some(n) = update.population.map(|n| n.max(1))
                    && n != p.population
                {
                    p.population = n;
                    grid_changed = true;
                }
                // Displayed sandboxes: telemetry only, the simulation is not touched
                if let Some(n) = update.display_count {
                    self.display_count = n.max(1);
                }
                if let Some(warp) = update.warp_speed {
                    self.warp = clamp_warp(warp);
                    log::info!("trainer warp set to {} (requested {})", self.warp, warp);
                }
                if let Some(e_r) = update.elite_ratio {
                    p.elite_ratio = e_r;
                }
                // `mutationRate` = max mutation ops per child (TCK-174; field name kept)
                if let Some(ops) = update.mutation_rate {
                    let lo = p.mutation_ops_min.max(1);
                    p.mutation_ops_max =
                        (ops.round().max(0.0) as u32).clamp(lo, ctx.config.trainer.mutation_ops_limit.max(lo));
                }
                if let Some(i_r) = update.inflow_rate {
                    p.inflow_rate = i_r;
                }
                if let Some(h_r) = update.hof_rate {
                    p.hof_rate = h_r;
                }
                if let Some(mt) = update.multi_trial {
                    p.multi_trial = mt;
                }
                if let Some(hl) = update.is_headless {
                    self.headless = hl;
                }
                if let Some(lam) = update.lamarckian
                    && lam != p.lamarckian
                {
                    p.lamarckian = lam;
                    grid_changed = true;
                }
                if let Some(run_id) = update.run_id
                    && run_id != self.run_id
                {
                    self.run_id = run_id;
                    generation_base = 1;
                    grid_changed = true;
                    self.run_state_loaded = false;
                }
                if grid_changed {
                    self.restore(generation_base, ctx.store, ctx.config);
                }
                self.display_count = self.display_count.min(self.sim.params.population);
                // Broadcast the validated hyperparameters back to the UI (self-healing UI)
                self.emit_state(ctx);
            }
        }
    }

    /// Up to `warp` steps. Frame cost scales with population × steps (TCK-192): `step(true)` steps the sandboxes on
    /// the rayon pool (parallel == sequential: seeded per-sandbox and per-slot streams); at ~1 M sandbox-ticks/s
    /// (runner, pop 300) warp 1 costs ~0.3 ms per frame and the cap of 35 steps ~10 ms. A frame over 16.7 ms only
    /// skips the sleep (actions are still read every frame).
    fn step(&mut self, ctx: &mut ModeCtx) {
        if !self.running {
            return;
        }
        for _ in 0..self.warp {
            if let Some(event) = self.sim.step(true) {
                self.on_event(event, ctx);
            }
        }
    }

    /// Only the displayed sandboxes (first N, elites first), so the payload does not grow with the population (TCK-192).
    fn emit_telemetry(&mut self, ctx: &mut ModeCtx) {
        if self.telemetry.due(ctx.now) {
            let tick = TrainerTelemetryTick::build(&self.sim, self.display_count, self.selected_sandbox, self.running);
            ctx.emit(TrainerUiEvent::TrainerTelemetryTick(tick));
        }
    }
}
