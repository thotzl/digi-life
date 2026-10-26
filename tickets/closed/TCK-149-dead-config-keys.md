---
id: TCK-149
title: Dead Config Keys: Implement or Remove
status: closed
priority: high
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S01
epic_phase: Training Track
---

# TCK-149: Dead Config Keys: Implement or Remove

> Decision 2026-10-21 (training track, epic TCK-171): included because several dead keys shape the creature model (adrenaline brain input always 0, biolum/reserved outputs cost synapses without effect, steering/drag keys). Default grouping (confirm per key with the user at start):
> - implement (needed by other tickets): `maxCreatures` (TCK-143), `seed` (TCK-173/158), `creatureMaxAgeTicks` (TCK-146), `bitingBaseEnergyGain` (TCK-145);
> - implement as clamps after checking intent in code: `creatureMatureAgeFloor`, `lamarckianAssimilationAgeFloor`, `reproductionStomachThresholdFloor`;
> - remove (re-introduce with their research ticket): `adrenaline*` (TCK-165), `biome*`, `bioluminescenceFlashCost` (TCK-167), `predatorSavageThrust*`, `restockFounderGeneInheritChance`;
> - physics keys `steerTorqueBaseMultiplier`, `dragLateralCoefficient`: decide together with TCK-150 (thrust rescale) and TCK-180 (walls);
> - remove from `config.json`: `obstacles*`, `vents*`, `disableVentsForce` (unknown to the struct);
> - biolum (motor 3) and reserved (motor 4) outputs: decide together with the golden-master change of TCK-179 (remove or keep).
> Afterwards `#[serde(deny_unknown_fields)]` and a config test. Findings (RV-S01) below.

## Description
16 of 44 `SimulationRules` are never read (adrenaline, biolum, biomes, predator savage, lateral drag, ...); `maxCreatures`/`seed` unused; 9 keys in config.json unknown to the struct.

## Proposed Scope
Decide per key; afterwards `deny_unknown_fields`.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S01 · P1 · verified** — 16 of 44 `SimulationRules` are never read: `creatureMaxAgeTicks`, `creatureMatureAgeFloor`, `lamarckianAssimilationAgeFloor`, `adrenalineMaxMultiplier`, `adrenalineIncreaseRate`, `adrenalineDecayRate`, `bioluminescenceFlashCost`, `biomeHazardDamageScale`, `biomeAlgaeSporeSpawnThreshold`, `bitingBaseEnergyGain`, `reproductionStomachThresholdFloor`, `steerTorqueBaseMultiplier`, `predatorSavageThrustThreshold`, `predatorSavageThrustMultiplier`, `dragLateralCoefficient`, `restockFounderGeneInheritChance`. `AppConfig`: `maxCreatures`, `seed` unused. 9 keys in `config.json` do not exist in the struct (`obstacles*`, `vents*`, `disableVentsForce`) and are silently ignored.
- Consequences: adrenaline constant 1.0 (brain input always 0, metabolic surcharge always 1); biolum output (motor 3) and "reserved" (4) have no effect but cost synapses/BMR; biomes have no simulation effect.
- Decision per key (user): implement or remove. Target: `#[serde(deny_unknown_fields)]`, every key has at least one read site; config test that every key deserializes.

## Progress P1 (2026-10-21, v1.12.15)
Decisions (user, 2026-10-21): groups A–E as proposed; values introduced by the fixes live in `config.json`, not in code.
- Implemented: `seed` (Ocean world, `engine.rs` `generate_world`, INIT_STATE), `maxCreatures` (guard arm for `INJECT_URZELLE`/`SPAWN_CLONE`, mitosis condition `population < max_creatures`), `creatureMaxAgeTicks` as upper bound with new key `creatureLifespanMatureAgeMultiplier` (12, formerly a literal), `creatureMatureAgeFloor` and `reproductionStomachThresholdFloor` as floors at the mitosis read site (`parse_genome` unchanged, golden master unchanged), `lamarckianAssimilationAgeFloor` as Ocean-only age gate for Lamarckian offsets.
- Removed: `adrenaline*` (4, incl. the always-neutral BMR surcharge), `bioluminescenceFlashCost`, `biome*` (2), `predatorSavageThrust*` (2), `restockFounderGeneInheritChance`; from `config.json`: `obstacles*`, `vents*`, `disableVentsForce`.
- `#[serde(deny_unknown_fields)]` on `AppConfig` and `SimulationRules`.

## Remaining (ticket stays ongoing)
- P3 (with TCK-179 golden-master change): agent field `adrenaline` and brain input `adrenaline_level` (always 0); motor 3 (biolum) and motor 4 (reserved): remove or keep.
- P5 (TCK-150/TCK-180): `steerTorqueBaseMultiplier`, `dragLateralCoefficient`.
- P6 (TCK-145): `bitingBaseEnergyGain`.
- These three keys are listed in `KEYS_WITHOUT_READ_SITE` (`src/shared/types.rs`); remove each entry when its key gets a read site.

## Verification (P1)
- `cargo test`: 45 passed, 0 failed, incl. new `shared::types::tests::test_every_config_key_has_read_site`, `test_unknown_config_key_is_rejected`, existing `test_embedded_config_matches_config_json`, golden master unchanged.
- `cargo build`: 4 warnings before and after (no new warnings). `npm run build`: green.
- Not run: the app itself (manual smoke checklist of TCK-186 does not exist yet; it is built in P2).

## Progress P3 (v1.12.17)
- Motor 3 (biolum), motor 4 (reserved) and the adrenaline brain input plus `CreatureAgent.adrenaline` removed with the Brain v2 golden-master change (TCK-179).
- Remaining: `steerTorqueBaseMultiplier`, `dragLateralCoefficient` (P5, TCK-150/TCK-180), `bitingBaseEnergyGain` (P6, TCK-145).

## Progress P5 (v1.12.19)
- `steerTorqueBaseMultiplier` replaced by `turnRateAtRest` (TCK-150), `dragLateralCoefficient` removed (TCK-180). Remaining: `bitingBaseEnergyGain` (P6, TCK-145).

## Verification (P6, v1.12.20)
- `bitingBaseEnergyGain` read by the predation rule (TCK-145). Every config key now has a read site; the exemption list `KEYS_WITHOUT_READ_SITE` is removed. All keys of RV-S01 are implemented or removed (P1, P3, P5, P6).
