---
sources: [.agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, tickets/closed/TCK-136-trainer-sandbox-epoch-ticks.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/biology/scenarios.rs, src/biology/trainer_sim.rs, src/biology/trainer_engine.rs, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md]
---

# Trainer

Neuroevolution in isolated sandboxes (training chambers) to breed fit progenitor genomes. Since TCK-173 the trainer is an IO-free simulation (`TrainerSim`) driven either by the engine thread (UI) or by the headless runner ([[wiki/simulation/analytics-and-balancing]]).

## Structure

- `src/biology/trainer_sim.rs` — `TrainerSim`: population of sandboxes, epoch/trial loop, breeding, parents and hall of fame; no IO. API: `new` (generation 1), `restore` (continue from parents + HoF), `step(parallel) -> Option<TrainerEvent>`, `set_scenario`, `assign_creature`. Types `TrainerParams`, `GenomeRecord`, `GenerationStats`, `TrainerEvent::GenerationCompleted { generation, best_fitness, avg_fitness, best_genome, elites, stats }` (TCK-173).
- `src/biology/trainer_engine.rs` — `TrainerSandbox` (one creature + its food), `SandboxOrigin` enum (`elite`/`hof`/`mutant`/`random`, ts-rs), `breed_genome`, `init_rust_sandbox`, `TrainerSandbox::reset_trial`, `step_trainer_sandbox_physics` (eating, respawn, wall counter, 8×8 coverage mask), `TrainerTelemetrySandbox`.
- `src/biology/scenarios.rs` — `Scenario` enum (`standard`/`exploration`, ts-rs), `DietClass` (`CARNIVORE_MIN` 0.6, `HERBIVORE_MAX` 0.4), `TrainingScenarioPlugin` trait (`chamber_size`, `epoch_ticks`, `respawn_min_distance`, `trial_min_distance`, `initialize`, `visible_foods`, `calculate_fitness`) returned as `&'static dyn` by `Scenario::plugin()`; one shared `step_physics` (sensing, brain, kinematics, food drift) with the `visible_foods` hook; helpers `spawn_at_min_distance` and `target_distance_by_diet` (TCK-173).
- Engine (`src/server/engine.rs`) keeps only the UI state: active/running flags, warp (cap 35, [[wiki/architecture/invariants]]), selected sandbox, run id, 25 Hz telemetry and DB writes. One function builds the `TRAINER_STATE_CHANGED` / `TRAINER_RESET_COMPLETED` payload ([[wiki/architecture/simulation-engine]]).
- Creatures are spawned by the shared `CreatureFactory` (`src/biology/creature_factory.rs`, TCK-185), identical to Ocean spawns except position, heading, id and generation. Each sandbox holds exactly one creature; prey mocking ("meatballs") provides meat targets (TCK-106).

## Rooms

| | `standard` | `exploration` |
| :--- | :--- | :--- |
| Chamber | 1000 px | 3500 px |
| Epoch (one trial) | 300 ticks | 1800 ticks |
| Food | 1 plant + 1 meat | 3 plants + 3 meat |
| Visible to sensors | all food | nearest plant and nearest meat only |
| Min. food distance: init / new trial / respawn after eating | 200 / 200 / 200 px | 650 / 1200 / 550 px |

- All values come from the scenario plugin; the engine has no chamber/epoch literals (TCK-173, RV-A05 trainer part).
- Spawns use `spawn_at_min_distance` with at most 1000 tries (`SPAWN_MAX_TRIES`), then the farthest candidate (25 px margin) — the unbounded rejection loops are gone (TCK-173). The distances are unchanged: trial 1 of the large room is easier than trials 2/3 (RV-S08).
- `SET_TRAINER_MODE` switches the room of the running population (`TrainerSim::set_scenario`): every sandbox restarts its trial in the new chamber; genomes, the epoch tick counter and the trial accumulators are kept.
- Sensor range `scale · 550`, measured 256 / 575 / 926 px (p10/p50/p90) over letter-soup genomes; food is in range at spawn for 86 % of structured generation-1 genomes in the small and 24 % in the large room (letter soup: 57 % / 14 %; TCK-172 probe, [[wiki/biology/sensing]]).

## Start state and energy

- Start energy = `stomach_capacity · creatureStartStomachRatio` (0.60), set by the factory and again by every trial reset (TCK-185; before: fixed 100). The brain input `internal_energy = energy / 100` therefore differs per body instead of being 1.0.
- No metabolism in the trainer: energy is never reduced during a trial.
- `reset_trial`: agent to the centre with a random heading, velocity/bend/age reset, start energy, neuron states cleared, food respawned at `trial_min_distance`; learned synapse weights are kept.

## Generation loop

- `step` advances every sandbox one tick; `parallel` steps them on the rayon pool. The UI engine calls `step(true)`.
- Fitness is computed per tick only for the UI telemetry (`track_live_fitness`); the runner computes it at epoch end only, with identical results (TCK-173).
- **Multi-trial (TCK-120, default on, 3 trials):** fitness accumulated per trial, then averaged; no catch in any trial → × 0.1, otherwise + 150 per successful trial.
- Every sandbox rolls its own food layout, so candidates of one generation face different tasks.
- `GenerationStats` are computed at generation end over the final fitness, before breeding ([[wiki/simulation/analytics-and-balancing]] for the fields).
- **Selection:** sandboxes sorted by fitness; `elite_count = clamp(round(N · eliteRatio), 1, N)` elites become the parents and enter the hall of fame (sorted by fitness, bounded to `trainerGenomeRetentionPerRun`, 200).
- **Slot plan** of the next generation: elites (unmodified) | hall-of-fame copies (`round(N · hofRate)`, not mutated) | random immigrants (`round(N · inflowRate)`) | mutants of the parents round-robin. Missing parents/HoF entries fall back to mutants of parent 0.
- **Random genomes** (generation 1, where every slot is random, and immigrants): `generate_structured_genome` with config `genomeComposition` (TCK-184): every genome has all body genes, an `OUT` gene and 1–3 organs; 99 % have a food-sensing organ, 31 % a `HOX` pair (before: letter soup with 21 % organless and 67 % food-sensing genomes, RV-S09) ([[wiki/biology/genome-compiler]]).
- **Mutants:** `round(len · mutationRate)` calls of `mutate_genome` (default 0.06 → ≈ 15 at 256 bases); each call draws creep, point, insertion, deletion, gene duplication or gene deletion by config `mutationOperators` (TCK-184).
- **Lamarckian toggle** (default off, TCK-128): elites and mutants receive the source parent's methylations and, per plastic synapse with the same key in the (possibly mutated) child genome, with probability `lamarckianAssimilationChance` (0.25) the parent's full learned offset, else the genome weight (`inherit_learned_weights`, TCK-179, TCK-152). The draws come from the slot's seeded breed stream, so runs stay deterministic. An unmutated elite also inherits only ~25 % of its parent's learned weights; fixed reflexes and new synapses start at their genome weight. Same rule as Ocean mitosis ([[wiki/biology/metabolism-and-reproduction]]).
- Defaults (`TrainerParams::default`, same as the UI): population 16, elite 0.15, mutation 0.06, inflow 0.15, HoF 0.15, multi-trial on, 3 trials, Lamarck off.
- Fixed while extracting (TCK-173): Lamarck source index underflow for fallback mutants; Lamarck mutants were built from the unmutated parent genome; catalogue assignment kept the old `antisense` and neuron buffers.
- `trainer_genomes` rows from an older model version (before Brain v2, before Oja learning) are dropped by the model-version reset (TCK-183, `MODEL_VERSION` 3 since TCK-152, [[wiki/architecture/persistence]]).
- Kept as is (TCK-173 equivalence): HoF slots are not mutated although a mutation rate is passed; `avg_fitness` divides by the population size.

## Determinism

- One seed per run: `seed_from_str("{config.seed}:{run_id}")` in the UI, `seed_from_str(--seed)` in the runner ([[wiki/architecture/simulation-engine]]).
- RNG streams (`src/shared/rng.rs`): sandbox `i` in generation `g`, trial `t` uses `derive(seed, [g, t, i])`; breeding slot `s` uses `derive(seed, [g, BREED_STREAM, s])`. Results do not depend on `parallel` or the thread count (tests in [[wiki/quality/testing]]).
- The per-tick path uses `src/shared/fmath.rs` (pure-Rust `libm`) for transcendental functions, so reports are identical across platforms.

## Persistence

- On `GenerationCompleted` the engine emits `TRAINER_GENERATION_COMPLETED` (`generation`, `bestFitness`, `avgFitness`, `bestGenome`) and calls `save_trainer_elites`: previous rows of the run become inactive, the elites are inserted as active, then retention prunes the run ([[wiki/architecture/persistence]], TCK-183).
- `SET_MODE trainer`, a new `runId`, a population change or the Lamarck toggle restore the population from the DB (`load_trainer_genomes`: active parents up to N, HoF up to the retention); the generation continues from the newest parent. `TRAINER_RESET` deletes the run's rows and starts a fresh `TrainerSim`.
- `ASSIGN_SANDBOX_CREATURE` rebuilds the agent through the factory (position, heading and motion kept, trial counters restart) and stores it as an active row with fitness 0.

## Fitness

- Standard success: `(yield · 1000 + path_efficiency + speed_bonus) · wall_penalty`, `speed_bonus = (epoch − finish_tick) · 0.2`; failure: `(base − distance · 0.20) · penalty · wall_penalty`.
- Exploration success: `(yield · 3000 + 1000 · path_efficiency + speed_bonus + coverage_bonus) · wall_penalty`, speed factor `0.5`; failure rewards displacement from the centre (`· 1.2`) plus a homing bonus within 550 px, kinetic waste `distance · 0.02`; tight looping (average radius < 180 px) × 0.25.
- `sb.epoch_ticks` increments every tick (TCK-136), so `finish_tick` records the real consumption tick. Fitness of runs after TCK-135/136 is not comparable with older HoF entries.

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

## Controls

- Start/pause/reset, warp (capped 35), hyperparameters, sandbox selection/assignment, Lamarckian synapse inheritance toggle (TCK-128), `apply_champion`, add catalogue creatures to training.

## Key Takeaways

- `TrainerSim` is IO-free, seeded and parallel-safe; the engine and the headless runner drive the same code.
- Two plugin rooms with multi-trial evaluation and elitist selection; spawn loops are bounded, spawn distances still differ per trial (RV-S08).
- P2 baseline: success rate 0.040 (small) and 0.023 (large) at population 16 after 30 generations; after Brain v2 0.122 and 0.033; after Oja learning (P4) 0.207 and 0.046; after Locomotion v2 (P5) 0.218 and 0.033 (10 seeds: 0.193 and 0.060).
- Generation 1 and immigrants are structured random genomes; Lamarckian weights follow synapse keys and are inherited per plastic synapse with probability 0.25 (TCK-152).

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-105, TCK-106, TCK-120, TCK-128, TCK-136, TCK-150, TCK-152, TCK-173, TCK-179, TCK-180, TCK-183, TCK-184, TCK-185, `docs/revisit/04-simulation-model.md`, `src/biology/scenarios.rs`, `src/biology/trainer_sim.rs`.
