# 🧬 Pixel DNA Life - Native Desktop Lab

A high-performance, fully native 2D biomorphic ecosystem and reinforcement learning neuroevolution trainer. Built as a native desktop application using **Tauri v2**, **Rust 2024**, **SQLite**, and **Preact Signals**.

---

## 🚀 Execution Commands

All commands are executed directly from the **repository root directory**. There is no need to enter subfolders.

| Command | Action |
| :--- | :--- |
| `npm run tauri:dev` | **Launch Tauri Desktop Application** (Launches the main Ozean-Labor & Evolutionary Trainer) |
| `npm run build` | **Compile Production Frontend Bundle** (Prepares HTML assets for Tauri compile) |
| `npm run types:generate` | **Auto-Generate Types from Rust Backend** (Triggers TS-RS compilation to export types) |
| `npm run test:all` | **Run Unified Verification Test Suite** (Funs Vitest tests, Cargo tests, and balance sim checks) |
| `cargo test --manifest-path src-tauri/Cargo.toml` | **Run Native Backend Unit Tests** (Tests DNA parsing, CTRNN brain determinism, and database schema) |

---

## 📁 Repository Architecture & SSOT

The repository is structured cleanly around a unified configuration model:

*   **`config.json` (Single Source of Truth):** The central configuration file. All biological decay constants, photosynthesis gains, biting damage, mitosis thresholds, and fluid drag multipliers are loaded dynamically from this file at boot by the Rust backend.
*   **`src-tauri/` (Native Rust Core):**
    *   `src/main.rs`: High-performance 60Hz physics, collision partitioning, and spatial grid loops.
    *   `src/biology/trainer_engine.rs`: Parallel sandbox stepper, 60Hz brain integrations, and standstill/circular penalties.
    *   `src/shared/physics.rs`: Decoupled biomorphic steering equations and boundary reflections.
    *   `src/database.rs`: SQLite session, species extinction tracking, and champion genome persistence.
*   **`src/` (Webview HUD UI):**
    *   `src/tauri_ocean.ts`: Real-time canvas telemetry rendering and inspector panel.
    *   `src/tauri_trainer.ts`: Lightweight spectator renderer and slider action dispatches.
*   **`OLD_SIMS_ARCHIVE.md` (Legacy Docs):** Contains documentation and commit history details for the legacy, browser-based web assets that have been successfully retired.

---

## 🔄 Automatic TypeScript Type Generation (TS-RS)

To completely eliminate manual API sync errors and enforce 100% stable communication over the Tauri IPC bridge, this workspace integrates **`ts-rs`** inside the Rust core.

### How it works:
1. **Annotated Rust Structs:** Structures such as `CreatureAgent`, `FoodSpore`, and `CreaturePhenotype` are decorated with the `#[derive(TS)]` and `#[ts(export)]` macros.
2. **On-Test Compilation:** Whenever backend tests are executed (`npm run types:generate`), the Rust compiler automatically analyzes the fields (including nested structs and enums) and exports bit-perfect TypeScript interfaces into `src-tauri/bindings/`.
3. **Frontend Type Adapter:** The frontend library at `src/shared/types.ts` is configured as a clean type-adapter that re-exports all 12 auto-generated files from `src-tauri/bindings/` while maintaining purely client-side payload definitions.

To manually trigger a type regeneration cycle, run:
```bash
npm run types:generate
```

---

## 🧪 Testing Standards & Guidelines

Our native backend is covered by an automated test suite verifying core biological and physical determinism:

*   **Execution:** Run `cargo test --manifest-path src-tauri/Cargo.toml` from the repository root.
*   **Decoded Phenotypic Parity:** Verified via `biology::dna::tests` to prove that the Rust DNA parser decodesgenomes exactly identical to the TS specifications.
*   **Neuron Determinism:** Verified via `shared::brain::tests::test_execute_brain_determinism` to ensure that CTRNN temporal integrations produce bit-perfect deterministic neural potentials across compiler optimizations.
*   **Relational Schema Integrity:** Verified via `database::tests` to ensure that cascading foreign key constraints are honored inside SQLite databases.

---

## 🖥️ End-User Installations & Standalone Binaries

The compiled production installers are compiled automatically in the cloud on every tag push via **GitHub Actions**. End-users do **not** need to install any compilation or developer dependencies (`apt-get`, `cargo`, `node`, etc.) to run the standalone binaries:

### 🍏 macOS (Apple Silicon & Intel)
*   **Format:** `.dmg` (Disk Image) / `.app` (App Bundle)
*   **Launch:** Open the `.dmg`, drag **Pixel DNA Life** to your `Applications` folder, and double-click to launch.
*   **Webview Engine:** Uses Apple's native Safari-based **WKWebView** pre-installed on all macOS installations. No dependencies required.

### 🪟 Windows (10 & 11)
*   **Format:** `.exe` (Standalone Installer) / `.msi` (Windows Installer)
*   **Launch:** Double-click the `.exe` or `.msi` file to install and launch.
*   **Webview Engine:** Uses Microsoft's native **Edge WebView2**. It is pre-installed on 99% of Windows 10/11 machines. If missing, the installer will automatically download and install it in the background.

### 🐧 Linux (Debian, Ubuntu, Mint, etc.)
*   **Format:** `.deb` (Debian Package) / `.AppImage` (Portable Standalone Binary)
*   **Launch (Debian/Ubuntu):** Install the `.deb` package via double-click in your software manager or run `sudo apt install ./pixel-life.deb`. This automatically resolves and downloads any runtime system dependencies.
*   **Launch (AppImage):** Right-click the `.AppImage` file, check **"Allow executing file as program"** (or run `chmod +x pixel-life.AppImage`), and double-click to run!
*   **Webview Engine:** Uses the system-wide **WebKitGTK** layout engine.

---

## 🔧 Developer & User Troubleshooting Guide

### 1. Developer Compilation Missing Dependencies (Linux)
If compilation or `npm run tauri:dev` fails on your local Linux developer machine with missing C-library headers, install the compilation toolchain:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

### 2. Live Changes Not Reflecting in Webview (Linux)
Webview assets are heavily cached by WebKitGTK on Linux under `~/.cache/webkitgtk-4.1`. If your TypeScript edits do not reflect in the app:
*   Right-click anywhere inside the Tauri application window, select **"Inspect"** (or press `Ctrl+Shift+I` to open DevTools), go to the **Network** tab, and check **"Disable Cache"**.
*   Alternatively, purge the local WebKitGTK cache folder:
    ```bash
    rm -rf ~/.cache/webkitgtk-4.1
    ```

### 3. Database Schema Mismatch or Startup Freezes
If you experience a thread freeze on startup after changing structural Rust database models, purge the previous SQLite file to force a clean, migrated database rebuild:
```bash
rm -f pixel_life_local.db
```
