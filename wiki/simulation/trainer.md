---
sources: [.agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, tickets/closed/TCK-106-advanced-carnivore-training-and-adaptive-ctrnn-physiology.md, tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, tickets/closed/TCK-136-trainer-sandbox-epoch-ticks.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/biology/scenarios.rs]
---

# Trainer

Neuroevolution in isolated sandboxes (training chambers) to breed fit progenitor genomes.

## Structure

- `src/biology/trainer_engine.rs`: `TrainerSandbox`, `init_rust_sandbox`, `step_trainer_sandbox_physics`, `TrainerTelemetrySandbox` (25 Hz telemetry).
- `src/biology/scenarios.rs`: `TrainingScenarioPlugin` trait with `calculate_fitness`; plugins `StandardScenario` (`standard`) and `ExplorationScenario` (`exploration`), selected by `get_scenario_plugin`.
- Grid of N sandboxes (16 chambers in the current UI), one creature + target spores each, runs in the engine thread ([[wiki/architecture/simulation-engine]]).
- Prey mocking ("meatballs") provides meat targets for carnivore training (TCK-106). There are no live opponents; each sandbox holds exactly one creature (`TrainerSandbox.agent`).
- Two rooms exist: `standard` 1000 px and `exploration` 3500 px (the UI option reads "2500x2500", `frontend/views/TrainerView.ts:546`). `SET_TRAINER_MODE` switches the room of the running population immediately (`engine.rs:774-806`); epoch ticks, trial index and accumulated fitness keep running across the switch.
- No metabolism in the trainer: `energy` starts at 100 and is never reduced (`trainer_engine.rs:101`); the Ocean drains BMR every tick. Trainer creatures are constructed differently from Ocean creatures (energy, `antisense`, neuron buffers).
- Sensor range `scale · 550` (≈ 190–990 px, [[wiki/biology/sensing]]): in the small room food is usually in range, in the large room usually not at start.

## Generation loop

- Epoch of fixed ticks (engine literals `300`/`1800`, chamber size `1000`/`3500`, standard/exploration).
- **Multi-trial (TCK-120):** optionally 3 trials per generation; agents reset (position, velocity, neuron states), spores re-randomized, fitness averaged; no catch in any trial → fitness × 0.1, otherwise +150 per successful trial (`engine.rs:1889-1895`).
- Every sandbox rolls its own food layout (`scenarios.rs:53-76,197-209`, re-trial `engine.rs:1832-1845`), so candidates of one generation face different tasks.
- Selection: 16 sandboxes, `eliteRatio` 0.15 → ~2 elites copied unmodified; the rest is filled from Hall of Fame, inflow and mutants of the parents (`engine.rs:157-200`, TCK-105); elites saved to `trainer_genomes` every generation without pruning; Hall of Fame per run, sorted by raw fitness across rooms (`engine.rs:145`).
- Default `mutationRate` 0.06 means `genome.len() · 0.06` mutation operations per mutant (15–30), see [[wiki/biology/genome-compiler]].
- `sb.epoch_ticks` increments every tick (TCK-136, RV-B05), so `finish_tick` records the real consumption tick.
- Generation 1 with origin `random` uses `generate_random_genome(256)` instead of `base_dna` (RV-S09).

## Spawn distances (exploration)

- Initialization `min_dist = 650` (`scenarios.rs`), re-trial `1200` (`engine.rs`), respawn after eating `550` (`trainer_engine.rs`); trial 1 is easier than trials 2/3. The rejection-sampling loops have no iteration limit (RV-S08).

## Fitness

- Standard success: `(yield · 1000 + path_efficiency + speed_bonus) · wall_penalty`, `speed_bonus = (epoch − finish_tick) · 0.2`; failure: `(base − distance · 0.20) · penalty · wall_penalty`.
- Exploration success: `(yield · 3000 + 1000 · path_efficiency + speed_bonus + coverage_bonus) · wall_penalty`, speed factor `0.5`; failure kinetic waste `distance · 0.02`.
- Speed bonus uses `saturating_sub`. Fitness of runs after TCK-135/136 is not comparable with older HoF entries.

## Controls

- Start/pause/reset, warp (capped 35), hyperparameters, sandbox selection/assignment, Lamarckian synapse inheritance toggle (TCK-128), `apply_champion`, add catalogue creatures to training.

## Key Takeaways

- Plugin-based scenarios (2) with multi-trial evaluation and elitist selection.
- The speed bonus is effective since TCK-136; spawn distances differ per trial (RV-S08).

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-105, TCK-106, TCK-120, TCK-128, TCK-136, `docs/revisit/04-simulation-model.md`, `src/biology/scenarios.rs`.
