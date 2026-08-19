---
name: server-simulation
description: Rules and guidelines for the headless simulation engine, physics, WebSockets replication protocol, and backend endpoints of Pixel DNA Life.
---
# server-simulation: Headless CTRNN Physics & Networking Engine

## I. Headless Simulator Loop
- **Frequency:** Locked to ~60 updates per second (using high-precision intervals or loops in `src/server/index.ts`).
- **State Properties:** Maintains two primary arrays:
  - `creatures: CreatureAgent[]`
  - `foodPellets: FoodSpore[]`
- **Physics Integration:** Implements Newtonian mechanics for creature agents. Velocity is updated via applied brain motor outputs, drag, boundary collisions, and physical impulses (e.g., predatory biting or bouncing off structures).

## II. WebSocket State Sync Protocol
- **Server Port:** `3002`.
- **Broadcast:** Streams high-frequency delta or full-state payloads containing positions, velocities, orientation, energy levels, active synapses, and brain state updates to all connected web-client monitors.
- **Synchronization Model:** One-way headless truth. The client is a reactive monitor and does not authoritatively mutate the positions of creatures; it only requests actions (such as manual spawn injection or specimen tracking).

## III. Persistence & Database Synchronization
- **State Resumption:** The simulator loads state on startup from `simulation_state.json` and writes periodic auto-saves back to it.
- **Species Ledger:** Synchronizes active and extinct species to `species_db.json`.
- **Integrity Rule:** Ensure all filesystem reads/writes are non-blocking where possible, or handled synchronously strictly during startup/shutdown phases.
