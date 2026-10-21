---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, .tickets/closed/TCK-103-improve-petri-dish.md, .tickets/closed/FIX-127-restore-biomes.md, .tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/server/engine.rs]
---

# Ocean

The open-world mode: a closed basin with a free-living population.

## World

- Size `19200 × 10800` px, hard elastic walls (no torus since TCK-103).
- Procedural world generated in Rust (`src/shared/map_generator.rs`, `generate_world(seed, w, h)`, Mulberry32 PRNG): biomes, reef obstacles (`ProceduralObstacle`), current vents, vertices. Sent to the client in `INIT_STATE` on `CLIENT_READY`/`RESET_WORLD`; the client only renders it (FIX-127).
- Reefs collide physically and are felt by tactile organs (FIX-131), see [[simulation/physics]].
- Biomes are generated and rendered but have **no simulation effect** (RV-S01).

## Population and food

- `foodSporeCount` spores (300) spawned around nutrient centers; ~1 spore per 830 × 830 px.
- Restocking refills to `targetPopulation` from catalogue clones or random genomes (catalogue loaded once per pass). `maxCreatures` exists but is not read.
- Creatures see/smell nearby plants and peers via the spatial grid (TCK-129), see [[biology/sensing]].
- Catalogue releases are scattered randomly to avoid stacking (TCK-129).
- Mitosis, death, decomposition and predation: [[biology/metabolism-and-reproduction]].

## Auto-save

- Every 10 s `simulation_state` (`current_state`) and a `simulation_history` row; loaded on startup ([[architecture/persistence]]).

## Known defects and hypotheses

- **RV-B01 (P0, TCK-143):** no surviving offspring — no heredity in the Ocean.
- **RV-S02 (P1 hypothesis, TCK-150) "motionless Ocean":** thrust `≈ stiffness · (pulse·1000)² · 3.4 · eta · out_thrust`. With `pulse = 0.0015` (genome minimum) and `stiffness = 0.15` → ~1.1 · eta; mass `r^1.5 · L/25` ≈ 230–1,180; damping 0.94 → terminal speed 10⁻³–10⁻² px/tick. Such genomes are effectively immobile; negative `out_thrust` swims backwards. Measure the speed distribution over 1,000 random genomes headlessly before changing constants.
- **RV-S07 (hypothesis, TCK-150):** ecology very sparse — 300 spores, sensor range `scale · 550` (190–990 px), 25 agents; random food finding is rare. Balance only after RV-B01/S02, with measurement series.
- These two findings superseded the earlier hypothesis that the render time scale in `frontend/ocean.ts` caused the motionless Ocean.

## Key Takeaways

- Rust-generated world, rendered by the client; biomes are cosmetic.
- Population is held by restocking only; offspring are dropped (TCK-143).
- Low mobility and sparse food are measured-first hypotheses (TCK-150).

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, FIX-127, TCK-129, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`, `HANDOFF.md`.
