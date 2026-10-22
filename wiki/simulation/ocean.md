---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, tickets/closed/TCK-103-improve-petri-dish.md, tickets/closed/FIX-127-restore-biomes.md, tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/server/engine.rs, tickets/closed/TCK-125-dry-kiss-cleanup.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md]
---

# Ocean

The open-world mode: a closed basin with a free-living population.

## World

- Size `19200 × 10800` px, hard elastic walls (no torus since TCK-103).
- Procedural world generated in Rust (`src/shared/map_generator.rs`, `generate_world(seed, w, h)` with config `seed`, Mulberry32 PRNG, TCK-149): biomes, reef obstacles (`ProceduralObstacle`), current vents, vertices. Sent to the client in `INIT_STATE` on `CLIENT_READY`/`RESET_WORLD`; the client only renders it (FIX-127).
- Reefs collide physically and are felt by tactile organs (FIX-131), see [[wiki/simulation/physics]].
- Biomes are generated and rendered but have no simulation effect; the unused biome config keys were removed (TCK-149).

## Population and food

- `foodSporeCount` spores (300) spawned around nutrient centers; ~1 spore per 830 × 830 px. Sensor range `scale · 550` (190–990 px).
- Restocking refills to `targetPopulation` from catalogue clones or, with an empty catalogue, structured random founders (`generate_structured_genome`, TCK-184; catalogue loaded once per pass). `INJECT_URZELLE` and the empty-catalogue branch of `RESET_EVOLUTION` (25 founders) use structured random genomes as well; the fresh-start seeding mutates the wildtype literal. `maxCreatures` (45) caps `INJECT_URZELLE`/`SPAWN_CLONE` (rejected with a `LOG_EVENT`, `logType` system) and Ocean mitosis (TCK-149).
- Creatures see/smell nearby plants and peers via the spatial grid (TCK-129), see [[wiki/biology/sensing]].
- Catalogue releases are scattered randomly to avoid stacking (TCK-129).
- Mitosis, death, decomposition and predation: [[wiki/biology/metabolism-and-reproduction]].

## Auto-save

- Every 10 s `simulation_state` (`current_state`) and a `simulation_history` row; loaded on startup ([[wiki/architecture/persistence]]).

## Observed behaviour

- **No heredity:** mitosis children are not simulated; the population is held by restocking (RV-B01).
- **Locomotion (RV-S02 resolved, TCK-150):** the "motionless Ocean" was attributed to thrust ∝ `stiffness · (pulse·1000)²`, which left low-pulse genomes near zero speed (speed probe before P5: 23 % of structured random genomes below 0.2 px/tick). Since P5 the Ocean uses the same speed band as the trainer (`step_creature_kinematics` in `src/server/engine.rs`): every genome swims 1–5 px/tick at full thrust and can turn in place; wall and reef contact reflect the heading (TCK-180, [[wiki/simulation/physics]]). Negative `out_thrust` swims backwards (≈ half of random brains have a negative mean thrust, steering probe). The Ocean population itself has not been measured.
- **Sparse ecology:** with 300 spores, 25 agents and the sensor range above, random food finding is rare (hypothesis RV-S07, unmeasured; left for the Ocean phase of TCK-169, TCK-150).
- The earlier explanation (render time scale in `frontend/ocean.ts`) is superseded by RV-S02 (fixed) and RV-S07 (open).

## Key Takeaways

- Rust-generated world, rendered by the client; biomes are cosmetic.
- Population is held by restocking only (RV-B01).
- Locomotion is fixed for every genome (1–5 px/tick, turning at rest; RV-S02, TCK-150); sparse food remains unmeasured (RV-S07).

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, FIX-127, TCK-129, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`, `HANDOFF.md`, TCK-150, TCK-172, TCK-180, TCK-184.
