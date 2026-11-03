---
sources: [.agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-156-engine-decomposition.md, tickets/closed/TCK-139-ocean-hot-loop-performance.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-176-automatic-training-curriculum.md, tickets/closed/TCK-177-trainer-ui-progress-metrics.md, src/server/engine/mod.rs, src/server/engine/mode.rs, src/server/engine/ocean_mode.rs, src/server/engine/trainer_mode.rs, src/server/engine/sink.rs, src/server/engine/store.rs, src/sim/ocean/mod.rs, src/server/trainer_ipc.rs]
---

# Simulation Engine

`src/server/engine/` (TCK-156) — `spawn_simulation_thread(window, rx, config)` is started from `main.rs` setup. It
builds an `Engine` (both modes, the store, the config) and runs `run_loop`. Only `spawn_simulation_thread` touches
Tauri (the window becomes a `TauriSink`); `Engine` and `run_loop` run headless with `VecSink` and `MemoryDb` in
tests. Contracts: [[wiki/contracts/mode]], [[wiki/contracts/event-sink]].

| File | Content |
| :--- | :--- |
| `mod.rs` | `Engine` (`new`, `handle`, `frame`), `EngineInput`, `run_loop`, `spawn_simulation_thread` |
| `mode.rs` | `Mode` trait (`on_enter`, `on_exit`, `handle`, `step`, `emit_telemetry`), `ModeCtx`, `MAX_STEPS_PER_FRAME`, `clamp_warp`, `Interval` |
| `ocean_mode.rs` | `OceanMode`: UI side of the Ocean (run state, warp, selection, newborn queue, throttles), species writes, snapshot |
| `trainer_mode.rs` | `TrainerMode`: UI side of the trainer (run state, warp, run id, display count, selected sandbox), elites and run state to the store |
| `sink.rs` | `EventSink`, `TauriSink`, `VecSink`, `drain_coalesced` |
| `store.rs` | `EngineStore` (the four repository traits), `open_store` (in-memory fallback), `SnapshotWriter`, `read_snapshot` |

The simulations are IO-free structs: the Ocean `sim::ocean::OceanSim` ([[wiki/simulation/ocean]]) and the trainer
`trainer::sim::TrainerSim` ([[wiki/simulation/trainer]]).

## Frame and timing

- Frame ~60 Hz (`FRAME` 16.67 ms sleep budget): all queued inputs in order, then the active mode's `step` and
  `emit_telemetry`, then one `sink.flush()`.
- **Warp cap: 35 steps per frame** (`MAX_STEPS_PER_FRAME`) for both modes; `clamp_warp` caps the requested warp when
  it is set and the clamped value is echoed (`SIM_STATE.speed`, `TRAINER_STATE_CHANGED.warpSpeed`). Unbounded steps
  block the action channel and the UI ([[wiki/architecture/invariants]] 2).
- Each mode has its own throttles (`Interval`): Ocean telemetry 25 Hz, food 2 Hz, snapshot 10 s; trainer telemetry
  25 Hz. Before TCK-156 Ocean and trainer shared one emit clock.
- `run_loop` returns after the frame in which all senders were dropped (app shutdown, tests).

## Modes

- One engine, two modes: `SET_MODE` `ocean` | `trainer`, only through the `set_mode` command of the router
  ([[wiki/architecture/invariants]] 1). A switch calls `on_exit` of the active mode; the requested mode's `on_enter`
  runs on every `SET_MODE`, so a reloaded trainer page gets `TRAINER_STATE_CHANGED` again.
- Actions reach their mode in every mode; only the active mode is stepped and streams telemetry.
- `CLIENT_READY` answers with `INIT_STATE`, `DATABASE_CHANGED` and a log line and never changes the mode (TCK-186).
- Entering the trainer rebuilds its population from the store only if the run state is not loaded yet or a command
  changed the run's genomes (`EngineInput::TrainerStoreChanged`, sent by `add_catalogue_creature_to_training` and
  `clear_trainer_history`); otherwise the running generation continues (before TCK-156 every navigation rebuilt it).

## Events

- Modes emit typed `UiEvent`s into the `EventSink`; it buffers them and sends them once per frame in order,
  coalescing repeated `DATABASE_CHANGED` (TCK-139, RV-P05; [[wiki/contracts/event-sink]]).
- One Tauri event channel `simulation-state` for everything, see [[wiki/architecture/ipc-protocol]].
- `OceanSim` returns `OceanEvent`s (eat, bite, death, mitosis, extinction, restock); `OceanMode` maps them onto
  `EAT_EVENT`, `BITE_EVENT` and log lines.

## Trainer actions and events (TCK-177)

- The commands parse and validate the actions (`ClientAction::from_value`; unknown or malformed actions are rejected
  there with a logged `IpcError`) and queue them as `EngineInput::Action` (TCK-157).
- `SET_TRAINER_MODE` and `SET_TRAINER_LEVEL` only request a room switch (`TrainerSim::set_scenario` / `set_level`);
  it applies at the next generation boundary, or at once if no tick of the current generation has run
  ([[wiki/simulation/trainer]]). `SET_TRAINER_MODE` with the current mode and nothing pending is a no-op; every request
  answers `TRAINER_STATE_CHANGED` with `pendingMode`.
- `TrainerMode` emits `TRAINER_TELEMETRY_TICK` (25 Hz, chamber size and level per sandbox from `sb.room`),
  `TRAINER_GENERATION_COMPLETED` (with the generation's `GenerationStats`), then **`TRAINER_STATE_CHANGED` after every
  generation** (applied switch, gate level, adapted mutation ops). `TrainerMode::state` builds the one `TrainerState`.
  The generation end is matched exhaustively (`match TrainerEvent`), so a new trainer event is a compile error there.

## Persistence

- The desktop app runs `init_db` (migrations) once in `main.rs` for the commands' connection; the engine opens its own
  connection with `open_db` (no second migration). If that fails, the engine runs on a `MemoryDb` and repeats
  "Database unavailable … running without persistence" on every `CLIENT_READY` (before TCK-156 an `expect` panicked the
  engine thread without any message to the UI).
- Ocean species: `OceanSim` keeps a pure `SpeciesLedger` and returns `SpeciesChange`s; `OceanMode` writes the changes
  of a frame's steps once per frame (`apply_species`) and reloads the ledger from the store at start and after a
  retention prune.
- Ocean snapshot (`OceanSnapshot`, typed JSON): every 10 s through the `SnapshotWriter`, which in the app is a writer
  thread with its own connection, so serialising does not run on the frame thread. A reset's clear goes through the
  same channel.
- Trainer: elites and run state after every generation (`trainer_store`), the assigned creature on
  `ASSIGN_SANDBOX_CREATURE`, delete and run state on `TRAINER_RESET`.

## Creature spawning and RNG

- Ocean founders (fresh start, inject, clone, reset, restock) come from one `OceanSpawner::founder(Random | Wildtype |
  Genome)`; mitosis children from `sim::systems::reproduce::child`. Both build agents through `CreatureFactory`
  (`spawn_from_phenotype`, TCK-185, TCK-152).
- Ocean: seeded `SimRng` streams `derive(seed, [STREAM_FOOD | STREAM_MITOSIS | STREAM_RESTOCK, tick])`,
  `[STREAM_ACTION, release count]`, `[STREAM_POPULATE]`; seed `ocean_seed(core.seed, nonce)` (a reset draws a clock
  nonce), stored with tick and release count in the snapshot. Same seed and inputs → same Ocean.
- Trainer: seeded `SimRng`, run seed `run_seed(config.seed, run_id, nonce)` (TCK-194), sandboxes stepped in parallel
  with `TrainerSim::step(true)`; reproducible per run ([[wiki/simulation/trainer]]).

## Hot-loop rules

- No SQL per step: species changes are collected per frame, the catalogue is read only when the population is below
  the target (once per restock).
- Spatial grid allocated once (240 × 135 cells of 80 px) and `clear()`ed per step; buckets store vector indices (RV-P02).
- Spore pushes use `SporePusher` kinematic snapshots, prey/bite lookups use `PeerSnapshot` — no agent clones (RV-P03);
  `OceanSim` reuses its buffers (`CreatureStep`, peers, damage map) across ticks.
- Details: [[wiki/architecture/performance]].

## Known limitations

- Mitosis children are reported to the UI but not simulated (RV-B01, TCK-143).
- The frontend still has its own world-size literals (`camera.ts`, `ocean.ts`); the backend sends the size in
  `INIT_STATE.world` (RV-A05, frontend part with TCK-160).
- The nearest-food lookup scans all spores and peers per creature (RV-P04).

## Key Takeaways

- `Engine` + `Mode` (`OceanMode`, `TrainerMode`) + `EventSink` + `EngineStore`; only `spawn_simulation_thread` needs Tauri.
- One thread, ~60 Hz, max 35 steps per frame (clamped and echoed), events flushed once per frame.
- The Ocean is the IO-free, seeded `OceanSim` on the shared creature systems; the trainer is `TrainerSim`.
- Store failures no longer kill the engine; the trainer is rebuilt from the store only when needed.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-129, TCK-139, TCK-149, TCK-156, TCK-173, TCK-176, TCK-177, TCK-184, TCK-185, TCK-186.
