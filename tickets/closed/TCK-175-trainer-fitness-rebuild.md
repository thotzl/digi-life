---
id: TCK-175
title: Trainer Fitness Rebuild
status: closed
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
- **Connection cost as a selection objective:** selecting for performance and low connection cost yields more modular and more evolvable networks (Clune, Mouret, Lipson 2013, https://arxiv.org/pdf/1207.2743.pdf); a trainer rule on top of the in-world energy cost of TCK-151.

Sources: see epic TCK-171.

## Verification (P7 step 2, 2026-10-26)
Implementation: one shared function `scenarios::calculate_fitness` (live UI telemetry, epoch end in UI and runner, `probes::evaluate_genome`), in catch units (1.0 = one catch at full diet fit), same scale in both rooms:
`fitness = yield + firstCatchBonus · (1 − finish_tick / epoch_ticks) + approachBonus · approach`
- yield = Σ diet fit per catch (meat: carnivory, plant: 1 − carnivory); time bonus only with a catch, in [0, 0.5); approach = `1 − d_min / d_sight` towards the nearest diet target, where `d_sight` is the distance when that target first came within the creature's food-sensing range (`food_sense_range`, max of scale · 550 over eye/nose/thermal organs); restarts per trial and after each catch; 0 if no target was ever in range.
- Removed: centre-distance term, wall/looping/standstill/path-efficiency multipliers, trial gate (×0.1 / +150). Generation fitness = aggregate of the trial fitness (`trainerTrialAggregation`).
- Config `trainerFitness` (`firstCatchBonus` 0.5, `approachBonus` 0.3; a test keeps both in [0, 1)). A wrong-diet catch (yield < 0.4) can score below a no-catch trial with full approach (max 0.3); matching catches always win.
- Generation 1 / immigrants minimal criterion (RV-S09, config `trainerMinimalCriterion`, on): structured generator with food-organ probability 1.0, redrawn (≤ `maxRedraws` 50) until the genome has an eye or nose and moves ≥ 5 px within 100 ticks; deterministic from the slot's breed stream. Runner flag `--minimal-criterion`.
- `MODEL_VERSION` 5 (stored fitness from the old formula, ~10²–10³, would outrank new values in HoF/retention/run list).
- Frontend: fitness shown with 2 decimals (trainer, catalogue, Ocean, diagnostics).
- Tests: `cargo test` 137 passed (11 new: one catch = 1.0 with bonuses < 1, no cliffs, same scale in both rooms, approach tracking, minimal criterion, live == epoch-end fitness, generation fitness = trial mean, determinism).

Measurement (pop 16, 30 gens, mean success of the last 5 gens, 20 seeds b1..b20; raw data `.agents/artifacts/p7/comb`):

| Variant | small | large | time to first catch large / small (ticks) |
| :--- | :--- | :--- | :--- |
| TCK-174 state (old fitness, 1/5 rule) | 0.432 (SD 0.132) | 0.229 (SD 0.107) | 944 / 219 |
| new fitness | 0.425 (0.163) | 0.264 (0.144) | 662 / 192 |
| new fitness + minimal criterion (chosen) | 0.434 (0.123) | 0.306 (0.109) | 722 / 178 |

Acceptance met: large-room success +0.077 (≈ 2.3 standard errors) and time to first catch −222 ticks; small room unchanged within noise. Large-room target (≥ 0.30) reached at pop 16; small-room target (≥ 0.50) is open for the curriculum (TCK-176). Escalations (novelty, Pareto, connection cost) not needed so far; revisit after P8.
