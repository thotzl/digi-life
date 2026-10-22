---
sources: [.agents/skills/testing-strategies/SKILL.md, README.md, docs/revisit/07-testing-ci-tooling.md, tickets/closed/TCK-102-test-setup.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/ongoing/TCK-183-trainer-persistence-and-db-reset.md]
---

# Testing

Agent rules for writing tests live in `.agents/skills/testing-strategies/SKILL.md`. This article holds the inventory and validation facts.

## Running

- `cargo test` from the repo root (= `npm run test:rust`, `npm run types:generate`); also regenerates `bindings/`. As of P2 of TCK-171: 49 unit tests plus one ts-rs export test per exported type (19).
- `npm run test` — Vitest configured (`frontend/**/*.test.ts`), **no frontend test files exist**. The TS-era tests from TCK-102 were deleted with the TS backend (TCK-115/124). Stryker is configured without tests.
- DB tests use `init_db(":memory:")`.

## Rust test inventory

- `biology::dna::tests` — codon/complement helpers, degenerate promoter matching, payload extraction, mutation robustness, progenitor parsing, neuro-generation, Hebbian/epigenetics, pruning, HOX bilateral symmetry.
  - **Golden master** `test_parse_genome_golden_master`: FNV fingerprint of `parse_genome` over 300 deterministic genomes (Latin name excluded), `0xa7b82dc4fe552ac8` as of TCK-140. Any change to phenotypes or brains changes it — update `GOLDEN_FINGERPRINT` only for intended model changes and say so in the ticket.
  - `test_read_evolution_rates_matches_parse_genome`, `test_latin_name_is_deterministic`.
- `biology::trainer_engine::tests` — `finish_tick` records the consumption tick; `reset_trial` restores the trial start state (centre, cleared neuron states, kept learned weights, food at the trial distance).
- `biology::trainer_sim::tests` (TCK-173) — same seed → same generations, other seed differs; `restore` continues with the given parents; Lamarck fallback mutants (former index underflow) and phenotype = `parse_genome(genome, parent methylations)` after breeding; `assign_creature` rebuilds the agent; `test_parallel_step_equals_sequential`, `test_parallel_breeding_equals_sequential`; quantile interpolation.
- `biology::trainer_report::tests` — `test_same_seed_identical_report_and_parallel_equals_sequential` (report minus `perf`).
- `biology::scenarios::tests` — `spawn_at_min_distance` respects the distance and terminates on an unreachable distance; `DietClass` thresholds.
- `biology::creature_factory::tests` (TCK-185) — trainer-style and Ocean-style spawn of one genome are identical except id/position/heading/generation; mismatching learned weights fall back to the genome weights.
- `biology::probes::tests` — no mutation → identical heredity profile; forced mutation kinds change the length as named; speed probe deterministic; Pearson/distribution helpers.
- `shared::rng::tests` — same seed/stream → same sequence; different stream indices, order or seed → different sequence.
- `shared::physics::tests` — wall reflection on all four walls; spore push from `SporePusher` snapshot equals full agent.
- `shared::types::tests` — embedded config equals `config.json`; every config key has a read site in `src/` (`test_every_config_key_has_read_site`, exemptions in `KEYS_WITHOUT_READ_SITE`); unknown keys are rejected (`test_unknown_config_key_is_rejected`) (TCK-149).
- `shared::brain::tests` — determinism, sensory input correctness.
- `database::tests` — in-memory schema, catalogue CRUD, WAL on file DBs, one-time legacy DB copy; trainer retention keeps the best rows plus the active parents per run (TCK-183).

## Determinism and regression

- Simulation tests can be exact: the trainer, the runner and the probes use seeded `SimRng` streams (`src/shared/rng.rs`) and `libm` math (`src/shared/fmath.rs`), so a seed gives a bit-identical report on every platform and thread count (TCK-173).
- **CI reference run** (TCK-186): `trainer_runner run --seed ci-reference --generations 3 --scenario standard --population 16 --parallel true`, compared by `tests/reference/compare-trainer-report.mjs` against the committed `tests/reference/trainer-ci.json`; `perf` and `meta.version` are ignored, every other value must match exactly. An intended model change updates the reference in the same commit, like the golden master ([[wiki/quality/ci-and-tooling]]).
- **Mechanics probes** (TCK-172): `trainer_runner probe heredity|speed|equipment|steering|wall|hebbian|all` — deterministic measurements of the genome/physics/brain model, written as JSON (default `.agents/artifacts/probes/`); results in [[wiki/biology/genome-compiler]], [[wiki/simulation/physics]], [[wiki/biology/ctrnn-brain]], [[wiki/biology/sensing]].

## Per-phase smoke checklist (TCK-186)

Run at the end of every phase of the training epic:
1. `cargo test` and `npm run build` green; `cargo build` without warnings.
2. CI reference run matches.
3. UI (`tauri dev`, app DB backed up and restored): Ocean starts without NaN and inject works; the trainer completes a generation, room switch, pause stops generation completions, reset returns to generation 1; neural inspector and gene tooltips; no console errors. The agent drives the UI part via WebView2 remote debugging (local tooling, not in the repo).

## Not covered

- `engine.rs` (needs a Tauri window, RV-A01), `api.rs`, scenario fitness functions, the frontend.
- No behavioural tests for Ocean mitosis, property-based genome parsing or the warp cap (RV-T02).
- The Ocean still uses `thread_rng`; Ocean behaviour can only be tested statistically (RV-T04).

## CTRNN validation facts

- `tau` within `[0.1, 2.5]`; no NaN/inf after many steps.
- Output ranges: `tanh`/`sin` `[-1, 1]`, `sigmoid` `[0, 1]`, `relu` `[0, ∞)`.
- Identical genome + inputs → bit-identical states.

## Key Takeaways

- Rust unit tests + golden master guard the biology model; the CI reference report guards trainer behaviour; the frontend has no tests.
- The engine itself is untested.

Sources: `.agents/skills/testing-strategies/SKILL.md`, `docs/revisit/07-testing-ci-tooling.md`, `README.md`, TCK-102, TCK-172, TCK-173, TCK-183, TCK-185, TCK-186.
