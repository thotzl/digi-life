---
sources: [.agents/skills/simulation-analytics/SKILL.md, docs/revisit/04-simulation-model.md, docs/revisit/07-testing-ci-tooling.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-188-targets-adr-and-analytics-skill.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-184-genome-operators.md, src/bin/trainer_runner.rs, src/biology/trainer_report.rs, src/biology/probes.rs, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md]
---

# Analytics and Balancing

Trainer measurements run headless with `trainer_runner` (TCK-173): reproducible, without UI and warp cap, in memory. It replaced the outdated `cli_trainer` (RV-S11). Agent rules for measuring: `.agents/skills/simulation-analytics/SKILL.md`.

## Headless runner

`src/bin/trainer_runner.rs` (CLI) + `src/biology/trainer_report.rs` (run and report) drive the same `TrainerSim` as the UI ([[wiki/simulation/trainer]]). Build with `--release` for measurements; run from the repo root (a `config.json` in the working directory overrides the embedded one, [[wiki/architecture/configuration]]).

```
cargo run --release --bin trainer_runner -- run --seed <text> --generations <N> --scenario standard|exploration|curriculum --population <N>
    [--schedule <room>:<gens>,...] [--eval-final standard,exploration] [--eval-trials <N>]
    [--curriculum-upper <x>] [--curriculum-lower <x>] [--curriculum-step <x>] [--start-level <0..1>]
    [--rehearsal <share 0..1>] [--ramp-start <x>] [--layout-policy hold|rotate]
    [--trials <N>] [--aggregation mean|median] [--one-fifth true|false] [--elite-ratio <0..1>]
    [--minimal-criterion true|false] [--mutation-channels mixed|separate]
    [--multi-trial true|false] [--parallel true|false] [--persist --db <path>] [--run-id <id>] [--out <file>]
cargo run --release --bin trainer_runner -- probe heredity|speed|equipment|steering|wall|hebbian|topology|metabolism|all --seed <text>
    [--out <file; directory for all>] [--db <path>] [--hof-limit 64] [--samples <N>]
```

- `run`: `--seed`, `--population` and `--scenario` or `--schedule` are required, `--generations` too unless `--schedule` gives it; defaults from `config.json` / `TrainerParams::default` (`RunConfig::with_defaults`): `--trials` = `trainerTrials` (8 since P8), `--layout-policy` = `trainerLayoutPolicy` (rotate), `--aggregation` = `trainerTrialAggregation` (mean), `--one-fifth` = `mutationOneFifthRule` (true), `--elite-ratio` 0.25 (must be in (0, 1]), `--minimal-criterion` = `trainerMinimalCriterion.enabled` (true), `--mutation-channels` = `mutationChannels` (mixed), `--multi-trial true`, `--parallel false` (P7, TCK-174, TCK-175, TCK-181); gate, rehearsal and ramp flags default to `trainerCurriculum` (`--curriculum-upper` 0.3, `--curriculum-lower` 0.1, `--curriculum-step` 0.1, `--start-level` 0, `--rehearsal` 0, `--ramp-start` 1.0; the rehearsal level has no flag), validated (`--start-level`, `--rehearsal` in [0, 1], step in (0, 1], lower ≤ upper) (P8, TCK-176). All other trainer parameters are the UI defaults (mutation ops 1–3 from `mutationOpsMin`/`mutationOpsMax`, inflow 0.15, HoF 0.15, Lamarck off). Unknown flags are rejected (exit code 2 with usage). Progress goes to stderr; the report goes to `--out` or stdout.
- In memory by default: no DB file is opened or created. `--persist` requires `--db <path>` and writes every generation's elites with retention (run id `--run-id`, default `headless-<seed>`) ([[wiki/architecture/persistence]]).
- **Curriculum and schedule** (TCK-176): `--scenario curriculum` runs the curriculum from `--start-level` with the gate on. `--schedule standard:30,exploration:30,standard:10` runs room per generation span (rooms `standard` | `exploration` | `curriculum`); the population and the generation counter carry over, the switch applies at the generation boundary; the span total sets `--generations` (a differing `--generations` or `--scenario` is rejected).
- **Final evaluation** (TCK-176): `--eval-final standard,exploration` evaluates the final population after the last generation (its elites unmodified plus the offspring bred for the next generation, not yet evaluated) in each listed fixed room without breeding, `--eval-trials` trials per sandbox (default `--trials`) on the evaluation layouts `derive(seed, [EVAL_STREAM, trial])`, which no training generation uses (`TrainerSim::evaluate_population`); learned weights carry across the trials as in training. `curriculum` is rejected here.
- `npm run sim:balance` = standard room, seed `baseline`, 20 generations, population 16, parallel, report to `.agents/artifacts/runner/balance.json`.
- Determinism: the text seed is hashed with `seed_from_str`; same seed and flags → identical report except `perf`, independent of `--parallel` and thread count and of the platform ([[wiki/quality/testing]]). `--parallel true` only changes the speed.

## Report (`TrainerReport`)

| Field | Content |
| :--- | :--- |
| `meta` | `seed`, `scenario` (first schedule room for a schedule), `population`, `generations`, `trials`, `trial_aggregation`, `multi_trial`, `elite_ratio`, `mutation_ops_min`, `mutation_ops_max` (at run start; the 1/5 rule may adapt it), `one_fifth_rule`, `minimal_criterion`, `mutation_channels`, `layout_policy` (P8), `curriculum` (`CurriculumParams`: `upper`, `lower`, `step`, `start_level`, `rehearsal_share`, `rehearsal_level`, `metabolic_ramp_start`; P8), `schedule` (list of `{scenario, generations}`, only with `--schedule`), `version` (crate version) |
| `generations[]` | one `GenerationStats` per generation (below) |
| `final_eval` | only with `--eval-final`: per room `{scenario, level, trials, success_rate, mean_fitness, elite_success_rate, elite_mean_fitness}` (`FinalEval` = room + flattened `PopulationEval`; elite values over the unmodified elite slots, `null` without elites) (P8, TCK-176) |
| `perf` | `ticks_per_second_total` (sandbox-ticks/s over all sandboxes), `ticks_per_second_per_sandbox`, `wall_seconds`, `generation_end_seconds` (time in ticks that ended a generation: fitness, stats, breeding) |

`GenerationStats` (computed at generation end over the final fitness, before breeding):
- `success_rate` — share of sandbox-trials with at least one catch (population × trials).
- `mean_time_to_first_catch`, `median_time_to_first_catch` — ticks from trial start to the first catch over successful trials; `null` without any catch.
- `best_fitness`, `median_fitness`, `mean_fitness`, `fitness_quantiles` (p10, p25, p50, p75, p90, linear interpolation) — over the generation's final fitness in catch units (1.0 = one catch at full diet fit; aggregate of the trial fitness, no gate; TCK-175, [[wiki/simulation/trainer]]).
- `mean_genome_length` — over the generation's sandboxes.
- `immobile_share` — share of sandbox-trials with less than 5 px travelled.
- `mean_end_energy` — mean energy at trial end over the sandbox-trials (TCK-178).
- `starved_share` — share of sandbox-trials in which the creature ran out of energy (TCK-178).
- `mean_organ_count` — mean organ count of the generation's sandboxes.
- `food_organ_share` — share of the generation's sandboxes with at least one eye or nose (sensor-equipment check under the energy budget, TCK-178).
- `layout_id` — shared layout the generation was evaluated on; with layout policy `rotate` (default since P8) it increases every generation, with `hold` when all elites solved every trial (TCK-174).
- `mutation_ops_max` — max mutation ops per child used to breed this generation (adapted by the 1/5 rule, TCK-174).
- `child_success_share` — share of mutants with a higher fitness than their parent's unmodified elite slot of the same generation, input of the 1/5 rule; `null` without such mutants (TCK-174).
- `mutation_channels` — mutant slots (origin `Mutant`) per channel `body` / `brain` / `both` / `neutral`, each with `mutants`, `beat_parent_share`, `elite_share` (`null` without mutants), plus `organ_mutants` (mutants with a call in an organ gene, counted as body) (TCK-181).
- `elite_last_mutation` — channel of the last mutation in each elite's lineage, elites in rank order; `null` for a lineage without mutation (generation 1, immigrant) or a restored parent (TCK-181).
- `scenario` — training mode of the generation (`standard` / `exploration` / `curriculum`) (TCK-176).
- `level` — difficulty level of the generation's non-rehearsal trials (fixed rooms 0 / 1) (TCK-176).
- `level_success_rate` — success rate of the non-rehearsal trials, the gate input; equals `success_rate` without rehearsal (TCK-176).
- `rehearsal_trials`, `rehearsal_success_rate` — trials per sandbox run at the rehearsal level and their success rate (`null` without rehearsal) (TCK-176).
- `level_move` — gate decision after the generation, `up` / `down` / `hold`; `null` when the gate is off (fixed room, manual level, or a pending manual switch) (TCK-176).

The UI receives a subset of these fields as `TRAINER_GENERATION_COMPLETED.stats` (`TrainerGenerationStats`, same values; [[wiki/architecture/ipc-protocol]]).

## Probes (TCK-172)

`src/biology/probes.rs`, measurement only. Each probe is deterministic for its seed (own RNG sub-stream per probe, so `all` equals single runs) and writes JSON (default `.agents/artifacts/probes/<name>.json`). `--db` loads up to `--hof-limit` distinct hall-of-fame genomes read-only from a DB; without it the HoF sets are skipped and the Hebbian probe uses the best elites of a seeded 5-generation trainer run instead.

Genome sets (since TCK-184): `letter_soup` = random 256-base genomes (the former random immigrants, same RNG stream as before, so pre-P3 numbers stay comparable) and `structured` = `generate_structured_genome` with config `genomeComposition` (trainer generation 1 and immigrants). Heredity, speed, equipment, steering, wall and topology report both sets; heredity also uses the wildtype and HoF parents.

| Probe | Measures | Results |
| :--- | :--- | :--- |
| `heredity` | parent–child correlation of non-`SY` tape weights and `SY` weights on homologous synapses (same key), body traits, speed and organ changes after 1/5/15 ops: each forced `MutationKind`, `mixed` (`mutate_genome`, config operator mix) and `legacy` (the pre-TCK-184 character mix) | [[wiki/biology/genome-compiler]] |
| `speed` | terminal speed at forced full thrust (1,800 ticks), turning radius and rate at full thrust and bend; `turn_at_rest`: ticks to a 90° heading change at thrust 0 and full bend from rest, and the share reaching it within 1,800 ticks (TCK-150) | [[wiki/simulation/physics]] |
| `equipment` | eyes, noses, `HOX` pairs, sensor range, food stimulus at spawn per room | [[wiki/biology/sensing]] |
| `steering` | heading change toward food at ±45° vs. control, mean thrust sign | [[wiki/biology/ctrnn-brain]] |
| `wall` | getting free from a wall within 600 ticks, forced thrust (heading changes only through wall contact) vs. real brain | [[wiki/simulation/physics]] |
| `hebbian` | trainer evaluation of the same genomes with and without Hebbian learning, both rooms | [[wiki/biology/ctrnn-brain]] |
| `topology` | synapse, neuron, organ count and genome length distributions per set, wildtype synapse/neuron count (TCK-182) | [[wiki/biology/ctrnn-brain]] |
| `metabolism` | inputs of the energy-cost calibration per set: body mass, organ load, neurons, mean synaptic activity and mean |thrust| over one standard-room trial (no drain), resting cost, typical total cost (resting + movement at that thrust), brain + senses share, full-thrust / resting cost (TCK-151); `--samples` default 1000 | [[wiki/biology/metabolism-and-reproduction]] |

## Comparing runs

- Use fixed seed sets and identical flags for before/after; never compare runs of different seeds one by one.
- Exact equality (refactorings, "behaviour-preserving" changes): compare the reports minus `perf` and `meta.version`, e.g. with `node tests/reference/compare-trainer-report.mjs <a.json> <b.json>`; the CI reference run does exactly this ([[wiki/quality/ci-and-tooling]]).
- Model changes: aggregate per configuration over several seeds (the P2 baseline uses 5 seeds × 30 generations at population 16), using the mean of the last 5 generations and generation 1 for the learning slope. Single seeds vary strongly (P2, small room: best generation 0.125 vs. mean 0.040). The 5-seed mean of the last-5 success rate has a standard error of ≈ 0.05 (small room) and ≈ 0.01–0.02 (large room) at population 16; smaller differences need more seeds (P5 added b6–b10, [[wiki/simulation/trainer]]).
- `perf` is machine- and load-dependent: compare only on the same machine, release build, same `--parallel`.
- **Evaluation protocol from P8 on** (TCK-174 post-fix, TCK-176): read capability on the final population on unseen layouts, not on training success: `--eval-final standard,exploration --eval-trials 10`, compare `final_eval[].success_rate` (population) and `elite_success_rate` per room, mean over the seed set (b1–b10 for decisions). P8 runs: pop 16, 8 trials, `--layout-policy rotate`, 60 generations. Training success overstated capability under `hold` (P7: 0.438 training vs. 0.149 unseen in the small room); under `rotate` the two are close. The local helper scripts `.agents/artifacts/baseline/curr.sh` (one configuration over a seed list, with these flags) and `evalsum.py` (mean training success of the last 5 generations and mean `final_eval` success per room and configuration) are not in the repo.
- Target values per phase of the training epic and the baseline they derive from are kept in TCK-171 ("Target values"); measured baselines: [[wiki/simulation/trainer]].

## Caveats

- **Fitness** is in catch units since P7 and on the same scale in both rooms (TCK-175); it is not comparable with reports or HoF entries from before P7 (old formula, ~10²–10³; `MODEL_VERSION` 5). `success_rate` and time to first catch stay the metrics comparable across phases.
- **Shared layouts:** since P7 all candidates of one generation solve the same task (TCK-174); since P8 (`rotate`) the layout changes every generation, so generation-to-generation `success_rate` includes layout difficulty noise; under `hold` a jump can come from a layout change, and a held layout can be memorised ([[wiki/simulation/trainer]]). Respawns are shared only for catches at the same position.
- **Training success ≠ capability:** `success_rate` in `generations[]` is measured on the layouts the population is selected on; use `final_eval` for capability (above).
- **Curriculum runs:** `success_rate` of a curriculum generation is measured at its own level; compare across levels only via `final_eval` in the fixed rooms.
- **HoF-based probes depend on the local DB** (`--db`): their numbers are reproducible only with the same DB content.
- **Config keys:** every key has a test-enforced read site, without exemptions (TCK-149, TCK-145, [[wiki/architecture/configuration]]), so every tuning has an effect.
- **Energy metrics depend on the room length:** the 300-tick small room hardly drains the budget (`starved_share` 0 after P6), the 1800-tick large room does (≈ 0.18); compare energy metrics only within one room.
- **Ocean:** no headless runner and no seeded RNG (`thread_rng`, RV-T03/RV-T04); no Ocean offspring (RV-B01), so Ocean lineage analysis is meaningless; species = genome string (RV-B10), so species counts measure mutation volume, not cladogenesis.

## DB data (Ocean, persisted trainer runs)

SQLite `pixel_life_local.db` in the app data dir ([[wiki/architecture/persistence]]). Open read-only (`sqlite3 -readonly`) while the app runs.

| Table | Useful columns |
| :--- | :--- |
| `simulation_state` | `state_json` → `creatures[]` (`energy`, `age`, `generation`, `neuronStates`), `foodPellets`, `highestGeneration` |
| `simulation_history` | `generation`, `population_count`, `average_fitness`/`max_fitness` (actually avg/max energy) |
| `species_records` | `latin_name`, `parent_name`, `status`, `peak_population`, `generation`, `carnivory`, `birth_time`, `extinction_time` |
| `trainer_genomes` | `run_id`, `generation`, `fitness`, `active`, `genome`, `methylations`, `synapse_weights` (best 200 per run, TCK-183) |
| `creature_catalogue` | `source`, `fitness`, `carnivory` |

- Ancestry via `species_records.parent_name → latin_name`.
- Rows from an older model version are gone: the first start after a `MODEL_VERSION` bump resets all model tables (TCK-183, [[wiki/architecture/persistence]]).
- Ocean heuristics: herbivore/predator ratio ~75 % with `carnivory < bitingCarnivoryThreshold`; energy drain (`metabolism` block, derived constants, [[wiki/biology/metabolism-and-reproduction]]) vs. spore availability (`foodSporeCount`, `foodSporeEnergy`) and predator income (`bitingBaseEnergyGain`, `biteCooldownTicks`); `reproductionSplitLossRatio` must keep growth bounded; `maxCreatures` caps spawn actions and mitosis (TCK-149).

## Key Takeaways

- `trainer_runner run` gives a reproducible JSON report per generation; `probe` measures the mechanics; both are deterministic per seed. Since P8 it also runs the curriculum, room schedules and a final evaluation on unseen layouts.
- Compare reports minus `perf`, over fixed seed sets, against the TCK-171 targets; read targets on `final_eval` (unseen layouts), not on training success.
- The Ocean has no reproducible measurement path yet; trainer data is the evolution signal.

Sources: `.agents/skills/simulation-analytics/SKILL.md`, `docs/revisit/04-simulation-model.md`, `docs/revisit/07-testing-ci-tooling.md`, TCK-149, TCK-150, TCK-151, TCK-172, TCK-173, TCK-174, TCK-175, TCK-176, TCK-178, TCK-181, TCK-182, TCK-183, TCK-184, TCK-186, TCK-188, `src/bin/trainer_runner.rs`, `src/biology/trainer_report.rs`, `src/biology/probes.rs`.
