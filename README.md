# 🧬 Pixel DNA Life - Native Desktop Lab

A native 2D biomorphic ecosystem and neuroevolution trainer, built as a desktop application with **Tauri v2**, **Rust 2024**, **SQLite**, and **Preact Signals**.

Current status and work plan: [`docs/revisit/roadmap.md`](docs/revisit/roadmap.md).

---

## 🚀 Execution Commands

All commands are executed from the **repository root directory** (`Cargo.toml` and `package.json` both live there).

| Command | Action |
| :--- | :--- |
| `npm run tauri:dev` | **Launch the desktop application** (Vite dev server + Rust build) |
| `npm run build` | **Type-check and build the frontend** (`tsc` + Vite into `dist/`) |
| `cargo test` | **Run Rust unit tests** (DNA compiler, CTRNN brain, database schema); also regenerates `bindings/` |
| `npm run types:generate` | Alias for `cargo test` (ts-rs export) |
| `npm run test` | Vitest (configured; no frontend test files exist yet) |

### Windows developer setup
1. Install Rust via [rustup](https://rustup.rs) with the `stable-msvc` toolchain.
2. Install the **Visual Studio C++ Build Tools** (MSVC + Windows SDK).
3. Ensure **Microsoft Edge WebView2** is present (preinstalled on Windows 10/11).
4. Install Node.js, then `npm ci` and `npm run tauri:dev`.

---

## 📁 Repository Architecture & SSOT

*   **`config.json` (Single Source of Truth):** Simulation parameters (metabolism, photosynthesis, biting, mitosis, drag, world generation). It is compiled into the binary; a `config.json` in the working directory overrides it during development.
*   **`src/` (Rust core):**
    *   `main.rs`: Tauri setup, command registration, starts the simulation thread.
    *   `server/engine.rs`: Ocean and Trainer simulation loop (~60 Hz, warp capped at 35 steps per frame), emits the `simulation-state` event.
    *   `server/api.rs`: Tauri commands (`handle_client_action`, species, trainer runs, catalogue).
    *   `biology/dna.rs`: Genome compiler (`parse_genome`) and mutation.
    *   `biology/trainer_engine.rs`, `biology/scenarios.rs`: Trainer sandboxes and scenario plugins.
    *   `shared/brain.rs`, `shared/physics.rs`, `shared/spatial_grid.rs`, `shared/map_generator.rs`: CTRNN, kinematics, neighbor grid, world generation.
    *   `database.rs`: SQLite schema.
*   **`frontend/` (Webview UI):**
    *   `main.ts`, `core/router.ts`: View mounting and navigation (Main Menu, Ocean, Trainer, Catalogue).
    *   `ocean.ts`, `trainer.ts`: Event handling and render loops.
    *   `render/`: Procedural creature and brain renderers. `views/`: UI panels.
*   **IPC:** The frontend calls `invoke("handle_client_action", { action })` and other commands; the backend pushes all updates through a single Tauri event, `simulation-state`.
*   **Database:** `pixel_life_local.db` in the Tauri app data directory (Windows: `%APPDATA%\com.pixeldna.life\`). An older `target/pixel_life_local.db` is copied there once on first start.

---

## 🔄 TypeScript Type Generation (TS-RS)

Rust structs such as `CreatureAgent`, `FoodSpore`, and `CreaturePhenotype` carry `#[derive(TS)]` / `#[ts(export)]`. Running `cargo test` (or `npm run types:generate`) exports TypeScript interfaces into `bindings/`.

The frontend does **not** consume these bindings yet; it uses the handwritten `frontend/shared/types.ts`. Switching to `bindings/` is planned (roadmap Phase 1, RV-A03).

---

## 🧬 Sequence-Based Genetics & CTRNN Brain

The genome compiler in `src/biology/dna.rs` uses a sequence-based model instead of fixed positional genes:

### 1. Promoter/terminator genes
*   Genes are recognized by promoter motifs (e.g. `COL`, `STF`, `EYE`, `NEU`) followed by a payload and a terminator (`EN`/`SP`), with degenerate matching by biochemical character class.
*   Payloads of any length are hashed into normalized values that drive colors, stiffness, size, wave parameters, sensory organelles, and brain parameters.
*   Starting wildtypes can be organless; sensory organs (eyes, noses, tactile organs, photophores) emerge through mutation.

### 2. Chromatin and CTRNN brain
*   Positions covered by recognized genes form the open chromatin (`chromatin_state`); the resulting `active_dna` feeds brain-parameter hashing.
*   The CTRNN (`src/shared/brain.rs`) mixes `tanh`, `relu`, `sigmoid`, and `sin` neurons, with time constants in `[0.1, 2.5]`, a Thalamus preprocessing layer, and Hebbian weight learning during lifetime.

---

## 🧪 Testing

*   **Execution:** `cargo test` from the repository root.
*   **Genome compiler:** `biology::dna::tests` cover payload extraction, degenerate matching, mutation robustness, neuro-generation, pruning, and Hox symmetry.
*   **Neuron determinism:** `shared::brain::tests::test_execute_brain_determinism` checks that identical inputs produce identical neural states.
*   **Schema:** `database::tests` run against an in-memory SQLite database (schema, catalogue CRUD, column migration).
*   **CI:** `.github/workflows/ci.yml` runs `cargo test` and `npm run build`; `release.yml` runs only on `v*` tags.

---

## 🖥️ End-User Installations & Standalone Binaries

Installers are built by **GitHub Actions** (`release.yml`) when a `v*` tag is pushed. End-users do **not** need any developer dependencies to run them:

### 🍏 macOS (Apple Silicon & Intel)
*   **Format:** `.dmg` (Disk Image) / `.app` (App Bundle)
*   **Launch:** Open the `.dmg`, drag **Pixel DNA Life** to your `Applications` folder, and double-click to launch.
*   **Webview Engine:** Uses the system **WKWebView**.

### 🪟 Windows (10 & 11)
*   **Format:** `.exe` (Standalone Installer) / `.msi` (Windows Installer)
*   **Launch:** Double-click the `.exe` or `.msi` file to install and launch.
*   **Webview Engine:** Uses Microsoft **Edge WebView2**, preinstalled on current Windows 10/11. If missing, the installer downloads it.

### 🐧 Linux (Debian, Ubuntu, Mint, etc.)
*   **Format:** `.deb` (Debian Package) / `.AppImage` (Portable Standalone Binary)
*   **Launch (Debian/Ubuntu):** Install the `.deb` package via your software manager or `sudo apt install ./pixel-life.deb`.
*   **Launch (AppImage):** Mark the file as executable (`chmod +x pixel-life.AppImage`) and run it.
*   **Webview Engine:** Uses the system **WebKitGTK**.

---

## 🔧 Developer & User Troubleshooting Guide

### 1. Missing build dependencies (Linux)
If `npm run tauri:dev` fails with missing C-library headers:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

### 2. Live changes not reflecting in the webview (Linux)
WebKitGTK caches assets under `~/.cache/webkitgtk-4.1`. Open DevTools (`Ctrl+Shift+I`), enable **Disable Cache** in the Network tab, or purge the cache:
```bash
rm -rf ~/.cache/webkitgtk-4.1
```

### 3. Database schema mismatch or startup freezes
Close the app and delete `pixel_life_local.db` (plus `-wal`/`-shm` files if present) from the Tauri app data directory. The schema is recreated on next start. This deletes all species, trainer runs, and catalogue entries.
