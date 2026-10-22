---
id: TCK-186
title: "Regression Safety: CI Runner Run and Per-Phase Smoke Checks"
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173
---

# TCK-186: Regression Safety: CI Runner Run and Per-Phase Smoke Checks

Part of epic TCK-171. Set up in phase P2, applied at the end of every phase.

## Description
Brain, physics and energy are shared by Ocean and trainer; every phase of the epic changes the Ocean immediately as well (e.g. removal of the `* 0.45` BMR literal). Unintended behaviour changes should show up at once, not at the next measurement round.

## Scope
- CI job: headless runner with fixed seed and few generations, report compared against a committed reference report; an intended model change updates the reference in the same commit (like the golden master).
- Smoke checklist per phase (in this ticket, results per phase in the phase commit):
  - Ocean runs N ticks headless or in the app without NaN/inf, creatures move and eat.
  - Trainer UI: start/pause/reset, room switch, generation completes.
  - Neural inspector and gene tooltips show the current model (TCK-187).
  - `cargo test`, `npm run build` green.

## Acceptance
- CI job fails on an unintended change of the reference report.
- Every closed phase of TCK-171 records its smoke-check result.

## Smoke checklist (per phase)
1. `cargo test` and `npm run build` green; `cargo build` without warnings.
2. CI reference run matches (`trainer_runner run --seed ci-reference --generations 3 --scenario standard --population 16 --parallel true`, compared by `tests/reference/compare-trainer-report.mjs`); an intended model change updates `tests/reference/trainer-ci.json` in the phase commit.
3. UI (app via `tauri dev`, WebView2 remote debugging, app DB backed up and restored): Ocean starts, population/generation/spores without NaN, inject works; trainer completes a generation (backend log), fitness labels update, room switch, pause stops generation completions, reset returns to generation 1; no console errors. From P3: neural inspector and gene tooltips show the current model (TCK-187).

## Verification (P2, v1.12.16)
- CI step "trainer reference run" in `.github/workflows/ci.yml`; the comparator fails on a single changed value (tested with a modified report) and ignores `perf` and `meta.version`.
- P2 smoke result: 1-3 passed (UI smoke 2026-10-22: Ocean 15 -> 16 after inject, trainer generations completed, pause 0 completions in 8 s, reset -> 1, 0 console errors). UI fixes found on the way: `styled.ts` class props, `CLIENT_READY` forcing trainer mode off.
