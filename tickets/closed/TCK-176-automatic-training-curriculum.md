---
id: TCK-176
title: Automatic Training Curriculum with One Difficulty Knob
status: closed
priority: medium
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-175
---

# TCK-176: Automatic Training Curriculum with One Difficulty Knob

Part of epic TCK-171. Small core of TCK-123 (which planned ten discrete scenarios plus playlists); TCK-123 is re-scoped after this ticket.

## Description
The jump from "food in sight" (small chamber) to "food out of sight" (large chamber) is a cliff. A continuous difficulty knob driven by the success rate makes it gradual.

## Scope
- One difficulty parameter controlling start distance and/or chamber size, or food density (more food further away gives an earlier signal; density decreases along the curriculum).
- Gate in the spirit of POET's minimal criterion ("neither too easy nor too hard"): raise difficulty when the success rate is above an upper threshold, lower it below a lower threshold. Start values 60 % / 30 % are calibration values, not literature values; measure and adjust. Alternative: teacher-student rule (choose the step with the steepest learning progress).
- Difficulty level part of the report and the UI.
- **Retention across rooms:** a share of the trials always runs in earlier difficulty levels / rooms (rehearsal), so skills learned in an easier room are not lost after switching. Share is a measured calibration value.
- **Switch only at generation boundaries:** automatic and manual room switches (today `SET_TRAINER_MODE`, `src/server/engine.rs:774-806`) take effect at the next generation; epoch ticks, trial index and accumulated fitness reset with the switch.
- **Manual mode:** the user can set the difficulty / room by hand during a run (same mechanism as the knob).
- **Metabolic pressure ramp:** the energy cost scale of TCK-151/TCK-178 ramps from 0 to 1 along the curriculum (or over the first generations), so sense organs can prove useful before their full price applies (bootstrap problem).

## Acceptance
- Headless runs reach the large-chamber difficulty and keep a measurable success rate there, compared with direct large-chamber training from the TCK-173 baseline.
- Switch test: train in the small room, switch to the large room (and back); success rate in the small room stays within a tolerance and recovers/increases in the large room over generations.
- Mechanism documented in `wiki/simulation/trainer.md`.

Sources: see epic TCK-171.

## Verification (P8, 2026-10-26)
Implementation: one level plugin `LevelScenario` (`src/biology/scenarios.rs`) is the only `TrainingScenarioPlugin`; the small room is level 0, the large room level 1 (results of both fixed rooms unchanged, checked against the P7 runner). Chamber size, epoch ticks and spawn distance interpolate geometrically in the level, food per type linearly (1 → 3); room endpoints in config `trainerCurriculum.chamberSize/epochTicks/foodPerType`. Curriculum mode (`Scenario::Curriculum`, `TrainerParams.level/level_auto`): gate per generation on the success of the non-rehearsal trials, > `upper` → level + `step`, < `lower` → level − `step`, clamped to [0, 1]; optional rehearsal trials at `rehearsalLevel`, optional metabolic ramp (`metabolic_scale()` hook). Room and level switches (`SET_TRAINER_MODE`, new `SET_TRAINER_LEVEL`) are pending until the next generation boundary; epoch, trial index and fitness reset with them. Runner: `--scenario curriculum`, `--schedule room:gens,...`, `--eval-final <rooms>` with `--eval-trials` (final population on unseen layouts, `EVAL_STREAM`), gate/rehearsal/ramp overrides. Report: `level`, `level_success_rate`, rehearsal fields, `level_move`, `meta.curriculum`, `final_eval`. Tests: 14 (levels 0/1 equal the fixed rooms, gate up/down/clamp, switches only at generation boundaries, rehearsal selection, ramp, schedule, eval determinism).

Evaluation measure: the final population on 10 unseen layouts per room (training success overstates capability, see the TCK-174 post-fix). Settings for all runs: pop 16, 8 trials, layout policy rotate.

Calibration (60 gens, unseen layouts, population; seeds b1..b5 unless noted):

| Training | small | large | final level |
| :--- | :--- | :--- | :--- |
| direct small room (b1..b10) | 0.383 | 0.044 | — |
| direct large room (b1..b10) | 0.109 | 0.083 | — |
| curriculum, gate 0.3 / 0.1 (b1..b10, chosen) | 0.281 | 0.096 | mean 0.61 (2 of 10 seeds at 1.0) |
| curriculum + rehearsal 0.25 (b1..b10) | 0.270 | 0.071 | mean 0.54 |
| curriculum + rehearsal + ramp start 0.5 | 0.135 | 0.061 | — |
| curriculum, gate 0.2 / 0.05 + rehearsal | 0.163 | 0.065 | — |
| curriculum, gate 0.6 / 0.3 (contract start values) | gate never fires: small-room success stays below 0.6 | | 0 |

Chosen: gate `upper` 0.3, `lower` 0.1, `step` 0.1, `startLevel` 0, `rehearsalShare` 0, `metabolicRampStart` 1.0 (no ramp; the ramp lowered success in both rooms).

Switch test (schedule, unseen layouts, b1..b10): 30 gens small → small 0.266 / large 0.044; then 30 gens large → small 0.133 / large 0.041; then 10 gens small → small 0.329 / large 0.050.

Acceptance:
- Curriculum reaches the large-room difficulty only on 2 of 10 seeds within 60 gens (mean level 0.61); its large-room success (0.096) equals direct large-room training (0.083, within noise) while keeping much more small-room skill (0.281 vs. 0.109). Partly met.
- Switch test: the small-room skill drops while training in the large room (−0.13, outside a −0.05 tolerance) and recovers above its earlier level after 10 gens back (0.329); the large room does not improve over the switch. Partly met.
- The large-room target (≥ 0.30 on unseen layouts) is not reached; next steps are the escalations listed in TCK-175 (novelty blend, multi-objective, connection cost) and longer runs / larger populations, to be planned after the epic.
