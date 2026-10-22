---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/project-baseline/SKILL.md, README.md, docs/revisit/02-architecture.md, docs/revisit/06-frontend.md, src/main.rs, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-187-gene-spans-from-backend.md]
---

# IPC Protocol

## Frontend → backend

- `invoke("handle_client_action", { action: JSON.stringify({ type, ... }) })` — forwarded over an mpsc channel to the engine and matched on `type`.
- Action types: `CLIENT_READY`, `TOGGLE_SIMULATION`, `INJECT_URZELLE`, `SPAWN_CLONE`, `RESET_EVOLUTION`, `SELECT_AGENT`, `SET_SIMULATION_SPEED`, `SET_MODE`, `SET_TRAINER_MODE`, `START_TRAINING`, `PAUSE_TRAINING`, `TRAINER_RESET`, `SELECT_TRAINER_SANDBOX`, `ASSIGN_SANDBOX_CREATURE`, `UPDATE_TRAINER_HYPERPARAMS`, ...
- Unknown types are logged to stderr (TCK-141); missing or mistyped fields are silently ignored (`as_u64()` → `None`) (RV-A02).
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
- Both `frontend/ocean.ts` and `frontend/trainer.ts` listen to all events and dispatch on `type` (RV-F05).
- Field casing is mixed (`load_learned_synapses`, `sandbox_id` vs `warpSpeed`, `runId`) (RV-A02).
- `INIT_STATE` carries all creatures incl. phenotype/brain and the procedural world (once); `FOOD_TICK` all spores at 2 Hz.
- Trainer telemetry does not carry the per-sandbox world (removed in TCK-140).
- Navigation switches the backend mode via several fire-and-forget actions from `router.ts` (RV-F05).

## Types (ts-rs)

- Rust structs with `#[derive(TS)]` / `#[ts(export)]` (`CreatureAgent`, `FoodSpore`, `CreaturePhenotype`, `TelemetryCreature`, `TrainerTelemetrySandbox`, world types, ...) are exported to `bindings/` (27 files, incl. the trainer enums `Scenario` and `SandboxOrigin` used by `TrainerTelemetrySandbox` since TCK-173) when `cargo test` runs (= `npm run types:generate`).
- Brain v2 types (TCK-179, TCK-144, TCK-187): `NeuronRole` (tagged by `role`: `systemic`/`sensor`/`thalamus`/`hidden`/`motor`), `GeneRef`, `GeneSpan`, `OrganKind`, `OrganSide`, `SystemicInput`, `Motor`, `ThalamusKind`. `CTRNNNeuron` has `role`; `CTRNNSynapse` has `key` (53-bit, exact as a JS number) and `owner` (index into `CreaturePhenotype.genes`); `SensoryPatch` has `kind` and `side`; `CreaturePhenotype.genes` lists the gene spans. `CreatureAgent`/`TelemetryCreature` lost `adrenaline` (TCK-149).
- CI fails on uncommitted binding drift (`git diff --exit-code -- bindings/`), see [[wiki/quality/ci-and-tooling]].
- **Partly consumed** (RV-A03 partly resolved, TCK-187): `frontend/shared/types.ts` re-exports `CTRNNNeuron`, `CTRNNSynapse`, `BrainTopology`, `GeneSpan` from `bindings/`; `shared/genome.ts`, `render/brainRenderer.ts` and `views/UnifiedDiagnosticsPanel.ts` import binding types directly. The rest of `types.ts` (phenotype, agent, telemetry) is still handwritten and deviates (e.g. `OrganelleLocus` `x?/y?` vs Rust `SensoryPatch` `geneStartIndex/geneEndIndex/kind/side`) (RV-A03, RV-F01). ~40 `any`, mostly at the IPC boundary.
- Timestamp units are mixed across payloads and tables (RV-A06), see [[wiki/architecture/persistence]].

## Key Takeaways

- Commands in, one stringly-typed event out; the contract is not type-checked across the boundary (RV-A02).
- `bindings/` is generated and drift-checked in CI; brain and gene types are consumed by the frontend, the rest of the IPC types are still handwritten (RV-A03 partly).

Sources: `.agents/skills/server-simulation/SKILL.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/02-architecture.md`, `src/main.rs`, TCK-149, TCK-173, TCK-179, TCK-187.
