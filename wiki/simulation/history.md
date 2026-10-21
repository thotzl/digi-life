---
sources: [.tickets/closed/TCK-103-improve-petri-dish.md, .tickets/closed/TCK-104-better-steering-and-neurons.md, .tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, .tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, .tickets/closed/FIX-127-restore-biomes.md, .tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, .tickets/closed/TCK-129-architecture-refactoring.md, .tickets/closed/TCK-135-wall-reflection-sign-fix.md, .tickets/closed/TCK-136-trainer-sandbox-epoch-ticks.md]
---

# Simulation History (implemented)

| Ticket | Date | What was implemented | Article |
| :--- | :--- | :--- | :--- |
| TCK-103 | 2026-08-19 | Closed basin with elastic walls, procedural world (Mulberry32), `config.json` limits, spatial grid 80 px cells, creature/spore collisions | [[simulation/ocean]], [[simulation/physics]] |
| TCK-104 | 2026-08-19 | Continuous thrust + bending steering, speed-coupled turning, visual spine curvature, WASD preview sandbox (TS era) | [[simulation/physics]] |
| TCK-105 | 2026-08-19 | Evolutionary trainer: sandbox grid, epoch fitness, culling bottom 50 %, elite cloning/mutation, champion export | [[simulation/trainer]] |
| TCK-106 | 2026-08-19 | Prey mocking for carnivore training; 3-column trainer dashboard; deletable training runs | [[simulation/trainer]] |
| FIX-127 | 2026-09-03 | World generation re-implemented in Rust (`map_generator.rs`), sent in `INIT_STATE`, rendered by the client | [[simulation/ocean]] |
| TCK-120 | 2026-08-31 | 3-trial evaluation loop; minimum spawn distance (originally 200 px, now 550–1200 px, see RV-S08) | [[simulation/trainer]] |
| TCK-129 | 2026-09-03 | Shared `compute_sensory_inputs`; Ocean creatures get real sensing; meat decomposition cycle; unified warp slider with 35-step cap; scattered catalogue spawns; DNA-dependent lifespan `mature_age * 12` | [[simulation/ocean]], [[biology/metabolism-and-reproduction]] |
| TCK-135 | 2026-10-19 | Wall reflection sign fix on left/top (RV-B02) | [[simulation/physics]] |
| TCK-136 | 2026-10-19 | Trainer `epoch_ticks` increments; speed bonus effective (RV-B05) | [[simulation/trainer]] |

## Key Takeaways

- Ocean and Trainer share physics and sensing since TCK-129.
- Phase 0 fixed two fitness-distorting bugs (walls, speed bonus); older trainer fitness values are not comparable.

Sources: closed tickets listed in the frontmatter.
