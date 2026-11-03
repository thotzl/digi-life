---
sources: [tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-192-trainer-population-independent-of-grid.md, tickets/closed/TCK-171-training-track-epic.md, wiki/simulation/trainer.md]
---

# ADR-008: Trainer Evaluation and Selection

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-26
- Tickets: TCK-174 (incl. P8 post-fix), TCK-181, TCK-192; epic TCK-171 phases P7/P8 and wrap-up

## Context

Selection was dominated by noise (TCK-174): every sandbox rolled its own food layout, only ~2 parents survived (16 sandboxes, elite ratio 0.15) and each child received 15–30 mutation ops. Measurement-based decisions of the epic are taken by the agent and documented with numbers in the ticket (user, 2026-10-21, TCK-171 execution decisions).

## Decision

1. **Shared layouts:** one food layout per trial, identical for all candidates of a generation (TCK-174).
2. **Mutation strength:** 1–3 ops per child, adapted at population level by the 1/5 rule (`mutationOneFifthRule` on, limit 8) instead of `len · rate`. Measured on 10 seeds: 3 trials mean 0.338 / 0.193 vs. with 1/5 rule 0.444 / 0.260 (≈ 2.4 and 2.1 SE).
3. **Parents:** elite ratio 0.25 (truncation selection; tournament selection was only an optional variant and was not chosen).
4. **Trials and layout policy (corrected in P8):** P7 chose 3 trials, mean, layouts held until all elites solve them. The P8 post-fix found that "hold" let the population memorise its training layouts (b1: 0.52 on its training layout, 0.02–0.25 on unseen layouts). Corrected decision: **8 trials, mean, layout policy `rotate`** (new shared layout every generation): best generalisation in the small room (unseen small 0.266 / elites 0.355 vs. 0.149 / 0.170 for hold, 3), and training success then equals performance on unseen layouts. From P8 on, targets are read on unseen layouts (`--eval-final`).
5. **No body/brain mutation decoupling** (TCK-181): "separate" vs. "mixed" channels, 10 seeds: small 0.415 vs. 0.438 (−0.4 SE), large 0.335 vs. 0.283 (+1.2 SE), "no clear benefit". Default stays `mutationChannels` "mixed"; the instrumentation and the "separate" option are kept for the active body traits planned after the epic.
6. **Population independent of the visible grid, default 100** (TCK-192, after the user's decision on the epic report): population is its own trainer parameter (`trainerPopulation`); the grid shows the first N sandboxes. Pop 100 (5 seeds) small direct 0.708, curriculum large 0.376 vs. pop 300 (10 seeds) 0.709 / 0.395 at ~⅓ of the compute.

## Alternatives considered

| Alternative | Measured result (source) |
| :--- | :--- |
| 3 / 5 / 8 trials, mean vs. median (P7) | more trials helped the small room only within noise and cost runtime; in the large room they lowered success (TCK-174) |
| Layout policy hold | memorisation of training layouts (TCK-174 post-fix) |
| Population 16 / 300 | pop 16: 0.454 / 0.227 (targets missed); pop 300: same success as 100 at ~3× compute (TCK-192) |
| Separate mutation channels | no clear benefit (TCK-181) |

## Consequences

- `layout_id`, mode, level and max mutation ops are stored with the run state (TCK-191), so a restored run continues its layouts.
- Training success numbers of P7 (small 0.434 / large 0.306) overstate capability; later comparisons use unseen layouts.
- Measurement tables: [[wiki/history/trainer-measurements-p2-p8]]; current trainer: [[wiki/simulation/trainer]].
