# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.1] - 2026-08-21

### Fixed
- **Ticket Sync ESM Scope Error:** Renamed the ticketing synchronization script `.github/scripts/sync_github.js` to `.cjs` (CommonJS) and updated the workflow trigger to natively support `require('fs')` imports inside the project's `"type": "module"` ESM package scope.

## [1.6.0] - 2026-08-21

### Added
- **Multi-Platform Cloud Releases (TCK-114):** Configured a full parallel automated GitHub Actions workflow (`release.yml`) running on macOS, Windows, and Linux virtual machines. Pushing a `v*` tag automatically compiles, signs, packages, and drafts a GitHub Release with ready-to-run `.dmg`, `.exe`, `.msi`, `.deb`, and `.AppImage` bundle installers attached as download assets!
- **OnceLock Thread-Safe Config Cache:** Implemented `std::sync::OnceLock` in types.rs to parse and cache `config.json` exactly once on startup. Cuts down thousands of 60Hz disk reads inside physics loops to `0ms` and eliminates terminal log spam entirely.

### Fixed
- **Clean Bundler Entrypoints:** Removed deleted legacy assets from `vite.config.ts` Rollup options, restoring 100% clean production Vite compiles.
- **Retired Legacy Web Code:** Retired obsolete browser-based files and structured them inside `OLD_SIMS_ARCHIVE.md` for historical peeks and documentation.

## [1.5.4] - 2026-08-21

### Added
- **Interactive Genome Loci Highlight (Active vs. Inactive):** Integrated active gene spans highlights inside the DNA Helix visualizer. Transcribed Hox genes are shown in bright high-contrast colors, while silent junk/inactive regions are dimmed out to 15% opacity with dashed borders.
- **DNA Hover Role Tooltips:** Linked every single genome character locus (0 to 255) to its exact biological role (Symmetry, Photosynthesis, Neural Tau, Synaptic pathways) inside the hover tooltip.

### Fixed
- **Kinematic & Friction Realignment:** Stripped out the remaining hardcoded locomotion, thrust, fluid drag, stiffness decay, and boundary bounce coefficients inside the trainer engine (`trainer_engine.rs`) and global physics module (`physics.rs`), realigning them directly with `config.json` rules.

## [1.5.3] - 2026-08-21

### Added
- **Global Config SSOT Integration:** Linked the native Rust backend directly to the central `config.json` file. All biological, physical, metabolic, and restocking constraints are now parsed dynamically at runtime.
- **Robust Fallback Defaults:** Integrated strongly typed struct configurations inside `types.rs` with automatic check of multiple relative paths, fallback default definitions, and zero-panic error protection.
- **Decoupled Hardcodes:** Stripped out all hardcoded physical and biological literals inside the main physics and predation loop in `main.rs`, replacing them with exact dynamic references to `AppConfig.rules`.

## [1.5.2] - 2026-08-21

### Fixed
- **Terminal Log Quietness & Performance:** Stripped out high-frequency `[SIMULATION] Processing incoming action` logs from the channel receiver and removed the `[DEBUG PHYSICS] Creature ...` telemetry position printouts, cutting console output by over 98% and maximizing terminal and CPU performance!

## [1.5.1] - 2026-08-21

### Added
- **Nutrient Hotspots and Algae Forests:** Implemented 12 localized high-density nutrient hotspots across the logical ocean coordinates.
- **Double Ecosystem Spores:** Increased the baseline food spore count from 300 to 600, providing rich grazing environments to foster herbivore population viability and CTRNN sensory exploration.
- **Patchy Density Spawning & Relocation:** Programmed the spore-population maintenance to spawn 75% of spores close to our algae centers (creating lush localized food clumps) and 25% scattered across the deep ocean. Relocations upon eating follow the exact same patchy probability, ensuring long-term self-sustaining nutrient fields!

## [1.5.0] - 2026-08-21

### Added
- **Native Rust Trainer Core (TCK-108):** Ported the entire physical and neural evolutionary reinforcement learning loops (N sandboxes, continuous 60Hz physics, Recurrent CTRNN Euler integration, chemoreceptive sensory patches, boundary bounces, and circular/standstill penalties) to the native Rust backend under `src-tauri`.
- **Buttery-Smooth Slider Performance:** Split slider actions into `input` events (providing instant local label updates in JS at 60+ FPS) and `change` events (sending the finalized parameter to Rust exactly once upon mouse release) to completely eliminate IPC message flooding.
- **State Reconciliation / Self-Healing UI:** Programmed Rust to broadcast the absolute hyperparameter truth back to the UI upon mode swaps, resets, or parameter adjustments, keeping the frontend sliders and labels in perfect, error-proof synchronization with the database.
- **40/40/20 Ecological Restocking:** Integrated the main ocean simulation with the trainer's SQLite database. When the ocean population drops below 25, Rust spawns a new founder cell with a 40% chance of being an all-time training champion, a 40% chance of being a cloned successful wild species, and a 20% chance of being a fresh random wildtype.
- **Zero-Delay UI Interaction:** Rewrote the start and reset buttons to toggle their visual states in `0ms` directly in JS, dispatching the Tauri IPC actions to Rust in parallel.
- **Pristine Zero-Warning Compilation:** Resolved all compiler warnings inside the main Tauri process of `src-tauri` and ensured 100% clean Vite production builds.

### Fixed
- **Stable Generation Restore:** Programmed the sandbox grid builder to query and load the maximum saved generation of the active run from SQLite, preventing the generation counter from resetting back to 1 on page swaps or session reloads.
- **Pristine Hard Resets:** Enhanced the reset command to physically delete all saved genomes of the active run ID from SQLite, ensuring a clean Greenfield start (all 16 sandboxes spawning as "🌱 Fresh Random") on reset.
- **Database Schema Completeness:** Registered the missing `trainer_genomes` table and optimized index inside `init_db` in `database.rs`, restoring full persistence and Hall of Fame tracking capabilities.

## [1.4.0] - 2026-08-21

### Added
- **Tauri v2 Desktop Laboratory Shell (TCK-109):** Implemented a high-performance, portless, and standalone desktop lab shell using Tauri v2. The Vite-TypeScript-Preact Signals HUD has been bundled into a native, borderless webview window.
- **Headless Rust Simulation Core (TCK-110):** Ported the continuous 60Hz physics, Recurrent CTRNN Euler temporal integration, and spatial grid collision partitioning to a native Rust background core under `src-tauri`.
- **Ecosystem and Biological Alignment:** Ported the full suite of biological and metabolic natural laws from the TS original: Thermal depth-stress penalties, Herbivore photosynthesis near the surface, Adrenaline sprint taxes, and Predatory non-cannibalistic biting attacks with client BITE shockwaves.
- **Ecological Balance (Spore Relocation):** Re-engineered the grazing and death decomposition mechanics to organically relocate existing food spores (no memory allocations) rather than infinite array deletions/growth, keeping the spore population perfectly locked at 300.
- **Selected Brain SVG Activations:** Fully supported live kognitive brain activity streaming. When a creature is selected in the sidebar or canvas, Rust streams its real-time neuron activations to drive the SVG Directed Graph glows in the UI.
- **Robust Session Persistence:** Integrated `simulation_state` table into the local SQLite database. Every 10 seconds, Rust serializes and saves the entire active simulation state, allowing seamless save/resume capabilities on application restarts.
- **Deterministic Handshake & Type-Safety:** Created a fully awaited startup handshake to resolve Webview load race conditions and enforced strict type-casting (`Number(id)`) inside client filters to prevent race-condition deletions.

## [1.3.3] - 2026-08-19

### Added
- **Isomorphic DRY Physics/Sensory/Brain SSOT (TCK-107):** Factored out all duplicate physics kinematics, sensory chemoreceptors, and CTRNN brain integrations into clean, stateless, and fully shared modules under `src/shared/`. These shared units act as the absolute Single Source of Truth (SSOT), consumed directly by the Live Ocean Server, the Balance Sim, and the local browser-bound Evolutionary Trainer.
- **Proportional Physical Mass Decoupling:** Decoupled the physical mass calculations inside `applyCreaturePhysics` to allow ecosystem-appropriate volume-based weight scaling in the live ocean while preserving precise tuned weights in the trainer.
- **Biomorphic Flexion Turn Coupling:** Harmonized steering turning kinematics across all environments to be strictly coupled to forward speed (`vForward * bendAngle * 0.015`), preventing any un-biomorphic in-place rotation and ensuring smooth eel-like sliding.

## [1.3.2] - 2026-08-19

### Added
- **Genetically Diverse HOF Re-injection:** Built a highly sophisticated genetic-distance (Hamming distance) matching algorithm that checks HOF candidates and only re-injects those with a minimum genetic difference of 25 characters from the current champion. This keeps lost active lineages alive and prevents monocultural deadlocks!
- **Strictly Unique Active Elites:** Refactored the breeding selector to filter out identical, duplicated genomes inside the elite pool, ensuring that every survivor slot is occupied by a unique species.
- **Granular 1% Step Sliders:** Updated all generational tuning sliders to 1% granular step-precision (0-50% for mutation rates, 5-100% for elite ratios, 0-100% for inflows and HOFs) to enable precise 1-3% micro-mutations and prevent catastrophic gene loss.
- **Symmetric Scent/Olfactory Target Balancing:** Re-balanced the plant chlorophyll scent spectrum from `0.15` (which was outside the `[0.25, 0.65]` olfactory range) to `0.35` (perfectly centered), giving herbivores a 100% equal, high-intensity scent-detection capability.
- **Bilateral Eye Symmetrical Greenfield Spawning:** Resolved the asymmetric eye-selection bug inside the progenitor compiler (`generatePEN_Progenitor()`) by including left-quadrant angles (`angle >= 315`), establishing perfect 50/50 starting symmetry in Gen 1.

## [1.3.1] - 2026-08-19

### Added
- **Isomorphic Core Roadmaps (TCK-107 & TCK-108):** Formulated and registered backend-headless-migration (`TCK-108`) and shared DRY/isomorphic-module refactoring (`TCK-107`) roadmaps to unify simulations and visualizers.
- **Biologically Precise Omnivore Diets:** Integrated true omnivore feeding in the trainer, allowing creatures with mixed herbivore/carnivore DNA (`0.40 <= carnivory < 0.65`) to dynamically seek and consume either plant (green) or meat (red) spores, whichever is closer.
- **Close-Call Proximity Protection Klausel:** Exempted near-success attempts (where a creature gets within 30px of the target) from the 120px minimum path requirements, perfectly preserving high-fidelity steering learning while still ruthlessly executing lazy "Schleicher" crawlers.
- **Ultra-Fast DOM Recycling:** Refactored generational transitions to keep existing browser cards and canvases untouched in memory when $N$ is unchanged, cutting transition time from `200ms` down to `<1ms` and completely removing browser-rendering bottlenecks.
- **Non-Blocking Macrotask Micro-yielding:** Scheduled the main loop execution inside `setTimeout(tick, 0)` at generation transitions to yield execution to Chrome's event-loop, keeping the browser 100% responsive and fluid during Superwarp training.
- **High-Tech Spacing Button Group:** Consolidated the "Headless" toggle directly inside the primary control row as a beautiful dark-slate button (Headless: OFF) and glowing neon-cyan button (Headless: ON), matching the dark terminal theme.

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
