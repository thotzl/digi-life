---
sources: [src/lib.rs, src/main.rs, frontend/main.ts, package.json, Cargo.toml, tickets/closed/TCK-156-engine-decomposition.md, tickets/closed/TCK-157-typed-ipc-contract.md, tickets/closed/TCK-159-db-migrations-retention.md, tickets/closed/TCK-160-frontend-structure.md, tickets/closed/TCK-198-r1-sim-context-config-utils.md, tickets/closed/TCK-199-r2-genome-and-organ-registry.md, tickets/closed/TCK-200-r3-world-and-creature-systems.md, tickets/closed/TCK-201-r3-rooms-evaluator-breeder.md]
---

# Module Map

One row per module and per registered plugin, with the contract that governs it (TCK-203; replaces the former
repository map). Layout: `Cargo.toml`, `tauri.conf.json` and `package.json` at the root, `src/` = Rust, `frontend/` =
webview, `bindings/` = generated TS types. No functions or line numbers here; the contract names the public surface.
Update when a module or plugin is added, removed or changes responsibility. Runtime picture:
[[wiki/architecture/system-overview]].

## Rust (`src/`)

| Module | Responsibility | Contract / article |
| :--- | :--- | :--- |
| `main.rs`, `lib.rs` | Tauri setup, command registration, DB setup, config load, starts the engine; module tree | [[wiki/architecture/system-overview]] |
| `config/` | `AppConfig` (one field per section), loader, config tests; section `core`, `ocean` (`ocean.rs`) | [[wiki/contracts/config-section]] |
| `server/engine/` | `Engine`, `run_loop`, `Mode` trait and the modes `OceanMode`, `TrainerMode`; `EventSink` (`TauriSink`, `VecSink`); `EngineStore`, `SnapshotWriter` | [[wiki/contracts/mode]], [[wiki/contracts/event-sink]] |
| `server/ipc.rs`, `ocean_ipc.rs`, `trainer_ipc.rs` | typed actions, events, `IpcError`, `SimMode` per group (engine, Ocean, trainer) | [[wiki/contracts/ipc-message]] |
| `server/api/` | Tauri commands `actions`, `species`, `trainer`, `catalogue` with typed results; helpers `with_store`, `collect_rows`, `send_to_engine` | [[wiki/contracts/ipc-message]] |
| `persistence/` | the only module with SQL: `init_db`, migrations (`schema.rs`), repository traits (`repos.rs`), `Db` per table module, `MemoryDb`, DTOs | [[wiki/contracts/persistence-repo]] |
| `sim/` | shared creature kernel: `World`, `WorldSettings`, `TickCtx`, `step_creature`, `systems/` (one module per dynamic), `model_tests.rs` | [[wiki/contracts/creature-system]] |
| `sim/ocean/` | IO-free, seeded Ocean `OceanSim`: spawner, spore field, species ledger, snapshot | [[wiki/contracts/mode]], [[wiki/simulation/ocean]] |
| `trainer/` | IO-free trainer `TrainerSim` (`sim.rs`), `chamber.rs`, `sandbox.rs`, `breeder.rs`, `curriculum.rs`, `genomes.rs`, `mode.rs`, `stats.rs`, `streams.rs` | [[wiki/simulation/trainer]] |
| `trainer/room/` | `Room` trait, registry `ROOMS`, `LevelRoom` and its fitness | [[wiki/contracts/room]] |
| `trainer/evaluator.rs` | `Evaluator` trait, `SoloTrials`, `aggregate_trials` | [[wiki/contracts/evaluator]] |
| `biology/genome/` | alphabet, promoter registry, gene scan and table, payload reader, generator, mutation; section `genome` | [[wiki/biology/genome-compiler]] |
| `biology/phenotype/` | `parse_genome`, `CreaturePhenotype`, body decoders, Latin names | [[wiki/biology/genome-compiler]] |
| `biology/organs/` | `OrganKind`, `OrganSpec`, one file per kind, shared spectral/contact sensing | [[wiki/contracts/organ-kind]] |
| `biology/brain/` | brain model types (roles, motors, `Activation`) and the Brain v2 compiler | [[wiki/biology/ctrnn-brain]] |
| `biology/creature_factory.rs`, `metabolism.rs`, `diet.rs` | the one spawn path; the one energy function (section `metabolism`); diet classes | [[wiki/biology/metabolism-and-reproduction]] |
| `biology/trainer_config.rs`, `trainer_store.rs` | section `trainer`; adapter `TrainerSim` ↔ `TrainerRepo` | [[wiki/contracts/config-section]], [[wiki/contracts/persistence-repo]] |
| `shared/brain.rs` | CTRNN execution, Oja learning, sensory inputs; section `brain` | [[wiki/biology/ctrnn-brain]] |
| `shared/physics.rs`, `spatial_grid.rs` | kinematics, collisions, spore physics, neighbour grid; section `physics` | [[wiki/simulation/physics]] |
| `shared/` (rest) | `dto.rs` (simulation state structs), `map_generator.rs`, `rng.rs` (`SimRng`), `fmath.rs`, `temperature.rs`, `geom.rs`, `time.rs`, `json_col.rs`, `stats.rs` | [[wiki/architecture/invariants]] 16 |
| `diagnostics/` | runner feature and tests only: trainer report, Ocean report, mechanics probes | `wiki/simulation/analytics-and-balancing.md` |
| `bin/trainer_runner.rs` | headless runner CLI `run`, `probe`, `ocean` (feature `runner`) | `wiki/simulation/analytics-and-balancing.md` |
| `logging.rs` | `log` backend: stderr (dev, runner), rotating file (release app) | [[wiki/architecture/system-overview]] |

## Frontend (`frontend/`)

| Module | Responsibility | Contract / article |
| :--- | :--- | :--- |
| `main.ts`, `core/router.ts`, `core/view.ts`, `core/scope.ts` | boot, view registry routing, backend mode, lifecycle scope | [[wiki/contracts/frontend-view]] |
| `core/ipc.ts` | typed IPC client `call`, `send`, `setMode`, `onUiEvent`, error toast; the only Tauri caller | [[wiki/contracts/ipc-message]] |
| `core/` (rest) | camera, phenotype cache, `styled`, escaping, toasts | [[wiki/frontend/structure]] |
| `views/` | registry `VIEWS` (`index.ts`) and one folder per view; happy-dom suite `views.dom.test.ts` | [[wiki/contracts/frontend-view]] |
| `state/selection.ts` | app selection tagged by source; vitals and activations signals | [[wiki/contracts/frontend-view]] |
| `components/` | `Modal`, `CreaturePreview`, `CataloguePicker`, `DiagnosticsPanel`, icons | [[wiki/frontend/structure]] |
| `render/` | creature renderer, organ painters per `OrganKind`, world layer, brain renderer, progress chart | [[wiki/frontend/structure]] |
| `shared/` | display tables (genes, roles), diet labels, warp mapping | [[wiki/frontend/structure]] |

## Registered plugins

| Extension point | Registration point | Registered today | Contract |
| :--- | :--- | :--- | :--- |
| Organ kind | `OrganKind` + `OrganKind::spec` (`src/biology/organs/mod.rs`) | `Eye`, `Nose`, `Thermal`, `Bristle`, `LateralLine`, `Proprioceptor` | [[wiki/contracts/organ-kind]] |
| Creature system | `sim::System` + `step_creature`; lists in `src/sim/world.rs` | `Sense`, `Think`, `Metabolize`, `Death`, `Move`, `Eat` (+ owner-called `food_drift`, `bite`, `reproduce`) | [[wiki/contracts/creature-system]] |
| Room | `ROOMS` (`src/trainer/room/registry.rs`) | `standard`, `exploration` (`LevelRoom`) | [[wiki/contracts/room]] |
| Evaluator | `TrainerSim::set_evaluator` | `SoloTrials` | [[wiki/contracts/evaluator]] |
| Engine mode | `SimMode` + `Engine` fields and arms (`src/server/engine/mod.rs`) | `OceanMode`, `TrainerMode` | [[wiki/contracts/mode]] |
| Event sink | `Engine::new` / `run_loop` argument | `TauriSink`, `VecSink` | [[wiki/contracts/event-sink]] |
| IPC message | action/event enums per group; `generate_handler!` + `Commands` | 3 action groups, 3 event groups, 12 commands | [[wiki/contracts/ipc-message]] |
| Config section | `AppConfig` field + `config.json` object | `core`, `ocean`, `trainer`, `genome`, `brain`, `metabolism`, `physics` | [[wiki/contracts/config-section]] |
| Persistence repo | trait in `repos.rs`, `TABLES`, `schema::MIGRATIONS` | `SpeciesRepo`, `CatalogueRepo`, `TrainerRepo`, `OceanSnapshotRepo` | [[wiki/contracts/persistence-repo]] |
| Frontend view | `VIEWS` (`frontend/views/index.ts`) | `main-menu`, `ocean`, `trainer`, `catalogue` | [[wiki/contracts/frontend-view]] |

## Root

| Path | Responsibility |
| :--- | :--- |
| `config.json` | every simulation tunable, embedded into the binary |
| `bindings/` | ts-rs output, regenerated by `cargo test`, committed |
| `tests/reference/` | CI reference reports (trainer, Ocean) and the comparator |
| `tauri.conf.json`, `tauri.dev.conf.json`, `capabilities/`, `icons/` | Tauri config (CSP, bundle icons), dev identifier override, permissions |
| `scripts/` | doc link check `check-links.mjs`, portable Windows ZIP packaging |
| `.github/workflows/` | CI (tests, fmt, clippy, lint, references, link check), releases |
| `rustfmt.toml`, `eslint.config.js`, `vite.config.ts`, `vitest.config.ts`, `tsconfig.json` | tooling ([[wiki/quality/ci-and-tooling]]) |
| `wiki/`, `tickets/`, `.agents/`, `CHANGELOG.md`, `ideas.md` | knowledge, work items, agent rules and state, release notes, research dossier |

Largest source files (2026-11-03): `diagnostics/probes.rs` (~2,000 lines), `trainer/sim/tests.rs` (~1,340),
`trainer/sim.rs`, `persistence/tests.rs`, `server/trainer_ipc.rs`, `frontend/views/trainer/layout.ts` (~650–780).

## Key Takeaways

- Every extension point has one registration point and one contract; the plugin table lists what is registered.
- Rust: `server/` = engine, IPC, commands; `sim/` = creature kernel and Ocean; `trainer/` = trainer; `biology/` =
  genome, organs, brain, metabolism; `persistence/` = all SQL; `config/` + section structs = tunables.
- Frontend: router + view registry + one IPC client; renderers and components are shared.
