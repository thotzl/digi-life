---
name: project-baseline
description: Baseline configuration, developer toolchains, environment scripts, and structural setup of the Pixel DNA Life project. Read this before running builds, scripts, or managing dependencies.
---
# project-baseline: Core Architecture & Developer Toolchain

## I. Universal Directives
- **Single Source of Truth:** All AI operations MUST respect the modular skills in `.agents/skills/`, which supersede tool-specific configs (e.g. `.cursorrules`).
- **Current work plan:** `docs/revisit/roadmap.md` (phased plan, finding IDs `RV-*`). Open tickets live in `.tickets/{open,ongoing,closed}/`.

## II. Tech Stack
- **Shell:** Tauri v2 desktop app. `Cargo.toml` (package `src-tauri`, edition 2024) and `package.json` both live at the repository root. There is no `src-tauri/` directory.
- **Backend:** Rust core in `src/`:
  - `main.rs` — Tauri builder, registers commands, spawns the simulation thread.
  - `server/engine.rs` — simulation thread (Ocean + Trainer loop, ~60 Hz, emits `simulation-state`).
  - `server/api.rs` — Tauri commands (`handle_client_action` + DB/catalogue/trainer queries).
  - `biology/dna.rs` — genome compiler (`parse_genome`), mutation; `biology/trainer_engine.rs` + `biology/scenarios.rs` — trainer sandboxes and `TrainingScenarioPlugin` trait.
  - `shared/brain.rs` (CTRNN + sensory inputs), `shared/physics.rs`, `shared/spatial_grid.rs`, `shared/map_generator.rs`, `shared/types.rs` (agent structs, `AppConfig`).
  - `database.rs` — SQLite schema (`init_db`).
  - `bin/cli_trainer.rs` — outdated balance CLI (3-input model, writes to the production DB; see RV-S11). Do not trust its output.
- **Frontend:** `frontend/` — Vite 5 + TypeScript 5 + `@preact/signals-core`, no UI framework. Views in `frontend/views/`, renderers in `frontend/render/`, loop modules `frontend/ocean.ts` / `frontend/trainer.ts`, router `frontend/core/router.ts`.
- **Persistence:** SQLite file `pixel_life_local.db` in the Tauri app data dir (resolved via `app_data_dir` at startup; a legacy `target/pixel_life_local.db` is copied once). Tables: `sessions`, `species_records`, `simulation_history`, `simulation_state`, `trainer_genomes`, `creature_catalogue`. No JSON database files exist.
- **IPC:** Frontend -> `invoke("handle_client_action", { action: JSON.stringify({ type, ... }) })` plus the other commands in `api.rs`. Backend -> one Tauri event `"simulation-state"` whose payload `type` is a string (`INIT_STATE`, `SIM_STATE`, `TELEMETRY_TICK`, `FOOD_TICK`, `TRAINER_*`, `DATABASE_CHANGED`, `LOG_EVENT`, ...). There is no WebSocket and no network port.
- **Types:** `ts-rs` exports Rust types to `bindings/` when `cargo test` runs. The frontend does NOT import them yet; it uses the handwritten `frontend/shared/types.ts` (RV-A03).

## III. Developer Commands (run from repository root)
| Command | Purpose |
| :--- | :--- |
| `npm run tauri:dev` | Launch the desktop app (Vite dev server + Rust build). |
| `npm run build` | `tsc` type check + Vite build into `dist/`. |
| `cargo test` (= `npm run test:rust`, `npm run types:generate`) | Rust unit tests; also regenerates `bindings/`. |
| `cargo clippy` | Lint. |
| `npm run test` | Vitest — currently there are no frontend test files. |

- **Toolchain (Windows):** rustup `stable-msvc`, MSVC Build Tools, WebView2, Node.js.
- **CI:** `.github/workflows/ci.yml` runs `npm run build`, `cargo test` and a `bindings/` drift check; `release.yml` builds installers only on `v*` tags. `.gitattributes` pins generated `bindings/` and `gen/` files to LF.
- **Versioning:** one patch bump per closed ticket, synchronized in `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`/`Cargo.lock`, with a `CHANGELOG.md` entry referencing the ticket.

## IV. Core Mandates
- **config.json (SSOT):** Simulation parameters live in root `config.json`, loaded into `AppConfig` (`src/shared/types.rs`). The file is compiled into the binary via `include_str!`; a `config.json` in the working directory overrides it for development. Do not hardcode new tunables in Rust; add them to `config.json` + `SimulationRules`. Note: several existing rules are never read (RV-S01).
- **No DB work in the hot loop:** Keep SQLite access out of the per-step physics path; batch writes on timers or events (RV-P01).
- **Bindings:** When changing a `#[derive(TS)]` struct, run `cargo test` and commit the regenerated `bindings/`.
