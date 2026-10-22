---
sources: [.agents/skills/testing-strategies/SKILL.md, README.md, docs/revisit/07-testing-ci-tooling.md, tickets/closed/TCK-102-test-setup.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-179-stable-synapse-identity-and-trait-locality.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-144-motion-vision-channel-design.md, tickets/closed/TCK-153-brain-evaluation-order.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-187-gene-spans-from-backend.md, tickets/closed/TCK-152-hebbian-plasticity-model.md]
---

# Testing

Agent rules for writing tests live in `.agents/skills/testing-strategies/SKILL.md`. This article holds the inventory and validation facts.

## Running

- `cargo test` from the repo root (= `npm run test:rust`, `npm run types:generate`); also regenerates `bindings/`. As of P4 of TCK-171: 71 unit tests (plus 1 ignored measurement) and one ts-rs export test per exported type (27), 98 passing in total.
- `npm run test` — Vitest (`frontend/**/*.test.ts`, node environment): `frontend/shared/genome.test.ts` (TCK-187, 5 tests: promoter table covers every expressed promoter, position → gene span mapping, silent copies, role and synapse descriptions). Not run in CI. The TS-era tests from TCK-102 were deleted with the TS backend (TCK-115/124). Stryker is configured without use.
- DB tests use `init_db(":memory:")`.

## Rust test inventory

- `biology::dna::tests` — codon/complement helpers, degenerate promoter matching, payload extraction, mutation robustness, progenitor parsing, neuro-generation, learning/epigenetics (Oja rule with the learning rate as the only parameter, TCK-152), targeted wiring and pruning (`test_targeted_wiring_and_active_pruning`), HOX bilateral symmetry.
  - **Golden master** `test_parse_genome_golden_master`: FNV fingerprint of `parse_genome` over 300 deterministic genomes (Latin name excluded). `0xa7b82dc4fe552ac8` (TCK-140 to P2) → `0x65f701a6937c1e71` with Brain v2 (TCK-171 P3; intermediate `0x0c404a33923c8dd9` before gene identity by spelling) ([[wiki/decisions/adr-002-brain-v2]]) → `0xe0a2bfd1c7310306` with the per-synapse `plastic` flag (TCK-152, P4; unchanged without the flag). Any change to phenotypes or brains changes it — update `GOLDEN_FINGERPRINT` only for intended model changes and say so in the ticket.
  - **Property tests** (TCK-179, TCK-184): `parse_genome` never panics on 400 random ASCII strings of length 0–1024 (valid indices, owners, finite weights); `mutate_genome` keeps `[128, 512]` over 10,000 ops; a point mutation outside every gene changes no brain, organ or trait; a point mutation inside gene G changes only synapses owned by or touching G, neurons of G and traits of G; adding an organ gene keeps every unrelated synapse key and weight; letter mutations never hit a promoter spelling; gene duplication/deletion keep unrelated synapses (`test_gene_duplication_and_deletion_keep_unrelated_synapses`).
  - Operators (TCK-184): structured genomes parse to exactly the planned genes; gene duplication adds exactly one gene of its promoter; gene deletion removes one and never the last essential gene; creep changes a tape weight by at most the step; generator and mutation are deterministic per seed.
  - `measure_gene_operator_key_shift` (`#[ignore]`, run with `--ignored --nocapture`): key and weight retention of unrelated synapses under gene operators on 1,000 structured genomes.
  - `test_read_evolution_rates_matches_parse_genome`, `test_latin_name_is_deterministic`.
- `biology::trainer_engine::tests` — `finish_tick` records the consumption tick; `reset_trial` restores the trial start state (centre, cleared neuron states, kept learned weights, food at the trial distance).
- `biology::trainer_sim::tests` (TCK-173) — same seed → same generations, other seed differs; `restore` continues with the given parents; Lamarck fallback mutants (former index underflow) and phenotype = `parse_genome(genome, parent methylations)` after breeding; `assign_creature` rebuilds the agent; `test_parallel_step_equals_sequential`, `test_parallel_breeding_equals_sequential`; quantile interpolation; Lamarck by synapse key across gene duplication and in breeding (TCK-179): only plastic offsets land on their key, an unmutated elite inherits some but not all plastic learned weights, and the inherited share in breeding matches `lamarckianAssimilationChance` (TCK-152).
- `biology::trainer_report::tests` — `test_same_seed_identical_report_and_parallel_equals_sequential` (report minus `perf`).
- `biology::scenarios::tests` — `spawn_at_min_distance` respects the distance and terminates on an unreachable distance; `DietClass` thresholds.
- `biology::creature_factory::tests` (TCK-185) — trainer-style and Ocean-style spawn of one genome are identical except id/position/heading/generation; mismatching learned weights fall back to the genome weights; `test_lamarck_chance_is_a_probability_per_synapse` (chance 1 → learned plastic weights, fixed reflexes keep the genome weight; chance 0 → genome weights; chance 0.25 → each plastic synapse takes the full learned weight or the genome weight, share ≈ 0.25 over > 3,000 synapses) and `test_lamarck_draws_are_seeded` (TCK-152).
- `biology::probes::tests` — no mutation → identical heredity profile; forced mutation kinds change the length as named; speed probe deterministic; Pearson/distribution helpers.
- `shared::rng::tests` — same seed/stream → same sequence; different stream indices, order or seed → different sequence.
- `shared::physics::tests` — wall reflection on all four walls; spore push from `SporePusher` snapshot equals full agent.
- `shared::types::tests` — embedded config equals `config.json`; every config key has a read site in `src/` (`test_every_config_key_has_read_site`, exemptions in `KEYS_WITHOUT_READ_SITE`); unknown keys are rejected (`test_unknown_config_key_is_rejected`) (TCK-149).
- `shared::brain::tests` — determinism, sensory input correctness, signed ΔI slot and 1-tick sensor → motor latency (`test_delta_channel_and_one_tick_latency`), exponential integration bounded and non-overshooting for τ in `[0.1, 2.5]` (TCK-144, TCK-153); `test_reflexes_are_fixed_and_the_rest_plastic` (exactly the sensor → thalamus synapses are fixed, weight ±1) and `test_oja_learning_stays_bounded_and_reflexes_stay_fixed` (24 genomes × 36,000 ticks: weights within ±4, reflexes unchanged, < 5 % of plastic weights at `|w| ≥ 3.9`, learning active) (TCK-152).
- `database::tests` — in-memory schema, catalogue CRUD, WAL on file DBs, one-time legacy DB copy; trainer retention keeps the best rows plus the active parents per run; model-version reset on `:memory:` and on reopening a file DB (TCK-183).

## Determinism and regression

- Simulation tests can be exact: the trainer, the runner and the probes use seeded `SimRng` streams (`src/shared/rng.rs`) and `libm` math (`src/shared/fmath.rs`), so a seed gives a bit-identical report on every platform and thread count (TCK-173).
- **CI reference run** (TCK-186): `trainer_runner run --seed ci-reference --generations 3 --scenario standard --population 16 --parallel true`, compared by `tests/reference/compare-trainer-report.mjs` against the committed `tests/reference/trainer-ci.json`; `perf` and `meta.version` are ignored, every other value must match exactly. An intended model change updates the reference in the same commit, like the golden master ([[wiki/quality/ci-and-tooling]]).
- **Mechanics probes** (TCK-172): `trainer_runner probe heredity|speed|equipment|steering|wall|hebbian|topology|all` — deterministic measurements of the genome/physics/brain model, written as JSON (default `.agents/artifacts/probes/`); results in [[wiki/biology/genome-compiler]], [[wiki/simulation/physics]], [[wiki/biology/ctrnn-brain]], [[wiki/biology/sensing]].

## Per-phase smoke checklist (TCK-186)

Run at the end of every phase of the training epic:
1. `cargo test` and `npm run build` green; `cargo build` without warnings.
2. CI reference run matches.
3. UI (`tauri dev`, app DB backed up and restored): Ocean starts without NaN and inject works; the trainer completes a generation, room switch, pause stops generation completions, reset returns to generation 1; neural inspector and gene tooltips; no console errors. The agent drives the UI part via WebView2 remote debugging (local tooling, not in the repo).

## Not covered

- `engine.rs` (needs a Tauri window, RV-A01), `api.rs`, scenario fitness functions; the frontend except `shared/genome.ts`.
- No behavioural tests for Ocean mitosis or the warp cap (RV-T02); genome parsing and mutation have property tests since P3.
- The Ocean still uses `thread_rng`; Ocean behaviour can only be tested statistically (RV-T04).

## CTRNN validation facts

- `tau` within `[0.1, 2.5]`; integration factor `1 − exp(−1/τ)` in `(0, 1)`; no NaN/inf after many steps.
- Output ranges: `tanh`/`sin` `[-1, 1]`, `sigmoid` `[0, 1]`, `relu` `[0, 4]` (state clamp).
- Identical genome + inputs → bit-identical states.

## Key Takeaways

- Rust unit tests, property tests and the golden master guard the biology model; the CI reference report guards trainer behaviour; Vitest covers the frontend gene/role mapping only.
- The engine itself is untested.

Sources: `.agents/skills/testing-strategies/SKILL.md`, `docs/revisit/07-testing-ci-tooling.md`, `README.md`, TCK-102, TCK-144, TCK-153, TCK-172, TCK-173, TCK-179, TCK-182, TCK-183, TCK-184, TCK-185, TCK-186, TCK-187.
