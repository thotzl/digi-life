---
id: TCK-175
title: Trainer Fitness Rebuild
status: open
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173, TCK-154
---

# TCK-175: Trainer Fitness Rebuild

Part of epic TCK-171. Includes TCK-154 (scenario parameters, spawn distances, RV-S09).

## Description
The large chamber suffers from the bootstrap problem: early on all candidates fail equally, and the failure fitness offers no useful gradient. It rewards displacement from the centre (`src/biology/scenarios.rs:344-347`) instead of approaching food. Many hard multipliers create cliffs and invite exploits: ×0.05 / ×0.3 (`scenarios.rs:154-160`), ×0.1 standstill (`scenarios.rs:340-342`), ×0.25 looping (`scenarios.rs:363-367`), quadratic path efficiency (`scenarios.rs:147-149`), 90 % starvation filter and +150 per catch (`src/server/engine.rs:1889-1895`).

## Scope
- Few interpretable terms per scenario: primary = eaten amount / yield; secondary = time to first catch; shaping = approach achieved towards the nearest food that was in sensor range. No distance-from-centre term, no multiplicative cliffs.
- From TCK-154: spawn distances from the scenario plugin, uniform across trials (today 650 / 1200 / 550); iteration limit for rejection sampling; re-trial calls `plugin.initialize`.
- Chamber size and epoch length from the plugin (`chamber_size()`, `epoch_ticks()`) instead of engine literals (`engine.rs:777,1753`); the UI shows the plugin value (today the option reads "Exploration (2500x2500)" in `frontend/views/TrainerView.ts:546` while the engine uses 3500).
- Fitness comparable across rooms: normalise per scenario or keep elites/HoF per scenario; today the HoF query sorts the whole run by raw fitness (`engine.rs:145`) although the large room scores on a different scale.
- Decide RV-S09 (should generation 1 start blind with random genomes?). Option: minimum equipment or a generation-1 filter discarding immobile/blind genomes (minimal-criterion idea).

## Acceptance
- Before/after report vs. TCK-174 state: success rate and time to first catch in the large chamber improve; no regression in the small chamber.
- Fitness formula documented in `wiki/simulation/trainer.md`.

## Escalation options (only if this ticket and TCK-176 are insufficient in the large chamber)
- **Novelty blend:** fitness plus a novelty term (behaviour = end position, visited sectors), normalised weighted sum (Cuccu/Gomez 2011) or minimal-criterion novelty. Pure novelty search is not recommended: it struggles in large behaviour spaces with a fixed goal (Lehman/Stanley 2011 and follow-ups).
- **Multi-objective (Pareto):** eating, path and energy as separate objectives; ancillary objectives address the bootstrap problem (Mouret/Doncieux).

Sources: see epic TCK-171.
