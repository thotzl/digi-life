---
id: TCK-149
title: Dead Config Keys: Implement or Remove
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S01
---

# TCK-149: Dead Config Keys: Implement or Remove

> Pool ticket: requires a user decision before implementation. Findings (RV-S01) below.

## Description
16 of 44 `SimulationRules` are never read (adrenaline, biolum, biomes, predator savage, lateral drag, ...); `maxCreatures`/`seed` unused; 9 keys in config.json unknown to the struct.

## Proposed Scope
Decide per key; afterwards `deny_unknown_fields`.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S01 · P1 · verified** — 16 of 44 `SimulationRules` are never read: `creatureMaxAgeTicks`, `creatureMatureAgeFloor`, `lamarckianAssimilationAgeFloor`, `adrenalineMaxMultiplier`, `adrenalineIncreaseRate`, `adrenalineDecayRate`, `bioluminescenceFlashCost`, `biomeHazardDamageScale`, `biomeAlgaeSporeSpawnThreshold`, `bitingBaseEnergyGain`, `reproductionStomachThresholdFloor`, `steerTorqueBaseMultiplier`, `predatorSavageThrustThreshold`, `predatorSavageThrustMultiplier`, `dragLateralCoefficient`, `restockFounderGeneInheritChance`. `AppConfig`: `maxCreatures`, `seed` unused. 9 keys in `config.json` do not exist in the struct (`obstacles*`, `vents*`, `disableVentsForce`) and are silently ignored.
- Consequences: adrenaline constant 1.0 (brain input always 0, metabolic surcharge always 1); biolum output (motor 3) and "reserved" (4) have no effect but cost synapses/BMR; biomes have no simulation effect.
- Decision per key (user): implement or remove. Target: `#[serde(deny_unknown_fields)]`, every key has at least one read site; config test that every key deserializes.
