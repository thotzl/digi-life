---
sources: [tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-171-training-track-epic.md, wiki/simulation/trainer.md]
---

# ADR-009: Trainer Fitness in Catch Units and a Minimal Criterion for Generation 1

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-26
- Tickets: TCK-175 (incl. TCK-154, RV-S09); epic TCK-171 phase P7

## Context

The large room suffered from the bootstrap problem: early on all candidates failed equally, and the failure fitness rewarded displacement from the centre instead of approaching food. Many hard multipliers created cliffs (×0.05 / ×0.3, ×0.1 standstill, ×0.25 looping, quadratic path efficiency, 90 % starvation filter, +150 per catch). The HoF sorted the whole run by raw fitness although the rooms scored on different scales. RV-S09 asked whether generation 1 should start blind with random genomes (TCK-175).

## Decision

1. **One fitness function in catch units**, shared by live UI telemetry, epoch end and probes, same scale in every room:
   `fitness = yield + firstCatchBonus · (1 − finish_tick / epoch_ticks) + approachBonus · approach`
   (`firstCatchBonus` 0.5, `approachBonus` 0.3; one catch at full diet fit = 1.0). Approach is measured towards the nearest diet target from the moment it first came into food-sensing range.
2. **Removed:** centre-distance term, wall/looping/standstill/path-efficiency multipliers, trial gate. Generation fitness = aggregate of the trial fitness.
3. **Minimal criterion for generation 1 and immigrants** (RV-S09, `trainerMinimalCriterion` on): structured genomes are redrawn (≤ 50 times) until they have an eye or nose and move ≥ 5 px within 100 ticks.
4. **Escalations not adopted:** novelty blend, multi-objective (Pareto) selection and connection cost were listed as options only if this ticket and the curriculum were insufficient; "not needed so far; revisit after P8" (TCK-175). They remain planned options (`.agents/state/next-tasks.md`).

## Alternatives considered

Measured on 20 seeds (pop 16, 30 gens, mean success of the last 5 gens), TCK-175:

| Variant | small | large | first catch large / small (ticks) |
| :--- | :--- | :--- | :--- |
| old fitness (TCK-174 state) | 0.432 | 0.229 | 944 / 219 |
| new fitness | 0.425 | 0.264 | 662 / 192 |
| new fitness + minimal criterion (chosen) | 0.434 | 0.306 | 722 / 178 |

## Consequences

- `MODEL_VERSION` 5: stored fitness of the old formula (~10²–10³) would have outranked new values in HoF, retention and run list.
- A wrong-diet catch can score below a no-catch trial with full approach; matching catches always win.
- Fitness is shown with two decimals in the UI. Current formula: [[wiki/simulation/trainer]].
