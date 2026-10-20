---
name: testing-strategies
description: Testing methodologies, unit testing guidelines, CTRNN mathematical validation, and behavioral mocking for biological models in Pixel DNA Life.
---
# testing-strategies: Mathematical Validation & Bio-Mocking

## I. Principles
- **Reproduction first:** before fixing a behavioral bug (mitosis, brain signals, collision physics), write a failing test that reproduces it.
- **Fast and pure:** test pure functions (`parse_genome`, `execute_brain_with_learning`, `compute_sensory_inputs`, `step_creature_kinematics`, scenario `calculate_fitness`) directly. The engine closure in `engine.rs` needs a real Tauri window and is not unit-testable until RV-A01; extract logic into pure functions instead of launching the app.
- **No production DB in tests:** use `init_db(":memory:")` as `database::tests` does.

## II. Existing Rust Tests (`cargo test`)
- `biology::dna::tests` — codon/complement helpers, degenerate promoter matching, payload extraction, mutation robustness, progenitor parsing, neuro-generation, Hebbian/epigenetics, synaptic pruning, Hox bilateral symmetry.
  - **Golden master** `test_parse_genome_golden_master`: FNV fingerprint of `parse_genome` over 300 deterministic genomes (latin name excluded). Any change that alters phenotypes or brains changes the fingerprint — update `GOLDEN_FINGERPRINT` only for intended model changes and say so in the ticket.
  - `test_read_evolution_rates_matches_parse_genome`, `test_latin_name_is_deterministic`.
- `biology::trainer_engine::tests` — `finish_tick` records the consumption tick.
- `shared::physics::tests` — wall reflection on all four walls, spore push from `SporePusher` snapshot.
- `shared::types::tests` — embedded config equals `config.json`.
- `shared::brain::tests` — brain determinism, sensory input correctness.
- `database::tests` — in-memory schema init, catalogue CRUD, WAL on file DBs, one-time legacy DB copy that never overwrites.
- Frontend: Vitest is configured but no test files exist.

## III. CTRNN Validation
- `tau` must stay within the compiler clamp `[0.1, 2.5]`; assert no NaN/inf after many steps.
- Output range depends on the neuron's activation: `tanh`/`sin` in `[-1, 1]`, `sigmoid` in `[0, 1]`, `relu` in `[0, inf)`.
- Determinism: identical genome + inputs must yield bit-identical states.

## IV. Physics & Behavior
- Build `CreatureAgent` fixtures from `parse_genome` output with fixed inputs to isolate motor/steering math.
- Regression guards to encode as tests: warp cap 35 steps per frame; mitosis children enter the simulated list (RV-B01, once fixed).
