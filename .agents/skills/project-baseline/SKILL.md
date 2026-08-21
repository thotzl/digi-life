---
name: project-baseline
description: Baseline configuration, developer toolchains, environment scripts, and structural setup of the Pixel DNA Life project. Read this before running builds, scripts, or managing dependencies.
---
# project-baseline: Core Architecture & Developer Toolchain

## I. Universal Directives
- **Single Source of Truth:** All AI operations MUST respect the modular skills in `.agents/skills/` directory, which supersedes general global instructions or local config overrides (e.g., `.cursorrules`, `.gemini/`).
- **Self-Configuration Mandate:** Register and load the local skills dynamically. Treat this repository as a fully portable AI workspace.

## II. Tech Stack Discovery
- **Runtime:** Tauri v2 Desktop Environment.
- **Frontend Framework:** Vite v5 (TypeScript v5) serving high-fps HTML5 Canvas & Preact Signals client UI.
- **Backend Framework:** Native, headless Rust 2024 simulation and neuroevolution core running in a parallel background thread.
- **Persistence Layer:** Local SQLite database (`pixel_life_local.db`) for tracking training runs, HOF lineages, and active/extinct species records.
- **Core Dependencies:**
  - `@preact/signals-core`: Fine-grained, reactive state tracking for UI overlays.
  - `tauri`: Cross-platform desktop application framework.
  - `@tauri-apps/api`: Type-safe IPC bridge and command invoker.

## III. Primary Workflows & Developer Commands
All commands must be executed directly from the **repository root directory**.
- **Launch Desktop Application:**
  ```bash
  npm run tauri:dev
  ```
  Launches the main Ozean-Labor substrate and Evolutionary Trainer Tauri windows simultaneously.
- **Production Frontend Build:**
  ```bash
  npm run build
  ```
  Runs `tsc` type checking and compiles client assets through Vite into `dist/` for Tauri compilation.
- **Run Native Backend Tests:**
  ```bash
  npm run test:rust
  ```
  Executes the automated Rust unit-tests suite (DNA parsing, CTRNN brain determinism, SQLite constraints).

## IV. Core Mandates
- **config.json (SSOT):** All physical, biological, and metabolic parameters are loaded dynamically from the central `config.json` file in the root directory. Hardcoding literals in Rust is prohibited.
- **SQLite Transactions:** DB writes of Elite Champions must be performed asynchronously inside the background loop to prevent thread freezes.

## V. Legacy Simulations Archive (Checkout Guide)
The legacy, browser-based web assets (Express, standalone WebSockets, and WASD preview sandbox) have been retired in favour of the native desktop app.
* **Deletion Commit:** `chore: remove legacy browser-based assets and archive configurations` (Commit Hash: **`dd36b90`**).
* **Retrieval Guide:** Devs and AI agents can temporarily check out the retired web assets at any time using:
  ```bash
  git checkout dd36b90^
  ```
  Refer to `OLD_SIMS_ARCHIVE.md` in the root directory for full code structures and operational notes on the legacy views.
