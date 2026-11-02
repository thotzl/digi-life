---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/project-baseline/SKILL.md, README.md, src/main.rs, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-187-gene-spans-from-backend.md, tickets/closed/TCK-177-trainer-ui-progress-metrics.md, tickets/closed/TCK-176-automatic-training-curriculum.md, src/server/ipc.rs, src/server/ocean_ipc.rs, src/server/trainer_ipc.rs, src/server/api/mod.rs, src/server/engine.rs, frontend/core/ipc.ts]
---

# IPC Protocol

Every message is typed end to end (TCK-157): Rust enums and structs exported to `bindings/` by ts-rs, consumed by
the frontend IPC client `frontend/core/ipc.ts`, the only module that calls Tauri. Rules for adding a message:
[[wiki/contracts/ipc-message]] and [[wiki/guides/add-ipc-message]].

## Frontend → backend: actions

- `send(action)` → command `handle_client_action(action)` (the action object itself, no JSON string) →
  `ClientAction::from_value` → mpsc channel `Sender<ClientAction>` → the engine thread drains it at the start of every
  frame and matches the variant.
- `ClientAction` (`src/server/ipc.rs`, untagged) = `EngineAction` | `OceanAction` | `TrainerAction`; each group is
  tagged by `type` (SCREAMING_SNAKE_CASE), the type names are disjoint.
- Rejected with a logged `IpcError` (`invalidAction`) and a toast, never forwarded: an unknown type ("unknown client
  action type 'X'"), a known type with missing or mistyped fields ("malformed client action 'X': ..."), and `SET_MODE`
  (only via the `set_mode` command). Before TCK-157 an unknown type was only logged by the engine and missing fields
  were ignored silently (RV-A02).

| Group | Action | Fields | Effect |
| :--- | :--- | :--- | :--- |
| Engine | `CLIENT_READY` | — | answers `INIT_STATE`, `DATABASE_CHANGED`, a log line; never changes the mode ([[wiki/architecture/invariants]] 1) |
| Engine | `SET_MODE` | `mode`: `ocean` \| `trainer` | only from the `set_mode` command; `trainer` restores the trainer population from the DB and answers `TRAINER_STATE_CHANGED` |
| Ocean | `TOGGLE_SIMULATION` | `running` | answers `SIM_STATE` |
| Ocean | `INJECT_URZELLE` | — | one structured random founder; at `ocean.maxCreatures` a "Population cap reached" log line instead |
| Ocean | `SPAWN_CLONE` | `SpawnClone`: `genome`, optional `methylations`, `synapse_weights`, `load_learned_synapses` | sent by the command `spawn_catalogue_creature_to_ocean`; same cap rule |
| Ocean | `RESET_EVOLUTION` | — | clears species, snapshot, creatures and the newborn queue ([[wiki/architecture/invariants]] 3), repopulates, answers `INIT_STATE` |
| Ocean | `SELECT_AGENT` | `id` (optional) | creature whose activations `TELEMETRY_TICK.selectedBrain` carries |
| Ocean | `SET_SIMULATION_SPEED` | `speed` | Ocean steps per frame, clamped to 35 ([[wiki/architecture/invariants]] 2) |
| Trainer | `UPDATE_TRAINER_HYPERPARAMS` | `TrainerHyperparamsUpdate`, all optional: `N` (visible sandboxes only, TCK-192), `population` (restores the population from the DB when it changes), `warpSpeed`, `eliteRatio`, `mutationRate` (max ops per child, name kept), `inflowRate`, `hofRate`, `multiTrial`, `isHeadless`, `runId`, `lamarckian` | absent fields unchanged; population, Lamarck or run change restores the population from the DB; answers `TRAINER_STATE_CHANGED` |
| Trainer | `START_TRAINING` / `PAUSE_TRAINING` | — | running flag; answers `TRAINER_STATE_CHANGED` |
| Trainer | `TRAINER_RESET` | — | deletes the run's rows, fresh `TrainerSim` (automatic curriculum back to `startLevel`); answers `TRAINER_RESET_COMPLETED` |
| Trainer | `SET_TRAINER_MODE` | `mode`: `standard` \| `exploration` \| `curriculum` | **deferred** to the next generation boundary (at once at a generation start); `curriculum` turns the gate on; answers `TRAINER_STATE_CHANGED` with `pendingMode` (TCK-176) |
| Trainer | `SET_TRAINER_LEVEL` | `level` in [0, 1] | manual difficulty level, gate off; deferred like `SET_TRAINER_MODE` |
| Trainer | `SELECT_TRAINER_SANDBOX` | `id` (optional) | brain view source |
| Trainer | `ASSIGN_SANDBOX_CREATURE` | `sandbox_id`, `genome`, optional `methylations`, `synapse_weights`, `load_learned_synapses` | rebuilds the sandbox creature, stores it; answers `DATABASE_CHANGED` |

- Field casing: camelCase, except `SpawnClone` and `AssignSandboxCreature` (snake_case kept).
- Removed in TCK-157: `SPAWN_FOOD` (right-click on empty water; the engine never handled it, so right-click food spawn
  still does nothing, RV-B09) and `CLIENT_LOG` (newborn log line of the Ocean view; unknown to the engine, only
  produced warnings). Neither had an effect before.
- Tests: `ipc::tests::test_client_actions_parse_into_their_group`, `test_unknown_and_malformed_actions_are_rejected_with_the_type`, `trainer_ipc::tests::test_trainer_actions_parse_from_ui_json`.

## Tauri commands (`src/server/api/`, registered in `src/main.rs`)

Every command returns `Result<T, IpcError>`; `T` is a ts-rs DTO. `IpcError { kind, message }` with `kind` =
`invalidAction` | `notFound` | `database` | `engineUnavailable`. Store access goes through `with_store` /
`collect_rows` (`src/server/api/mod.rs`), which log store errors.

| Module | Command | Args | Result |
| :--- | :--- | :--- | :--- |
| `actions.rs` | `handle_client_action` | `action: ClientAction` | `null` |
| `actions.rs` | `set_mode` | `mode: SimMode` | `SimMode` (queued after the actions sent before it) |
| `species.rs` | `get_registered_species` | — | `SpeciesEntry[]` |
| `species.rs` | `get_fossil_phenotype` (async, runs `parse_genome`) | `genome` | `CreaturePhenotype` |
| `trainer.rs` | `get_trainer_runs` | — | `TrainerRunInfo[]` (`runId`, `maxGeneration`, `maxFitness`) |
| `trainer.rs` | `clear_trainer_history` | `runId` | `null` |
| `catalogue.rs` | `get_catalogue_creatures` | — | `CatalogueCreature[]` |
| `catalogue.rs` | `save_to_catalogue` | `id`, `name`, `genome`, `source`, `fitness`, `methylations`, `synapseWeights` | `null` |
| `catalogue.rs` | `delete_from_catalogue` | `id` | `null` |
| `catalogue.rs` | `rename_catalogue_creature` | `id`, `newName` | `null` |
| `catalogue.rs` | `spawn_catalogue_creature_to_ocean` | `id`, `loadLearnedSynapses` | `null` (sends a typed `SPAWN_CLONE`; `notFound` for an unknown id) |
| `catalogue.rs` | `add_catalogue_creature_to_training` | `creatureId`, `runId`, `generation` | `null` (`notFound` for an unknown id) |

- Removed in TCK-157 (no caller in `frontend/`, checked by grep): `get_trainer_population`, `get_trainer_hof`,
  `save_trainer_generation` (deactivated a run's genomes and dropped their weights), `apply_champion` (wrote the
  never-read `progenitor_genome` key, RV-D06; its repository method went with it).
- All DB commands share one mutex-guarded connection (`DbState`), see [[wiki/architecture/persistence]].

## Backend → frontend: events

- One Tauri event `simulation-state` (`UI_EVENT_NAME`), payload `UiEvent` (untagged) = `EngineEvent` |
  `OceanUiEvent` | `TrainerUiEvent`, each tagged by `type`. The engine buffers typed events (`emit_state`) and emits
  them in order once per frame; repeated `DATABASE_CHANGED` within a frame are coalesced.
- The frontend has one listener (`onUiEvent` in `frontend/core/ipc.ts`); `ocean.ts` and `trainer.ts` register a
  handler each and switch on `type` (per-view listener lifecycle: TCK-160).

| Group | Event | Payload |
| :--- | :--- | :--- |
| Engine | `DATABASE_CHANGED` | — (UI refetches roster, runs) |
| Engine | `LOG_EVENT` | `message`, `logType`: `system` \| `mutation` \| `repair` (Ocean log terminal) |
| Ocean | `INIT_STATE` | `OceanInitState`: `highestGeneration`, `running`, `seed`, `world` (procedural world, once), `rules.sporeEnergy`, `foodPellets`, `creatures` (full, incl. phenotype/brain) |
| Ocean | `SIM_STATE` | `running` |
| Ocean | `TELEMETRY_TICK` (25 Hz) | `OceanTelemetryTick`: `highestGeneration`, `creatures` (`TelemetryCreature`), `newCreatures` (full, sent once), `selectedBrain` (`{ id, activations }` or null) |
| Ocean | `FOOD_TICK` (2 Hz) | `foodPellets` |
| Ocean | `EAT_EVENT` | `x`, `y` (no consumer in the UI) |
| Ocean | `BITE_EVENT` | `attackerId`, `victimId`, `x`, `y` |
| Trainer | `TRAINER_TELEMETRY_TICK` (25 Hz) | `TrainerTelemetryTick`: `generation`, `isRunning`, `timeStr`, `sandboxes` (a sample of `N` visible sandboxes covering every origin group, `display_slots`, TCK-192, TCK-194; `TrainerTelemetrySandbox` incl. `scenario` and `level`), `selectedBrain` |
| Trainer | `TRAINER_GENERATION_COMPLETED` | `TrainerGenerationCompleted`: `generation`, `bestFitness`, `avgFitness`, `bestGenome`, `mutationOpsMax`, `trainingMode`, `nextLevel`, `stats` (`TrainerGenerationStats`, built `From<&GenerationStats>`, one source for UI and headless report; `layoutId` as a decimal string) |
| Trainer | `TRAINER_STATE_CHANGED` / `TRAINER_RESET_COMPLETED` | `TrainerState`: `isRunning`, `population`, `N`, `warpSpeed`, `eliteRatio`, `mutationRate`, `mutationOpsMin`, `mutationOpsLimit`, `inflowRate`, `hofRate`, `multiTrial`, `isHeadless`, `runId`, `lamarckian`, `trainingMode`, `level`, `levelAuto`, `pendingMode`, `generation`; sent after every state-changing trainer action and after every completed generation |

- Number spelling: events are serialized from the typed values, so f32 fields keep their shortest f32 spelling
  (before TCK-157 the `json!` payloads widened them to f64, e.g. 0.9 → 0.8999999761581421); the parsed f32 values on
  the way back are identical.
- Tests: `test_engine_events_and_errors_keep_their_wire_names`, `ocean_ipc::tests::test_ocean_events_keep_type_tag_and_ui_field_names`, `trainer_ipc::tests::test_trainer_events_keep_type_tag_and_ui_field_names`.

## Frontend IPC client (`frontend/core/ipc.ts`)

- `call(command, args)` — typed by the `Commands` map; resolves with the DTO, on failure reports the `CallError`
  (`IpcError` or `transport` for Tauri-side failures such as bad arguments) through the error hook and rejects.
  Outside Tauri (plain `vite dev`) it rejects with `unavailable` without a toast.
- `send(action)` — `Action` = `ClientAction` without `SET_MODE`; resolves `true` / `false` (accepted / rejected), never
  rejects.
- `setMode(mode)` — the `set_mode` command; called by `navigateTo` and by the trainer's unload reset to Ocean only
  ([[wiki/architecture/invariants]] 1). `navigateTo` awaits `TOGGLE_SIMULATION false` → `PAUSE_TRAINING` →
  `set_mode` in this order.
- `onUiEvent(handler)` — the one `simulation-state` listener; `onIpcError(handler)` replaces the default error hook,
  a toast (`frontend/core/toast.ts`, text only). Replaces `safeInvoke` (`frontend/api.ts`), which returned `null` on
  errors (RV-F04); the phenotype cache moved to `frontend/core/phenotypes.ts`.

## Types (ts-rs)

- Every IPC type has `#[derive(TS)] #[ts(export)]` and is written to `bindings/` by `cargo test` (=
  `npm run types:generate`); CI fails on uncommitted drift (`git diff --exit-code -- bindings/`, [[wiki/quality/ci-and-tooling]]).
- Protocol types: `ClientAction`, `EngineAction`, `OceanAction`, `SpawnClone`, `TrainerAction`,
  `TrainerHyperparamsUpdate`, `AssignSandboxCreature`, `SimMode`, `UiEvent`, `EngineEvent`, `LogKind`, `OceanUiEvent`,
  `OceanInitState`, `OceanRules`, `OceanTelemetryTick`, `SelectedBrain`, `TrainerUiEvent` and its payloads,
  `IpcError`, `IpcErrorKind`; command results `SpeciesEntry`, `SpeciesLifeStatus`, `TrainerRunInfo`,
  `CatalogueCreature`; model types `CreatureAgent`, `FoodSpore`, `TelemetryCreature`, `CreaturePhenotype`, world and
  brain types.
- `i64` fields are typed `number` (`#[ts(type = "number")]`); u64 values are sent as decimal strings.
- `frontend/shared/types.ts` re-exports the brain, gene, species, food and world types from `bindings/`; the
  phenotype and agent model types used by the renderers are still handwritten (the binding `CreatureAgent` is
  assignable to them; alignment in TCK-160, RV-A03 partly, RV-F01).
- Timestamp units are mixed across payloads and tables (RV-A06), see [[wiki/architecture/persistence]].

## Key Takeaways

- Actions in through `handle_client_action` / `set_mode`, events out on `simulation-state`, all typed by ts-rs
  enums; no untyped payload is left (TCK-157).
- Invalid actions are rejected at the command with a typed error that reaches the user as a toast.
- Commands return typed DTOs and `IpcError`; four dead commands were removed.

Sources: `.agents/skills/server-simulation/SKILL.md`, `src/main.rs`, `src/server/ipc.rs`, `src/server/ocean_ipc.rs`, `src/server/trainer_ipc.rs`, `frontend/core/ipc.ts`, TCK-157, TCK-173, TCK-176, TCK-177, TCK-179, TCK-187.
