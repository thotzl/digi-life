//! The engine thread (TCK-156): `Engine` holds the two modes (`OceanMode`, `TrainerMode`), the store and the config;
//! `run_loop` reads the inputs, steps the active mode once per ~60 Hz frame and flushes the `EventSink`. Nothing here
//! needs Tauri except `spawn_simulation_thread` (window → `TauriSink`), so the loop runs headless in tests with
//! `VecSink` and `MemoryDb`. Contracts: `wiki/contracts/mode.md`, `wiki/contracts/event-sink.md`.
//!
//! | File | Content |
//! | :--- | :--- |
//! | `mod.rs` | `Engine`, `EngineInput`, `run_loop`, `spawn_simulation_thread` |
//! | `mode.rs` | `Mode` trait, `ModeCtx`, warp cap `MAX_STEPS_PER_FRAME`, `Interval` |
//! | `sink.rs` | `EventSink`, `TauriSink`, `VecSink` (buffer + `DATABASE_CHANGED` coalescing) |
//! | `store.rs` | `EngineStore` (repository ports), `open_store` with in-memory fallback, `SnapshotWriter` |
//! | `ocean_mode.rs` | `OceanMode` over the IO-free `sim::ocean::OceanSim` |
//! | `trainer_mode.rs` | `TrainerMode` over `trainer::sim::TrainerSim` |

pub mod mode;
pub mod ocean_mode;
pub mod sink;
pub mod store;
#[cfg(test)]
mod tests;
pub mod trainer_mode;

use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::AppConfig;
use crate::server::ipc::{ClientAction, EngineAction, LogKind, SimMode, UiEvent};

pub use mode::{MAX_STEPS_PER_FRAME, Mode, ModeCtx, clamp_warp};
pub use ocean_mode::OceanMode;
pub use sink::{EventSink, TauriSink, VecSink};
pub use store::{EngineStore, SnapshotWriter, open_store};
pub use trainer_mode::TrainerMode;

/// Frame budget of the engine thread (~60 Hz).
pub const FRAME: Duration = Duration::from_micros(16_667);

/// What reaches the engine thread: typed client actions (commands, TCK-157) and store notifications of commands that
/// write rows the engine caches.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineInput {
    Action(ClientAction),
    /// A command changed the stored trainer genomes of `run_id` (`add_catalogue_creature_to_training`,
    /// `clear_trainer_history`)
    TrainerStoreChanged {
        run_id: String,
    },
}

impl From<ClientAction> for EngineInput {
    fn from(action: ClientAction) -> Self {
        EngineInput::Action(action)
    }
}

pub struct Engine {
    config: AppConfig,
    store: Box<dyn EngineStore>,
    mode: SimMode,
    ocean: OceanMode,
    trainer: TrainerMode,
    /// Why the store could not be opened (the engine then runs on an in-memory store); repeated on every handshake
    store_error: Option<String>,
}

impl Engine {
    /// Loads the Ocean (snapshot or fresh) and a fresh trainer; starts in Ocean mode. Emits `DATABASE_CHANGED` so a
    /// listening UI fetches the seeded species roster.
    pub fn new(
        config: AppConfig,
        store: Box<dyn EngineStore>,
        snapshots: SnapshotWriter,
        store_error: Option<String>,
        sink: &mut dyn EventSink,
        now: Instant,
    ) -> Self {
        let (ocean, trainer) = {
            let mut ctx = ModeCtx {
                sink,
                store: &*store,
                config: &config,
                now,
            };
            let ocean = OceanMode::load(&mut ctx, snapshots);
            ctx.emit(UiEvent::database_changed());
            (ocean, TrainerMode::new(&config, now))
        };
        Self {
            config,
            store,
            mode: SimMode::Ocean,
            ocean,
            trainer,
            store_error,
        }
    }

    pub fn mode(&self) -> SimMode {
        self.mode
    }

    pub fn ocean(&self) -> &OceanMode {
        &self.ocean
    }

    pub fn trainer(&self) -> &TrainerMode {
        &self.trainer
    }

    pub fn store(&self) -> &dyn EngineStore {
        &*self.store
    }

    /// One input. Mode actions go to their mode in every mode; `SET_MODE` (router only, invariant 1) exits the
    /// active mode on a real switch and always enters the requested one; `CLIENT_READY` answers with the Ocean state
    /// and never changes the mode.
    pub fn handle(&mut self, input: EngineInput, sink: &mut dyn EventSink, now: Instant) {
        let mut ctx = ModeCtx {
            sink,
            store: &*self.store,
            config: &self.config,
            now,
        };
        match input {
            EngineInput::Action(ClientAction::Ocean(action)) => self.ocean.handle(action, &mut ctx),
            EngineInput::Action(ClientAction::Trainer(action)) => self.trainer.handle(action, &mut ctx),
            EngineInput::Action(ClientAction::Engine(EngineAction::ClientReady)) => {
                self.ocean.on_client_ready(&mut ctx);
                if let Some(error) = &self.store_error {
                    ctx.emit(UiEvent::log(
                        format!("Database unavailable ({}): running without persistence.", error),
                        LogKind::System,
                    ));
                }
            }
            EngineInput::Action(ClientAction::Engine(EngineAction::SetMode { mode })) => {
                if mode != self.mode {
                    match self.mode {
                        SimMode::Ocean => self.ocean.on_exit(&mut ctx),
                        SimMode::Trainer => self.trainer.on_exit(&mut ctx),
                    }
                    self.mode = mode;
                }
                match mode {
                    SimMode::Ocean => self.ocean.on_enter(&mut ctx),
                    SimMode::Trainer => self.trainer.on_enter(&mut ctx),
                }
            }
            EngineInput::TrainerStoreChanged { run_id } => self.trainer.store_changed(&run_id),
        }
    }

    /// One frame of the active mode: steps, then its streams.
    pub fn frame(&mut self, sink: &mut dyn EventSink, now: Instant) {
        let mut ctx = ModeCtx {
            sink,
            store: &*self.store,
            config: &self.config,
            now,
        };
        match self.mode {
            SimMode::Ocean => {
                self.ocean.step(&mut ctx);
                self.ocean.emit_telemetry(&mut ctx);
            }
            SimMode::Trainer => {
                self.trainer.step(&mut ctx);
                self.trainer.emit_telemetry(&mut ctx);
            }
        }
    }
}

/// The engine loop: per frame all queued inputs in order, one frame of the active mode, one flush of the sink, then
/// sleep for the rest of `FRAME`. Returns after the frame in which every sender was dropped.
pub fn run_loop(engine: &mut Engine, rx: &Receiver<EngineInput>, sink: &mut dyn EventSink) {
    loop {
        let frame_start = Instant::now();
        let disconnected = loop {
            match rx.try_recv() {
                Ok(input) => engine.handle(input, sink, frame_start),
                Err(TryRecvError::Empty) => break false,
                Err(TryRecvError::Disconnected) => break true,
            }
        };
        engine.frame(sink, frame_start);
        sink.flush();
        if disconnected {
            return;
        }
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME {
            thread::sleep(FRAME - elapsed);
        }
    }
}

/// Starts the engine thread. `config` is the app config of the entry point (`main.rs`); the thread owns it. The
/// database must already be prepared by `init_db` (`main.rs`); the engine opens its own connection without migrating
/// and falls back to an in-memory store (with a log line on every handshake) if that fails.
pub fn spawn_simulation_thread(window: tauri::WebviewWindow, rx: Receiver<EngineInput>, config: AppConfig) {
    thread::spawn(move || {
        let mut sink = TauriSink::new(window);
        let db_path = crate::persistence::db_path();
        let (store, store_error) = open_store(&db_path);
        let snapshots = if store_error.is_none() {
            SnapshotWriter::background(db_path)
        } else {
            SnapshotWriter::Inline
        };
        let mut engine = Engine::new(config, store, snapshots, store_error, &mut sink, Instant::now());
        run_loop(&mut engine, &rx, &mut sink);
        log::info!("engine thread stopped: all action senders dropped");
    });
}
