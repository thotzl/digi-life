---
id: TCK-202
title: "R5 Model Corrections (intended behaviour changes, one MODEL_VERSION bump)"
status: open
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R5
depends_on: TCK-200, TCK-201, TCK-156
---

# TCK-202: Model Corrections

All behaviour-changing fixes found in the 2026-11-02 review, bundled so the refactor phases stay behaviour-preserving. Absorbs TCK-146 (lifespan source of truth) and TCK-147 (SPAWN_FOOD / CLIENT_LOG). One `MODEL_VERSION` bump, new golden master, new trainer references, training results re-measured on unseen layouts.

## Corrections
1. **Thermal tolerance** `dna.rs:1963` (module path after TCK-199): `-5.0f32.max(x)` parses as `-(5.0.max(x))`, so `thermal_tolerance_min` ≤ −5 always; with water 12–38 °C cold stress never fires. Regression test on the tolerance range.
2. **Eat radius** one rule from config (`grazingRadiusMultiplier`/offset) for trainer and Ocean; today the trainer hard-codes `max(r+10, r·0.75+8 / r·0.8+5)` (`trainer_engine.rs:374-381`), so creatures bred in the trainer meet different catch geometry in the Ocean.
3. **Food drift order** identical in all worlds (trainer: after the move, Ocean: before).
4. **Clock and genome math via `fmath`** (Ocean `engine.rs:963` and `parse_genome` 1326/2046 use std `sin`/`cos`).
5. **Limbs**: only limb organs drive locomotion; today every organ with `expression_style ≥ 0.72` counts, eyes included (`physics.rs:115`, renderer, `probes.rs:266`).
6. **Carnivory thresholds** from one source (today 0.50 `is_predator`, 0.55 ecology, 0.60/0.40 diet strings, 0.6/0.4 `DietClass`, 0.35 bite config, 0.55/0.3 in `ocean.ts`).
7. **Temperature**: one model for sensing and thermal stress (today centre gradient in `brain.rs:358` vs y position in the Ocean).
8. **Lateral line**: signed flow is clamped to [0, 1] (`brain.rs:405`); keep the sign or document it as intended.
9. **Death**: one threshold (`<= 0.0` at `engine.rs:1011` vs `> 0.1` at 1227); a creature dying this tick cannot be bitten for energy (1105).
10. **Hall of fame dedup** (`trainer_sim.rs:972`): surviving elite copies are re-added every generation, HoF slots mostly re-mutate the current parents.
11. **Lifespan** one source (TCK-146: `min(mature·mult, cap)` at `engine.rs:1009`, the cap never binds).
12. **SPAWN_FOOD / CLIENT_LOG** (TCK-147, RV-B09): implement as typed Ocean actions or remove from `ocean.ts:536-541,758`.
13. **Mutation insertion** can land inside a promoter when no unprotected locus exists (`dna.rs:982`).
14. Literals into config: wall cooldown 20, 25 px spawn margin, spore `amount: 15.0` vs `foodSporeEnergy`.

## Acceptance
- Each correction has a test that fails before and passes after.
- `MODEL_VERSION` bumped once (backup from TCK-195 keeps old saves); new `GOLDEN_FINGERPRINT` and trainer references committed with the reason.
- Training re-measured (pop 100, curriculum, unseen layouts, 5 seeds) and compared with the TCK-171 wrap-up (small 0.708, large 0.376); a drop below the epic targets (0.50 / 0.30) is reported to the user before closing.
- TCK-146 and TCK-147 moved to closed as absorbed.
