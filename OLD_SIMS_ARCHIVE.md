# 🏛️ Pixel DNA Life - Legacy Simulations Archive

This document preserves the documentation, mechanics, and structures of the legacy browser-based simulations that are being removed from the repository root to transition fully into the high-performance Tauri Desktop application.

The legacy codebase is permanently archived in git and can be retrieved from any commit prior to:
`chore: remove legacy browser-based assets and archive configurations` (Commit Hash: `5a37c25` and earlier).

---

## 🔬 1. The Legacy Standalone Petri Dish
* **Files:** `index.html` (UI), `src/beta.ts` (State & Websocket), `src/beta.css` (Styles)
* **Status:** Fully Ported to Tauri `tauri_ocean.html` & `src/tauri_ocean.ts`.

### Mechanics & Features
The Petri Dish was the original web-based ecosystem simulation that allowed real-time observation of the biological lifecycle of Urzelles.
* **WebSocket Replication:** Connected to a standalone Node.js server via WebSockets (`ws://localhost:3002/api/simulation`) to receive real-time positions of up to 45 creatures and 300 food spores.
* **Ecosystem Metrics:** Displayed active population count, highest evolved generation, and food spore quantity.
* **Watson-Crick DNA Helix Grid:** Drawn dynamically as an interactive loci panel (`inspect-genome-grid`). Color-coded promoter regions ($0-15$) and active gene segments (Hox-transcribed codons) while dimming out silent junk DNA.
* **Diagnostics Focus:** Clicking any creature locked the camera onto it and displayed its physical state (energy, age, adrenaline) and real-time CTRNN brain activations.

---

## 🏆 2. The Legacy Evolutionary Trainer
* **Files:** `trainer.html` (UI), `src/trainer.ts` (Zucht-Core & Web)
* **Status:** Fully Ported to Tauri `tauri_trainer.html` & `src/tauri_trainer.ts` (with parallel background thread in `src-tauri/src/biology/trainer_engine.rs`).

### Mechanics & Features
The Web Trainer calculated evolutionary reinforcement learning loops in parallel sandboxes directly inside the browser's Javascript engine.
* **Local Physics Stepper:** Ran 16 parallel simulation chambers (each `1000x1000` logical canvas coordinates downscaled by CSS).
* **Standstill & Circular Movement Penalties:** Automatically filtered out stagnant or aimless circular creatures by assigning a `0.0` fitness score at epoch ends.
* **HTTP REST POST Backends:** Made HTTP POST requests to save Elite Champions and generation statistics into an SQLite database handled by the Express.js server.
* **Warp Speed Sliders:** Allowed speeding up Javascript loop execution (prone to heavy browser throttling when minimized/unfocused).

---

## 🕹️ 3. The Interactive Player Sandbox
* **Files:** `preview.html` (UI), `src/preview.ts` (Player Controls & Physics)
* **Status:** Scheduled for Creator UI integration (TCK-111).

### Mechanics & Features
The Preview page was a standalone sandbox that allowed the user to **manually pilot and control a single creature** using the keyboard inside a procedurally generated biome world.
* **Keyboard WASD / Arrow Controls:** Intercepted keyboard events to manually apply forward muscle thrust and lateral steering angles to a player-agent creature.
* **Fluid Current Drift:** Intersected the player creature's position with thermal current vectors (`getVectoredCurrentAt`) to slide and drift the creature realistically.
* **Obstacle Collisions:** Ran local axis-aligned bounding box (AABB) checks against polar-coordinate jagged obstacles (`checkObstacleCollision`) to bounce the player creature back upon impact.
* **Manual Gene Editing:** Intended to allow editing the genome character by character and clicking "Re-Sprout" to observe how the phenotypic tail harmonics, speed, and size changed instantly on the canvas.
