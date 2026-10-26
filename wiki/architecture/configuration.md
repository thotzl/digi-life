---
sources: [.agents/skills/project-baseline/SKILL.md, README.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-138-embedded-config.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-145-predation-energy-and-bite-cooldown.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md]
---

# Configuration (`config.json`)

## Mechanism

- Root `config.json` is the single source of truth for simulation parameters (metabolism, photosynthesis, biting, mitosis, drag, world generation), loaded into `AppConfig` / `SimulationRules` (`src/shared/types.rs`).
- Compiled into the binary via `include_str!` (TCK-138). A `config.json` in the working directory overrides it for development (`tauri dev` runs from the repo root).
- An invalid override is logged as an error and the embedded config is used.
- Test `shared::types::tests::test_embedded_config_matches_config_json` asserts embedded = `config.json`.
- `AppConfig`, `SimulationRules` and the nested block structs (`GenomeComposition`, `MutationOperators`, `MetabolismConfig`, `TrainerFitnessConfig`, `TrainerSpawnMinDistance`, `TrainerCurriculumConfig`, `RoomPair`, `TrainerMinimalCriterion`) use `#[serde(deny_unknown_fields)]` (TCK-149): an unknown key fails deserialization. The embedded config then panics (test/build); an invalid working-dir override is logged and the embedded config is used. Test `test_unknown_config_key_is_rejected`.
- Before TCK-138 release builds used a divergent hand-written fallback (food 600 vs 300, thrust 6.8 vs 3.4, max age 5400 vs 36000) and swallowed parse errors (RV-B04).

## Rule for new tunables

- Do not hardcode tunables in Rust; add them to `config.json` + `SimulationRules`.
- Every key needs a read site: test `test_every_config_key_has_read_site` scans `src/` (excluding `shared/types.rs`) for `.<snake_case_key>` per `config.json` key, including the keys of every nested block, and fails on unread keys (TCK-149, TCK-184).
- There are no exemptions: the former list `KEYS_WITHOUT_READ_SITE` was removed when TCK-145 wired `bitingBaseEnergyGain` (P6). A new key without a read site fails the test.

## Key usage (RV-S01, TCK-149)

- `config.json`: 13 top-level scalar keys (`seed`, `targetPopulation`, `foodSporeCount`, `maxCreatures`, `trainerGenomeRetentionPerRun`, since P7 `trainerTrials`, `trainerTrialAggregation`, `mutationOpsMin`, `mutationOpsMax`, `mutationOneFifthRule`, `mutationOpsLimit`, `mutationChannels`, since P8 `trainerLayoutPolicy`), the blocks `trainerFitness` (2 keys), `trainerSpawnMinDistance` (2), `trainerCurriculum` (7 scalars + 3 room pairs of 2), `trainerMinimalCriterion` (4), `genomeComposition` (12 keys), `mutationOperators` (7 keys) and `metabolism` (6 keys), and `rules` (33 `SimulationRules`). Every key is read; all keys of the RV-S01 review are implemented or removed (TCK-149 closed in P6).
- Removed (TCK-149): `adrenalineMaxMultiplier`, `adrenalineIncreaseRate`, `adrenalineDecayRate`, `adrenalineMetabolicSurchargeScale` (with the always-1 surcharge factor in the Ocean BMR), `bioluminescenceFlashCost`, `biomeHazardDamageScale`, `biomeAlgaeSporeSpawnThreshold`, `predatorSavageThrustThreshold`, `predatorSavageThrustMultiplier`, `restockFounderGeneInheritChance`, and the 7 struct-less keys `obstacles*`, `vents*`, `disableVentsForce`. Adrenaline keys return with TCK-165, biolum with TCK-167.
- Wired by TCK-149:
  - `seed` → Ocean world generation (`generate_world`) and the `seed` field of `INIT_STATE` (frontend does not read it); since TCK-173 also the trainer run seed `seed_from_str("{seed}:{run_id}")`. The Ocean simulation RNG is still `thread_rng` (RV-T04).
  - `maxCreatures` → cap for `INJECT_URZELLE`/`SPAWN_CLONE` and Ocean mitosis ([[wiki/simulation/ocean]]).
  - `creatureMaxAgeTicks` and new `creatureLifespanMatureAgeMultiplier` → lifespan; `creatureMatureAgeFloor`, `reproductionStomachThresholdFloor` → mitosis floors; `lamarckianAssimilationAgeFloor` → Lamarck age gate ([[wiki/biology/metabolism-and-reproduction]]).
- Added in P2 of TCK-171:
  - `trainerGenomeRetentionPerRun` (top level, 200) → `trainer_genomes` rows kept per run and size of the in-memory trainer hall of fame (TCK-183, [[wiki/architecture/persistence]]).
  - `rules.creatureStartStomachRatio` (0.60, renamed from `restockInitialStomachRatio`) → start energy `stomach_capacity · ratio` of every spawned creature, Ocean and trainer (`creature_factory::start_energy`, TCK-185, [[wiki/biology/metabolism-and-reproduction]]).
- Added in P3 of TCK-171 (TCK-184), structs `GenomeComposition` / `MutationOperators` in `dna.rs`, both `deny_unknown_fields`:
  - `genomeComposition` → `generate_structured_genome` (trainer generation 1 and immigrants, Ocean random founders): `targetLength` 256, `payloadLengthMin`/`Max` 8/16, `organCountMin`/`Max` 1/3, `pFoodOrgan` 0.9 (first organ is an eye or nose), `foodOrganEyeShare` 0.5, `pHox` 0.3, `neuCountMin`/`Max` 1/3, `syCountMin`/`Max` 0/2 ([[wiki/biology/genome-compiler]]).
  - `mutationOperators` → relative operator weights of `mutate_genome`: `creep` 0.6, `point` 0.15, `insertion` 0.05, `deletion` 0.05, `geneDuplication` 0.075, `geneDeletion` 0.075, `creepMaxStep` 2; the `EVO` gene scales the indel weights.
- Removed in P4 of TCK-171 (TCK-152, Oja learning): `hebbianLearningStiffnessDecay` (passed to the brain step but never read) and `hebbianForgettingDecay` (redundant with the decay term of Oja's rule). `hebbianLearningRateBase` (0.00015) is the only learning key: η of the Oja rule ([[wiki/biology/ctrnn-brain]]). `lamarckianAssimilationChance` (0.25) is a probability per synapse since TCK-152 ([[wiki/biology/metabolism-and-reproduction]]).
- Changed in P5 of TCK-171 (TCK-150, TCK-180, [[wiki/simulation/physics]]):
  - Added `rules.locomotionSpeedMin` (1.0) / `rules.locomotionSpeedMax` (5.0): terminal speed at full thrust of the slowest / fastest genome in px/tick; thrust is derived from it (`target_terminal_speed`, `thrust_for_terminal_speed`). Added `rules.turnRateAtRest` (0.008): heading change per tick per unit bend independent of speed.
  - Removed `thrustBaseMultiplier` (replaced by the speed band), `steerTorqueBaseMultiplier` (never read; turning is `turnRateAtRest` plus the speed-coupled curvature) and `dragLateralCoefficient` (never read; the lock-on heading leaves no lateral velocity) (TCK-149).
- Changed in P6 of TCK-171 (Energy v2, [[wiki/biology/metabolism-and-reproduction]]):
  - Added block `metabolism` (`MetabolismConfig` in `src/biology/metabolism.rs`), the constants of the shared energy function, derived from share targets and scaled so that the median typical total cost equals the old Ocean drain 0.0243/tick (TCK-151): `bodyCostPerMass` 2.171e-5, `organCostPerLoad` 6.369e-4, `neuronCost` 1.589e-5, `activityCost` 9.26e-5, `movementCostPerMass` 2.554e-5; `trainerMetabolicScale` 1.0 = factor on the trainer drain only (TCK-178).
  - Added `rules.foodSporeEnergy` (15.0): energy of one food spore before diet efficiency, the one food energy source of Ocean and trainer (replaces the literal 15 in `engine.rs`, TCK-178). Added `rules.biteCooldownTicks` (60): ticks before the same attacker can bite the same victim again (TCK-145).
  - `bitingBaseEnergyGain` (35) is now read: attacker gain per bite (TCK-145).
  - Removed `basalMetabolicRateMultiplier` (top level) and `rules.bmrBaseScale` with the multiplicative BMR (TCK-151).
- Added in P7 of TCK-171 (evaluation and fitness, [[wiki/simulation/trainer]]); top level, structs `TrainerFitnessConfig`, `TrainerSpawnMinDistance`, `TrainerMinimalCriterion` and enums `TrialAggregation`, `MutationChannelMode` in `src/shared/types.rs`:
  - `trainerTrials` (3 in P7, 8 since P8) → trials per generation with multi-trial on; `trainerTrialAggregation` (`"mean"` | `"median"`, default mean) → generation fitness from the trial fitness (`aggregate_trials`) (TCK-174).
  - `mutationOpsMin` / `mutationOpsMax` (1 / 3) → mutation operator calls per mutant and HoF child, uniform in [min, max] (`draw_mutation_ops`); replace the former trainer parameter `mutation_rate` (`round(len · rate)`). `mutationOneFifthRule` (true) → population-level 1/5 success rule on the max (`one_fifth_update`). `mutationOpsLimit` (8) → upper bound of the max for the 1/5 rule and the UI slider; lower bound max(1, `mutationOpsMin`) (TCK-174).
  - `mutationChannels` (`"mixed"` | `"separate"`, default mixed) → body/brain mutation channels of trainer children; `separate` mutates one channel per child, 50/50 (TCK-181, experimental).
  - `trainerFitness` → weights of `scenarios::calculate_fitness` in catch units: `firstCatchBonus` 0.5 (bonus of a first catch at tick 0, falling linearly to 0 at trial end), `approachBonus` 0.3 (full relative approach to the nearest diet target in sensing range); both must stay below 1 (TCK-175).
  - `trainerSpawnMinDistance` → minimum food distance per room for trial 0, re-trials and respawns: `standard` 200, `exploration` 500 px (`spawn_min_distance`, TCK-154; reasoning in the `TrainerSpawnMinDistance` doc comment).
  - `trainerMinimalCriterion` → random trainer genomes (generation 1, immigrants): `enabled` (true), `maxRedraws` (50; redraws after the first candidate, else the last one is kept), `mobilityTicks` (100) and `mobilityMinDistance` (5.0 px) of the mobility check; plus at least one eye or nose (`meets_minimal_criterion`, RV-S09, TCK-175).
- Added / changed in P8 of TCK-171 (curriculum, [[wiki/simulation/trainer]]); enum `LayoutPolicy` and structs `TrainerCurriculumConfig`, `RoomPair<T>` in `src/shared/types.rs`:
  - `trainerTrials` 3 → **8** and new `trainerLayoutPolicy` (`"hold"` | `"rotate"`, **rotate** in `config.json`) → when the shared trial layout changes: `rotate` every generation, `hold` until every elite solves it (`TrainerParams::layout_policy`, runner `--layout-policy`). P8 post-fix of TCK-174: the P7 setting (`hold`, 3 trials) memorised layouts. The Rust enum's `Default` is `Rotate` as well.
  - `trainerCurriculum` → `LevelScenario` and the curriculum gate (TCK-176): `upper` 0.3 / `lower` 0.1 (gate thresholds on the success rate of the non-rehearsal trials; above `upper` the level rises, below `lower` it falls), `step` 0.1 (level change per gate decision), `startLevel` 0.0 (level of a new curriculum run), `rehearsalShare` 0.0 (share of trials at the rehearsal level; 0 = off), `rehearsalLevel` 0.0 (level of rehearsal trials, capped at the current level), `metabolicRampStart` 1.0 (drain factor `rampStart + (1 − rampStart) · d` on `trainerMetabolicScale` in curriculum mode; 1 = no ramp), and the room endpoints `chamberSize` {`standard` 1000.0, `exploration` 3500.0} px and `epochTicks` {300, 1800} (geometric in the level), `foodPerType` {1, 3} (linear, rounded). The gate started at 0.6 / 0.3 in the P8 design and was calibrated to 0.3 / 0.1; rehearsal and ramp stay off after calibration.
  - The spawn distance of a level is geometric between `trainerSpawnMinDistance.standard` and `.exploration` (no separate key).
- P3 also removed the last adrenaline and biolum model parts (agent field `adrenaline`, brain input, motors 3/4; TCK-149). Biomes have no simulation effect.

## Key Takeaways

- One config, embedded in the binary, overridable in dev; unknown keys are rejected.
- Every key has a read site, enforced by test, without exemptions (TCK-149, TCK-145).
- Energy constants live in the `metabolism` block and are derived, not tuned (TCK-151).
- Trainer evaluation, mutation ops, fitness weights, spawn distances and the minimal criterion are config keys since P7 (TCK-174, TCK-175, TCK-154, TCK-181); the layout policy, the room endpoints and the curriculum gate, rehearsal and ramp since P8 (TCK-174 post-fix, TCK-176).

Sources: `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/04-simulation-model.md`, TCK-138, TCK-145, TCK-149, TCK-150, TCK-151, TCK-152, TCK-154, TCK-174, TCK-175, TCK-176, TCK-178, TCK-173, TCK-180, TCK-181, TCK-183, TCK-184, TCK-185.
