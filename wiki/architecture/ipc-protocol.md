---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/project-baseline/SKILL.md, README.md, docs/revisit/02-architecture.md, docs/revisit/06-frontend.md, src/main.rs, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-187-gene-spans-from-backend.md, tickets/closed/TCK-177-trainer-ui-progress-metrics.md, tickets/closed/TCK-176-automatic-training-curriculum.md, src/server/trainer_ipc.rs, src/server/engine.rs]
---

# IPC Protocol

## Frontend → backend

- `invoke("handle_client_action", { action: JSON.stringify({ type, ... }) })` — forwarded over an mpsc channel to the engine and matched on `type`.
- Action types: `CLIENT_READY`, `TOGGLE_SIMULATION`, `INJECT_URZELLE`, `SPAWN_CLONE`, `RESET_EVOLUTION`, `SELECT_AGENT`, `SET_SIMULATION_SPEED`, `SET_MODE`, ... (untyped, matched on the raw JSON) and the typed trainer actions (below).
- Unknown types are logged to stderr (TCK-141); in the untyped actions missing or mistyped fields are silently ignored (`as_u64()` → `None`) (RV-A02). A malformed trainer action does not parse as `TrainerAction` and is logged and ignored as a whole ("unknown or malformed client action type").

### Typed trainer actions (TCK-177)

`TrainerAction` (`src/server/trainer_ipc.rs`, `#[serde(tag = "type")]`, ts-rs binding `TrainerAction`) is parsed first in `engine.rs`; the frontend sends it through `sendTrainerAction(action: TrainerAction)` (`frontend/trainer.ts`).

| Action | Fields | Effect |
| :--- | :--- | :--- |
| `UPDATE_TRAINER_HYPERPARAMS` | `TrainerHyperparamsUpdate`, all optional: `N`, `warpSpeed`, `eliteRatio`, `mutationRate` (max ops per child, name kept), `inflowRate`, `hofRate`, `multiTrial`, `isHeadless`, `runId`, `lamarckian` | absent fields unchanged; population, Lamarck or run change restores the population from the DB; answers `TRAINER_STATE_CHANGED` |
| `START_TRAINING` / `PAUSE_TRAINING` | — | running flag; answers `TRAINER_STATE_CHANGED` |
| `TRAINER_RESET` | — | deletes the run's rows, fresh `TrainerSim` (automatic curriculum back to `startLevel`); answers `TRAINER_RESET_COMPLETED` |
| `SET_TRAINER_MODE` | `mode`: `standard` \| `exploration` \| `curriculum` | **deferred**: pending until the next generation boundary (at once at a generation start); `curriculum` turns the gate on; answers `TRAINER_STATE_CHANGED` with `pendingMode` (TCK-176; before P8 it switched immediately) |
| `SET_TRAINER_LEVEL` (new in P8) | `level` in [0, 1] | manual difficulty level, gate off; deferred like `SET_TRAINER_MODE`; answers `TRAINER_STATE_CHANGED` |
| `SELECT_TRAINER_SANDBOX` | `id` (optional / null) | brain view source |
| `ASSIGN_SANDBOX_CREATURE` | `sandbox_id`, `genome`, optional `methylations`, `synapse_weights`, `load_learned_synapses` (snake case kept) | rebuilds the sandbox creature, stores it; answers `DATABASE_CHANGED` |

Tests `test_trainer_actions_parse_from_ui_json`, `test_non_trainer_actions_are_not_trainer_actions`.
- `SPAWN_FOOD` (right-click) and `CLIENT_LOG` are sent by `frontend/ocean.ts` but unknown to the engine; right-click food spawn has no effect (RV-B09).

### Tauri commands (`src/server/api.rs`, registered in `src/main.rs`)

| Area | Commands |
| :--- | :--- |
| Engine | `handle_client_action` |
| Species | `get_registered_species` |
| Trainer | `get_trainer_runs`, `get_trainer_population`, `get_trainer_hof`, `save_trainer_generation`, `clear_trainer_history`, `apply_champion` |
| Phenotype | `get_fossil_phenotype` (runs `parse_genome`) |
| Catalogue | `get_catalogue_creatures`, `save_to_catalogue`, `delete_from_catalogue`, `rename_catalogue_creature`, `spawn_catalogue_creature_to_ocean`, `add_catalogue_creature_to_training` |

- All DB commands share one mutex-guarded connection (`DbState`), see [[wiki/architecture/persistence]].

## Backend → frontend

- Single event `"simulation-state"`; payload `type` is a string: `INIT_STATE`, `SIM_STATE`, `TELEMETRY_TICK`, `FOOD_TICK`, `TRAINER_*`, `DATABASE_CHANGED`, `LOG_EVENT`, `EAT_EVENT`, `BITE_EVENT`, ...
- **Trainer events are typed** (TCK-177): enum `TrainerUiEvent` (`src/server/trainer_ipc.rs`, tag `type`, serialized by `to_payload`), listened to as `TrainerUiEvent | { type: "DATABASE_CHANGED" }` in `frontend/trainer.ts`:
  - `TRAINER_TELEMETRY_TICK` (`TrainerTelemetryTick`, 25 Hz): `generation`, `isRunning`, `timeStr`, `sandboxes` (`TrainerTelemetrySandbox`, incl. `scenario` = training mode and `level` of the sandbox's trial room), `selectedBrain` (`TrainerSelectedBrain` `{ id, activations }` or null).
  - `TRAINER_GENERATION_COMPLETED` (`TrainerGenerationCompleted`): `generation`, `bestFitness`, `avgFitness`, `bestGenome`, `mutationOpsMax`, `trainingMode`, `nextLevel`, `stats` (`TrainerGenerationStats`, built `From<&GenerationStats>`: success rate, level success rate, rehearsal fields, best/median/mean fitness, mean/median time to first catch, immobile and starved share, scenario, level, `levelMove`, `mutationOpsMax`, `layoutId` as a decimal string). One source for UI and headless report (test `test_generation_stats_match_report_serialization`).
  - `TRAINER_STATE_CHANGED` / `TRAINER_RESET_COMPLETED` (`TrainerState`): `isRunning`, `N`, `warpSpeed`, `eliteRatio`, `mutationRate`, `mutationOpsMin`, `mutationOpsLimit`, `inflowRate`, `hofRate`, `multiTrial`, `isHeadless`, `runId`, `lamarckian`, `trainingMode`, `level`, `levelAuto`, `pendingMode`, `generation`. Sent after every trainer action that changes state and **after every completed generation** (right after `TRAINER_GENERATION_COMPLETED`), so the UI sees applied switches, gate moves and adapted ops (test `test_trainer_events_keep_type_tag_and_ui_field_names`).
- Both `frontend/ocean.ts` and `frontend/trainer.ts` listen to all events and dispatch on `type` (RV-F05); the trainer narrows the payload by the typed union.
- Field casing is mixed (`load_learned_synapses`, `sandbox_id` vs `warpSpeed`, `runId`) (RV-A02).
- `INIT_STATE` carries all creatures incl. phenotype/brain and the procedural world (once); `FOOD_TICK` all spores at 2 Hz.
- Trainer telemetry does not carry the per-sandbox world (removed in TCK-140).
- Navigation switches the backend mode via several fire-and-forget actions from `router.ts` (RV-F05).

## Types (ts-rs)

- Rust structs with `#[derive(TS)]` / `#[ts(export)]` (`CreatureAgent`, `FoodSpore`, `CreaturePhenotype`, `TelemetryCreature`, `TrainerTelemetrySandbox`, world types, ...) are exported to `bindings/` (37 files, incl. the trainer enums `Scenario` and `SandboxOrigin` used by `TrainerTelemetrySandbox` since TCK-173, and since P8 the trainer IPC types `TrainerUiEvent`, `TrainerTelemetryTick`, `TrainerSelectedBrain`, `TrainerGenerationCompleted`, `TrainerGenerationStats`, `TrainerState`, `TrainerAction`, `TrainerHyperparamsUpdate`, `AssignSandboxCreature`, `LevelMove`, TCK-177) when `cargo test` runs (= `npm run types:generate`).
- Brain v2 types (TCK-179, TCK-144, TCK-187): `NeuronRole` (tagged by `role`: `systemic`/`sensor`/`thalamus`/`hidden`/`motor`), `GeneRef`, `GeneSpan`, `OrganKind`, `OrganSide`, `SystemicInput`, `Motor`, `ThalamusKind`. `CTRNNNeuron` has `role`; `CTRNNSynapse` has `key` (53-bit, exact as a JS number) and `owner` (index into `CreaturePhenotype.genes`); `SensoryPatch` has `kind` and `side`; `CreaturePhenotype.genes` lists the gene spans. `CreatureAgent`/`TelemetryCreature` lost `adrenaline` (TCK-149).
- CI fails on uncommitted binding drift (`git diff --exit-code -- bindings/`), see [[wiki/quality/ci-and-tooling]].
- **Partly consumed** (RV-A03 partly resolved, TCK-187, TCK-177): the trainer view (`frontend/trainer.ts`, `frontend/trainerMetrics.ts`) imports its event, action and stats types from `bindings/`; `frontend/shared/types.ts` re-exports `CTRNNNeuron`, `CTRNNSynapse`, `BrainTopology`, `GeneSpan` from `bindings/`; `shared/genome.ts`, `render/brainRenderer.ts` and `views/UnifiedDiagnosticsPanel.ts` import binding types directly. The rest of `types.ts` (phenotype, agent, telemetry) is still handwritten and deviates (e.g. `OrganelleLocus` `x?/y?` vs Rust `SensoryPatch` `geneStartIndex/geneEndIndex/kind/side`) (RV-A03, RV-F01). ~40 `any`, mostly at the IPC boundary.
- Timestamp units are mixed across payloads and tables (RV-A06), see [[wiki/architecture/persistence]].

## Key Takeaways

- Commands in, one event channel out; the trainer events and actions are typed end to end by ts-rs (TCK-177), the Ocean contract is still stringly typed and not type-checked across the boundary (RV-A02).
- `bindings/` is generated and drift-checked in CI; brain, gene and trainer IPC types are consumed by the frontend, the rest of the IPC types are still handwritten (RV-A03 partly).
- Trainer room switches (`SET_TRAINER_MODE`, `SET_TRAINER_LEVEL`) are deferred to the next generation boundary; `TRAINER_STATE_CHANGED` follows every generation.

Sources: `.agents/skills/server-simulation/SKILL.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/02-architecture.md`, `src/main.rs`, `src/server/trainer_ipc.rs`, TCK-149, TCK-173, TCK-176, TCK-177, TCK-179, TCK-187.
