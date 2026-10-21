---
name: testing-strategies
description: Testing rules for Pixel DNA Life - reproduction-first tests, pure-function testing, CTRNN validation and golden master handling.
---
# testing-strategies: Testing Rules

## I. Knowledge (read first)
- Test inventory, golden master, validation facts, gaps: `wiki/quality/testing.md`

## II. Rules
- **Reproduction first:** before fixing a behavioural bug, write a failing test that reproduces it; record the failure message in the ticket's `## Verification`.
- **Fast and pure:** test pure functions (`parse_genome`, `execute_brain_with_learning`, `compute_sensory_inputs`, `step_creature_kinematics`, scenario `calculate_fitness`) directly. The engine closure needs a Tauri window; extract logic into pure functions instead of launching the app.
- **No production DB in tests:** use `init_db(":memory:")`.
- **Fixtures:** build `CreatureAgent` fixtures from `parse_genome` output with fixed inputs.
- **CTRNN checks:** `tau` within `[0.1, 2.5]`, no NaN/inf after many steps, output range per activation, bit-identical results for identical inputs.
- **Golden master:** a changed fingerprint is either a bug or an intended model change — never update it silently.
- Run `cargo test` (and `npm run build` for frontend changes) before closing a ticket.
