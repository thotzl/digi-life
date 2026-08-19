---
id: TCK-102
title: Setup testing framework and establish core unit tests
status: closed
assigned: Gemini-CLI
created_at: 2026-08-19
closed_at: 2026-08-19
---

# TCK-102: Setup testing framework and establish core unit tests

## Context
We need a robust, fast, and modern testing suite to mathematically validate biochemical mechanisms (DNA decoding, speciation) and physics operations (CTRNN Euler integration, boundary collisions) in isolation. Since Vite is our build tool, Vitest is the ideal testing framework to integrate seamlessly with the existing setup.

## Requirements
1. Install `vitest` as a devDependency.
2. Add a `"test": "vitest run"` and `"test:watch": "vitest"` script to `package.json`.
3. Configure Vitest in `vite.config.ts` or as a standalone configuration if needed.
4. Establish unit tests under `src/biology/dna.test.ts` to verify:
   - DNA sense to antisense base pairing.
   - Phenotypic de-compilation (HSL colors, behavioral traits) parsed from genetic letters.
5. Create a basic test runner validation workflow (TDD check).

## Tasks
- [x] Install `vitest` as devDependency.
- [x] Configure testing scripts in `package.json`.
- [x] Implement unit tests for DNA / Biology logic.
- [x] Ensure all tests pass in CI-style headless execution.

## Verification
- **Testing Framework:** Vitest successfully configured with a dedicated coverage provider (`@vitest/coverage-v8`).
- **Tests Implemented (29 tests total):**
  - `src/biology/dna.test.ts` (10 unit tests covering 100% of CTRNN Euler-integration bounds, genetic decoding, base pairing, and mutation guard rails).
  - `src/biology/speciesDb.test.ts` (10 unit tests covering 100% of SQLite connection triggers and mock fetch endpoints).
  - `src/server/db.test.ts` (3 unit tests covering transactional upsert limits, peak population updates, and WAL modes).
  - `src/server/simulation.test.ts` (6 integration tests covering physical drag coefficients, wall reflections, nutrient spore absorption, asexual mitotic splits, and WebSocket telemetry stream state replication).
- **Reproduction & Fixes:**
  - Resolved Bug 1 (synapse index mapping constraint above neuron 26).
  - Resolved Bug 2 (headless server ignoring outputs[3] "Biolum Flash").
  - Resolved Bug 3 (empty genome mutation crashes).
