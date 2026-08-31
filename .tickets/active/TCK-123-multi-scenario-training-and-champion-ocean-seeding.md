---
id: TCK-123
title: Multi-Scenario Training and Champion Ocean Seeding
status: active
assigned: Gemini-CLI
created_at: 2026-08-31
---

# TCK-123: Multi-Scenario Training and Champion Ocean Seeding

## Description
Eradicate isolated random ocean restocking and uncoordinated single-target training by establishing a fully closed-loop evolutionary pipeline. Introduce 10 distinct, biophysically-rich training scenarios selectable within any training run, allowing progressive scenario rotation (transfer learning) to train versatile generalist survivors. Program a local database-driven "Export Champion/HOF" interface that saves elite training lineages into a unified prepared template table, which the main Ocean simulation prioritizes during restocking with zero-race-condition deduplication.

## Requirements & Scope

### 1. Database Schema Extension (database.rs)
- Create a new SQLite table `prepared_creatures`:
  - Fields: `id` (TEXT PRIMARY KEY), `genome` (TEXT UNIQUE NOT NULL), `run_id` (TEXT), `source_run` (INTEGER), `fitness` (REAL), `exported_at` (TEXT).
- Ensure the table is initialized cleanly inside `init_db` without wiping existing databases.

### 2. Multi-Scenario Training Engine (trainer_engine.rs)
- Register 10 distinct environmental scenarios as an enum `TrainingScenario`:
  1. `StaticForaging`: Calm water, static plant spore.
  2. `PreyChase`: Moving meat spore with sinusoidal drifting velocity.
  3. `Rheotaxis`: Constant fluid current (East to West force vector) challenging forward thrust.
  4. `ThermalStress`: Hot thermal bands causing kinetic waste; safe zone huddled near spores.
  5. `ReefAvoidance`: Circular rock obstacles block direct line of sight; requires haptic avoidance.
  6. `MetabolicStarvation`: Synaptic BMR coefficient is tripled, causing rapid energy depletion.
  7. `PredatorEvasion`: A hostile predator agent chases the creature, requiring active evasion.
  8. `BlinkNavigation`: Ambient visibility is pitch black; food only visible when the creature flashes its bioluminescence.
  9. `SensorySwarm`: Decoy spores emit distractor signals; requires precise chemical affinity.
  10. `TidalRotation`: Rotates environmental challenges dynamically every 100 ticks (Calm -> Current -> Reef).
- Program these scenario physics parameters inside `step_trainer_sandbox_physics`.
- Add `scenario` field to `TrainerSandbox` and implement real-time switching of scenarios inside a running training.

### 3. "Export Champion/HOF" Interface & Command (tauri_trainer.ts / api.rs)
- Implement a Tauri command `export_trainer_champions(run_id: i64) -> Result<(), String>`:
  - Query the SQLite database `trainer_genomes` for the top 3 genomes with the highest fitness associated with `run_id`.
  - Insert or update them into the `prepared_creatures` table, saving their maximum fitness and timestamps.
- Add an "Export Champion/HOF" button to the Trainer UI sidebar. Triggering it exports the top 3 Hall of Fame members of the active training.
- Add a dropdown inside the Trainer UI sidebar to select the active scenario (or select "Auto Rotation") and sync the selected scenario to Rust in real-time.

### 4. Dynamic Ocean Seeding & Deduplication (engine.rs / main.rs)
- Modify the main Ocean simulation population restocking engine (`engine.rs`):
  - When the ocean population drops below 25 (or during initial startup seeding), query the `prepared_creatures` table.
  - Pull places 1-3 from all existing training runs in equal portions (round-robin or fair proportional distribution).
  - Deduplicate loaded templates by genome string or ID (if the same genome is already active in the ocean or pulled from multiple runs, unify it to prevent redundant clumping).
  - Spawn these elite, highly evolved champion clones into the deep ocean as pioneer wildtypes.

### 5. Verification & Testing
- Validate all Rust compiles with zero warnings and passes the automated test suite.
- Write a unit test verifying that the `prepared_creatures` table correctly stores, deduplicates, and retrieves elite genomes.
- Verify that selected sandboxes load, switch, and render different scenario obstacles (such as reefs) on the canvas.
