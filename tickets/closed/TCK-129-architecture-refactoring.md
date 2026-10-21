---
id: TCK-129
title: Unified Biology, Warp Speed, and Ecological Decomposition
status: closed
assigned: Gemini-CLI
created_at: 2026-09-03
closed_at: 2026-09-03
---

# TCK-129: Unified Biology, Warp Speed, and Ecological Decomposition

## Background & Motivation
The simulation suffered from redundant architectures and isolated logic between the Trainer environment and the Ocean substrate. This caused behavioral inconsistencies (Ocean creatures had a "dummy" brain input loop and acted blindly), redundant state management (two separate warp speed variables), and an incomplete ecological cycle (dead creatures decomposed into green plants instead of red meat). Furthermore, creatures died far too quickly due to a short lifespan setting in the configuration. 

This ticket tracked the unified biological logic, merged duplicate controls, extended lifespan, and solved the food web.

## Scope & Impact
- **Biology/Engine:** Moved sensory calculation logic out of the trainer and into a shared biology module in `brain.rs` (`compute_sensory_inputs`).
- **Ocean Engine:** Fed real-time surrounding environment data (plants, peers) into the sensory calculation for Ocean creatures so they have active vision and smell. Modified spawn coordinates when spawning from the Catalogue to prevent stacking. Fixed the decomposition logic to drop meat spores (`typeId: 2`) instead of plant spores.
- **Warp Control:** Merged duplicate controls into a unified slider inside the Ocean HUD sidebar, capping warp steps in Rust to 35 steps to prevent thread starvation.
- **Configuration:** Extended the base lifespan in `config.json`.

## Solution Implemented
1.  **Sensory Unification:** Decoupled and moved `compute_sensory_inputs` into `src/shared/brain.rs` as the shared Single Point of Truth.
2.  **Ocean Real-time Sight:** Implemented real-time localized nearest plant and prey/peer searches using the `SpatialGrid` and fed them to the shared sensory compiler.
3.  **Ecological Meat-Cycle:** Programmed dead carcasses to drop crimson red meat spores (`type_id = 2`). Optimized grazing logic so that herbivores eat plants, carnivores eat meat carcasses, and eaten meat carcasses recycle back into Algae plants.
4.  **Warp Sliders:** Created the custom high-fps speedup slider in the Ocean sidebar, capping loop iterations at 35 steps in Rust.
5.  **Distributed Spawns:** Scattered catalogue clonal releases randomly across the Ocean substrate.
6.  **Lifespan Extension:** Extended maximum lifespan to 10 minutes, and subsequently implemented dynamic, DNA-dependent lifespans (`mature_age * 12`).
