---
sources: [.agents/skills/project-baseline/SKILL.md, README.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-138-embedded-config.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-185-creature-factory.md, tickets/ongoing/TCK-183-trainer-persistence-and-db-reset.md]
---

# Configuration (`config.json`)

## Mechanism

- Root `config.json` is the single source of truth for simulation parameters (metabolism, photosynthesis, biting, mitosis, drag, world generation), loaded into `AppConfig` / `SimulationRules` (`src/shared/types.rs`).
- Compiled into the binary via `include_str!` (TCK-138). A `config.json` in the working directory overrides it for development (`tauri dev` runs from the repo root).
- An invalid override is logged as an error and the embedded config is used.
- Test `shared::types::tests::test_embedded_config_matches_config_json` asserts embedded = `config.json`.
- `AppConfig` and `SimulationRules` use `#[serde(deny_unknown_fields)]` (TCK-149): an unknown key fails deserialization. The embedded config then panics (test/build); an invalid working-dir override is logged and the embedded config is used. Test `test_unknown_config_key_is_rejected`.
- Before TCK-138 release builds used a divergent hand-written fallback (food 600 vs 300, thrust 6.8 vs 3.4, max age 5400 vs 36000) and swallowed parse errors (RV-B04).

## Rule for new tunables

- Do not hardcode tunables in Rust; add them to `config.json` + `SimulationRules`.
- Every key needs a read site: test `test_every_config_key_has_read_site` scans `src/` (excluding `shared/types.rs`) for `.<snake_case_key>` per `config.json` key and fails on unread keys (TCK-149).
- Keys kept without a read site must be listed in `KEYS_WITHOUT_READ_SITE` (`src/shared/types.rs`) with the ticket that wires them up.

## Key usage (RV-S01, TCK-149)

- `config.json`: 6 top-level keys (`seed`, `targetPopulation`, `foodSporeCount`, `maxCreatures`, `basalMetabolicRateMultiplier`, `trainerGenomeRetentionPerRun`) + `rules` (34 `SimulationRules`). All are read except the 3 exempt keys below.
- Exempt, unread on purpose: `steerTorqueBaseMultiplier`, `dragLateralCoefficient` (TCK-150/TCK-180), `bitingBaseEnergyGain` (TCK-145).
- Removed (TCK-149): `adrenalineMaxMultiplier`, `adrenalineIncreaseRate`, `adrenalineDecayRate`, `adrenalineMetabolicSurchargeScale` (with the always-1 surcharge factor in the Ocean BMR), `bioluminescenceFlashCost`, `biomeHazardDamageScale`, `biomeAlgaeSporeSpawnThreshold`, `predatorSavageThrustThreshold`, `predatorSavageThrustMultiplier`, `restockFounderGeneInheritChance`, and the 7 struct-less keys `obstacles*`, `vents*`, `disableVentsForce`. Adrenaline keys return with TCK-165, biolum with TCK-167.
- Wired by TCK-149:
  - `seed` → Ocean world generation (`generate_world`) and the `seed` field of `INIT_STATE` (frontend does not read it); since TCK-173 also the trainer run seed `seed_from_str("{seed}:{run_id}")`. The Ocean simulation RNG is still `thread_rng` (RV-T04).
  - `maxCreatures` → cap for `INJECT_URZELLE`/`SPAWN_CLONE` and Ocean mitosis ([[wiki/simulation/ocean]]).
  - `creatureMaxAgeTicks` and new `creatureLifespanMatureAgeMultiplier` → lifespan; `creatureMatureAgeFloor`, `reproductionStomachThresholdFloor` → mitosis floors; `lamarckianAssimilationAgeFloor` → Lamarck age gate ([[wiki/biology/metabolism-and-reproduction]]).
- Added in P2 of TCK-171:
  - `trainerGenomeRetentionPerRun` (top level, 200) → `trainer_genomes` rows kept per run and size of the in-memory trainer hall of fame (TCK-183, [[wiki/architecture/persistence]]).
  - `rules.creatureStartStomachRatio` (0.60, renamed from `restockInitialStomachRatio`) → start energy `stomach_capacity · ratio` of every spawned creature, Ocean and trainer (`creature_factory::start_energy`, TCK-185, [[wiki/biology/metabolism-and-reproduction]]).
- Without effect outside the config: agent field `adrenaline` is constant 1.0 (brain input `adrenaline_level` always 0); motor 3 (biolum) and motor 4 (reserved) have no effect but cost synapses/BMR; biomes have no simulation effect.

## Key Takeaways

- One config, embedded in the binary, overridable in dev; unknown keys are rejected.
- Every key has a read site, enforced by test; 3 documented exemptions (TCK-149).

Sources: `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/04-simulation-model.md`, TCK-138, TCK-149, TCK-173, TCK-183, TCK-185.
