---
id: TCK-202
title: "R5 Model Corrections (intended behaviour changes, one MODEL_VERSION bump)"
status: closed
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

## Verification (2026-11-03, v1.12.41)
- Reproduction first: `src/sim/model_tests.rs`, one test per correction, each failing before its fix: (1) thermal minimum `max(-5, x)` (`tolerance -57.0`); (2) one eat radius `r * grazingRadiusMultiplier + grazingRadiusOffset` in every world, keys moved to `metabolism` (`food at 36.14 px (radius 36.64) not eaten`); (3) food drifts once per tick before the creatures move in every world, one `CREATURE_SYSTEMS` list (`spore not pushed from the tick-start position`); (4) `fmath` for the clock input, chamber count, mitosis recoil, `polar_offset` and the world generator (world layout platform-independent; no std trig left in non-test simulation code except the steering-probe layout in `diagnostics/probes.rs`); (5) only limb organs drive locomotion (`OrganSpec::limb`, eye/nose/thermal false; renderer reads it from the registry) (`left: 0.69 right: 0.64`); (6) one carnivory classification `src/biology/diet.rs` (0.4 / 0.6, exported to `bindings/diet.ts`), biting needs omnivore/carnivore, photosynthesis herbivores only, `ocean.bitingCarnivoryThreshold` removed (`carnivory 0.36 left: true right: false`); (7) one depth temperature field for sensing and thermal stress (`src/shared/temperature.rs`) (`same depth, same temperature left: 0.816 right: 0.307`); (8) lateral line kept and documented as intended (no change); (9) one death threshold (energy <= 0), bites and mitosis in a second pass, a creature that died in its own step is no bite target (`victim died with energy left`, `Bite{gain: 1e-6}` on a starved creature); (10) hall of fame deduplicated by genome (`["A","A","B"]` vs `["A","B"]`); (11) lifespan = mature age x multiplier, `creatureMaxAgeTicks` removed (TCK-146); (12) `SPAWN_FOOD` / `CLIENT_LOG` already removed in TCK-157 (TCK-147; docs only); (13) letter mutations fall back to letters outside every read gene promoter/terminator, else not applicable (`mutate_genome` may return `None`) (`Insertion at 45 inside a promoter`); (14) `trainer.trainerWallCollisionCooldown` 20, `trainer.trainerSpawnMargin` 25, spore amount and `INIT_STATE.rules.sporeEnergy` from `foodSporeEnergy`.
- Remaining world settings (real world features): targets, metabolic scale, energy modifiers, death policy (Freeze in trainer rooms: the trial resets age and lasts at most 1800 ticks, below the shortest lifespan 3600; Decompose in the Ocean), respawn.
- `MODEL_VERSION` 6 -> 7; `GOLDEN_FINGERPRINT` `0x6b78c31a4d19b0e8` -> `0x84dd8ffb7b9f87dd` (thermal minimum). References `trainer-ci`, `trainer-ci-curriculum`, `trainer-ci-rehearsal`, `ocean-ci` regenerated with the `ci.yml` commands (intended model change); two fresh reruns match. Ocean `ci-ocean-2`: births 2 -> 3, bites 15 -> 7, deaths starved/aged/hunted 43/3/3 -> 57/5/1.
- Training (pop 100, 60 gens, seeds b1-b5, unseen layouts, 10 eval trials; elites in brackets): small room direct 0.676 (0.885), wrap-up 0.708 (0.926); curriculum large 0.408 (0.562), wrap-up 0.376 (0.481); curriculum small 0.451 (0.467). Targets 0.50 / 0.30 met; changes within the seed spread (SD 0.13 small, 0.21 large). Raw reports `.agents/artifacts/tck202/train/`; history `wiki/history/model-corrections-r5.md`.
- Gate on master (`.agents/artifacts/gate.sh`): fmt, clippy `-D warnings`, `cargo test` 296 passed / 1 ignored, four references match, `npm test` 49, lint, build, doc links OK, bindings committed (`organRegistry.ts` with `limb`, new `diet.ts`). UI checks run separately: smoke, selection/reset, dialogs OK, 0 console errors; the dev app logged `backup before the model reset: ...model-v6.bak.db` and `model version 6 -> 7: model tables reset` (the TCK-195 backup works).
- Docs: creature-system, organ-kind, add-dynamic, trainer, ocean, physics, analytics, metabolism-and-reproduction, genome-compiler, sensing, configuration, persistence, ipc-protocol, frontend structure, testing; invariant 8 (four references, fmath rule); `wiki/history/model-corrections-r5.md`.
- Note: installed apps reset their trained creatures on the first start of this version (model version 7); the database is backed up first.
