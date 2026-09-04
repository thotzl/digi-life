---
id: TCK-123
title: Curriculum Learning Framework, Multi-Scenario Playlist, and Champion Ocean Seeding
status: active
assigned: Gemini-CLI
created_at: 2026-08-31
---

# TCK-123: Curriculum Learning Framework, Multi-Scenario Playlist, and Champion Ocean Seeding (Master Epic)

## Description
Establish a comprehensive, closed-loop evolutionary pipeline. This Epic consolidates and merges all sub-tickets (`TCK-112`, `TCK-130`, `TCK-132`) into a unified Curriculum Learning framework. 

Introduce 10 distinct, biophysically-rich training scenarios, scenario-specific fitness evaluations, a Setup Configurator when launching a new training run, and an Auto-Rotating Curriculum Playlist triggered by average population fitness stability. Additionally, implement local database-driven "Export HOF member" pipelines to seed elite genomes back into the Ocean simulation, and support the Match Arena and spectator HUD enhancements.

---

## Requirements & Scope

### 1. Trainer Setup Configurator (Initial Conditions)
Implement a modal/panel in `TrainerView.ts` when initiating a new training run to define the following parameters (synced via `TrainerSetupConfig` struct to Rust):
- **Population Seeding Composition:** Define proportions of:
  - *Catalog Seeds:* Select specific genomes from the local catalogue.
  - *Hall of Fame (HOF) members:* Pull from previous training runs.
  - *Random Mutants:* Freshly generated random genomes.
- **Synapse Weight Transmission:** 
  - *With weights (fine-tuning / transfer learning):* Preserves learned synaptic weights from the catalogue parents.
  - *Without weights (structural training):* Strips weights to baseline, training purely neural topology robustness.
- **Base Config Template:** A structured JSON format capturing setup, active rules, and active scenarios.

### 2. Multi-Scenario Training Engine (10 Scenarios)
Register 10 distinct environmental scenarios as an enum `TrainingScenario`:
1. `StaticTarget`: Calm water, static plant spore spawned at standard distance (250px).
2. `DistantTarget`: Long-range target spawned at extreme distance (450px - 600px).
3. `WanderingSpore`: Targets drift slowly in random brownian motion.
4. `WallHugger`: Target spawns near boundaries, requiring precise deceleration/turning.
5. `MultiFood`: Multiple target spores spawn simultaneously, testing decision priority.
6. `DietShowdown`: Candidate + database's best opposing diet champion in the same room. Implements Predator-Prey and Territorial Rivalry.
7. `DriftingCurrent`: Constant side currents drift the candidate, requiring tacking.
8. `FastPrey`: Meat target actively flees at high speed upon approach.
9. `ObstacleReef`: A single circular grey reef blocks the direct path, requiring haptic reef circumvention (now fully supported by physical and haptic reef physics).
10. `TidalRotation`: Rotates challenges dynamically every 100 ticks.

### 3. Dynamic Scenario-Specific Fitness (`calculate_sandbox_fitness`)
Eliminate the simulation gap where search exploration is penalized as distance waste:
- **Standard Mode/Scenarios (Static, Diet, etc.):** Keep baseline kinetic waste penalty (`distance_traveled * 0.05`) to discourage chaotic circular paths.
- **Exploration Mode/Scenarios (Distant, Obstacle, Drift):** Set `kinetic_waste = 0.0` or drastically reduce it. Reward continuous territory coverage and exploration distance.

### 4. Curriculum Playlists & Auto-Rotation
- **The Playlist:** Users can configure a sequential order of training scenarios.
- **Auto-Rotation Trigger:** When the moving average fitness of the sandbox population remains stable over $N$ generations and crosses a target fitness threshold, the simulator automatically advances to the next scenario in the playlist.
- **Echtzeit Live Overrides:** Expose sidebar controls to toggle Auto-Rotation off/on, or manually switch the active scenario in real-time.

### 5. Local Database Champion Export & Dynamic Ocean Seeding
- Create SQLite table `prepared_creatures` with fields: `id` (TEXT PRIMARY KEY), `genome` (TEXT UNIQUE NOT NULL), `run_id` (TEXT), `source_run` (INTEGER), `fitness` (REAL), `exported_at` (TEXT).
- Implement Tauri command `export_trainer_champions` to save the top 3 Hall of Fame members.
- **Dynamic Seeding:** When Ocean population drops below 25, query `prepared_creatures` and proportionally seed clones into the ocean as pioneer wildtypes.

### 6. Match Arena & Spectator HUD
- Configure competitive local match duels (1v1, team, Free-For-All) utilizing saved local species.
- **HUD Upgrades:** Frame-interpolate canvas rendering (from 25Hz to smooth 60/120Hz), add spectator camera tracking, and include a real-time neural synapse firing inspector.

---

## Verification & Testing
- Write Rust unit tests for `prepared_creatures` CRUD operations.
- Write unit tests for scenario-specific fitness calculations, asserting Standard vs. Exploration scoring differences.
- Verify full TypeScript compilation and Vite build with zero errors.
