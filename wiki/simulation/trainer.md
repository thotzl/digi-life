---
sources: [.agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-201-r3-rooms-evaluator-breeder.md, tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, tickets/closed/TCK-194-trainer-tiles-show-all-origins-and-reset-reseeds.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md, tickets/closed/TCK-177-trainer-ui-progress-metrics.md, src/trainer/mod.rs, src/trainer/sim.rs, src/trainer/room/level.rs, src/trainer/chamber.rs, src/trainer/breeder.rs, src/trainer/evaluator.rs, src/server/trainer_ipc.rs, src/server/engine/trainer_mode.rs, src/biology/trainer_config.rs, frontend/views/trainer/index.ts, frontend/views/trainer/metrics.ts, frontend/render/progressChart.ts]
---

# Trainer

Neuroevolution in isolated training chambers to breed fit progenitor genomes. The trainer is an IO-free simulation
(`TrainerSim`, TCK-173) driven by the engine thread (UI) or the headless runner ([[wiki/simulation/analytics-and-balancing]]).
Rooms, evaluation and breeding are separate seams since TCK-201: [[wiki/contracts/room]],
[[wiki/contracts/evaluator]], guide [[wiki/guides/add-room-or-mode]]. Measurements of the training epic (P2–P8):
[[wiki/history/trainer-measurements-p2-p8]]; replaced rules: [[wiki/history/replaced-models]].

## Structure

`src/trainer/` (module list: [[wiki/architecture/module-map]]): `TrainerSim` (`sim.rs`) only orchestrates trials, the
generation boundary, room switches and run state; rooms (`room/`), the chamber tick (`chamber.rs`), evaluation
(`evaluator.rs`), breeding (`breeder.rs`) and the curriculum (`curriculum.rs`) are separate modules.

- Engine mode `TrainerMode` ([[wiki/contracts/mode]]) is a thin wrapper that keeps only UI state (running flag,
  clamped warp, selected sandbox, run id, 25 Hz telemetry) and the store writes; its events and actions are typed in
  `src/server/trainer_ipc.rs` ([[wiki/architecture/ipc-protocol]]).
- Creatures come from the shared `CreatureFactory` (TCK-185), identical to Ocean spawns except position, heading, id
  and generation. Prey mocking ("meatballs") provides meat targets (TCK-106).

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

One tick (`Chamber::step`): food drift once, then per creature the shared systems `CREATURE_SYSTEMS` with the room's
`world_settings` ([[wiki/contracts/creature-system]]; the creature model is the Ocean's), then the bookkeeping: age + 1
(also when starved), 8×8 coverage mask, wall counter (`trainerWallCollisionCooldown` 20 ticks), approach tracking,
then the catches (`CreatureEvent::Ate`): yield, first-catch tick, respawn, approach restart.

- World features of a trainer room vs. the Ocean: targets `All` (one food per type) / `NearestPerType` (Ocean
  `NearestPerTypeWithPrey`); metabolic scale `trainerMetabolicScale` · room factor (Ocean 1); no energy modifiers (no
  light zone, no thermal stress); death `Freeze` (frozen for the rest of the trial, no age limit; Ocean `Decompose`);
  respawn `MinDistance` with `trainerSpawnMargin` 25 px from the walls (Ocean `Patchy`).
- **Energy** (TCK-178): start energy `stomach_capacity · creatureStartStomachRatio`; drain = Ocean cost ×
  `trainerMetabolicScale` (1.0); at energy ≤ 0 the creature is `starved` for the rest of the trial.
- **Trial start** (`Chamber::start_trial`): creatures at rest, start energy, age 0, neuron state cleared, learned
  weights kept, trial records cleared; then the room's layout and the respawn seed from the trial's layout stream.

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
  (its best record).
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
- Defaults: `TrainerParams::from_config` (population 100, 16 tiles shown; the rest as above).

## Determinism

- One seed per run: UI `run_seed(config.seed, run_id, nonce)`; `TRAINER_RESET` adds a clock nonce (TCK-194). The
  seed is stored with the run state. Runner: `seed_from_str(--seed)`.
- RNG streams (`src/trainer/streams.rs`): layouts `[LAYOUT, l, t]`, rehearsal choice `[LAYOUT, l, REHEARSAL_PICK]`,
  breeding `[g, BREED, s]`, final evaluation `[EVAL, t]`, respawns `derive(respawn_seed, [k])`; reserved `PAIRING`
  (evaluators that pair slots) and `BIRTHS` (rooms with births). Results do not depend on `parallel` or the thread count.

## Persistence and UI

- Generation end: `TRAINER_GENERATION_COMPLETED` (`TrainerGenerationStats`), `TRAINER_STATE_CHANGED`, then elites and
  run state saved (`trainer_store`, [[wiki/architecture/persistence]]).
- `SET_MODE trainer`, a new run id, a population change or the Lamarck toggle restore the population from the DB
  (active parents, HoF) and the stored run state (mode, level, gate, layout, max ops, population, seed).
- `N` visible tiles show a sample with every origin group (`display_slots`, TCK-192/194); it never touches the run.
- **`ASSIGN_SANDBOX_CREATURE`** (TCK-201): the creature is rebuilt through the factory and restarts the current trial
  on the shared layout, keeping the trial clock; the slot's earlier trial fitness and lineage are cleared (it leaves
  the 1/5 rule and channel statistics); stored as an active row with fitness 0.
- UI controls, metrics and progress curve: `frontend/views/trainer/` ([[wiki/frontend/structure]]).

## Key Takeaways

- `TrainerSim` is IO-free, seeded and parallel-safe and only orchestrates rooms, evaluator, breeder and curriculum.
- `standard` and `exploration` are levels 0 and 1 of `LevelRoom`; the curriculum moves the level by a success gate.
- Every sandbox of a generation runs the same shared layout, 8 trials, fitness in catch units.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-105, TCK-106, TCK-120, TCK-128, TCK-150, TCK-151, TCK-152,
TCK-154, TCK-173, TCK-174, TCK-175, TCK-176, TCK-177, TCK-178, TCK-179, TCK-181, TCK-183, TCK-184, TCK-185, TCK-194,
TCK-201, `src/trainer/`, `src/server/trainer_ipc.rs`, `src/biology/trainer_config.rs`, `frontend/views/trainer/`.
