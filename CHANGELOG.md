# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.11.0] - 2026-09-04

### Added
- **Symmetrische Sensor-Fusion via HOX-Interneurone (TCK-133 Phase 3 & 100% Performance-Optimierung):** Vollständige Abschaffung der fehleranfälligen Geburts-Synapsenspiegelung. Das Gehirn generiert bei bilateraler Symmetrie nun automatisch Summen- ($S_c = L_c + R_c$) und Differenz-Interneurone ($D_c = L_c - R_c$). Durch die vollständige Abkoppelung dieser Fusionsreize aus dem vollvernetzten Exuberanz-Graphen hin zu dedizierten, isolierten Schaltkreisen (je 3 Synapsen pro Fusionsknoten) konnte die Synapsenanzahl bei 2 Organen um **65%** und bei 4 Organen um **78%** drastisch gesenkt werden – die Warp-Performance ist zu 100% stabilisiert.
- **Dezentralisierung der Propriozeption (TCK-133 Phase 2):** Einführung eines permanenten Zentralnervensystem-Moduls aus 6 unzerstörbaren Basis-Inputs (Hunger Clock, lineare Geschwindigkeit, Drehgeschwindigkeit, Energie/Mageninhalt, Adrenalin-Pegel, Kollisionsschmerz). Periphere Organellen sind nun vollständig von propriozeptiven Signalen isoliert und fokussieren sich zu 100% auf die Außenwelt (Exterozeption).
- **Haptische Spezialisierung & Direktionale Strömung (TCK-133 Phase 4):** Aufteilung haptischer Organellen via `expression_style` in drei biologische Ausprägungen: Taktile Borsten (Oberflächenkontakt & lokaler Schmerz), Seitenlinien-Poren (richtungsabhängiger, vektorieller Wasserdruck `local_drag` $[-1.0, 1.0]$ für Rheotaxis & Gyrodämpfung) sowie Thermo-Propriozeptoren (Wassertemperatur & normierte Wirbelsäulen-Biegung `bend_angle` $[-1.0, 1.0]$).
- **Phasische Sehkanäle & Bewegungserkennung (TCK-133 Phase 5):** Ergänzung visueller/olfaktorischer Organe um temporale Differenzialsensoren ($\Delta I$), welche Bewegungen durch Helligkeitsänderungen über Zeit erfassen. Die ansonsten ungenutzten Zustandspotenziale der Input-Neurone werden als flüchtige frameübergreifende Speicherzellen genutzt.
- **Standardisierter Symmetrischer Winkelraum (TCK-133 Phase 1):** Vereinheitlichung aller Richtungsberechnungen auf den Bereich von $[-170^{\circ}, 170^{\circ}]$ mit $0^{\circ}$ geradeaus, links negativ, rechts positiv. Der TypeScript `BrainRenderer` im Frontend wurde so generalisiert, dass er alle Knotenkoordinaten (Säulen links/mitte/rechts) vollkommen dynamisch und ohne Magic Numbers anordnet.

## [1.10.1] - 2026-09-04

### Fixed
- **Beseitigung der Fitness-Klippe im großen Raum (ExplorationScenario):** Einordnung der Explorationsleistung (Displacement + Sektorenabdeckung) als stetige Fitness-Basis, ergänzt durch einen rein additiven Homing-Bonus bei Unterschreitung der 550px-Sensorgrenze der Nahrung.
- **Korrektur der Multi-Trial-Distanzeichung (engine.rs):** Behebung des Indexierungsfehlers für Nahrungselemente im Rundenreset 2 & 3 des großen Raums, sodass Carnivoren, Herbivoren und Omnivoren exakt auf ihre biologischen Nahrungstypen (Pflanze/Fleisch) referenziert werden.
- **Wiederherstellung des kleinen Trainingsraums (StandardScenario):** Vollständiger Rollback der geänderten Fitnessregeln im kleinen Raum zur verlässlichen Reaktivierung des bewährten, hochstabilen evolutionären Verhaltens und Beseitigung künstlicher Loop-Strafen.
- **Kompilierungsbereinigung:** Ergänzung unbenutzter Variablen in der Methodensignatur von `StandardScenario::calculate_fitness` mit Unterstrichen zur Gewährleistung eines 100% warnungsfreien Cargo-Builds.

## [1.10.0-BETA] - 2026-09-03 (STABLE PRODUCTION-READY RELEASE)

### Added
- **Progressive Training Modes & Circular Exploration (TCK-123 & TCK-130):** Implemented an on-the-fly "Training Mode" dropdown selector inside the Trainer sidebar. Users can transition running training sessions from a Standard chamber (1000x1000, standard spawns) to a massive Exploration chamber (3500x3500) where plant and meat spores spawn uniformly outside of initial sensory range (1400..1650 px), forcing the brain to learn search patterns.
- **GPU-Accelerated Labor-Zoom:** Implemented a smooth "Laboratory Zoom (Tile Size)" slider in the Trainer sidebar to dynamically scale sandbox cards and HTML5 canvases between `100px` (zoomed-out bird's-eye overview) and `400px` (deep close-up details) utilizing browser-native CSS variables and canvas matrix scaling (`ctx.scale()`).
- **Ecological Closed-Loop (Meat-Pellets & Decomposition):** Enabled dead carcasses to decompose and drop crimson-red meat spores (`typeId: 2`) at their death coordinates. Optimized grazing so that herbivores only eat plants, carnivores only eat meat, and eaten meat spores recycle back into Algae plants, closing the environmental cycle.
- **Balanced Start-Chances & Taxonomy HUD:** Set the predator threshold `is_predator` to exactly 50% (`carnivory >= 0.50`), removing starting letter bias. Restored real-time taxonomical names and diet class text labels in the Trainer Diagnostics sidebar HUD.
- **Unified Diagnostics HUD Sidebar (`UnifiedDiagnosticsPanel.ts`):** Standardized real-time vitals, chromatin grid, and directed CTRNN neural activations graphs into a single reusable TypeScript module, integrated across the Ocean, Trainer, and Catalogue screens.
- **Persistent SQLite Catalogs & Auto-Migrations:** Standardized relational schemas for `creature_catalogue` and `trainer_genomes` with JSON text columns for epigenetic methylations and brain synapse weights. Implemented automatic schema evolution checks in Rust database initialization.
- **Premium "Pick and Add" Modals:** Built scrollable selection cards and circular 60Hz live-preview wiggler modals for both "Inject Species" (Ocean) and "Assign from Catalogue" (Trainer) views, replacing the default select boxes.
- **Lamarckian Pre-population in Trainer:** Enabled assigning catalog specimens to individual paused sandboxes (1 to 16) in-memory, immediately updating the sandbox's brain and body.

### Fixed
- **Unified Biology (Sensory & Movement Symmetrization):** Decoupled and unified brain sensory inputs (`compute_sensory_inputs`) and locomotion physics (`step_creature_kinematics`) under `src/shared/`. Both the Ocean and Trainer now run on identical code, completely fixing the blocker bug where Ocean brain outputs were zero and creatures appeared visually frozen.
- **Multi-Trial Reset Alignment:** Overhauled the multi-trial reset mechanics in `engine.rs` to dynamically re-center coordinates, boundaries, and 1200px minimum distance constraints for subsequent trial runs, ensuring seamless progressive learning.
- **Tauri Deserialization Key Mismatch:** Corrected `synapse_weights` to `synapseWeights` inside JS invoke arguments to match Tauri v2's automatic camelCase deserializer, fully restoring SQLite catalog saving and persistence.
- **Ocean Loop-Death & Resumption:** Fixed silent requestAnimationFrame loop termination by introducing a `loopRunning` tracker and a Preact effect route observer to automatically wake up the Ocean canvas drawing thread.
- **Ecosystem Balance & Locomotion:** Replaced static input vectors with an oscillating Central Pattern Generator (CPG) age clock in the Ocean simulation thread to drive organic locomotor swimming. Reduced Ocean basal metabolic rate decay by 55% (`* 0.45` scale) to double natural species lifespans.
- **UI Refinements:** Set pre-training synapse checkboxes to unchecked (`false`) by default and made modals close instantly on confirmation click.

## [1.8.0] - 2026-08-31

### Added
- **Biochemical Codon Groups & Golden-Middle Wobble-Matching (TCK-117):**
  - Mapped the 26 ASCII uppercase letters into 5 distinct biological groups (Alpha: Polar, Beta: Aromatic, Gamma: Hydrophobic, Delta: Acidic, Epsilon: Inert).
  - Implemented the "Golden-Middle" Wobble-Matching rule for physical organs (Eyes, Noses, Tactiles, Biolums) requiring exact matching of the first $N-1$ letters and wobble matching for the last letter, stabilizing random founder organ occurrence rates and preserving evolutionary mutations.
  - Enabled **fully-degenerate matching** for brain-related promoters (`"NEU"`, `"SY"`) allowing rapid, dense, and highly evolvable neural structures.
  - Upgraded `mutate_genome` to support structural **Insertion and Deletion mutations** driven dynamically by parent phenotypic rates, letting species shrink or expand their genomes (capped strictly between 128 and 512 characters).
- **Dynamic CTRNN Brain Compilation & Epigenetic Hebbian Learning (TCK-119):**
  - Scans for `"NEU"` promoters to compile a variable number of hidden neurons, hashing properties ($\tau$, bias, activation types, and continuous layout-depth layers) from payloads.
  - Scans for 2-character `"SY"` promoters to build synapses dynamically with absolute modulo-mapping index safety, completely purging the legacy sliding-window compiler and eliminating index out-of-bound errors.
  - Integrated real-time Hebbian synaptic learning and forgetting decays into both background physics loops (Ocean and Trainer sandbox ticks) utilizing the new `synapse_weights` vector.
  - Implemented transgenerational Lamarckian epigenetic inheritance. Mutated offspring created during mitosis copy parental learned synaptic offsets ($\Delta W$) to homologous child synapses (same `from_node` and `to_node`) scaled by 25%.
- **Real Multi-Trial Loop & Lamarckian Starvation Filter (TCK-120):**
  - Created a robust 3-trial epoch loop (3 x 300 ticks) in the Rust simulation background thread, ensuring average fitness calculations across consecutive evaluations to completely eliminate accidental lucky champions.
  - Enforced a minimum spore spawning distance of **`200.0` pixels** to prevent instant collisions.
  - Implemented a Lamarckian Starvation Filter (Viability Gate). Candidates that fail to eat in any of the 3 trials are penalized by 90% (average fitness multiplied by `0.1`), while successful hunts receive a massive `+150` points bonus per catch, creating a clean, stable evolutionary gradient from scratch.
- **Synaptic Exuberance and Active Lifetime Pruning (TCK-121):**
  - Implemented Synaptic Exuberance at birth. The brain now compiles as a fully connected graph of all inputs, hiddens, and outputs.
  - Specialized `"SY"` genes determine strong, genetically specialized starting weights (`[-2.0 .. 2.0]`), while all other connections are initialized as weak, exploratory synapses (`[-0.075 .. 0.075]`).
  - Integrated Active Lifetime Pruning inside `execute_brain_with_learning`. Synapses whose weights drop below the absolute threshold of `0.015` under forgetting decay are set to exactly `0.0` and permanently locked/pruned, saving metabolic energy and stabilizing CTRNN oscillations.
- **Multispectral Receptive Fields & Haptics (TCK-122):**
  - Implemented 5-Channel Multispectral Sensory Inputs. Each organelle de-compiles exactly 5 separate input neurons (receptive cones) mapped dynamically to standard spectral frequencies (UV/Pressure 0.10, Blue/Ester 0.30, Green/Sugar 0.50, Red/Amino Acid 0.70, IR/Pheromone 0.90).
  - Enforced uncoupled receptor evolution, allowing any physical organ (Eyes, Noses, Biolums) to freely evolve its spectral affinity and focus-bandwidth across the entire `[0.0 .. 1.0]` spectrum.
  - Programmed multidimensional biophysical environmental emissions (Algae emits on Blue/Green channels; Meat/Prey emits on Infrared/Red channels).
  - Expanded Tactile organs (`aff < 0.25`) into advanced haptics measuring mechanical hardness (Walls 1.0 vs Spores 0.3), fluid flow rheotaxis, surrounding hydrothermal vent temperature, proprioceptive spine bending strain, and high-impact physical pain on distinct channels.
- **Decoupled Modular Loci & Linear Allele Mapping Compiler (Relationship Fix):**
  - Completely decoupled physical, metabolic, and behavioral traits by removing the global `active_dna` whole-genome hashing loop inside `parse_genome`.
  - Every basic trait (Colors, Symmetry, Sizes, Stiffness, Waves, Pulses, Stomach, Tolerances, Carnivory) is compiled locally from its specific gene payload (e.g. `"COL"` payload for colors).
  - **Eliminated Hashing for Physical/Neural Attributes:** Replaced `hash_genome_slice` with `get_payload_linear_value_offset` across the entire genome-to-phenotype compilation (including color, size, stiffness, neural constants, biases, layer depths, and synapse weights).
  - **Smooth Mutational Drift (Real Inheritance Similarity):** Replaced cryptographic "hash avalanche" chaos with a continuous linear shift. A 1-point mutation in a payload now only shifts its compiled trait smoothly and gradually by 1-2% (e.g. slight color hue shift), leaving all other decoupled traits 100% stable and preserving parent skills/instincts perfectly.
- **Continuous Block-Free Foraging & Smooth Metabolism:**
  - Removed all binary/ternary diet thresholds and 'target_idx' eat-gating inside `step_trainer_sandbox_physics`.
  - Enabled 100% continuous foraging during the full 300 ticks of the trial, where creatures can consume both plants and meat at any time (plants give `1.0 - C` yield, meat gives `C` yield) followed by instant respawning of the consumed spore.
  - Aligned starting and minimum target distance calculations to use the closer spore on spawn/reset.
- **60Hz requestAnimationFrame Decoupled Preview Loop:**
  - Decoupled the diagnostics preview rendering from the 25Hz telemetry stream, moving it to a 60Hz requestAnimationFrame loop. The wiggling preview is now 100% butter-smooth and fluid.
  - Locked the preview heading to North (pointed straight up).
- **High-Contrast Triggered Neural Path Glow:**
  - Programmed a binary-triggered state (threshold > 0.02) where active neurons glow at 1.0 opacity with 5px shadows, and quiet ones drop to 0.22 opacity. Active synapses glow at 1.0 opacity, and quiet ones fade out cleanly.

### Fixed
- **Epigenetic Chromatin State Scanning:** Corrected the chromatin initialization to start closed (`false`) by default. Active chromatin states now open and transcribe all physical, metabolic, and neural promoters, fully restoring color and form diversity of starting species records and newly born cells.

## [1.7.0] - 2026-08-22

### Added
- **Sequence-Based Genetic Compiler & Hybrid Epigenetic GRN (TCK-116):** Fully refactored the core biological genetic translation engine inside the native Rust core to transition from a coordinate-mapped genome (rigid positional indices) to a revolutionary dynamic, pattern-scanning **Gene Regulatory Network (GRN)** model.
  - **Class 1 (Basic Traits & Brain Synapses):** Compiled strictly from the dynamic **`active_dna` string** (containing only the active character positions where `chromatin_state[idx] == true` opened by the transgenerational 3-wave epigenetic Hox cascade). Any epigenetic learning or transgenerational methylation change instantly and organically shifts the primary/secondary colors, muscle stiffness, body dimensions, and all 20 brain synapses in real-time (true Lamarckian Epigenetics)!
  - **Class 2 (Specialized Organs):** Scanned directly from the raw `clean_genome` using our robust, any-length promoter-payload model (`extract_raw_gene_payloads`). Completely removes payload length restrictions and allows specialized organs (Eyes, Noses, Tactiles, Biolums) to emerge randomly through mutation.
  - **Pristine Random Wildtypes:** Restored `generate_random_genome` to return a 100% pure random sequence of characters, starting fresh runs with blind organ-less proto-worms that organically evolve complex sensory and neural pathways over generations.
- **Automated TS-RS Type Generation (TCK-115):** Integrated `ts-rs` into the Rust backend to automatically compile and export type-bindings for core data schemas (`CreatureAgent`, `FoodSpore`, `CreaturePhenotype` etc.) on every build/test, eliminating any manual synchronization errors.
- **Unified Single-Page Application (SPA) Architecture (TCK-115):** Purged separate HTML documents and migrated the entire dashboard and training chamber into a single generic `index.html` skeleton. Toggling between views is now performed instantly via zero-latency CSS visibility classes driven by `src/main.ts`.
- **Automated Background Sim Pausing:** Configured the SPA switcher to automatically trigger a simulation-wide suspend (`TOGGLE_SIMULATION` with `running: false` and `PAUSE_TRAINING`) when swapping tabs to conserve CPU and GPU performance.

### Fixed
- **Vite Bundler & Database Simplification:** Permanently deleted the legacy `digilife.db` database and simplified the 240-line `vite.config.ts` down to a clean, generic 10-line Vite bundler configuration, successfully uninstalling the heavy native dependency `better-sqlite3`.
- **Evolved Brain Constellation Views:** Modularized directed-graph brain drawings into a unified, reusable `BrainRenderer` class. It unifies neural alignments and glow-updates, rendering the Ocean's networks organically via Rust's spring-embedded coordinates while maintaining structured column grids for the Trainer candidates.

## [1.6.3] - 2026-08-21

### Fixed
- **AppImage Icon Square Panic:** Created the required square PNG icons (`32x32`, `128x128`, `256x256`, `512x512`) inside `src-tauri/icons/` and registered them explicitly inside `tauri.conf.json`. This completely resolves the Linux AppImage bundling panic and completes the multi-platform compiler flow!

## [1.6.2] - 2026-08-21

### Added
- **Manual Workflow Dispatch Trigger:** Added `workflow_dispatch` to `.github/workflows/release.yml`, allowing developers to manually trigger full multi-platform desktop release builds directly inside the GitHub Actions web UI without needing to push tags.

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
- **Selected Brain SVG Activations:** Fully supported live cognitive brain activity streaming. When a creature is selected in the sidebar or canvas, Rust streams its real-time neuron activations to drive the SVG Directed Graph glows in the UI.
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
- **Close-Call Proximity Protection Clause:** Exempted near-success attempts (where a creature gets within 30px of the target) from the 120px minimum path requirements, perfectly preserving high-fidelity steering learning while still ruthlessly executing lazy "creeper" crawlers.
- **Ultra-Fast DOM Recycling:** Refactored generational transitions to keep existing browser cards and canvases untouched in memory when $N$ is unchanged, cutting transition time from `200ms` down to `<1ms` and completely removing browser-rendering bottlenecks.
- **Non-Blocking Macrotask Micro-yielding:** Scheduled the main loop execution inside `setTimeout(tick, 0)` at generation transitions to yield execution to Chrome's event-loop, keeping the browser 100% responsive and fluid during Superwarp training.
- **High-Tech Spacing Button Group:** Consolidated the "Headless" toggle directly inside the primary control row as a beautiful dark-slate button (Headless: OFF) and glowing neon-cyan button (Headless: ON), matching the dark terminal theme.

## [1.3.0] - 2026-08-19

### Added
- **Multi-Trial Sandbox Trainer (TCK-105):** Built a concurrent, multi-sandbox evolutionary reinforcement learning trainer UI (`trainer.html` and `src/trainer.ts`) to breed and select highly fit, direct-navigating progenitor cells.
- **Success-Gated Path Efficiency (Path Distance):** Implemented a high-fidelity path efficiency metric that rewards direct straight-line trajectories and penalizes winding zigzag paths, heavily weighted with up to 2000 points upon food consumption.
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
- **Physical Multi-Agent Collisions:** Added mass-based elastic overlap resolution and momentum transfer (impact bounce) between creatures themselves, as well as kinetic "bugwave" displacement pushing food spores out of creature bodies.

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