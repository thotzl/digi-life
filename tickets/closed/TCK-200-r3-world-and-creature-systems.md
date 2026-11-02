---
id: TCK-200
title: "R3 Shared World and Creature Systems for Ocean, Trainer and Probes"
status: closed
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R3
depends_on: TCK-199
---

# TCK-200: Shared World and Creature Systems

One per-creature tick for every simulation. A new dynamic (attack, digestion, reproduction) is one system module plus one entry in a world's system list.

## Findings (2026-11-02 review)
The tick exists four times and diverges:
| Step | Trainer | Ocean (`engine.rs`) |
| :--- | :--- | :--- |
| visible food | `scenarios::visible_foods` 254 | inline 863-960, includes prey peers, O(N·F) without grid |
| clock input | `fmath::sin` (scenarios 331) | std `sin` 963 |
| sensing/brain | shared fns, call sites written twice (scenarios 336, engine 977) | |
| drain | `drain_energy(scale·room)` 349 | `drain_energy(1.0)` 990 + photosynthesis/thermal 993-1006 |
| death | `starved` flag 350 | decompose 1009 and a second copy 1221 |
| food drift | after creature moves (373) | before all creatures (841) |
| eat radius | hard-coded in `trainer_engine.rs:374-381` | config `grazingRadiusMultiplier`/offset 1041 |
| predation, mitosis | none | inline 1101, 1134 |
Plus `probes::brain_tick` 241 ("mirrors step_physics") and `metabolism::mean_trial_activity_and_thrust` 142. The "absent" sentinel food at −99999 is built ~4×. Layering: `metabolism.rs` depends on `trainer_engine`.

## Scope
1. `sim/world.rs`: `World` (entities, food, obstacles, bounds, temperature field, explicit RNG streams) and `TickCtx` (config sections, tick index).
2. `sim/systems/{sense,think,metabolize,move,food_drift,eat,bite,death,reproduce}.rs`, each a plain function over the world (enum or ordered fn list; `dyn` only if needed), events collected and applied after the step (as `damage_map` does). `TargetSelector` replaces `visible_foods` and the Ocean's inline search (grid-based). `EnergyModifier` hooks for photosynthesis/thermal; `RespawnPolicy`, `DeathPolicy` (freeze vs decompose) as world settings.
3. Trainer (`scenarios::step_physics`, `trainer_engine` tick), probes and the metabolism probe use the shared systems; the Ocean adopts them in TCK-156.
4. Divergences are **kept as explicit world settings** in this ticket (trainer eat radius, drift order, `sin` variant) so the references stay identical; unifying them is TCK-202.

## Gate
Trainer references and golden master unchanged; probes produce identical numbers for fixed seeds (compare before/after JSON).

## Acceptance
- One implementation per system; `probes::brain_tick` and `scenarios::step_physics` removed.
- Contract `wiki/contracts/creature-system.md` + guide `wiki/guides/add-dynamic.md`.

## Verification (2026-11-03, v1.12.33)
- New `src/sim/`: `mod.rs` (`System` enum Sense/Think/Metabolize/Death/Move/FoodDrift/Eat, `step_creature(world, idx, ctx, step)` running a world's ordered system list, `CreatureStep` with reused buffers and results, `CreatureEvent::{Ate, Decomposed}` applied by the owner after the step), `world.rs` (`World<'a>` borrowed view: creatures, foods, obstacles, peers, optional grid, size, optional `TemperatureField`, `WorldSettings`; `TickCtx` with brain/metabolism/physics sections; system lists `TRAINER_SYSTEMS`, `OCEAN_SYSTEMS`, `BRAIN_ONLY_SYSTEMS`), `systems/` (`sense` with `TargetSelector` and `ClockVariant` and the single absent-target placeholder, `think`, `metabolize` with `EnergyModifier` photosynthesis/thermal, `death` with `DeathPolicy` Freeze/Decompose, `move`, `food_drift` with `FoodDriftOrder`, `eat` with `EatRadiusRule` / `RespawnPolicy` / spawn helpers, `bite`, `reproduce`).
- Removed duplicate ticks: `scenarios::step_physics`, `probes::brain_tick`; the `visible_foods` plugin hook became `world_settings()`; `metabolism::mean_trial_activity_and_thrust` moved to `probes.rs` (metabolism no longer depends on `trainer_engine`); the trainer sandbox tick calls `step_creature`.
- Divergences kept as explicit world settings (unified in TCK-202): clock Fmath vs Std, targets All/NearestPerType vs NearestPerTypeWithPrey, metabolic scale, energy modifiers, death Freeze vs Decompose, food drift after vs before, eat radius trainer rule vs linear config, respawn MinDistance vs Patchy. The Ocean step in `engine.rs` is untouched and adopts the systems in TCK-156 (plug-in steps in the agent report, Ocean world test already runs the Ocean settings through `step_creature`).
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 223 passed, 1 ignored (13 new `sim::` tests); golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum, rehearsal references match; `npm test` 13 passed; build ok; doc links OK (84 files); no bindings drift. Probes identical: `trainer_runner probe all` before/after for three seed/sample settings, `diff -r` empty over 24 JSON files. UI smoke OK (generation completed, 0 console errors).
- Docs: `wiki/contracts/creature-system.md`, `wiki/guides/add-dynamic.md`; trainer.md ("Creature tick" with the divergence table), physics.md, repository-map, testing, config-section, master index.
