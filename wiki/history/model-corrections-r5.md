---
sources: [wiki/simulation/trainer.md, wiki/simulation/ocean.md, wiki/contracts/creature-system.md, tickets/closed/TCK-171-training-track-epic.md]
---

# Model corrections R5 (history)

The one intended model change of the plugin-architecture epic (TCK-202, epic TCK-196 phase R5; absorbs TCK-146 and
TCK-147). It bundles the behaviour fixes of the 2026-11-02 review, so the refactor phases R1–R4 stayed
behaviour-preserving. `MODEL_VERSION` 6 → 7, golden master `0x6b78c31a4d19b0e8` → `0x84dd8ffb7b9f87dd`, all four CI
references regenerated. The current model is described in [[wiki/contracts/creature-system]],
[[wiki/simulation/trainer]], [[wiki/simulation/ocean]] and [[wiki/biology/metabolism-and-reproduction]]; the
regression tests are `sim::model_tests` (one per correction).

## Corrections

| # | Before | After |
| :--- | :--- | :--- |
| 1 | `-5.0f32.max(x)` parsed as `-(5.0.max(x))`: thermal tolerance minimum always ≤ −5 °C, cold stress never fired | minimum `max(-5, x)` |
| 2 | trainer eat radius `max(r + 10, ..)` hard-coded, Ocean `1.5 · r + 4` from config | one rule `r · grazingRadiusMultiplier + grazingRadiusOffset` (section `metabolism`) |
| 3 | food drift after each creature's move (trainer) vs once before all creatures (Ocean) | once per tick before the creatures in every world |
| 4 | Ocean clock and body-chamber count with std `sin`/`cos`; world generation, mitosis recoil, patchy spawn with std trig | `fmath` everywhere in the simulation path |
| 5 | every organ with `expression_style ≥ 0.72` was a limb, eyes included (physics, renderer, probes) | limb-capable kinds only (`Organ::is_limb`); distance senses never |
| 6 | carnivory thresholds 0.50 (predator flag), 0.55 (name), 0.35 (bite, photosynthesis), 0.4/0.6 (trainer, labels) | one `DietClass` (0.4 / 0.6), exported to the frontend (`bindings/diet.ts`); `bitingCarnivoryThreshold` removed |
| 7 | temperature sense: warm chamber centre; thermal stress: depth | one depth field `TemperatureField::WATER` for both |
| 8 | lateral line keeps only head-on flow (clamp to [0, 1]) | kept, documented as intended (directional receptor) |
| 9 | death at energy ≤ 0 in the step, ≤ 0.1 after bite damage; a creature dying in its own step could still be bitten for energy | one threshold (≤ 0); bites resolve after all creatures' steps, dying creatures are no targets |
| 10 | surviving elites re-entered the hall of fame every generation | every genome once (best record) |
| 11 | lifespan `min(mature_age · 12, creatureMaxAgeTicks 36000)`, the cap never bound | `mature_age · 12`, cap removed |
| 12 | `SPAWN_FOOD` / `CLIENT_LOG` | already removed in TCK-157; verified, nothing left |
| 13 | without an unprotected locus a letter mutation drew its locus over the whole genome (could split a promoter) | fallback to letters outside every read gene's promoter and terminator; none: not applicable |
| 14 | literals: wall cooldown 20, spawn margin 25 px, spore `amount` 15 | config `trainerWallCollisionCooldown`, `trainerSpawnMargin`, `foodSporeEnergy` |

Thirteen of fourteen regression tests failed before their correction (8 documents intended behaviour). Correction
13 showed that genomes without any unprotected locus are common in long mutation walks (degenerate motif matching
covers every letter after a few hundred random mutations), so the letter operators fall back instead of stopping.

## Training before and after

Protocol of the TCK-171 wrap-up: pop 100, 60 generations, 8 trials, layout policy `rotate`, seeds b1–b5, final
population on unseen layouts (`--eval-final standard,exploration --eval-trials 10`); mean success rate (elites in
brackets). Raw reports: `.agents/artifacts/wrapup/final` (before) and `.agents/artifacts/tck202/train` (after).

| Run | Room | Before (model 6) | After (model 7) | Target |
| :--- | :--- | :--- | :--- | :--- |
| small room direct, 60 gens | small | 0.708 (0.926) | 0.676 (0.885) | ≥ 0.50 |
| curriculum, 60 gens (final level 1.0) | large | 0.376 (0.481) | 0.408 (0.562) | ≥ 0.30 |
| curriculum, 60 gens | small | 0.467 (0.544) | 0.451 (0.567) | – |
| small room direct, 60 gens | large | 0.060 (0.080) | 0.033 (0.042) | – |

Per seed after (small direct / curriculum large): b1 0.721 / 0.152, b2 0.445 / 0.374, b3 0.702 / 0.738, b4 0.740 /
0.357, b5 0.772 / 0.420. Both targets stay met; the changes are within the seed spread (small direct SD ≈ 0.13,
curriculum large ≈ 0.21).

## Ocean reference before and after

`trainer_runner ocean --seed ci-ocean-2 --ticks 12000`: deaths 49 → 63 (starved 43 → 57, aged 3 → 5, hunted 3 → 1),
bites 15 → 7, births 2 → 3, restocks 59 → 73. More starvation follows from the working cold stress (correction 1)
and fewer biters (omnivores from 0.4 instead of 0.35, correction 6).

Sources: TCK-202, TCK-171 wrap-up.
