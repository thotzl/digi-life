---
sources: [.agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, tickets/closed/TCK-136-trainer-sandbox-epoch-ticks.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/biology/scenarios.rs, src/biology/trainer_sim.rs, src/biology/trainer_engine.rs, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, src/biology/metabolism.rs, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, src/shared/types.rs, src/biology/dna.rs]
---

# Trainer

Neuroevolution in isolated sandboxes (training chambers) to breed fit progenitor genomes. Since TCK-173 the trainer is an IO-free simulation (`TrainerSim`) driven either by the engine thread (UI) or by the headless runner ([[wiki/simulation/analytics-and-balancing]]).

## Structure

- `src/biology/trainer_sim.rs` — `TrainerSim`: population of sandboxes, epoch/trial loop, breeding, parents and hall of fame; no IO. API: `new` (generation 1), `restore` (continue from parents + HoF), `step(parallel) -> Option<TrainerEvent>`, `set_scenario`, `assign_creature`. Types `TrainerParams`, `GenomeRecord`, `GenerationStats`, `TrainerEvent::GenerationCompleted { generation, best_fitness, avg_fitness, best_genome, elites, stats }` (TCK-173).
- `src/biology/trainer_engine.rs` — `TrainerSandbox` (one creature + its food), `SandboxOrigin` enum (`elite`/`hof`/`mutant`/`random`, ts-rs), `breed_genome`, `init_rust_sandbox`, `TrainerSandbox::reset_trial`, `step_trainer_sandbox_physics` (eating and energy refill, respawn, wall counter, 8×8 coverage mask; nothing after starvation), `TrainerTelemetrySandbox`.
- `src/biology/scenarios.rs` — `Scenario` enum (`standard`/`exploration`, ts-rs), `DietClass` (`CARNIVORE_MIN` 0.6, `HERBIVORE_MAX` 0.4), `TrainingScenarioPlugin` trait (`chamber_size`, `epoch_ticks`, `spawn_min_distance`, `initialize`, `visible_foods`) returned as `&'static dyn` by `Scenario::plugin()`; one shared `step_physics` (sensing, brain, metabolism, kinematics, food drift) with the `visible_foods` hook; one shared `calculate_fitness` for both rooms (TCK-175, section Fitness); helpers `spawn_at_min_distance` and `target_distance_by_diet` (TCK-173).
- Engine (`src/server/engine.rs`) keeps only the UI state: active/running flags, warp (cap 35, [[wiki/architecture/invariants]]), selected sandbox, run id, 25 Hz telemetry and DB writes. One function builds the `TRAINER_STATE_CHANGED` / `TRAINER_RESET_COMPLETED` payload ([[wiki/architecture/simulation-engine]]).
- Creatures are spawned by the shared `CreatureFactory` (`src/biology/creature_factory.rs`, TCK-185), identical to Ocean spawns except position, heading, id and generation. Each sandbox holds exactly one creature; prey mocking ("meatballs") provides meat targets (TCK-106).

## Rooms

| | `standard` | `exploration` |
| :--- | :--- | :--- |
| Chamber | 1000 px | 3500 px |
| Epoch (one trial) | 300 ticks | 1800 ticks |
| Food | 1 plant + 1 meat | 3 plants + 3 meat |
| Visible to sensors | all food | nearest plant and nearest meat only |
| Min. food distance (trial 0, re-trials and respawns after eating) | 200 px | 500 px |

- All values come from the scenario plugin; the engine has no chamber/epoch literals (TCK-173, RV-A05 trainer part). The spawn distance is one `spawn_min_distance()` per plugin, read from config `trainerSpawnMinDistance` (`standard` 200, `exploration` 500); re-trials call `plugin.initialize` like trial 0 (`TrainerSandbox::reset_trial`), so trial 0, re-trials and respawns use the same distance (RV-S08 resolved, TCK-154, P7; before: large room 650 px trial 0, 1200 px trials 1+, 550 px respawn).
- Spawns use `spawn_at_min_distance` with at most 1000 tries (`SPAWN_MAX_TRIES`), then the farthest candidate (25 px margin) — the unbounded rejection loops are gone (TCK-173).
- Distance choice (TCK-154 sweep, 2000 structured genomes, food-sensing range p10/p50/p90/max = 256/575/926/990 px; reasoning in the doc comment of `TrainerSpawnMinDistance`, `src/shared/types.rs`): share of genomes with food in sensor range at spawn in the large room 46 % (300 px), **35 % (500 px, chosen**; 13 % also in the view cone), 23 % (650 px, old trial 0), 10 % (800 px), 0 % (1200 px, old trials 1+). 500 px is just below the median sensor range. Small room 200 px (farthest point ≈ 670 px from the centre): 89 %.
- `SET_TRAINER_MODE` switches the room of the running population (`TrainerSim::set_scenario`): every sandbox restarts its trial in the new chamber; genomes, the epoch tick counter and the trial accumulators are kept.
- Sensor range `scale · 550` (`food_sense_range`: max over eye/nose/thermal organs), measured 256 / 575 / 926 px (p10/p50/p90); spawn-time food signals per genome set in [[wiki/biology/sensing]] (TCK-172 probe; the large-room numbers there predate the 500 px distance).

## Start state and energy

- Start energy = `stomach_capacity · creatureStartStomachRatio` (0.60), set by the factory and again by every trial reset (TCK-185; before: fixed 100). The brain input `internal_energy = energy / 100` therefore differs per body instead of being 1.0.
- **Energy budget** (TCK-178, P6): after the brain step every tick, `metabolism::drain_energy` subtracts `trainerMetabolicScale` (1.0) × the Ocean cost (body, organs, neurons, synaptic activity, thrust; [[wiki/biology/metabolism-and-reproduction]]). Test `test_trainer_drain_equals_the_ocean_energy_function`: trainer and Ocean path give a bit-identical drain over 20 ticks.
- Eating refills `metabolism::food_energy` (the Ocean rule, `foodSporeEnergy` · diet efficiency), capped at `stomach_capacity`.
- **Starvation:** at energy ≤ 0 the sandbox is `starved`: energy and velocity are set to 0, and for the rest of the trial there is no sensing, brain step, movement or eating (the epoch tick counter keeps running). The trial counts as failed unless a catch came before; energy is not a fitness term. `starved` is cleared on every trial reset and new generation.
- `trainerMetabolicScale` scales only the trainer drain (the Ocean uses 1); it exists for the curriculum pressure ramp of TCK-176 and stays 1.0 because the budget did not collapse sensor equipment (below).
- `reset_trial(rng)`: agent to the centre, velocity/bend reset, age 0, start energy, `starved` cleared, neuron states cleared; heading, food positions (`plugin.initialize`, `spawn_min_distance`) and respawn seed drawn in this order from the trial's shared layout stream (section Evaluation); learned synapse weights are kept.
- **Age** (TCK-174): `agent.age` increments every tick (`step_trainer_sandbox_physics`) and resets per trial, so the brain's clock input `0.5 + 0.5 · sin(age · 0.1)` (`scenarios::step_physics`) oscillates as in the Ocean; before P7 the trainer age stayed 0 and the clock input was constant 0.5.

## Generation loop

- `step` advances every sandbox one tick; `parallel` steps them on the rayon pool. The UI engine calls `step(true)`.
- Fitness is computed per tick only for the UI telemetry (`track_live_fitness`); the runner computes it at epoch end only, with identical results (TCK-173; test `test_live_telemetry_fitness_equals_epoch_end_fitness`).
- `GenerationStats` are computed at generation end over the final fitness, before breeding ([[wiki/simulation/analytics-and-balancing]] for the fields).

### Evaluation

- **Shared layouts** (TCK-174, module doc of `src/biology/trainer_sim.rs`): every sandbox of a generation faces the same task. Trial `t` of layout `layout_id` uses one RNG stream `derive(seed, [LAYOUT_STREAM, layout_id, t])` (`layout_rng`), replayed for every sandbox; the sandbox draws from it, in this order, its start heading, its food positions and its respawn seed. Shared: start position (always the chamber centre), start heading, initial food positions, respawn seed. Before P7 every sandbox rolled its own layout.
- **Respawns:** after the k-th catch of a trial the candidate points come from `derive(respawn_seed, [k])`; the accepted point also depends on the creature's position at the catch (minimum distance), so two candidates get the same respawn only when they make their k-th catch at the same position. Not shared: food drift caused by the creature pushing it.
- **Layout advance:** the layout is kept across generations and advances (`layout_id + 1`) once every elite of a generation caught food in every trial (`elites_solved_layout`). `layout_id` is not persisted: a restored run starts at layout 0.
- **Trials and aggregation** (multi-trial on by default): `trainerTrials` (3) trials per generation; the generation fitness of a sandbox is `aggregate_trials` over its trial fitness, `trainerTrialAggregation` `mean` (default) or `median` (mean of the two middle values for an even count). No gate and no bonus per successful trial (TCK-175; before: × 0.1 without any catch, + 150 per successful trial).
- `avg_fitness` and `elite_count` divide by the actual sandbox count (test `test_avg_fitness_divides_by_the_sandbox_count`).

### Selection and breeding

- **Selection:** sandboxes sorted by fitness; `elite_count = clamp(round(N · eliteRatio), 1, N)` elites (`eliteRatio` 0.25, TCK-174; before 0.15) become the parents and enter the hall of fame (sorted by fitness, bounded to `trainerGenomeRetentionPerRun`, 200).
- **Slot plan** of the next generation: elites (unmodified) | hall-of-fame children (`round(N · hofRate)`, mutated like mutants since TCK-174; before: unmodified copies) | random immigrants (`round(N · inflowRate)`) | mutants of the parents round-robin. Missing parents/HoF entries fall back to mutants of parent 0.
- **Random genomes** (generation 1, where every slot is random, and immigrants): `trainer_engine::random_genome` → `generate_structured_genome` with config `genomeComposition` (TCK-184): every genome has all body genes, an `OUT` gene and 1–3 organs; 99 % have a food-sensing organ, 31 % a `HOX` pair ([[wiki/biology/genome-compiler]]).
- **Minimal criterion** (RV-S09 resolved, TCK-175, P7; config `trainerMinimalCriterion`, on): `random_genome` uses the same generator with `pFoodOrgan` = 1.0 and redraws (≤ `maxRedraws` 50, else the last candidate is kept) until `meets_minimal_criterion`: at least one eye or nose, and the creature's own brain moves it ≥ `mobilityMinDistance` (5 px) within `mobilityTicks` (100) in a fixed standard-room trial (`MOBILITY_CHECK_SEED`). All draws come from the slot's breed stream, so it is deterministic. The trainer has no `base_dna` parameter any more (TCK-154).
- **Mutation ops** (TCK-174): every mutant and HoF child gets `draw_mutation_ops` calls of `mutate_genome`, uniform in [`mutationOpsMin`, `mutationOpsMax`] = [1, 3]; each call draws creep, point, insertion, deletion, gene duplication or gene deletion by config `mutationOperators` (TCK-184). Replaces `round(len · mutation_rate)` (0.06 → ≈ 15 at 256 bases).
- **1/5 success rule** (`mutationOneFifthRule`, on; `one_fifth_update`): after each generation, if more than 1/5 of the mutants beat their parent (`child_success_share`: higher fitness than the parent's unmodified elite slot of the same generation) `mutation_ops_max + 1`, below 1/5 `− 1`, exactly 1/5 or no mutants unchanged; clamped to [max(1, `mutationOpsMin`), `mutationOpsLimit`] (8).
- **UI slider** "mutation": the maximum ops per child (range max(1, `mutationOpsMin`) .. `mutationOpsLimit`, default 3, label `1–k`); the IPC field keeps its old name `mutationRate` (`UPDATE_TRAINER_HYPERPARAMS`, state payloads). The frontend sends it only when the slider itself changed (`pushHyperparamsToRust(includeMutation)`), so other sliders do not overwrite the adapted value; the adapted value arrives with every `TRAINER_GENERATION_COMPLETED` (`mutationOpsMax`).
- **Mutation channels** (TCK-181): every mutated child records the gene channels its operator calls touched (`dna::classify_mutation`: body / brain / both / neutral; [[wiki/biology/genome-compiler]]); elites carry the channel of the last mutation in their lineage. Mode `mutationChannels`: `mixed` (default, every call may hit any gene) or `separate` (each child draws body or brain 50/50, `SEPARATE_BODY_SHARE`, from its breed stream and mutates only genes of that channel, `dna::mutate_genome_in_channel`). Statistics in the report (`mutation_channels`, `elite_last_mutation`).
- **Lamarckian toggle** (default off, TCK-128): elites and mutants receive the source parent's methylations and, per plastic synapse with the same key in the (possibly mutated) child genome, with probability `lamarckianAssimilationChance` (0.25) the parent's full learned offset, else the genome weight (`inherit_learned_weights`, TCK-179, TCK-152). The draws come from the slot's seeded breed stream, so runs stay deterministic. An unmutated elite also inherits only ~25 % of its parent's learned weights; fixed reflexes and new synapses start at their genome weight. Same rule as Ocean mitosis ([[wiki/biology/metabolism-and-reproduction]]).
- Defaults (`TrainerParams::default`, same as the UI; trials, aggregation, ops, 1/5 rule, minimal criterion and channels from `config.json`): population 16, elite 0.25, mutation ops 1–3, 1/5 rule on, inflow 0.15, HoF 0.15, multi-trial on, 3 trials, mean, minimal criterion on, channels `mixed`, Lamarck off.
- Fixed while extracting (TCK-173): Lamarck source index underflow for fallback mutants; Lamarck mutants were built from the unmutated parent genome; catalogue assignment kept the old `antisense` and neuron buffers.
- `trainer_genomes` rows from an older model version are dropped by the model-version reset (TCK-183, `MODEL_VERSION` 5 since TCK-175: stored fitness of the old formula, ~10²–10³, would outrank every value in catch units in HoF, retention and run list, [[wiki/architecture/persistence]]).

## Determinism

- One seed per run: `seed_from_str("{config.seed}:{run_id}")` in the UI, `seed_from_str(--seed)` in the runner ([[wiki/architecture/simulation-engine]]).
- RNG streams (`src/shared/rng.rs`): trial `t` of layout `l` uses `derive(seed, [LAYOUT_STREAM, l, t])` for every sandbox (TCK-174; before: one stream per sandbox `derive(seed, [g, t, i])`); respawns `derive(respawn_seed, [k])`; breeding slot `s` uses `derive(seed, [g, BREED_STREAM, s])`. Results do not depend on `parallel` or the thread count (tests in [[wiki/quality/testing]]).
- The per-tick path uses `src/shared/fmath.rs` (pure-Rust `libm`) for transcendental functions, so reports are identical across platforms.

## Persistence

- On `GenerationCompleted` the engine emits `TRAINER_GENERATION_COMPLETED` (`generation`, `bestFitness`, `avgFitness`, `bestGenome`, `mutationOpsMax`) and calls `save_trainer_elites`: previous rows of the run become inactive, the elites are inserted as active, then retention prunes the run ([[wiki/architecture/persistence]], TCK-183).
- `SET_MODE trainer`, a new `runId`, a population change or the Lamarck toggle restore the population from the DB (`load_trainer_genomes`: active parents up to N, HoF up to the retention); the generation continues from the newest parent. `TRAINER_RESET` deletes the run's rows and starts a fresh `TrainerSim`.
- `ASSIGN_SANDBOX_CREATURE` rebuilds the agent through the factory (position, heading and motion kept, trial counters restart) and stores it as an active row with fitness 0.

## Fitness

One function for both rooms, the UI telemetry and the epoch end (UI, runner, `probes::evaluate_genome`): `scenarios::calculate_fitness(sb, epoch_ticks, &config.trainer_fitness)`, in **catch units** (1.0 = one catch at full diet fit, same scale in both rooms; TCK-175):

`fitness = yield + firstCatchBonus · (1 − finish_tick / epoch_ticks) + approachBonus · approach`

- `yield` (primary): sum over the trial's catches of the diet fit, meat `carnivory`, plant `1 − carnivory` (≤ 1.0 per catch).
- Time to first catch (secondary): only with a catch; `finish_tick` in `1..=epoch_ticks`, so the bonus lies in `[0, firstCatchBonus)`.
- `approach` (shaping) in `[0, 1]`: `1 − d_min / d_sight` towards the nearest diet target (`target_distance_by_diet`), where `d_sight` is the distance at the tick that target first came within the creature's food-sensing range (`food_sense_range`, max of `scale · 550` over eye/nose/thermal organs) and `d_min` the smallest distance since; restarts per trial and after each catch (`reset_approach`); 0 if no target was ever in range.
- Config `trainerFitness`: `firstCatchBonus` 0.5, `approachBonus` 0.3; both stay below 1 (test `test_configured_bonuses_stay_below_one_catch`), so neither bonus is worth a full catch. A wrong-diet catch (yield < 0.4) can score below a no-catch trial with full approach (max 0.3); matching catches always win.
- No penalties: wall contact, looping, standstill, path efficiency and distance from the centre do not enter (the wall counter and coverage mask are still recorded); starvation ends the trial's catches (TCK-178) and is no separate term. Non-finite results count as 0.
- Removed in P7: the per-plugin `calculate_fitness` with `yield · 1000` / `yield · 3000`, path efficiency, speed and coverage bonus, wall/looping/standstill multipliers, centre-distance and homing terms, and the trial gate (× 0.1 / + 150). Fitness values before P7 (~10²–10³) are not comparable with catch units (`MODEL_VERSION` 5).
- `sb.epoch_ticks` increments every tick (TCK-136), so `finish_tick` records the real consumption tick.
- Frontend shows fitness with 2 decimals (trainer, catalogue, Ocean, diagnostics).

## Baseline (P2, TCK-173)

Headless runner, release build, `--parallel true`, current settings; mean of the last 5 generations (reports kept locally in `.agents/artifacts/baseline/p2/`).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.004 | 0.040 | 0.125 | ≈ 190 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.023 | 0.083 | ≈ 1210 ticks |
| small, pop 300, 20 gens, 1 seed | 0.004 | 0.165 | 0.179 | ≈ 216 ticks |
| large, pop 300, 10 gens, 1 seed | 0.006 | 0.054 | 0.064 | ≈ 880 ticks |

- Success rate = share of sandbox-trials with at least one catch. Immobile share (< 5 px travelled) in the small room: 0.108 in generation 1, 0.032 at the end.
- Throughput: standard pop 300 ≈ 206k sandbox-ticks/s total, exploration pop 16 ≈ 469k ([[wiki/architecture/performance]]).
- Mechanics limits measured by the TCK-172 probes (heredity, speed, steering, walls) are in [[wiki/biology/genome-compiler]], [[wiki/simulation/physics]], [[wiki/biology/ctrnn-brain]] and [[wiki/biology/sensing]].

## After Brain v2 (P3)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations), reports kept locally in `.agents/artifacts/p3/after-C/` ([[wiki/decisions/adr-002-brain-v2]]).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.008 | 0.122 | 0.292 | ≈ 200 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.033 | 0.125 | ≈ 860 ticks |

- Small room ×3 over the P2 baseline, large room ×1.4; both stay below the TCK-171 targets. Immobile share in the small room: 0.067 in generation 1, 0.009 at the end.
- Throughput at pop 16 roughly doubled with the compact brain (seed b1, same machine: small 158k → 323k, large 425k → 763k sandbox-ticks/s after slice A).

## After Oja learning (P4)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations, Lamarck off), reports kept locally in `.agents/artifacts/p4/` (TCK-152, [[wiki/biology/ctrnn-brain]]).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.008 | 0.207 (P3: 0.122) | 0.458 | ≈ 180 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.046 (P3: 0.033) | 0.125 | ≈ 890 ticks |

- The gain comes with bounded learning and fixed reflexes; the Hebbian probe still shows no measurable benefit of lifetime learning per genome, so the effect is not attributed to better within-life learning.
- Variants measured and rejected: no pruning (last 5 gens small 0.166, large 0.026); pruning only with active presynaptic input (no measurable change). Kept: Oja + pruning.
- Immobile share in the small room: 0.067 in generation 1, 0.008 at the end (last 5 gens).

## After Locomotion v2 (P5)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations, Lamarck off), reports kept locally in `.agents/artifacts/p5/` (TCK-150, TCK-180, [[wiki/simulation/physics]]).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.013 | 0.218 (P4: 0.207) | 0.396 | ≈ 190 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.033 (P4: 0.046) | 0.083 | ≈ 1040 ticks |

- Both changes are within the 5-seed noise (standard error ≈ 0.05 small, ≈ 0.01–0.02 large); on 10 seeds (b1–b10) small 0.150 → 0.193, large 0.027 → 0.060. Best fitness in the large room (mean of the last 5 generations) 934 → 1427.
- Calibration of `locomotionSpeedMin` / `locomotionSpeedMax` / `turnRateAtRest`, time-boxed to two adjustments (5-seed success small / large): 0.8 / 3.0 / 0.013 → 0.095 / 0.023; 1.0 / 4.0 / 0.008 → 0.168 / 0.055; 1.0 / 5.0 / 0.008 → 0.218 / 0.033, chosen for the best 10-seed total.
- Immobile share in the small room: 0.004 in generation 1, 0 at the end (P4: 0.067 / 0.008).
- Throughput standard pop 300, 10 generations, seed b1: 857k → 882k sandbox-ticks/s.

## After Energy v2 (P6)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations, Lamarck off), before = P5 code, after = final calibration (typical total cost = old Ocean drain); reports kept locally in `.agents/artifacts/p6/before/` and `.agents/artifacts/p6/after2/` (TCK-151, TCK-178).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.013 | 0.223 (P5: 0.218) | 0.396 | ≈ 189 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.050 (P5: 0.033) | 0.271 | ≈ 1007 ticks |

| Energy metrics, last 5 gens (P5 → P6) | small | large |
| :--- | :--- | :--- |
| `starved_share` | 0 → 0 | 0 → 0.182 (gen 1: 0.10) |
| `mean_end_energy` | 161.7 → 139.0 | 144.0 → 71.3 |
| `food_organ_share` (eye or nose) | 0.907 → 0.958 | 0.812 → 0.917 |
| `mean_organ_count` | 2.96 → 3.41 | 1.85 → 3.24 |
| best fitness | 1204 → 1318 | 1427 → 1582 |

- The 300-tick small room does not exhaust the budget; in the 1800-tick large room about a fifth of the trials end starved. Sensor equipment did not collapse under the cost, it grew (TCK-178 acceptance).
- Both success changes are within or near the 5-seed noise; the large room per seed 0.021 / 0.138 / 0.054 / 0.021 / 0.017, the gain carried mainly by seed b2.
- Throughput standard pop 300, 10 generations, seed b1: ≈ 1064k → 983k sandbox-ticks/s (−8 %; [[wiki/architecture/performance]]).
- Report fields added: `mean_end_energy`, `starved_share`, `mean_organ_count`, `food_organ_share` ([[wiki/simulation/analytics-and-balancing]]).
- Best-fitness values of P5/P6 are on the old fitness scale (before catch units, P7).

## After evaluation and fitness (P7)

Protocol: pop 16, 30 generations, mean success of the last 5 generations, SD between seeds in parentheses; raw data kept locally in `.agents/artifacts/p7/` (TCK-174, TCK-175, TCK-154, TCK-181).

**Evaluation and selection** (TCK-174, seeds b1–b5, old fitness):

| Variant | small | large | runtime |
| :--- | :--- | :--- | :--- |
| P6 state | 0.223 (0.104) | 0.050 (0.051) | — |
| 3 trials, mean | 0.343 (0.106) | 0.194 (0.020) | 1× |
| 3 trials, median | 0.386 (0.105) | 0.133 (0.077) | 1× |
| 5 trials, mean / median | 0.383 / 0.449 | 0.116 / 0.137 | ~1.7× |
| 8 trials, mean | 0.473 (0.188) | 0.077 (0.013) | ~2.7× |
| 3 trials, mean, 1/5 rule | 0.443 (0.076) | 0.220 (0.024) | 1× |

- Chosen: 3 trials, mean, 1–3 ops, 1/5 rule on, elite ratio 0.25. On 10 seeds the 1/5 rule gives 0.444 / 0.260 vs. 0.338 / 0.193 without (≈ 2.4 / 2.1 standard errors). More trials help the small room only within noise and lower large-room success. Between-seed CV drops from 0.47 to 0.22 (small) and 1.02 to 0.38 (large).
- Population size (b1–b3, chosen settings): 16 → 0.444 / 0.260, 100 → 0.333 / 0.300, 300 → 0.521 / 0.297 at ~7.5× wall time (8.2 s vs. 1.1 s per 30 gens; 883k sandbox-ticks/s at pop 300). Protocol and UI default stay at 16 (each UI sandbox is a visible tile); pop 300 is the option for headless curriculum training (P8, TCK-176).

**Fitness and minimal criterion** (TCK-175, seeds b1–b20):

| Variant | small | large | time to first catch large / small (ticks) |
| :--- | :--- | :--- | :--- |
| TCK-174 state (old fitness, 1/5 rule) | 0.432 (0.132) | 0.229 (0.107) | 944 / 219 |
| catch-unit fitness | 0.425 (0.163) | 0.264 (0.144) | 662 / 192 |
| catch-unit fitness + minimal criterion (chosen) | 0.434 (0.123) | 0.306 (0.109) | 722 / 178 |

- Large room +0.077 (≈ 2.3 standard errors), time to first catch −222 ticks; small room unchanged within noise. Large-room target (≥ 0.30) reached at pop 16; the small-room target (≥ 0.50) is left to the curriculum (TCK-176). Escalations (novelty, Pareto, connection cost) not needed so far.

**Mutation channels** (TCK-181, `mixed`, seeds b1–b10; share that beat the parent / reached the elite):

| Room | body only | brain only | both |
| :--- | :--- | :--- | :--- |
| small | 4.9 % / 11.4 % (n 1346) | 11.3 % / 11.5 % (n 451) | 15.5 % / 14.4 % (n 464) |
| large | 3.9 % / 10.1 % (n 1289) | 7.4 % / 9.0 % (n 458) | 17.3 % / 13.2 % (n 515) |

- Body-only mutants beat their parent less often but reach the elite as often as brain-only mutants; the most frequent last mutation of elites is a body mutation (small 672 of 1200, large 612 of 1200).
- `separate` vs. `mixed` (10 seeds): small 0.415 vs. 0.438 (−0.4 SE), large 0.335 vs. 0.283 (+1.2 SE), no clear benefit.
- **Decision (TCK-181, closed as not needed):** no body/brain decoupling; `mixed` stays the default, the instrumentation and the `separate` option are kept for the active body traits planned after the epic. The body share of `separate` is fixed at 50/50 (`SEPARATE_BODY_SHARE`, not configurable). `mixed` reproduces the pre-instrumentation results exactly.

## Controls

- Start/pause/reset, warp (capped 35), hyperparameters, sandbox selection/assignment, Lamarckian synapse inheritance toggle (TCK-128), `apply_champion`, add catalogue creatures to training.

## Key Takeaways

- `TrainerSim` is IO-free, seeded and parallel-safe; the engine and the headless runner drive the same code.
- Two plugin rooms with one spawn distance each (200 / 500 px, RV-S08 resolved); every sandbox of a generation runs the same shared layout, which advances once all elites solve it (TCK-174).
- Fitness in catch units, one function for both rooms: yield + time bonus (< 0.5) + approach bonus (< 0.3), generation fitness = mean over 3 trials, no gate or penalties (TCK-175).
- Elite ratio 0.25, 1–3 mutation ops per child adapted by the 1/5 rule, HoF children mutated; generation 1 and immigrants pass a minimal criterion (eye or nose and mobility, RV-S09 resolved).
- P2 baseline: success rate 0.040 (small) and 0.023 (large) at population 16 after 30 generations; after Brain v2 0.122 and 0.033; after Oja learning (P4) 0.207 and 0.046; after Locomotion v2 (P5) 0.218 and 0.033 (10 seeds: 0.193 and 0.060); after Energy v2 (P6) 0.223 and 0.050; after P7 0.434 and 0.306 (20 seeds).
- Trainer creatures pay the Ocean energy cost and can starve (TCK-178); about a fifth of the large-room trials end starved (0.182, P6), sensor equipment did not shrink.
- Body and brain genes mutate together (`mixed`); separating them showed no clear benefit (TCK-181). Lamarckian weights follow synapse keys and are inherited per plastic synapse with probability 0.25 (TCK-152).

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-105, TCK-106, TCK-120, TCK-128, TCK-136, TCK-150, TCK-151, TCK-152, TCK-154, TCK-174, TCK-175, TCK-178, TCK-173, TCK-179, TCK-180, TCK-181, TCK-183, TCK-184, TCK-185, `docs/revisit/04-simulation-model.md`, `src/biology/scenarios.rs`, `src/biology/trainer_sim.rs`, `src/biology/trainer_engine.rs`, `src/shared/types.rs`.
