# 🧬 Pixel DNA Life - Native Desktop Lab

A high-performance, fully native 2D biomorphic ecosystem and reinforcement learning neuroevolution trainer. Built as a native desktop application using **Tauri v2**, **Rust 2024**, **SQLite**, and **Preact Signals**.

---

## 🚀 Execution Commands

All commands are executed directly from the **repository root directory**. There is no need to enter subfolders.

| Command | Action |
| :--- | :--- |
| `npm run tauri:dev` | **Launch Tauri Desktop Application** (Launches the main Ozean-Labor & Evolutionary Trainer) |
| `npm run build` | **Compile Production Frontend Bundle** (Prepares HTML assets for Tauri compile) |
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

## 🧪 Testing Standards & Guidelines

Our native backend is covered by an automated test suite verifying core biological and physical determinism:

*   **Execution:** Run `cargo test --manifest-path src-tauri/Cargo.toml` from the repository root.
*   **Decoded Phenotypic Parity:** Verified via `biology::dna::tests` to prove that the Rust DNA parser decodesgenomes exactly identical to the TS specifications.
*   **Neuron Determinism:** Verified via `shared::brain::tests::test_execute_brain_determinism` to ensure that CTRNN temporal integrations produce bit-perfect deterministic neural potentials across compiler optimizations.
*   **Relational Schema Integrity:** Verified via `database::tests` to ensure that cascading foreign key constraints are honored inside SQLite databases.
