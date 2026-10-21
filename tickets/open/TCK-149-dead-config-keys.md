---
id: TCK-149
title: Dead Config Keys: Implement or Remove
status: open
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
