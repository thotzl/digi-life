---
sources: [tickets/closed/TCK-156-engine-decomposition.md, tickets/closed/TCK-196-plugin-architecture-epic.md, src/server/engine/mod.rs, src/server/engine/mode.rs, src/server/engine/ocean_mode.rs, src/server/engine/trainer_mode.rs, src/sim/ocean/mod.rs]
---

# Contract: engine mode

## Purpose

A mode is one simulation the engine thread can run. The engine (`src/server/engine/`, TCK-156) holds every mode,
routes each mode's typed actions to it, and steps and streams only the active one (`SimMode`, set by the router,
[[wiki/architecture/invariants]] 1). Two modes exist: `OceanMode` (over the IO-free `sim::ocean::OceanSim`) and
`TrainerMode` (over the IO-free `trainer::sim::TrainerSim`). A mode is a thin adapter: the simulation itself is
an IO-free struct with a `step()` that returns what happened; the mode turns that into UI events
([[wiki/contracts/event-sink]]) and store writes ([[wiki/contracts/persistence-repo]]). A planned mode (arena) is one
more simulation struct plus one adapter.

## Signature

```rust
// src/server/engine/mode.rs
pub const MAX_STEPS_PER_FRAME: usize = 35;            // warp cap, invariant 2
pub fn clamp_warp(requested: usize) -> usize;
pub struct ModeCtx<'a> { pub sink: &'a mut dyn EventSink, pub store: &'a dyn EngineStore, pub config: &'a AppConfig, pub now: Instant }
impl ModeCtx<'_> { pub fn emit(&mut self, event: impl Into<UiEvent>) }
pub trait Mode {
    type Action;                                       // OceanAction | TrainerAction
    fn on_enter(&mut self, ctx: &mut ModeCtx);
    fn on_exit(&mut self, ctx: &mut ModeCtx);
    fn handle(&mut self, action: Self::Action, ctx: &mut ModeCtx);
    fn step(&mut self, ctx: &mut ModeCtx);
    fn emit_telemetry(&mut self, ctx: &mut ModeCtx);
}
pub struct Interval;                                   // throttle: Interval::new(period, start), due(now) -> bool

// src/server/engine/mod.rs
pub enum EngineInput { Action(ClientAction), TrainerStoreChanged { run_id: String } }
pub struct Engine;  // new(config, store: Box<dyn EngineStore>, SnapshotWriter, store_error, sink, now)
                    // handle(input, sink, now), frame(sink, now), mode(), ocean(), trainer(), store()
pub fn run_loop(engine: &mut Engine, rx: &Receiver<EngineInput>, sink: &mut dyn EventSink);
pub fn spawn_simulation_thread(window, rx: Receiver<EngineInput>, config);   // the only Tauri-bound function

// src/server/engine/store.rs (ports of TCK-159)
pub trait EngineStore: SpeciesRepo + CatalogueRepo + TrainerRepo + OceanSnapshotRepo {}   // blanket impl: Db, MemoryDb
pub fn open_store(path) -> (Box<dyn EngineStore>, Option<String>);   // MemoryDb + error text if the DB cannot open
pub enum SnapshotWriter { Inline, Background(Sender<SnapshotCmd>) }    // save(snapshot, store), clear(store)

// The simulations behind the modes (IO-free)
OceanSim::{fresh, restore, reset, release, tick, step, restock, snapshot} -> OceanStep { events, species, born }
TrainerSim::{new, restore_at_layout, step} -> Option<TrainerEvent>
```

## Lifecycle / call order

1. `Engine::new`: `OceanMode::load` (stored `OceanSnapshot`, else `OceanSim::fresh` with
   `ocean_seed(core.seed, None)`; founders' species written, then the species ledger loaded from the store),
   `TrainerMode::new` (fresh `TrainerSim` of run `default_run`, run state not loaded), one `DATABASE_CHANGED`.
2. Per frame (`run_loop`, ~60 Hz, `FRAME`): every queued input in order through `Engine::handle`, then
   `Engine::frame` = active mode's `step` then `emit_telemetry`, then one `sink.flush()`, then sleep for the rest of
   the frame. The loop returns after the frame in which all senders were dropped.
3. `handle`: `ClientAction::Ocean` → `OceanMode::handle`, `ClientAction::Trainer` → `TrainerMode::handle`, in every
   mode (an action is never dropped because its mode is inactive). `CLIENT_READY` → `OceanMode::on_client_ready`
   (`INIT_STATE`, `DATABASE_CHANGED`, log line; plus the store error, if any) and never changes the mode.
   `SET_MODE m` → `on_exit` of the active mode if `m` differs, then always `on_enter` of `m` (a reloaded UI on the
   trainer page gets `TRAINER_STATE_CHANGED` again). `TrainerStoreChanged { run_id }` marks the trainer run dirty.
4. `step`: up to `warp` (≤ `MAX_STEPS_PER_FRAME`) simulation steps when running. Ocean: per step `OceanSim::tick`
   (step + restock; the catalogue is read only when the population is below target), events emitted, species changes
   collected and written once per frame; every `SNAPSHOT_PERIOD` (10 s) the snapshot goes to the `SnapshotWriter` and
   the species retention runs (ledger reloaded after a prune). Trainer: `TrainerSim::step(true)`; a
   `GenerationCompleted` emits the generation and state events and saves elites and run state.
5. `emit_telemetry`: Ocean `TELEMETRY_TICK` (25 Hz, newborn queue moved into it) and `FOOD_TICK` (2 Hz); trainer
   `TRAINER_TELEMETRY_TICK` (25 Hz). Each mode has its own `Interval`s (no shared emit clock).
6. `TrainerMode::on_enter` rebuilds the population from the store only if the run state is not loaded yet or a
   command changed the run's genomes (`add_catalogue_creature_to_training`, `clear_trainer_history` send
   `TrainerStoreChanged`); otherwise the running generation continues.

## Invariants

- The simulation structs do no IO: `OceanSim` and `TrainerSim` never touch SQL, Tauri or the clock; randomness only
  from seeded `SimRng` streams (Ocean: `derive(seed, [STREAM_*, tick | release count])`, trainer: run seed streams).
  A seed and the same inputs give the same Ocean (`test_same_seed_same_ocean_and_snapshot_continues_identically`).
- Warp is clamped with `clamp_warp` when it is set, and the clamped value is echoed (`SIM_STATE.speed`,
  `TRAINER_STATE_CHANGED.warpSpeed`) ([[wiki/architecture/invariants]] 2).
- `RESET_EVOLUTION` clears the newborn queue ([[wiki/architecture/invariants]] 3) and clears the stored snapshot
  through the same `SnapshotWriter` channel as the saves (a pending save cannot overwrite the reset).
- Store writes happen at most once per frame for the step path (species) or per action/event (trainer generation
  end), never inside a simulation step ([[wiki/architecture/invariants]] 6).
- Modes emit only through `ctx.sink`; nothing is sent before the frame's `flush` ([[wiki/contracts/event-sink]]).
- The mode switch belongs to the router; no mode changes `Engine::mode` ([[wiki/architecture/invariants]] 1).
- `TrainerMode` must not change trainer results: golden master and trainer references stay identical
  ([[wiki/architecture/invariants]] 8).

## Registration point

A new mode: a `SimMode` variant (`src/server/ipc.rs`, ts-rs), its action group in `ClientAction`, a field in `Engine`,
and one arm each in `Engine::handle` (actions, `SET_MODE` enter/exit) and `Engine::frame`. The compiler reports every
missing arm.

## Config keys

The modes pass the whole `AppConfig` to their simulations. `OceanSim` reads section `ocean` (world size
`worldWidth`/`worldHeight`, population, food, lifespan, grazing, biting, mitosis, decomposition, retention) and
`core.seed`; `TrainerMode` reads `trainer.trainerGenomeRetentionPerRun`, `trainer.mutationOpsLimit` and `core.seed`
([[wiki/contracts/config-section]]).

## Events / IPC

Consumes `ClientAction` (`Engine`, `Ocean`, `Trainer` groups) and the internal `EngineInput::TrainerStoreChanged`.
Emits `UiEvent`s: Ocean `INIT_STATE`, `SIM_STATE { running, speed }`, `TELEMETRY_TICK`, `FOOD_TICK`, `EAT_EVENT`,
`BITE_EVENT`; trainer `TRAINER_STATE_CHANGED`, `TRAINER_RESET_COMPLETED`, `TRAINER_GENERATION_COMPLETED`,
`TRAINER_TELEMETRY_TICK`; both `DATABASE_CHANGED` and `LOG_EVENT` ([[wiki/contracts/ipc-message]]).

## Test recipe

`cargo test server::engine::` (engine with `VecSink` + `MemoryDb`: mode switch, warp echo, reset, snapshot restore,
store fallback, `run_loop`) and `cargo test sim::ocean::` (headless Ocean: feeding, death and decomposition, bite,
restock, ledger, determinism, snapshot compatibility). Full gate: `cargo test` (golden master), the three trainer
reference runs of `.github/workflows/ci.yml`, `npm test`, `npm run build`.

## Minimal example

```rust
// A headless Ocean frame without Tauri
use std::time::Instant;
use crate::persistence::MemoryDb;
use crate::server::engine::{Engine, EngineInput, SnapshotWriter, VecSink};
use crate::server::ipc::ClientAction;
use crate::server::ocean_ipc::OceanAction;

let mut sink = VecSink::new();
let now = Instant::now();
let mut engine = Engine::new(config.clone(), Box::new(MemoryDb::new()), SnapshotWriter::Inline, None, &mut sink, now);
engine.handle(EngineInput::Action(ClientAction::Ocean(OceanAction::ToggleSimulation { running: true })), &mut sink, now);
engine.frame(&mut sink, now);
let events = sink.take();   // SIM_STATE, per-step events, DATABASE_CHANGED (coalesced)
```
