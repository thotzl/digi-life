---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/simulation-analytics/SKILL.md, tickets/closed/TCK-103-improve-petri-dish.md, tickets/closed/FIX-127-restore-biomes.md, tickets/closed/TCK-129-architecture-refactoring.md, tickets/closed/TCK-156-engine-decomposition.md, src/sim/ocean/mod.rs, src/server/engine/ocean_mode.rs, src/diagnostics/ocean_report.rs, tickets/closed/TCK-158-seedable-rng-headless-runner.md, tickets/closed/TCK-125-dry-kiss-cleanup.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-145-predation-energy-and-bite-cooldown.md]
---

# Ocean

The open-world mode: a closed basin with a free-living population.

## Simulation (TCK-156)

- `sim::ocean::OceanSim` (`src/sim/ocean/`) is the Ocean without IO: population, `SporeField` (spores + nutrient
  centres), `SpeciesLedger`, id counters, bite cooldowns, seed and tick. `tick()` = `step()` + restock and returns an
  `OceanStep` (`OceanEvent`s, `SpeciesChange`s, creatures born this step for the UI; they are already in the
  population, the engine's newborn queue only notifies the UI). The engine's `OceanMode` maps it onto UI
  events and the species store ([[wiki/contracts/mode]], [[wiki/architecture/simulation-engine]]).
- One step runs on the shared creature systems ([[wiki/contracts/creature-system]]): spatial grid, food drift before
  all creatures, peer snapshot; per creature `step_creature` with `WorldSettings::ocean` (sense, think, metabolize
  with photosynthesis and thermal stress, death, move, eat), the eat respawn and the decomposition applied right after
  that creature's step; then, after every creature's step, per surviving creature age + 1, bite (no target that died
  in its own step this tick) and mitosis; bite damage after all creatures; mitosis children join the population
  (TCK-143); species census; restock (order since
  TCK-202; before, bite and mitosis ran right after each creature's step, so a creature that was about to starve in
  its own step later in the tick could still feed an attacker).
- Seeded and reproducible: `ocean_seed(core.seed, nonce)` and per-tick streams `derive(seed, [stream, tick])` for food
  (respawn, decomposition), mitosis and restock, `[STREAM_ACTION, n]` for releases, `[STREAM_POPULATE]` for a fresh
  start or reset. The snapshot stores seed, tick, release count and nutrient centres, so a restored Ocean continues
  exactly as an uninterrupted one (`test_same_seed_same_ocean_and_snapshot_continues_identically`). A reset draws a new
  seed (clock nonce).
- Founders: `OceanSpawner::founder(Random | Wildtype | Genome)`: fresh start `FRESH_FOUNDERS` (15) wildtype founders
  (`genome::WILDTYPE_GENOME`, mutated with probability 0.6), reset `RESET_FOUNDERS` (25) random catalogue clones or
  structured random genomes, inject a structured random genome, clone a given genome; uniform 500 px inside the world.
- **Behaviour change of TCK-156 (by design, no Ocean reference existed):** the RNG is seeded instead of `thread_rng`,
  so every Ocean run differs from the former ones; the creature tick runs in the shared system order (eaten-spore
  respawn and corpse decomposition are applied after each creature's step rather than inside its eat loop; the
  creature systems and their arithmetic are those of `engine.rs` before). Nutrient centres survive a restart (before,
  a restart drew new centres for the restored spores). The stale "15 unique wildtype cells" line after a reset is gone.
- `OceanEvent::Died` carries a `DeathCause` (TCK-158): `Starved` (no energy left in its step, or after a tick without
  bite damage), `Aged` (lifespan reached with energy left), `Hunted` (bite damage this tick left no energy). One death
  threshold `death::is_dead_energy` (energy ≤ 0) in the step and after the damage pass (TCK-202; the damage pass
  formerly removed creatures with up to 0.1 energy). The engine logs "hunted down" for `Hunted` only, as before.
- Same creature model as the trainer rooms (TCK-202): `fmath` clock, eat radius from config, food drift before the
  creatures, the water temperature as one depth field for sensing and thermal stress
  ([[wiki/contracts/creature-system]]). World features of the Ocean only: prey targets, photosynthesis, thermal
  stress, lifespan and decomposition, bite and mitosis.
- Headless runs: `trainer_runner ocean --seed <text> --ticks <N>` runs a fresh `OceanSim` without UI, warp cap or DB
  and writes a time series (population, births, deaths by cause, species, energy, spores); CI checks a fixed run
  against `tests/reference/ocean-ci.json` (TCK-158, [[wiki/simulation/analytics-and-balancing]]).

## World

- Size `ocean.worldWidth × ocean.worldHeight` (19200 × 10800 px, config since TCK-156, RV-A05), sent to the UI as
  `INIT_STATE.world.width` / `height`; hard elastic walls (no torus since TCK-103).
- Procedural world generated in Rust (`src/shared/map_generator.rs`, `generate_world(seed, w, h)` with config `seed`, Mulberry32 PRNG, TCK-149; trigonometry via `fmath` since TCK-202, so the layout is platform-independent): biomes, reef obstacles (`ProceduralObstacle`), current vents, vertices. Sent to the client in `INIT_STATE` on `CLIENT_READY`/`RESET_WORLD`; the client only renders it (FIX-127).
- Reefs collide physically and are felt by tactile organs (FIX-131), see [[wiki/simulation/physics]].
- Biomes are generated and rendered but have no simulation effect; the unused biome config keys were removed (TCK-149).

## Population and food

- `foodSporeCount` spores (300) spawned around nutrient centers; ~1 spore per 830 × 830 px. Sensor range `scale · 550` (190–990 px).
- Restocking refills to `targetPopulation` from catalogue clones or, with an empty catalogue, structured random founders (`generate_structured_genome`, TCK-184; catalogue loaded once per pass). `INJECT_URZELLE` and the empty-catalogue branch of `RESET_EVOLUTION` (25 founders) use structured random genomes as well; the fresh-start seeding mutates the wildtype literal. `maxCreatures` (45) caps `INJECT_URZELLE`/`SPAWN_CLONE` (rejected with a `LOG_EVENT`, `logType` system) and Ocean mitosis (TCK-149; the mitosis cap counts the children of the same tick, TCK-143). Restock fills only up to `targetPopulation`, so mitosis children can lift the population above the target, up to the cap.
- Creatures see/smell nearby plants and peers via the spatial grid (TCK-129), see [[wiki/biology/sensing]].
- Catalogue releases are scattered randomly to avoid stacking (TCK-129).
- Mitosis, death, decomposition and predation: [[wiki/biology/metabolism-and-reproduction]].

## Energy economy (P6)

- Drain per tick: the shared energy function `metabolism::drain_energy` (body, organs, neurons, synaptic activity, thrust; TCK-151), right after the brain step, plus thermal stress outside the thermal tolerance. The former BMR drain with the `* 0.45` literal is gone; the median typical total cost of structured genomes (resting + movement at the mean |thrust| of a standard trial) equals that old drain (0.0243 per tick), resting alone is ≈ 0.0152, full thrust about doubles the resting cost ([[wiki/biology/metabolism-and-reproduction]]).
- Income: plant and meat spores (`metabolism::food_energy`, `foodSporeEnergy` 15 · diet efficiency), photosynthesis in the light zone (`photosynthesisEnergyGain`), and bites.
- **Predation** (TCK-145): one bite per attacker and tick on the nearest eligible victim, per-pair cooldown `biteCooldownTicks` (60); the attacker gains `bitingBaseEnergyGain · carnivory · bitingEfficiencyCarnivoreScale`, capped by the 35 damage and by what the victim still has after earlier bites in the same tick, and the bite counts as eating for mitosis (`has_eaten`). Predators can now live from biting; before P6 a bite gave no energy and the same victim could be bitten every tick (RV-B07 resolved). The `[BITE ATTACK]` log event shows damage and gain.
- Death log: "hunted down by predators" only for an agent that took bite damage in that tick, otherwise starvation/senescence (`death_log`); before P6 every death in the damage pass was reported as predation (P5: 101 "hunted" deaths with 0 bites in 90 s).
- **Measured economy check** (UI, 90 s at maximum warp, single run, before TCK-156 with `thread_rng`): total deaths P5 ≈ 149 vs. P6 ≈ 147; P6: 13 bites, 5 predation deaths, 142 starvation/senescence deaths, 157 restocks; the population stays at the target 25. The death turnover is unchanged against P5 and was held by restocking (children were not simulated before TCK-143); predation is rare.
- **Headless numbers** (TCK-158, `trainer_runner ocean`, empty catalogue, 5 seeds, 12,000–40,000 ticks): the population is back at the target 25 after every tick; 4.1–4.4 deaths per 1,000 ticks (84–92 % starved, 6–16 % aged, 0–3 hunted per run), ≈ 1 bite per 1,000 ticks with bursts, 0–5 mitosis splits per 40,000 ticks. The UI check above is not comparable per tick (its tick count was not recorded).
- **With simulated offspring** (TCK-143, `trainer_runner ocean`, seeds `m1`–`m5`, 40,000 ticks, before → after): mean population 25.00 → 25.00–25.06 (max 25 → 25–28, min 15 both: the fresh start), splits 0–3 → 0–15 per run (seed `m3`: 2 → 15, highest generation 2 → 4, 220 → 197 restocks), deaths 205–217 → 202–219 (90–95 % starved, 4–8 % aged, 1–5 hunted), species per run 210–220 → 192–220. Seeds without a split are bit-identical. No explosion: mitosis stays rare (food scarcity, RV-S07), so restocking still supplies at least 92 % of the newcomers per run; no config change needed.

## Auto-save

- Every 10 s the typed `OceanSnapshot` is saved to `simulation_state` (`current_state`; written by the snapshot writer thread off the frame thread) and extinct species beyond `speciesRecordRetention` are pruned; the snapshot is loaded on startup (snapshots of earlier versions without seed, tick or nutrient centres still load), failures are logged ([[wiki/architecture/persistence]]).

## Observed behaviour

- **Heredity (TCK-143, RV-B01 resolved):** mitosis children are simulated from the next tick, a clone in the parent's species, a mutant in a new species recorded once it lives (no phantom species). Splits are rare, so the population is still held mostly by restocking.
- **Locomotion (RV-S02 resolved, TCK-150):** the "motionless Ocean" was attributed to thrust ∝ `stiffness · (pulse·1000)²`, which left low-pulse genomes near zero speed (speed probe before P5: 23 % of structured random genomes below 0.2 px/tick). Since P5 the Ocean uses the same speed band as the trainer (`step_creature_kinematics` through `sim/systems/move.rs`): every genome swims 1–5 px/tick at full thrust and can turn in place; wall and reef contact reflect the heading (TCK-180, [[wiki/simulation/physics]]). Negative `out_thrust` swims backwards (≈ half of random brains have a negative mean thrust, steering probe). The Ocean population itself has not been measured.
- **Sparse ecology:** with 300 spores, 25 agents and the sensor range above, random food finding is rare (hypothesis RV-S07, unmeasured; left for the Ocean phase of TCK-169, TCK-150).
- The earlier explanation (render time scale in `frontend/ocean.ts`) is superseded by RV-S02 (fixed) and RV-S07 (open).

## Key Takeaways

- IO-free, seeded `OceanSim` on the shared creature systems; reproducible per seed (TCK-156).
- Rust-generated world, rendered by the client; biomes are cosmetic.
- Mitosis children are simulated since TCK-143 (heredity, no phantom species); splits are rare, so restocking still holds the population.
- Locomotion is fixed for every genome (1–5 px/tick, turning at rest; RV-S02, TCK-150); sparse food remains unmeasured (RV-S07).
- Measured headless since TCK-158 (`trainer_runner ocean`, CI reference `tests/reference/ocean-ci.json`): turnover ≈ 4 deaths per 1,000 ticks, mostly starvation, mitosis rare. Since TCK-202 every simulation path uses `fmath`, so the reference is platform-independent; reference run `ci-ocean-2` (12,000 ticks) after TCK-202: 63 deaths (57 starved, 5 aged, 1 hunted), 7 bites, 3 births (before: 49 deaths, 43 / 3 / 3, 15 bites, 2 births); after TCK-143 (children simulated, reference regenerated, first difference at the first split near tick 7,000): 62 deaths (56 / 5 / 1), 7 bites, 2 births, 70 restocks (73), 69 species (73), max population 26.
- Energy uses the shared activity-based cost (TCK-151); bites transfer energy with a cooldown (TCK-145); a 90 s UI check shows P5-level death turnover (≈ 147 deaths, 5 by predation) at a stable population of 25.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-103, FIX-127, TCK-129, TCK-145, TCK-156, TCK-158, TCK-150, TCK-151, TCK-172, TCK-180, TCK-184, TCK-202.
