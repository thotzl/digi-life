---
name: testing-strategies
description: Testing rules for BroodBattle - reproduction-first tests, pure-function testing, CTRNN validation and golden master handling.
---
# testing-strategies: Testing Rules

## I. Knowledge (read first)
- Test inventory, golden master, validation facts, gaps: `wiki/quality/testing.md`
- Test recipe of a seam: its contract in `wiki/contracts/` (section Test recipe)

## II. Rules
- **Reproduction first:** before fixing a behavioural bug, write a failing test that reproduces it; record the failure message in the ticket's `## Verification`.
- **Fast and pure:** test pure functions (`parse_genome`, `execute_brain_with_learning`, `compute_sensory_inputs`, `step_creature_kinematics`, scenario `calculate_fitness`) directly. The engine closure needs a Tauri window; extract logic into pure functions instead of launching the app.
- **No production DB in tests:** use `init_db(":memory:")` or `persistence::MemoryDb`.
- **Fixtures:** build `CreatureAgent` fixtures from `parse_genome` output with fixed inputs.
- **CTRNN checks:** `tau` within `[0.1, 2.5]`, no NaN/inf after many steps, output range per activation, bit-identical results for identical inputs.
- **Golden master and references:** `wiki/architecture/invariants.md` 8 and 9.
- **Frontend DOM tests:** Vitest runs in node; a DOM test opts in with `// @vitest-environment happy-dom` as its first line, drives the real views through `startRouter` with a fake `ViewIpc` and a fake `setMode`, mocks `@tauri-apps/api/core` / `event`, and stubs `requestAnimationFrame` (example: `frontend/views/views.dom.test.ts`). Pure helpers stay node tests.
- Run `cargo test` (and `npm run build` for frontend changes) before closing a ticket; CI also gates `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings` and `npm run lint`.
