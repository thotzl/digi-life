---
sources: [.agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-201-r3-rooms-evaluator-breeder.md, tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, tickets/closed/TCK-194-trainer-tiles-show-all-origins-and-reset-reseeds.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md, tickets/closed/TCK-177-trainer-ui-progress-metrics.md, src/trainer/mod.rs, src/trainer/sim.rs, src/trainer/room/level.rs, src/trainer/chamber.rs, src/trainer/breeder.rs, src/trainer/evaluator.rs, src/server/trainer_ipc.rs, src/server/engine/trainer_mode.rs, src/biology/trainer_config.rs, frontend/views/trainer/index.ts, frontend/views/trainer/metrics.ts, frontend/render/progressChart.ts]
---

# Trainer

Neuroevolution in isolated training chambers to breed fit progenitor genomes. The trainer is an IO-free simulation
(`TrainerSim`, TCK-173) driven by the engine thread (UI) or the headless runner ([[wiki/simulation/analytics-and-balancing]]).
Rooms, evaluation and breeding are separate seams since TCK-201: [[wiki/contracts/room]],
[[wiki/contracts/evaluator]], guide [[wiki/guides/add-room-or-mode]]. Measurements of the training epic (P2–P8):
[[wiki/history/trainer-measurements-p2-p8]].

## Structure (`src/trainer/`)

| Module | Content |
| :--- | :--- |
| `sim.rs` | `TrainerSim`: orchestrates trials, generation boundary, room switches; `TrainerParams`, `TrainerEvent::GenerationCompleted { generation, best_fitness, avg_fitness, best_genome, elites, stats }`, `TrainerRunState`, `run_seed`, `PopulationEval`. API: `new`, `restore`, `restore_at_layout`, `step(parallel)`, `set_scenario`, `set_mode`, `set_level`, `room`, `pending_scenario`, `evaluate_population`, `assign_creature`, `parents`, `hof`, `set_evaluator` |
| `room/` | `Room` trait, registry `ROOMS` (`standard`, `exploration`), `LevelRoom` and its score `calculate_fitness` |
| `chamber.rs` | `Chamber` (N creatures, food, one `EntityTrial` per creature, tick, respawn seed) and its tick; `DietClass`, `target_distance_by_diet` |
| `sandbox.rs` | `TrainerSandbox` (slot id, `SandboxOrigin` elite/hof/mutant/random, fitness, room, chamber): `spawn`, `reset_trial`, `step`, `score` |
| `evaluator.rs` | `Evaluator` trait, `SoloTrials`, `aggregate_trials` |
| `breeder.rs` | `Breeder` (parents, hall of fame, slot plan, breeding, selection, lineage), `GenomeRecord`, `one_fifth_update`, `elites_solved_layout` |
| `genomes.rs` | `breed_genome(_traced)`, `random_genome`, `meets_minimal_criterion`, `draw_mutation_ops` |
| `curriculum.rs` | `CurriculumParams`, `LevelMove`, `gate_level`, `snap_level`, `metabolic_ramp`, `rehearsal_trials` |
| `mode.rs` | `Scenario` (UI mode, ts-rs) and `TrainingMode` (registry room or curriculum) |
| `stats.rs`, `streams.rs` | `GenerationStats`; named RNG stream ids |

- Engine mode `TrainerMode` (`src/server/engine/trainer_mode.rs`, TCK-156, [[wiki/contracts/mode]]) is a thin wrapper over `TrainerSim` and keeps only the UI state: running flag, warp (clamped to 35 and echoed, [[wiki/architecture/invariants]]), selected sandbox, run id, 25 Hz telemetry and DB writes; it rebuilds the population from the store only on the first entry, after a run switch or when a command changed the run's genomes. One method (`TrainerMode::state`) builds the typed `TrainerState` of `TRAINER_STATE_CHANGED` / `TRAINER_RESET_COMPLETED`; all trainer events and actions are typed in `src/server/trainer_ipc.rs` (TCK-177, [[wiki/architecture/ipc-protocol]], [[wiki/architecture/simulation-engine]]).
- Creatures come from the shared `CreatureFactory` (`src/biology/creature_factory.rs`, TCK-185), identical to Ocean
  spawns except position, heading, id and generation. Prey mocking ("meatballs") provides meat targets (TCK-106).

## Rooms and levels

The registry rooms are levels of one room, `LevelRoom` (TCK-176): `standard` = level 0, `exploration` = level 1; the
curriculum builds `LevelRoom::at(d)` for `d` in between. Today's rooms hold one creature per chamber.

| | `standard` (d = 0) | d = 0.5 | `exploration` (d = 1) | law |
| :--- | :--- | :--- | :--- | :--- |
| Chamber | 1000 px | ≈ 1871 px | 3500 px | geometric, ×1.13 per 0.1 |
| Epoch (one trial) | 300 ticks | 735 ticks | 1800 ticks | geometric, ×1.20 per 0.1 |
| Min. food distance (trial start and respawns) | 200 px | ≈ 316 px | 500 px | geometric, ×1.10 per 0.1 |
| Food | 1 plant + 1 meat | 2 + 2 | 3 plants + 3 meat | linear, rounded |
| Visible to sensors | all food | nearest plant and nearest meat | nearest plant and nearest meat | `TargetSelector` |

- Endpoints from config `trainerCurriculum.chamberSize` / `epochTicks` / `foodPerType` and `trainerSpawnMinDistance`
  ([[wiki/architecture/configuration]]); both ends exact, so the curriculum at level 0/1 reproduces the registry rooms
  (test `test_curriculum_levels_zero_and_one_reproduce_the_fixed_rooms`). The spawn distance 500 px is just below the
  median food-sensing range (reasoning in `TrainerConfig::trainer_spawn_min_distance`).
- Layout (`LevelRoom::layout`): creature in the chamber centre with a heading from the layout stream, then plants and
  meat, each at least the spawn distance from it (`spawn_at_min_distance`, at most 1000 tries, then the farthest
  candidate).
- **Room switches wait for the generation boundary:** `SET_TRAINER_MODE` (`set_scenario`), `SET_TRAINER_LEVEL`
  (`set_level`) and `set_mode` store a pending switch (`pending_scenario`), applied at the end of the generation after
  the gate; requested before the first tick of a generation it applies at once by restarting trial 0. A generation
  never mixes rooms.
- **Modes:** `TrainerParams::scenario` is the UI mode; a registry room without a UI mode is selected through
  `TrainerParams::room` / `TrainingMode::Room(id)` (runner `--scenario <id>`).

## Curriculum

Training mode `curriculum` (TCK-176): every trial runs in `LevelRoom::at(TrainerParams::level)`; the level changes only at
generation ends.

- **Gate** (`gate_level`, on while `level_auto`): success rate of the generation's non-rehearsal trials above `upper`
  (0.3) → level + `step` (0.1), below `lower` (0.1) → level − `step`; clamped to [0, 1] and snapped to 1e-6. Decision in
  `GenerationStats::level_move`. New runs start at `startLevel` (0).
- **Manual level:** `set_level(level)` = curriculum at that level with the gate off; `set_scenario(Curriculum)` turns it
  on again. A pending manual switch replaces the gate decision. `TRAINER_RESET` restarts an automatic curriculum at
  `startLevel` and keeps a manual level.
- **Rehearsal** (off: `rehearsalShare` 0): `round(trials · share)` trials (at most all but one) at `min(rehearsalLevel, d)`,
  chosen per layout from the layout stream; they count for fitness, not for the gate.
- **Metabolic ramp** (off: `metabolicRampStart` 1.0): curriculum trials drain `rampStart + (1 − rampStart) · d` times
  `trainerMetabolicScale` (`LevelRoom::with_metabolic_scale`).

## Chamber tick

One tick of a chamber (`Chamber::step`, TCK-200/201): per creature the shared creature systems of `src/sim/`
([[wiki/contracts/creature-system]]) with the room's `world_settings`, then the trainer's bookkeeping. Trainer side
of the world features (since TCK-202 the creature model is the Ocean's: `fmath` clock, eat radius
`r · grazingRadiusMultiplier + grazingRadiusOffset` from config section `metabolism`, food drift once per tick before
the creatures, death at energy ≤ 0, the water temperature as a depth field for the temperature sense):

| Setting | Trainer room | Ocean (`WorldSettings::ocean`) |
| :--- | :--- | :--- |
| `systems` | `CREATURE_SYSTEMS`: sense → think → metabolize → death → move → eat | same |
| `targets` | `All` (one food per type) / `NearestPerType` | `NearestPerTypeWithPrey` |
| `metabolic_scale` | `trainerMetabolicScale` · room factor | 1 |
| `energy_modifiers` | none (no light zone, no thermal stress) | photosynthesis, thermal stress |
| `death` | `Freeze`: frozen for the rest of the trial, no age limit (the trial resets the age and ends long before the shortest lifespan, 3,600 ticks) | `Decompose`: lifespan and meat spores |
| `respawn` | `MinDistance` (spawn distance, `trainerSpawnMargin` 25 px from the walls) | `Patchy` |

Before TCK-202 the trainer had its own clock (`fmath` vs the Ocean's std `sin`), food drift order (after each
creature's move) and eat radius (`max(r + 10, ..)`, smaller than the Ocean's for r > 12), so trainer-bred creatures
met another catch geometry in the Ocean.

Per tick: food drift (`food_drift::drift_before_creatures`), then per creature the systems and after them: age + 1
(also when starved), 8×8 coverage mask, wall counter (`trainerWallCollisionCooldown` 20 ticks), approach tracking,
then the catches (`CreatureEvent::Ate`; energy already added): yield, first-catch tick, respawn, approach restart.

- **Energy** (TCK-178): start energy `stomach_capacity · creatureStartStomachRatio`; the drain is the Ocean cost ×
  `trainerMetabolicScale` (1.0); eating refills `metabolism::food_energy`. At energy ≤ 0 the creature is `starved`:
  no sensing, brain, movement or eating for the rest of the trial ([[wiki/biology/metabolism-and-reproduction]]).
- **Trial start** (`Chamber::start_trial`): creatures at rest, start energy, age 0, neuron state cleared, learned
  synapse weights kept, trial records cleared; then the room's layout and the respawn seed from the trial's layout stream.
- **Age** increments every tick and resets per trial, so the clock input oscillates as in the Ocean.

## Evaluation

- **Shared layouts** (TCK-174): trial `t` of layout `l` uses `derive(seed, [LAYOUT, l, t])` for every sandbox (start
  heading, food, respawn seed); respawn after catch `k` draws from `derive(respawn_seed, [k])` and depends on the
  creature's position. Layout policy `trainerLayoutPolicy`: `rotate` (default) = new layout every generation; `hold` =
  until every elite caught food in every trial (`elites_solved_layout`). `layout_id` is stored with the run state.
- **Score and aggregation** (`SoloTrials`, [[wiki/contracts/evaluator]]): the room scores every trial; `trainerTrials`
  (8) trials per generation; generation fitness = `trainerTrialAggregation` (`mean`, or `median`) of the trial fitness.
  Live fitness per tick only for the UI (`track_live_fitness`), identical results.
- **Final evaluation** (`evaluate_population`): clones of the current population run trials on the streams
  `derive(seed, [EVAL, t])`, which no training generation uses; this is the capability measure since P8.

## Fitness

`LevelRoom` scores with `room::calculate_fitness(trial, epoch_ticks, trainerFitness)` in **catch units** (TCK-175):

`fitness = yield + firstCatchBonus · (1 − finish_tick / epoch_ticks) + approachBonus · approach`

- `yield`: per catch the diet fit, meat `carnivory`, plant `1 − carnivory` (≤ 1.0 per catch).
- Time bonus only with a catch, in `[0, firstCatchBonus)`; `firstCatchBonus` 0.5, `approachBonus` 0.3 (both below 1).
- `approach` in `[0, 1]`: `1 − d_min / d_sight` towards the nearest diet target from the tick it entered the
  food-sensing range; restarts per trial and after each catch; 0 if never in range.
- No penalties (walls, looping, standstill, centre distance); non-finite results count as 0.

## Selection and breeding (`Breeder`)

- **Selection:** sandboxes ranked by fitness; `clamp(round(N · eliteRatio), 1, N)` elites (0.25) become the parents and
  enter the hall of fame (bounded to `trainerGenomeRetentionPerRun`, 200). The hall of fame holds every genome once
  (its best record); before TCK-202 surviving elites were re-added every generation, so the hof slots mostly
  re-mutated copies of the current parents.
- **Slot plan:** elites (unmodified) | hall-of-fame children (`round(N · hofRate)`, mutated) | random immigrants
  (`round(N · inflowRate)`) | mutants of the parents round-robin; missing parents/HoF entries fall back to mutants of
  parent 0. Generation 1: all random.
- **Random genomes:** `generate_structured_genome` (config `genomeComposition`); with the minimal criterion
  (`trainerMinimalCriterion`, on) redrawn until it has an eye or nose and moves ≥ 5 px in 100 ticks of a fixed
  standard-room trial.
- **Mutation ops:** uniform in [`mutationOpsMin`, `mutationOpsMax`] = [1, 3] calls of `mutate_genome` per mutant and
  HoF child; the **1/5 rule** (`mutationOneFifthRule`, on) moves the max by ±1 after each generation from the share of
  mutants beating their parent's unmodified elite slot, clamped to `mutationOpsLimit` (8). UI slider "mutation" = the
  max (IPC field `mutationRate`).
- **Mutation channels** (TCK-181): each mutated child records body/brain/both/neutral; mode `mutationChannels`
  `mixed` (default) or `separate` (50/50 body or brain per child).
- **Lamarckian toggle** (off): methylations and, per plastic synapse with the same key, with probability
  `lamarckianAssimilationChance` (0.25) the parent's learned offset (TCK-152, TCK-179).
- Defaults (`TrainerParams::from_config`, same as the UI): population 100 (16 tiles shown), elite 0.25, ops 1–3, 1/5
  rule on, inflow 0.15, HoF 0.15, multi-trial on, 8 trials, mean, `rotate`, minimal criterion on, `mixed`, Lamarck off,
  mode `standard`, gate 0.3 / 0.1, step 0.1, rehearsal and ramp off.

## Determinism

- One seed per run: UI `run_seed(config.seed, run_id, nonce)`; `TRAINER_RESET` adds a clock nonce (TCK-194). The
  seed is stored with the run state. Runner: `seed_from_str(--seed)`.
- RNG streams (`src/trainer/streams.rs`): layouts `[LAYOUT, l, t]`, rehearsal choice `[LAYOUT, l, REHEARSAL_PICK]`,
  breeding `[g, BREED, s]`, final evaluation `[EVAL, t]`, respawns `derive(respawn_seed, [k])`; reserved `PAIRING`
  (evaluators that pair slots) and `BIRTHS` (rooms with births). Results do not depend on `parallel` or the thread count.
- The per-tick path uses `src/shared/fmath.rs` (`libm`), so reports are identical across platforms.

## Persistence and UI

- On `GenerationCompleted` the engine emits `TRAINER_GENERATION_COMPLETED` (stats = `TrainerGenerationStats` from
  `GenerationStats`), then `TRAINER_STATE_CHANGED`, and saves the elites and the run state
  (`trainer_store::save_elites`, `save_run_state`; [[wiki/architecture/persistence]]).
- Population vs. visible tiles (TCK-192, TCK-194): `N` tiles show a sample with every origin group
  (`display_slots`); the display count never touches the simulation.
- `SET_MODE trainer`, a new run id, a population change or the Lamarck toggle restore the population from the DB
  (active parents, HoF) and apply the stored run state (mode, level, gate, layout, max ops, population, seed).
- **`ASSIGN_SANDBOX_CREATURE`** (`assign_creature`, TCK-201): the creature is rebuilt through the factory and the
  sandbox restarts the current trial like a trial reset on the trial's shared layout (start pose, energy, food, trial
  record), keeping the generation's trial clock; the slot's earlier trial fitness of the generation and its lineage are
  cleared, so it leaves the 1/5 rule and the channel statistics. Stored as an active row with fitness 0.
- Controls: start/pause/reset, warp (≤ 35), hyperparameters, training mode select and level slider, sandbox
  selection/assignment, metrics (`#stat-success`, `#stat-median-fit`, `#stat-first-catch`, `#stat-level`) and the
  progress curve (`frontend/views/trainer/metrics.ts`, `frontend/render/progressChart.ts`; [[wiki/frontend/structure]]).

## Key Takeaways

- `TrainerSim` is IO-free, seeded and parallel-safe and only orchestrates: rooms (`Room` registry), evaluation
  (`SoloTrials`), breeding (`Breeder`) and the curriculum are separate modules with contracts (TCK-201).
- Registry rooms `standard` and `exploration` are levels 0 and 1 of `LevelRoom`; the curriculum moves the level by a
  success gate at generation ends; switches wait for the generation boundary.
- Every sandbox of a generation runs the same shared layout, a new one every generation (`rotate`), 8 trials, fitness
  in catch units averaged over the trials.
- Capability is read on the final population on unseen layouts; the measured history is in
  [[wiki/history/trainer-measurements-p2-p8]].

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-105, TCK-106, TCK-120, TCK-128, TCK-150, TCK-151, TCK-152,
TCK-154, TCK-173, TCK-174, TCK-175, TCK-176, TCK-177, TCK-178, TCK-179, TCK-181, TCK-183, TCK-184, TCK-185, TCK-194,
TCK-201, `src/trainer/`, `src/server/trainer_ipc.rs`, `src/biology/trainer_config.rs`, `frontend/views/trainer/`.
