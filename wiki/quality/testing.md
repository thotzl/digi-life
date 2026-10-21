---
sources: [.agents/skills/testing-strategies/SKILL.md, README.md, docs/revisit/07-testing-ci-tooling.md, tickets/closed/TCK-102-test-setup.md, tickets/ongoing/TCK-149-dead-config-keys.md]
---

# Testing

Agent rules for writing tests live in `.agents/skills/testing-strategies/SKILL.md`. This article holds the inventory and validation facts.

## Running

- `cargo test` from the repo root (= `npm run test:rust`, `npm run types:generate`); also regenerates `bindings/`. 43 tests passing as of TCK-162.
- `npm run test` — Vitest configured (`frontend/**/*.test.ts`), **no frontend test files exist**. The TS-era tests from TCK-102 were deleted with the TS backend (TCK-115/124). Stryker is configured without tests.
- DB tests use `init_db(":memory:")`.

## Rust test inventory

- `biology::dna::tests` — codon/complement helpers, degenerate promoter matching, payload extraction, mutation robustness, progenitor parsing, neuro-generation, Hebbian/epigenetics, pruning, HOX bilateral symmetry.
  - **Golden master** `test_parse_genome_golden_master`: FNV fingerprint of `parse_genome` over 300 deterministic genomes (Latin name excluded), `0xa7b82dc4fe552ac8` as of TCK-140. Any change to phenotypes or brains changes it — update `GOLDEN_FINGERPRINT` only for intended model changes and say so in the ticket.
  - `test_read_evolution_rates_matches_parse_genome`, `test_latin_name_is_deterministic`.
- `biology::trainer_engine::tests` — `finish_tick` records the consumption tick.
- `shared::physics::tests` — wall reflection on all four walls; spore push from `SporePusher` snapshot equals full agent.
- `shared::types::tests` — embedded config equals `config.json`; every config key has a read site in `src/` (`test_every_config_key_has_read_site`, exemptions in `KEYS_WITHOUT_READ_SITE`); unknown keys are rejected (`test_unknown_config_key_is_rejected`) (TCK-149).
- `shared::brain::tests` — determinism, sensory input correctness.
- `database::tests` — in-memory schema, catalogue CRUD, WAL on file DBs, one-time legacy DB copy.

## Not covered

- `engine.rs` (needs a Tauri window, RV-A01), `api.rs`, scenario fitness functions, the frontend.
- No behavioural tests for Ocean mitosis, property-based genome parsing or the warp cap (RV-T02).
- No seeded RNG, so simulation tests can only be statistical (RV-T04).

## CTRNN validation facts

- `tau` within `[0.1, 2.5]`; no NaN/inf after many steps.
- Output ranges: `tanh`/`sin` `[-1, 1]`, `sigmoid` `[0, 1]`, `relu` `[0, ∞)`.
- Identical genome + inputs → bit-identical states.

## Key Takeaways

- Rust unit tests + golden master guard the biology model; the frontend has no tests.
- The engine itself is untested.

Sources: `.agents/skills/testing-strategies/SKILL.md`, `docs/revisit/07-testing-ci-tooling.md`, `README.md`, TCK-102.
