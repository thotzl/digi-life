# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-08-19

### Added
- **Multi-Trial Sandbox Trainer (TCK-105):** Built a concurrent, multi-sandbox evolutionary reinforcement learning trainer UI (`trainer.html` and `src/trainer.ts`) to breed and select highly fit, direct-navigating progenitor cells.
- **Success-Gated Path Efficiency (Wegstrecke):** Implemented a high-fidelity path efficiency metric that rewards direct straight-line trajectories and penalizes winding zigzag paths, heavily weighted with up to 2000 points upon food consumption.
- **Prey-Peer Mocking (Meatballs) & Sensory Scanning (TCK-106):** Expanded the trainer to spawn both plant (green) and meat (red) spores. Mocked meat spores as a living "prey peer" inside the spatial grid's creature layer with physical color, smell, vibration, and thermal attributes, allowing carnivores to learn visual, olfactory, and thermal tracking.
- **Adaptive CTRNN Activation Functions (TCK-106):** Enabled genetically evolving activation functions (tanh, ReLU, Sigmoid, Sine) for CTRNN hidden interneurons mapped from DNA Locus 21, allowing neurons to develop into period-oscillators (sin) or discrete gates (relu).
- **Immersive 3-Column Terminal Layout (TCK-106):** Redesigned the trainer interface into a 3-column layout featuring scrollable sandboxes in the center up to N=100, while locking generational hyperparameters on the left and live brain telemetry/zoomed preview on the right.
- **Live 60Hz Sidebar HTML Telemetry (TCK-106):** Integrated a real-time, flicker-free telemetry sidebar showing dynamic potential states, activations, decay time constants, and bias for hovered neurons.
- **Active Substrate Restocking (TCK-106):** Configured the live simulation backend to query SQLite and restock the live ocean substrate with active champions of all completed training sessions at a 40% probability rate.
- **Custom Deletable Dropdown Component (TCK-106):** Replaced standard dropdowns with a custom absolute-positioned session manager list featuring inline red `✕` delete buttons and auto-collapse on outside clicks.

## [1.2.0] - 2026-08-19

### Added
- **O(1) Spatial Grid Partitioning:** Implemented a high-performance 2D spatial grid partitioned into 80x80 pixel bins across the 19200x10800 simulation area (ref TCK-103). Neighborhood queries for collision detection and sensory inputs are optimized from $O(N)$ or $O(N \cdot M)$ down to $O(1)$.
- **Biomorphic Body Flexion (Bending):** Refactored locomotion physics on the headless server and balancing simulator from rectilinear angular torque to a realistic body flexion (bending) and thrust curve-turn kinematic model (ref TCK-104). Heading changes are strictly coupled to forward/backward velocity and flexion angle. Pure in-place turning and side-slippage are eliminated.
- **Interactive WASD Steering Sandbox:** Rewrote the standalone seed preview page (`preview.html` and `src/preview.ts`) to spawn an interactive player-controlled test creature. Added keyboard WASD/Arrow listeners to steer the creature, featuring a smooth camera focus follow and dynamic visual body bending.
- **Decoupled Ecosystem Balance Simulator:** Created a standalone command-line balancing utility (`src/server/balanceSim.ts`) that executes the synchronous physics and biology engine in-memory at warp speed (~300 Ticks/sec). Extracted all hardcoded biological, physical, and hazard parameters to a central config `"rules"` block in `config.json`.
- **Neuron Layout Coordinates:** Assigned layered graphical layout coordinates (`x`, `y`) to all compiled neurons inside the CTRNN directed graph compiler in `src/biology/dna.ts`, successfully restoring and fixing the broken visual brain graph on the frontend.
- **Hard Bound Wall Bouncing:** Disabled toroidal wrapping in favor of closed-basin elastic boundary wall collisions with 50% kinetic speed loss on both creatures and spores. Re-activated and calibrated the neuronal "Wall warn touch" sensor.
- **Physical Multi-Agent Collisions:** Added mass-based elastic overlap resolution and momentum transfer (impact bounce) between creatures themselves, as well as kinetic "bugwave" verdrängung pushing food spores out of creature bodies.

### Fixed
- **Unused Variables:** Resolved multiple TypeScript compiler warnings and errors for unused declarations (`isMutated`, `oldAgeRatio`, `outRight`) on both server and simulator scripts.

---

## [1.1.0] - 2026-08-19

### Added
- **SQLite Storage Integration:** Migrated from raw JSON file storage (`species_db.json` and `simulation_state.json`) to a fast, transactional, multi-connection-safe SQLite database (`digilife.db`) running in WAL (Write-Ahead Logging) mode.
- **Testing Architecture:** Added full Vitest integration with `@vitest/coverage-v8` and configured Stryker Mutator (`stryker.config.json`) for mutation testing.
- **Unit and Integration Tests (29 passing tests):**
  - Covered 100% of CTRNN Euler-integration bounds, genetic base-pairing decoding, and mutation guard rails.
  - Covered 100% of API endpoints, SQLite queries, and concurrent transactions.
  - Covered physical boundary reflections, fluid drag, nutrient spore absorption, and asexual mitosis.
- **Luminosity Flash Handling:** Headless server now parses the 4th CTRNN output node ("Biolum Flash"). Active flash ($>0.5$ intensity) triggers a metabolic energy tax of $0.05 \times \text{intensity}$ and broadcasts a `"FLASH_EVENT"` to all visual web clients.
- **Newborn Species Ledger:** Centralized a new `registerSpeciesIfNew` helper that tracks and records all newly evolved genomes (mitosis mutations, random founders, and manually injected specimens) to SQLite with strict lineage parent IDs.

### Fixed
- **Synaptic Index Mapping Bottleneck:** Fixed the genomic mapping bottleneck where synapses in brains with $>26$ total nodes could never choose presynaptic nodes $\ge 26$, leaving hidden layers isolated.
- **Mutation Engine Crash:** Patched `mutateGenome` to prevent runtime division-by-zero / NaN index crashes when passed an empty string.

---

## [1.0.0] - 2026-08-19

### Added
- Initial workspace repository setup (ref TCK-101).
- Scaffolded Vite v5 client monitor and tsx WebSocket simulation server.
- Built AI local skills structure under `.agents/skills/`.
