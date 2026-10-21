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

## Acceptance
- Headless runs reach the large-chamber difficulty and keep a measurable success rate there, compared with direct large-chamber training from the TCK-173 baseline.
- Mechanism documented in `wiki/simulation/trainer.md`.

Sources: see epic TCK-171.
