---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, tickets/closed/TCK-103-improve-petri-dish.md, tickets/closed/FIX-127-restore-biomes.md, tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/server/engine.rs, tickets/closed/TCK-125-dry-kiss-cleanup.md, tickets/ongoing/TCK-149-dead-config-keys.md]
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
- Restocking refills to `targetPopulation` from catalogue clones or random genomes (catalogue loaded once per pass). `maxCreatures` (45) caps `INJECT_URZELLE`/`SPAWN_CLONE` (rejected with a `LOG_EVENT`, `logType` system) and Ocean mitosis (TCK-149).
- Creatures see/smell nearby plants and peers via the spatial grid (TCK-129), see [[wiki/biology/sensing]].
- Catalogue releases are scattered randomly to avoid stacking (TCK-129).
- Mitosis, death, decomposition and predation: [[wiki/biology/metabolism-and-reproduction]].

## Auto-save

- Every 10 s `simulation_state` (`current_state`) and a `simulation_history` row; loaded on startup ([[wiki/architecture/persistence]]).

## Observed behaviour

- **No heredity:** mitosis children are not simulated; the population is held by restocking (RV-B01).
- **"Motionless Ocean":** many creatures barely move. Hypothesis RV-S02: thrust `≈ stiffness · (pulse·1000)² · 3.4 · eta · out_thrust`; with `pulse = 0.0015` (genome minimum) and `stiffness = 0.15` → ~1.1 · eta, mass `r^1.5 · L/25` ≈ 230–1,180, damping 0.94 → terminal speed 10⁻³–10⁻² px/tick. Negative `out_thrust` swims backwards. Not yet measured.
- **Sparse ecology:** with 300 spores, 25 agents and the sensor range above, random food finding is rare (hypothesis RV-S07).
- The earlier explanation (render time scale in `frontend/ocean.ts`) is superseded by RV-S02/RV-S07.

## Key Takeaways

- Rust-generated world, rendered by the client; biomes are cosmetic.
- Population is held by restocking only (RV-B01).
- Low mobility and sparse food are unmeasured hypotheses (RV-S02, RV-S07).

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, FIX-127, TCK-129, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`, `HANDOFF.md`.
