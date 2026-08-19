---
id: TCK-103
title: Improve Petri Dish simulation and environmental physics (Bio-Basin)
status: ongoing
assigned: Gemini-CLI
created_at: 2026-08-19
---

# TCK-103: Improve Petri Dish simulation and environmental physics (Bio-Basin)

## Context
Instead of a flat, sterile rectangular dish with wall boundaries, we will transform the world into a seed-generated, toroidal-wraparound **Cybernetic Bio-Basin** using deterministic procedural generation (similar to the Neumann simulation universe mechanics). To satisfy configuration and usability needs, we will add a central JSON configuration file and a standalone web-based Seed Preview Page.

## Requirements
1.  **Central Configuration (`config.json`):** Create a root-level config file containing variables like `seed`, `targetPopulation`, `basalMetabolicRateMultiplier`, etc. The server reads this file on startup.
2.  **Procedural Generator (`src/shared/mapGenerator.ts`):** 
    - Implement a seed-based PRNG (using the existing high-speed `mulberry32` algorithm) to generate:
      - **Obstacles:** Circular barrier reefs/rocks.
      - **Current Vents:** Thermal currents that push or pull objects (vector force fields).
      - **Biome Areas:** Specialized habitats (e.g., Algae Shallows, Acid Pools) with distinct spore spawn rates, energies, and damage multipliers.
3.  **Toroidal Wraparound Physics:**
    - Update `src/server/index.ts` coordinate calculations to wrap around logically when passing the bounds of $19200 \times 10800$.
    - Implement toroidal distance and vector computations so creatures can sense things across boundaries.
4.  **Spatial Grid Partitioning:**
    - Divide the $19200 \times 10800$ field into a 2D Spatial Grid to keep collision detection at $O(1)$ and guarantee 60 FPS performance.
5.  **Interactive Seed Preview Page (`preview.html` / `src/preview.ts`):**
    - A minimal frontend page loaded independently containing only:
      - A Seed text input.
      - A "Randomize" button.
      - A Canvas showing the procedural terrain layout (obstacles, biome zones, vents) - nothing else!
6.  **Simulation Renderer Sync:**
    - Update `src/render/creatureRenderer.ts` and `src/beta.ts` to procedurally render the barriers, currents (moving particle flows), and biomes.

## Tasks
- [ ] Create `config.json` at root.
- [ ] Implement deterministic procedural world generator in `src/shared/mapGenerator.ts`.
- [ ] Implement toroidal physics wraparound and Spatial Grid partitioning on headless server.
- [ ] Build standalone seed preview page (`preview.html` and `src/preview.ts`).
- [ ] Update frontend canvas renderer to render the procedural Bio-Basin.
- [ ] Establish automated unit and integration tests verifying the map generator and toroidal collisions.
