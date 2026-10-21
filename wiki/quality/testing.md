---
sources: [.agents/skills/testing-strategies/SKILL.md, README.md, docs/revisit/07-testing-ci-tooling.md]
---

# Testing

Agent rules for writing tests (reproduction first, pure functions, no production DB) live in `.agents/skills/testing-strategies/SKILL.md`. This article holds the inventory and the validation facts.

## Running

- `cargo test` from the repo root (= `npm run test:rust`, `npm run types:generate`); also regenerates `bindings/`. 43 tests passing as of TCK-162.
- `npm run test` — Vitest configured (`frontend/**/*.test.ts`), **no frontend test files exist**. Stryker configured without tests.
- DB tests use `init_db(":memory:")`.

## Rust test inventory

- `biology::dna::tests` — codon/complement helpers, degenerate promoter matching, payload extraction, mutation robustness, progenitor parsing, neuro-generation, Hebbian/epigenetics, pruning, HOX bilateral symmetry.
  - **Golden master** `test_parse_genome_golden_master`: FNV fingerprint of `parse_genome` over 300 deterministic genomes (Latin name excluded), `0xa7b82dc4fe552ac8` as of TCK-140. Any change to phenotypes or brains changes it — update `GOLDEN_FINGERPRINT` only for intended model changes and say so in the ticket.
  - `test_read_evolution_rates_matches_parse_genome`, `test_latin_name_is_deterministic`.
- `biology::trainer_engine::tests` — `finish_tick` records the consumption tick.
- `shared::physics::tests` — wall reflection on all four walls; spore push from `SporePusher` snapshot equals full agent.
- `shared::types::tests` — embedded config equals `config.json`.
- `shared::brain::tests` — determinism, sensory input correctness.
- `database::tests` — in-memory schema, catalogue CRUD, WAL on file DBs, one-time legacy DB copy.
- Not covered: `engine.rs` (needs a Tauri window, RV-A01), `api.rs`, `scenarios.rs` fitness.

## CTRNN validation facts

- `tau` within `[0.1, 2.5]`; no NaN/inf after many steps.
- Output ranges: `tanh`/`sin` `[-1, 1]`, `sigmoid` `[0, 1]`, `relu` `[0, ∞)`.
- Identical genome + inputs → bit-identical states.

## Gaps (RV-T02, pool TCK-161)

Behavioural tests still to write:
1. Ocean step: after mitosis `creatures.len()` +1 and the child moves next step (RV-B01).
2. Brain: two independent runs bit-identical; output range per activation.
3. Config: every key deserializes, no unknown keys (RV-S01).
4. Property tests (`proptest`): `parse_genome` never panics on ASCII 0..1024 chars; `mutate_genome` keeps length in `[128, 512]`.
5. Regression guards: warp cap 35; mitosis children enter the simulated list.
- Historic weaknesses: `test_execute_brain_determinism` originally fed 4 inputs into a ≥ 7-input brain; DB tests tested SQLite itself. No test would have caught RV-B01/B02/B05.
- Reproducible tests need a seeded RNG (RV-T04, TCK-158).

## Key Takeaways

- Rust unit tests + golden master guard the biology model; frontend has zero tests.
- The engine is untestable until TCK-156; behavioural suite is TCK-161.

Sources: `.agents/skills/testing-strategies/SKILL.md`, `docs/revisit/07-testing-ci-tooling.md`, `README.md`.
