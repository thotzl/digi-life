---
sources: [.agents/skills/simulation-analytics/SKILL.md, docs/revisit/04-simulation-model.md, docs/revisit/07-testing-ci-tooling.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-188-targets-adr-and-analytics-skill.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-184-genome-operators.md, src/bin/trainer_runner.rs, src/biology/trainer_report.rs, src/biology/probes.rs, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md]
---

# Analytics and Balancing

Trainer measurements run headless with `trainer_runner` (TCK-173): reproducible, without UI and warp cap, in memory. It replaced the outdated `cli_trainer` (RV-S11). Agent rules for measuring: `.agents/skills/simulation-analytics/SKILL.md`.

## Headless runner

`src/bin/trainer_runner.rs` (CLI) + `src/biology/trainer_report.rs` (run and report) drive the same `TrainerSim` as the UI ([[wiki/simulation/trainer]]). Build with `--release` for measurements; run from the repo root (a `config.json` in the working directory overrides the embedded one, [[wiki/architecture/configuration]]).

```
cargo run --release --bin trainer_runner -- run --seed <text> --generations <N> --scenario standard|exploration --population <N>
    [--trials 3] [--multi-trial true|false] [--parallel true|false] [--persist --db <path>] [--run-id <id>] [--out <file>]
cargo run --release --bin trainer_runner -- probe heredity|speed|equipment|steering|wall|hebbian|topology|metabolism|all --seed <text>
    [--out <file; directory for all>] [--db <path>] [--hof-limit 64] [--samples <N>]
```

- `run`: `--seed`, `--generations`, `--scenario`, `--population` are required; defaults `--trials 3`, `--multi-trial true`, `--parallel false`. All other trainer parameters are the UI defaults (elite 0.15, mutation 0.06, inflow 0.15, HoF 0.15, Lamarck off). Unknown flags are rejected (exit code 2 with usage). Progress goes to stderr; the report goes to `--out` or stdout.
- In memory by default: no DB file is opened or created. `--persist` requires `--db <path>` and writes every generation's elites with retention (run id `--run-id`, default `headless-<seed>`) ([[wiki/architecture/persistence]]).
- `npm run sim:balance` = standard room, seed `baseline`, 20 generations, population 16, parallel, report to `.agents/artifacts/runner/balance.json`.
- Determinism: the text seed is hashed with `seed_from_str`; same seed and flags → identical report except `perf`, independent of `--parallel` and thread count and of the platform ([[wiki/quality/testing]]). `--parallel true` only changes the speed.

## Report (`TrainerReport`)

| Field | Content |
| :--- | :--- |
| `meta` | `seed`, `scenario`, `population`, `generations`, `trials`, `multi_trial`, `version` (crate version) |
| `generations[]` | one `GenerationStats` per generation (below) |
| `perf` | `ticks_per_second_total` (sandbox-ticks/s over all sandboxes), `ticks_per_second_per_sandbox`, `wall_seconds`, `generation_end_seconds` (time in ticks that ended a generation: fitness, stats, breeding) |

`GenerationStats` (computed at generation end over the final fitness, before breeding):
- `success_rate` — share of sandbox-trials with at least one catch (population × trials).
- `mean_time_to_first_catch`, `median_time_to_first_catch` — ticks from trial start to the first catch over successful trials; `null` without any catch.
- `best_fitness`, `median_fitness`, `mean_fitness`, `fitness_quantiles` (p10, p25, p50, p75, p90, linear interpolation) — over the generation's final fitness (multi-trial average incl. the no-catch factor × 0.1 and +150 per successful trial).
- `mean_genome_length` — over the generation's sandboxes.
- `immobile_share` — share of sandbox-trials with less than 5 px travelled.
- `mean_end_energy` — mean energy at trial end over the sandbox-trials (TCK-178).
- `starved_share` — share of sandbox-trials in which the creature ran out of energy (TCK-178).
- `mean_organ_count` — mean organ count of the generation's sandboxes.
- `food_organ_share` — share of the generation's sandboxes with at least one eye or nose (sensor-equipment check under the energy budget, TCK-178).

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
- Target values per phase of the training epic and the baseline they derive from are kept in TCK-171 ("Target values"); measured baselines: [[wiki/simulation/trainer]].

## Caveats

- **Fitness is not comparable** across rooms, across fitness-function changes or with HoF entries from before TCK-135/136; `success_rate` and time to first catch are the comparable metrics.
- **Different food layouts per sandbox:** candidates of one generation solve different tasks, which adds noise to all per-generation numbers ([[wiki/simulation/trainer]]).
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

- `trainer_runner run` gives a reproducible JSON report per generation; `probe` measures the mechanics; both are deterministic per seed.
- Compare reports minus `perf`, over fixed seed sets, against the TCK-171 targets.
- The Ocean has no reproducible measurement path yet; trainer data is the evolution signal.

Sources: `.agents/skills/simulation-analytics/SKILL.md`, `docs/revisit/04-simulation-model.md`, `docs/revisit/07-testing-ci-tooling.md`, TCK-149, TCK-150, TCK-151, TCK-172, TCK-173, TCK-178, TCK-182, TCK-183, TCK-184, TCK-186, TCK-188, `src/bin/trainer_runner.rs`, `src/biology/trainer_report.rs`, `src/biology/probes.rs`.
