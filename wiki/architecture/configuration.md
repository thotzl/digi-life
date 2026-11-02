---
sources: [tickets/closed/TCK-198-r1-sim-context-config-utils.md, wiki/contracts/config-section.md, .agents/skills/project-baseline/SKILL.md, tickets/closed/TCK-138-embedded-config.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-145-predation-energy-and-bite-cooldown.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md]
---

# Configuration (`config.json`)

How a section or key is added: [[wiki/contracts/config-section]] (contract) and [[wiki/guides/add-config-key]].

## Mechanism

- Root `config.json` is the single source of truth for simulation tunables. It has one object per section; `AppConfig`
  (`src/config/mod.rs`) has one field per section, and each section struct lives next to the code that reads it
  (TCK-198).
- Compiled into the binary via `include_str!` (TCK-138). Debug builds (`tauri dev`, `cargo test`, debug runner) let a
  `config.json` in the working directory override it; an invalid override is logged and the embedded config is used.
  Release builds (installed app, CI runner `--release`) always use the embedded config.
- Loaded once by the entry points: Tauri setup (`main.rs`; managed state for the commands, owned copy for the engine
  thread) and the headless runner (`trainer_runner.rs`). Below them the config is passed explicitly (`&AppConfig` or
  one section); there is no global read in the model. `TrainerSim` keeps a clone; probes pass modified copies.
  Unit tests use `config::test_config()` (the embedded config).
- Every section struct and nested block uses `#[serde(deny_unknown_fields)]` (TCK-149): an unknown key or section
  fails to parse. No section has a `Default` with values, so there is no second source of values (RV-B04).

## Tests (`config::tests`)

- `test_config_roundtrip_keeps_every_key`: deserialize → serialize gives exactly the key paths of `config.json`.
- `test_every_config_key_has_read_site`: every field of every section (key list from the struct) is read as `.field`
  by production code (test modules, comments, strings excluded). No exemptions (TCK-145 wired the last unread key).
- `test_unknown_config_key_is_rejected`, `test_embedded_config_matches_config_json`.

## Sections

| Section | Struct (file) | Read by |
| :--- | :--- | :--- |
| `core` | `CoreConfig` (`src/config/mod.rs`) | Ocean world generation, `INIT_STATE.seed`, trainer run seeds |
| `ocean` | `OceanConfig` (`src/server/ocean_config.rs`) | Ocean step in `server::engine` |
| `trainer` | `TrainerConfig` (`src/biology/trainer_config.rs`) | `trainer_sim`, `trainer_engine`, `scenarios`, runner defaults |
| `genome` | `GenomeConfig` (`src/biology/genome/mod.rs`) | genome generator, mutation, Lamarckian inheritance |
| `brain` | `BrainConfig` (`src/shared/brain.rs`) | `parse_genome` (brain compile), brain step (learning) |
| `metabolism` | `MetabolismConfig` (`src/biology/metabolism.rs`) | energy function, start energy, food and bite energy |
| `physics` | `PhysicsConfig` (`src/shared/physics.rs`) | `step_creature_kinematics`, wall and reef contact |

### `core`

- `seed` (`"ALIFE_BASIN_77A"`) → Ocean `generate_world` and `INIT_STATE.seed` (the frontend does not read it); trainer
  run seed `seed_from_str("{seed}:{run_id}")`, after a trainer reset with a clock nonce (`run_seed`, TCK-173, TCK-194).
  The Ocean simulation RNG is still `thread_rng` (RV-T04).

### `ocean`

- `targetPopulation` 25 (restock target), `foodSporeCount` 300, `maxCreatures` 45 (cap for `INJECT_URZELLE` /
  `SPAWN_CLONE` and mitosis, TCK-149; [[wiki/simulation/ocean]]).
- Lifespan `min(mature_age · creatureLifespanMatureAgeMultiplier (12), creatureMaxAgeTicks (36000))`; mitosis floors
  `creatureMatureAgeFloor` 600 and `reproductionStomachThresholdFloor` 0.60; `reproductionSplitLossRatio` 0.4,
  `reproductionRecoilVelocityScale` 15; `lamarckianAssimilationAgeFloor` 1200 (younger parents pass no learned
  weights) ([[wiki/biology/metabolism-and-reproduction]]).
- `thermalStressPenaltyScale` 0.0012, `photosynthesisEnergyGain` 0.20 (green creatures below
  `bitingCarnivoryThreshold` in the light zone).
- Eat radius `mean_radius · grazingRadiusMultiplier (1.5) + grazingRadiusOffset (4)`; bite range
  `mean_radius · bitingRadiusMultiplier (1.6) + bitingRadiusOffset (5)` for carnivory ≥ `bitingCarnivoryThreshold`
  0.35; `biteCooldownTicks` 60 per attacker/victim pair (TCK-145).
- Corpse meat spores `floor(length · radius / decompositionSizeRatio (1200))` clamped to
  [`decompositionSporeMin` 1, `decompositionSporeMax` 5].
- `speciesRecordRetention` 2000: extinct `species_records` rows kept (newest extinction first; alive species always
  stay), pruned with the 10 s Ocean snapshot (TCK-159, [[wiki/architecture/persistence]]).

### `trainer`

- `trainerPopulation` 100: app default population, independent of the displayed tiles; the runner's `--population`
  overrides it; a missing key falls back to 100 (TCK-192). `trainerGenomeRetentionPerRun` 200: DB rows kept per run
  and in-memory hall of fame size (TCK-183, [[wiki/architecture/persistence]]).
- Evaluation (TCK-174): `trainerTrials` 8 trials per generation with multi-trial on; `trainerTrialAggregation`
  `"mean"` | `"median"` (mean); `trainerLayoutPolicy` `"hold"` | `"rotate"` (rotate: new shared layout every
  generation; hold: until every elite solves it).
- Mutation (TCK-174, TCK-181): `mutationOpsMin` / `mutationOpsMax` 1 / 3 operator calls per mutant and hall-of-fame
  child; `mutationOneFifthRule` true (population 1/5 success rule on the max); `mutationOpsLimit` 8 (upper bound of the
  max for the 1/5 rule and the UI slider, lower bound max(1, `mutationOpsMin`)); `mutationChannels` `"mixed"` |
  `"separate"` (experimental: one body or brain channel per child).
- `trainerFitness` (TCK-175, weights of `scenarios::calculate_fitness` in catch units, both below 1):
  `firstCatchBonus` 0.5, `approachBonus` 0.3.
- `trainerSpawnMinDistance` (`RoomPair<f32>`, TCK-154): minimum food distance for trial starts and respawns,
  `standard` 200, `exploration` 500 px; reasoning in the doc comment of `TrainerConfig::trainer_spawn_min_distance`.
- `trainerCurriculum` (TCK-176): gate thresholds `upper` 0.3 / `lower` 0.1 on the success rate of the non-rehearsal
  trials, `step` 0.1, `startLevel` 0.0, `rehearsalShare` 0.0 (off), `rehearsalLevel` 0.0, `metabolicRampStart` 1.0
  (no ramp), room endpoints `chamberSize` {1000, 3500} px and `epochTicks` {300, 1800} (geometric in the level),
  `foodPerType` {1, 3} (linear, rounded). The spawn distance of a level is geometric between the two
  `trainerSpawnMinDistance` values ([[wiki/simulation/trainer]]).
- `trainerMinimalCriterion` (RV-S09, TCK-175): `enabled` true, `maxRedraws` 50, `mobilityTicks` 100,
  `mobilityMinDistance` 5.0 px, plus at least one eye or nose (`meets_minimal_criterion`).

### `genome`

- `lamarckianAssimilationChance` 0.25: probability per plastic synapse with a homologue that a child takes the
  parent's learned offset (TCK-152; Ocean mitosis and trainer breeding).
- `genomeComposition` (TCK-184, `generate_structured_genome`: trainer generation 1, immigrants, Ocean founders):
  `targetLength` 256, `payloadLengthMin`/`Max` 8/16, `organCountMin`/`Max` 1/3, `pFoodOrgan` 0.9,
  `foodOrganEyeShare` 0.5, `pHox` 0.3, `neuCountMin`/`Max` 1/3, `syCountMin`/`Max` 0/2
  ([[wiki/biology/genome-compiler]]).
- `mutationOperators` (TCK-184, relative weights of `mutate_genome`): `creep` 0.6, `point` 0.15, `insertion` 0.05,
  `deletion` 0.05, `geneDuplication` 0.075, `geneDeletion` 0.075, `creepMaxStep` 2; the `EVO` gene scales the indel
  weights.

### `brain`

- `thalamusSumActivation` `"sigmoid"` | `"tanh"` (tanh, TCK-190): activation of the thalamus sum nodes; tanh keeps a
  silent channel at 0. Read by `parse_genome`, so it is part of the golden master.
- `hebbianLearningRateBase` 0.00015: η of the Oja rule, the only learning key (TCK-152,
  [[wiki/biology/ctrnn-brain]]).

### `metabolism`

- Energy function constants, derived not tuned (TCK-151, re-derived for model version 6 by TCK-190):
  `bodyCostPerMass` 2.241e-5, `organCostPerLoad` 6.575e-4, `neuronCost` 1.639e-5, `activityCost` 1.84e-4,
  `movementCostPerMass` 2.636e-5; `trainerMetabolicScale` 1.0 = factor on the trainer drain only (TCK-178).
- `creatureStartStomachRatio` 0.60: start energy `stomach_capacity · ratio` of every spawned creature, Ocean and
  trainer (`creature_factory::start_energy`, TCK-185).
- Food and bite energy, shared by Ocean and trainer: `foodSporeEnergy` 15 (TCK-178), diet efficiency
  `grazingEfficiencyHerbivoreScale` 1.50 / `bitingEfficiencyCarnivoreScale` 1.25; bite gain
  `bitingBaseEnergyGain (35) · carnivory · bitingEfficiencyCarnivoreScale`, capped by `bitingEnergyDamage` 35 and the
  victim's energy (TCK-145) ([[wiki/biology/metabolism-and-reproduction]]).

### `physics`

- Speed band (TCK-150): `locomotionSpeedMin` 1.0 / `locomotionSpeedMax` 5.0 px/tick terminal speed at full thrust of
  the slowest / fastest genome; thrust is derived from it (`target_terminal_speed`, `thrust_for_terminal_speed`).
- `turnRateAtRest` 0.008 rad per tick per unit bend; `dragForwardCoefficient` 0.015, `dragForwardStiffnessDecay` 0.3,
  `receptorBallastScale` 0.18 per organ; `elasticWallRestitution` 0.5 at walls and reefs (TCK-180)
  ([[wiki/simulation/physics]]).

## Key Takeaways

- One config, embedded in the binary, sectioned per subsystem; the debug build may override it from the working
  directory; unknown keys and sections are rejected.
- The config is loaded only by the entry points and passed explicitly; every key has a production read site
  (enforced by test).
- Energy constants are derived, not tuned (TCK-151); trainer evaluation, mutation, fitness, spawn distances,
  curriculum and the minimal criterion are config keys (TCK-174 to TCK-176).

Sources: TCK-198, [[wiki/contracts/config-section]], `.agents/skills/project-baseline/SKILL.md`, TCK-138, TCK-145,
TCK-149, TCK-150, TCK-151, TCK-152, TCK-154, TCK-173, TCK-174, TCK-175, TCK-176, TCK-178, TCK-180, TCK-181, TCK-183,
TCK-184, TCK-185.
