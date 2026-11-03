---
sources: [tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-171-training-track-epic.md, CHANGELOG.md, wiki/history/model-corrections-r5.md]
---

# ADR-014: Every Config Key Has an Effect; Unknown Keys Are Rejected

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-21
- Tickets: TCK-149 (RV-S01); epic TCK-171 phase P1 (v1.12.15), completed in P3/P5/P6 (v1.12.20)

## Context

The review of 2026-10-19 (RV-S01) found 16 of 44 `SimulationRules` never read (adrenaline, biolum, biomes, predator savage thrust, lateral drag, ...), `maxCreatures` and `seed` unused, and 9 keys in `config.json` unknown to the struct and silently ignored. Dead keys shaped the creature model: the adrenaline brain input was always 0, the biolum and reserved motors cost synapses without effect.

## Decision

User decisions of 2026-10-21 (TCK-149, P1): the proposed key groups A–E are accepted, and values introduced by the fixes live in `config.json`, not in code.

1. **Implement** keys needed by other tickets (`maxCreatures`, `seed`, `creatureMaxAgeTicks`, `bitingBaseEnergyGain`) and the floors (`creatureMatureAgeFloor`, `lamarckianAssimilationAgeFloor`, `reproductionStomachThresholdFloor`).
2. **Remove** keys without effect (`adrenaline*`, `biome*`, `bioluminescenceFlashCost`, `predatorSavageThrust*`, `restockFounderGeneInheritChance`; unknown `obstacles*`, `vents*`, `disableVentsForce`); research keys are re-introduced with their research ticket.
3. **Decide model-coupled keys with their model change:** biolum/reserved motors and adrenaline input with Brain v2 (removed, ADR-002); `steerTorqueBaseMultiplier`, `dragLateralCoefficient` with locomotion v2 (replaced/removed, ADR-012).
4. **Strict config:** `#[serde(deny_unknown_fields)]` on `AppConfig` and `SimulationRules`; a test asserts every key has a read site. The temporary exemption list `KEYS_WITHOUT_READ_SITE` was removed in P6 once every key was read.

## Consequences

- An unknown or misspelt key fails deserialization instead of being ignored (`test_unknown_config_key_is_rejected`).
- A new config key needs a read site in the same change (`test_every_config_key_has_read_site`); later literals moved into config (e.g. TCK-202: `trainerWallCollisionCooldown`, `trainerSpawnMargin`, `foodSporeEnergy`).
- Explicit passing of config sections to modules is a separate decision (ADR-003, point 3).
