---
sources: [.agents/skills/project-baseline/SKILL.md, README.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-138-embedded-config.md]
---

# Configuration (`config.json`)

## Mechanism

- Root `config.json` is the single source of truth for simulation parameters (metabolism, photosynthesis, biting, mitosis, drag, world generation), loaded into `AppConfig` / `SimulationRules` (`src/shared/types.rs`).
- Compiled into the binary via `include_str!` (TCK-138). A `config.json` in the working directory overrides it for development (`tauri dev` runs from the repo root).
- An invalid override is logged as an error and the embedded config is used.
- Test `shared::types::tests::test_embedded_config_matches_config_json` asserts embedded = `config.json`.
- Before TCK-138 release builds used a divergent hand-written fallback (food 600 vs 300, thrust 6.8 vs 3.4, max age 5400 vs 36000) and swallowed parse errors (RV-B04).

## Rule for new tunables

- Do not hardcode tunables in Rust; add them to `config.json` + `SimulationRules`.
- Verify a key is actually read in `src/` before tuning it (see below).

## Keys without effect (RV-S01)

- 16 of 44 `SimulationRules` are never read: `creatureMaxAgeTicks`, `creatureMatureAgeFloor`, `lamarckianAssimilationAgeFloor`, `adrenalineMaxMultiplier`, `adrenalineIncreaseRate`, `adrenalineDecayRate`, `bioluminescenceFlashCost`, `biomeHazardDamageScale`, `biomeAlgaeSporeSpawnThreshold`, `bitingBaseEnergyGain`, `reproductionStomachThresholdFloor`, `steerTorqueBaseMultiplier`, `predatorSavageThrustThreshold`, `predatorSavageThrustMultiplier`, `dragLateralCoefficient`, `restockFounderGeneInheritChance`.
- `AppConfig`: `maxCreatures`, `seed` unused.
- 9 keys in `config.json` do not exist in the struct (`obstacles*`, `vents*`, `disableVentsForce`) and are silently ignored by serde.
- Effects: adrenaline is constant (brain input always 0, metabolic surcharge always 1); biolum output (motor 3) and "reserved" (4) have no effect but cost synapses/BMR; biomes have no simulation effect.

## Key Takeaways

- One config, embedded in the binary, overridable in dev.
- More than a third of the rules have no effect; check usage before tuning.

Sources: `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/04-simulation-model.md`, TCK-138.
