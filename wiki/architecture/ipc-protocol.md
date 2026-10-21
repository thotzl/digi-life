---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/project-baseline/SKILL.md, README.md, docs/revisit/02-architecture.md, docs/revisit/06-frontend.md, src/main.rs]
---

# IPC Protocol

## Frontend → backend

- `invoke("handle_client_action", { action: JSON.stringify({ type, ... }) })` — forwarded over an mpsc channel to the engine and matched on `type`.
- Action types: `CLIENT_READY`, `TOGGLE_SIMULATION`, `INJECT_URZELLE`, `SPAWN_CLONE`, `RESET_EVOLUTION`, `SELECT_AGENT`, `SET_SIMULATION_SPEED`, `SET_MODE`, `SET_TRAINER_MODE`, `START_TRAINING`, `PAUSE_TRAINING`, `TRAINER_RESET`, `SELECT_TRAINER_SANDBOX`, `ASSIGN_SANDBOX_CREATURE`, `UPDATE_TRAINER_HYPERPARAMS`, ...
- Unknown types are logged to stderr (TCK-141); missing or mistyped fields are still silently ignored (`as_u64()` → `None`) (RV-A02).
- `SPAWN_FOOD` (right-click) and `CLIENT_LOG` are sent by `frontend/ocean.ts` but unknown to the engine; `SPAWN_FOOD` is a no-op (RV-B09, TCK-147).

### Tauri commands (`src/server/api.rs`, registered in `src/main.rs`)

| Area | Commands |
| :--- | :--- |
| Engine | `handle_client_action` |
| Species | `get_registered_species` |
| Trainer | `get_trainer_runs`, `get_trainer_population`, `get_trainer_hof`, `save_trainer_generation`, `clear_trainer_history`, `apply_champion` |
| Phenotype | `get_fossil_phenotype` (runs `parse_genome`) |
| Catalogue | `get_catalogue_creatures`, `save_to_catalogue`, `delete_from_catalogue`, `rename_catalogue_creature`, `spawn_catalogue_creature_to_ocean`, `add_catalogue_creature_to_training` |

- All DB commands share one mutex-guarded connection (`DbState`), see [[architecture/persistence]].

## Backend → frontend

- Single event `"simulation-state"`; payload `type` is a string: `INIT_STATE`, `SIM_STATE`, `TELEMETRY_TICK`, `FOOD_TICK`, `TRAINER_*`, `DATABASE_CHANGED`, `LOG_EVENT`, `EAT_EVENT`, `BITE_EVENT`, ...
- Both `frontend/ocean.ts` and `frontend/trainer.ts` listen to all events and dispatch on `type` (RV-F05).
- Field casing is mixed (`load_learned_synapses`, `sandbox_id` vs `warpSpeed`, `runId`) (RV-A02).
- `INIT_STATE` carries all creatures incl. phenotype/brain and the procedural world (once); `FOOD_TICK` all spores at 2 Hz.
- Trainer telemetry no longer carries the per-sandbox world (removed in TCK-140).

## Types (ts-rs)

- Rust structs with `#[derive(TS)]` / `#[ts(export)]` (`CreatureAgent`, `FoodSpore`, `CreaturePhenotype`, `TelemetryCreature`, `TrainerTelemetrySandbox`, world types, ...) are exported to `bindings/` (17 files) when `cargo test` runs (= `npm run types:generate`).
- CI fails on uncommitted binding drift (`git diff --exit-code -- bindings/`), see [[quality/ci-and-tooling]].
- **The frontend does not import `bindings/`**; it uses handwritten `frontend/shared/types.ts`, which already deviates (e.g. `OrganelleLocus` `x?/y?` vs Rust `SensoryPatch` `geneStartIndex/geneEndIndex`) (RV-A03, RV-F01). ~41 `any`, mostly at the IPC boundary.

## Target (pool TCK-157)

- `#[derive(Deserialize, TS)] #[serde(tag = "type")] enum Action { ... }` taken directly by a command (no `JSON.stringify`).
- `#[derive(Serialize, TS)] enum SimEvent { ... }` on separate events (`ocean://telemetry`, `trainer://telemetry`, `db://changed`, `log`).
- `frontend/shared/types.ts` re-exports from `bindings/`; one timestamp unit (RV-A06).
- Mode switch as one command with a return value instead of 3–4 fire-and-forget actions (RV-F05, TCK-160).

## Key Takeaways

- Commands in, one stringly-typed event out; contract is not type-checked across the boundary.
- `bindings/` is generated and drift-checked in CI but not consumed by the frontend.
- Typed `Action`/`SimEvent` enums are the planned fix (TCK-157).

Sources: `.agents/skills/server-simulation/SKILL.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/02-architecture.md`, `src/main.rs`.
