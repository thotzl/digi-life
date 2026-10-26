---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, tickets/closed/TCK-103-improve-petri-dish.md, tickets/closed/FIX-127-restore-biomes.md, tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/server/engine.rs, tickets/closed/TCK-125-dry-kiss-cleanup.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-145-predation-energy-and-bite-cooldown.md]
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

## Energy economy (P6)

- Drain per tick: the shared energy function `metabolism::drain_energy` (body, organs, neurons, synaptic activity, thrust; TCK-151), right after the brain step, plus thermal stress outside the thermal tolerance. The former BMR drain with the `* 0.45` literal is gone; the median typical total cost of structured genomes (resting + movement at the mean |thrust| of a standard trial) equals that old drain (0.0243 per tick), resting alone is ≈ 0.0152, full thrust about doubles the resting cost ([[wiki/biology/metabolism-and-reproduction]]).
- Income: plant and meat spores (`metabolism::food_energy`, `foodSporeEnergy` 15 · diet efficiency), photosynthesis in the light zone (`photosynthesisEnergyGain`), and bites.
- **Predation** (TCK-145): one bite per attacker and tick on the nearest eligible victim, per-pair cooldown `biteCooldownTicks` (60); the attacker gains `bitingBaseEnergyGain · carnivory · bitingEfficiencyCarnivoreScale`, capped by the 35 damage and by what the victim still has after earlier bites in the same tick, and the bite counts as eating for mitosis (`has_eaten`). Predators can now live from biting; before P6 a bite gave no energy and the same victim could be bitten every tick (RV-B07 resolved). The `[BITE ATTACK]` log event shows damage and gain.
- Death log: "hunted down by predators" only for an agent that took bite damage in that tick, otherwise starvation/senescence (`death_log`); before P6 every death in the damage pass was reported as predation (P5: 101 "hunted" deaths with 0 bites in 90 s).
- **Measured economy check** (UI, 90 s at maximum warp, single run, `thread_rng`): total deaths P5 ≈ 149 vs. P6 ≈ 147; P6: 13 bites, 5 predation deaths, 142 starvation/senescence deaths, 157 restocks; the population stays at the target 25. The death turnover is unchanged against P5 and is held by restocking (RV-B01); predation is rare. No headless Ocean runner, so there are no seeded multi-run numbers.

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
- Energy uses the shared activity-based cost (TCK-151); bites transfer energy with a cooldown (TCK-145); a 90 s UI check shows P5-level death turnover (≈ 147 deaths, 5 by predation) at a stable population of 25.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, FIX-127, TCK-129, `docs/revisit/01-critical-bugs.md`, `docs/revisit/04-simulation-model.md`, `HANDOFF.md`, TCK-145, TCK-150, TCK-151, TCK-172, TCK-180, TCK-184.
