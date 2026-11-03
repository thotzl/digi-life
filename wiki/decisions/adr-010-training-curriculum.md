---
sources: [tickets/closed/TCK-176-automatic-training-curriculum.md, tickets/closed/TCK-171-training-track-epic.md, wiki/simulation/trainer.md]
---

# ADR-010: Training Curriculum with One Difficulty Level

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-26
- Tickets: TCK-176; epic TCK-171 phase P8

## Context

The jump from "food in sight" (small room, 1000 px) to "food out of sight" (large room, 3500 px) was a cliff. TCK-176 asked for one continuous difficulty knob with a gate in the spirit of POET's minimal criterion ("neither too easy nor too hard"); its start thresholds 60 % / 30 % were declared calibration values, not literature values. Rehearsal and a metabolic pressure ramp were in scope as measured options.

## Decision

1. **One level plugin** (`LevelScenario`): the small room is level 0, the large room level 1; chamber size, epoch ticks and spawn distance interpolate geometrically, food per type linearly (1 → 3). Both fixed rooms keep their results.
2. **Gate per generation** on the success of the non-rehearsal trials: above `upper` → level + `step`, below `lower` → level − `step`, clamped to [0, 1]. Chosen: `upper` 0.3, `lower` 0.1, `step` 0.1, `startLevel` 0.
3. **No rehearsal, no metabolic ramp by default** (`rehearsalShare` 0, `metabolicRampStart` 1.0); both stay configurable.
4. **Room and level switches take effect at the next generation boundary**; epoch, trial index and fitness reset with them. Manual level setting uses the same mechanism (`SET_TRAINER_LEVEL`).

## Alternatives considered

Calibration (pop 16, 8 trials, rotate, 60 gens, unseen layouts), TCK-176:

| Training | small | large | final level |
| :--- | :--- | :--- | :--- |
| direct small room (b1..b10) | 0.383 | 0.044 | — |
| direct large room (b1..b10) | 0.109 | 0.083 | — |
| curriculum, gate 0.3 / 0.1 (chosen) | 0.281 | 0.096 | mean 0.61 |
| curriculum + rehearsal 0.25 | 0.270 | 0.071 | mean 0.54 |
| curriculum + rehearsal + ramp start 0.5 | 0.135 | 0.061 | — |
| curriculum, gate 0.2 / 0.05 + rehearsal | 0.163 | 0.065 | — |
| gate 0.6 / 0.3 (ticket start values) | gate never fires | | 0 |

## Consequences

- Acceptance was only partly met at pop 16 (large room 0.096 ≈ direct training, but much more small-room skill kept; small-room skill dropped −0.13 after switching rooms). The large-room target was met later at population 100/300 with the curriculum (TCK-171 wrap-up: 0.376 / 0.395).
- Teacher-student step selection (named alternative in TCK-176) was not evaluated.
- Current mechanism: [[wiki/simulation/trainer]].
