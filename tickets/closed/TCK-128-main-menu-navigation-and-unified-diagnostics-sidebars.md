---
id: TCK-128
title: Unified Laboratory Menu and Modular Diagnostics Sidebars
status: closed
assigned: Gemini-CLI
created_at: 2026-09-03
closed_at: 2026-09-03
---

# TCK-128: Unified Laboratory Menu and Modular Diagnostics Sidebars

## Description
Establish a game-like retro-cybernetic Main Menu to orchestrate simulation states with absolute hygiene. Empty the static `index.html` to a minimal mount point and build the entire SPA dynamically in pure TypeScript using a bespoke, typed CSS-in-JS `styled` engine. Refactor and unify the isolated Ocean and Trainer diagnostics sidebars into a highly reusable, modular `UnifiedDiagnosticsPanel` component. Implement the core database schema for a persistent, cross-simulation `creature_catalogue` to store, rename, and retrieve species genomes independently of ephemeral training runs. Support full **Cryo-Cloning** (saving DNA, Epigenetics, and live learned Brain Synapses) with a selective **Pre-Training Restoration Selector** upon loading/spawning, and introduce a **Lamarckian Synaptic Inheritance** toggle in the Trainer.

## Requirements & Scope

### 1. Persistent Database Schema (database.rs)
- Create the persistent SQLite table `creature_catalogue`:
  - `id` (TEXT PRIMARY KEY) - unique ID (UUID or generated string).
  - `name` (TEXT NOT NULL) - custom or generated Latin name.
  - `genome` (TEXT NOT NULL) - genetic string (A-Z).
  - `source` (TEXT NOT NULL) - provenance label (e.g., "Trainer Run 'FastRun'", "Ocean Gen 14", "Creator").
  - `fitness` (REAL NOT NULL DEFAULT 0.0) - last evaluated fitness.
  - `carnivory` (REAL NOT NULL DEFAULT 0.0) - diet index computed from the genome.
  - `methylations` (TEXT NOT NULL) - JSON array of epigenetic chromatid methylation values.
  - `synapse_weights` (TEXT NOT NULL) - JSON array of live active brain CTRNN synapse weights.
  - `created_at` (DATETIME DEFAULT CURRENT_TIMESTAMP).
- Add index on `genome` for fast queries.
- Ensure clean schema registration inside `init_db`.

- Expand the `trainer_genomes` table schema to also store:
  - `methylations` (TEXT NOT NULL) - JSON array representing epigenetic state.
  - `synapse_weights` (TEXT NOT NULL) - JSON array representing learned brain weights at the end of the epoch.

### 2. Tauri API Operations (api.rs)
- Implement five safe Tauri command bindings:
  - `get_catalogue_creatures() -> Result<Value, String>`: Returns all saved creatures sorted by `created_at` DESC.
  - `save_to_catalogue(name: String, genome: String, source: String, fitness: f64, methylations: Vec<f32>, synapse_weights: Vec<f32>) -> Result<bool, String>`: Inserts or replaces a catalog specimen, parsing and saving its exact `carnivory` index along with its epigenetic and synaptic snapshot state.
  - `delete_from_catalogue(id: String) -> Result<bool, String>`: Deletes a catalog specimen by ID.
  - `rename_catalogue_creature(id: String, new_name: String) -> Result<bool, String>`: Updates the custom name of a specimen.
  - `spawn_catalogue_creature_to_ocean(id: String, load_learned_synapses: bool) -> Result<bool, String>`: Finds the creature in the DB and dispatches a command to spawn its clone directly inside the active ocean thread.

### 3. Pure TypeScript Frontend & Bespoke `styled` Engine (frontend/core/styled.ts)
- Strip `index.html` down to a single blank mount container: `<div id="app"></div>`.
- Build a lightweight, typed CSS-in-JS `styled` engine in `frontend/core/styled.ts` to parse, register, and inject CSS styles dynamically into a head style block.

### 4. Retro-Cybernetic Main Menu & State-Router (frontend/core/router.ts)
- Implement a lightweight, state-based navigation router in TypeScript.
- Dynamically compile and mount the three primary SPA views:
  - **Main Menu View:** An immersive fullscreen hub displaying launchers for active modes (Ocean, Trainer, Catalogue) and locked placeholders.
  - **Ocean Simulation View:** Fully compiled on demand.
  - **RL Trainer View:** Compiled and managed dynamically.
  - **Fullscreen Catalogue View:** The standalone listing view.

### 5. Modular & Unified Diagnostics Sidebar (diagnostics.ts)
- Refactor the right-sidebars of Ocean, Trainer, and Catalogue into a unified component (`UnifiedDiagnosticsPanel`) containing 5 stackable, expandable sub-modules:
  - **HD Preview Module:** Circular dark preview viewport running a local high-fps Preact-bound rendering loop, showing spinal wiggling, limbs, and actual specimen colors.
  - **Vital & Meta Module:** Real-time health gauges (Stomach Energy, Stress Adrenaline, Age) and taxonomical names.
  - **Watson-Crick Double Helix Module:** Chromatid gene mapping layout.
  - **CTRNN Brain Module:** Live directed graph showing neural firing states and synaptic weight thickness.
  - **Raw DNA Copy Module:** Clean textarea with click-to-copy button.

### 6. Dual-Evolution Mode in Trainer
- Add a new tuning control inside the Trainer Sidebar: `Lamarckian Synapse Inheritance` [Default: False].
- Sync this toggle to the Rust engine in real-time and inherit parents' refined `synapse_weights` across generations.

### 7. Fullscreen Catalogue View & Seeding Modals
- Build the Fullscreen Catalogue view panel `#catalogue-view-container`.
- Build the modular Selection Dialog overlay with pre-training synaptic weight restorations.

---

## Current Status (As of 2026-09-03)
*   **Status:** Closed
*   **Progress:** 
    *   All core features, Main Menu SPA routing, and modular sidebars have been successfully implemented and verified through a clean 200ms production build.
    *   The wiggling blocker issue in the Ocean has been fully resolved by correcting the sensory input dimensionality mismatch.
