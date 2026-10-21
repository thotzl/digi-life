---
id: TCK-186
title: "Regression Safety: CI Runner Run and Per-Phase Smoke Checks"
status: open
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
