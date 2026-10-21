---
sources: [.agents/skills/server-simulation/SKILL.md, .tickets/closed/TCK-105-evolutionary-reinforcement-learning-trainer.md, .tickets/closed/TCK-120-multi-trial-evaluation-loop-and-min-distance-spawning.md, .tickets/closed/TCK-136-trainer-sandbox-epoch-ticks.md, .tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, src/biology/scenarios.rs]
---

# Trainer

Neuroevolution in isolated sandboxes (training chambers) to breed fit progenitor genomes.

## Structure

- `src/biology/trainer_engine.rs`: `TrainerSandbox`, `init_rust_sandbox`, `step_trainer_sandbox_physics`, `TrainerTelemetrySandbox` (25 Hz telemetry).
- `src/biology/scenarios.rs`: `TrainingScenarioPlugin` trait with `calculate_fitness`; plugins `StandardScenario` (`standard`) and `ExplorationScenario` (`exploration`), selected by `get_scenario_plugin`.
- Grid of N sandboxes (16 chambers in the current UI), one creature + target spores each, runs in the engine thread ([[architecture/simulation-engine]]).

## Generation loop

- Epoch of fixed ticks (literals `300`/`1800`, chamber size `1000`/`3500`, standard/exploration).
- **Multi-trial (TCK-120):** optionally 3 trials per generation; agents reset (position, velocity, neuron states), spores re-randomized, fitness averaged.
- Selection: elites kept (bottom 50 % culled), cloned and mutated to refill (TCK-105); elites saved to `trainer_genomes`; Hall of Fame per run.
- `sb.epoch_ticks` increments every tick since TCK-136 (RV-B05), so `finish_tick` records the real consumption tick.

## Fitness (current code)

- Standard success: `(yield · 1000 + path_efficiency + speed_bonus) · wall_penalty`, `speed_bonus = (epoch − finish_tick) · 0.2`; failure: `(base − distance · 0.20) · penalty · wall_penalty`.
- Exploration success: `(yield · 3000 + 1000 · path_efficiency + speed_bonus + coverage_bonus) · wall_penalty`, speed factor `0.5`; failure kinetic waste `distance · 0.02`.
- Speed bonus uses `saturating_sub`. Fitness of runs after TCK-135/136 is not comparable with older HoF entries.

## Controls

- Start/pause/reset, warp (capped 35), hyperparameters, sandbox selection/assignment, Lamarckian synapse inheritance toggle (TCK-128), `apply_champion`, add catalogue creatures to training.

## Open issues (pool)

- **RV-S08 (TCK-154):** inconsistent exploration spawn distances — init `min_dist = 650` (`scenarios.rs`), re-trial `1200` (`engine.rs`), respawn after eating `550` (`trainer_engine.rs`); trial 1 is systematically easier. Rejection-sampling loops have no iteration limit. Target: `spawn_min_distance()` in the plugin; re-trial calls `plugin.initialize`.
- **RV-S09 (TCK-154):** generation 1 replaces `base_dna` with random genomes (likely intended; then remove the parameter).
- **RV-A05 (TCK-156):** epoch length and chamber size belong in the plugin (`epoch_ticks()`, `chamber_size()`).
- **TCK-123 (ongoing):** curriculum framework — 10 scenarios, setup configurator, playlists with auto-rotation, `prepared_creatures` export and champion seeding into the Ocean, match arena. Depends on RV-S08 and TCK-156, see [[roadmap]].

## Key Takeaways

- Plugin-based scenarios (2 today) with multi-trial evaluation and elitist selection.
- The speed bonus works since TCK-136; spawn distances are still inconsistent (TCK-154).
- Curriculum learning (TCK-123) builds on the plugin trait.

Sources: `.agents/skills/server-simulation/SKILL.md`, TCK-105, TCK-120, TCK-128, TCK-136, `docs/revisit/04-simulation-model.md`, `src/biology/scenarios.rs`.
