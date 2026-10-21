---
id: TCK-176
title: Automatic Training Curriculum with One Difficulty Knob
status: open
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
