---
id: TCK-174
title: Fair Trainer Evaluation and Selection
status: open
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173
---

# TCK-174: Fair Trainer Evaluation and Selection

Part of epic TCK-171. Depends on TCK-173 (runner and baseline).

## Description
Selection is dominated by noise: every sandbox rolls its own food layout (`src/biology/scenarios.rs:53-76,197-209`, `src/server/engine.rs:1832-1845`), only ~2 parents survive (16 sandboxes, elite ratio 0.15, `engine.rs:96,157`), and each child receives 15–30 mutation ops (`engine.rs:97`, `src/biology/trainer_engine.rs:78-84`).

## Scope
1. **Shared layouts:** one layout (food positions, start heading) per trial, identical for all candidates. Hold layouts across generations and replace a layout once all elites solve it (DelFly behaviour-tree study).
2. **Trial count:** measure 3 vs. 5 vs. 8 trials and mean vs. median; pick by result variance per runtime, not by default (more evaluations are not always better).
3. **Mutation strength:** 1–3 mutation ops per child instead of `len * rate`; then test a success-based adjustment at population level (1/5-rule family, Doerr et al.).
4. **Parents:** more parents (~20–30 % of the population); tournament selection only as an optional variant (benefit over truncation is disputed: Blickle/Thiele vs. Motoki).
5. **Population size:** measure 16 vs. 100 vs. 300 in the headless runner after 1–4 (larger populations alone do not counter large noise efficiently).

## Acceptance
- Before/after report vs. the TCK-173 baseline: lower fitness variance between seeds and higher success rate (small and large chamber).
- Chosen values documented with the measurement in `wiki/simulation/trainer.md`.

Sources: see epic TCK-171.
