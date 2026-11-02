---
sources: [wiki/contracts/creature-system.md, src/sim/mod.rs]
---

# Guide: add a dynamic

A dynamic is a creature system: something every creature does or suffers each tick (digestion, an attack variant,
fatigue). Read first: [[wiki/contracts/creature-system]] and [[wiki/architecture/invariants]].

## Steps

1. Add `src/sim/systems/<name>.rs` with `pub fn <name>(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep)`.
   Write only creature `idx`, `step` and food positions; draw no random numbers. Anything the owner must do after the
   step (move other entities, use an RNG) becomes a variant of `sim::CreatureEvent`, pushed to `step.events`.
2. Declare it in `src/sim/systems/mod.rs` (`pub mod <name>;`).
3. Registration: a variant in `sim::System` and its arm in `step_creature` (`src/sim/mod.rs`).
4. Put the variant into the system list of each world that runs it, at its place in the order:
   `TRAINER_SYSTEMS` / `OCEAN_SYSTEMS` / `BRAIN_ONLY_SYSTEMS` in `src/sim/world.rs`.
5. If Ocean and trainer need different rules, add a field to `WorldSettings` and set it in
   `LevelScenario::world_settings` (`src/biology/scenarios.rs`), `WorldSettings::ocean` and `probes::brain_world`.
6. Tunables: a key in the section that owns the rule (`metabolism`, `ocean`, `trainer`); no literals in the system.
7. New events: apply them in the owner after the step (`trainer_engine::step_trainer_sandbox_physics`; the Ocean
   from TCK-156).
8. Tests: a `#[cfg(test)]` module in the new file with a hand-built `World::single` (fixture creature from
   `spawn_creature`), covering the rule's edges and that it draws no random numbers.
9. Docs: the system list and settings table in [[wiki/contracts/creature-system]].

## Done when

- `cargo test sim::` passes, including the new tests.
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green.
- A new dynamic in an existing world is a model change: the golden master is unchanged (it covers the phenotype only),
  the trainer references and probe JSON change and are updated on purpose (invariants 8 and 9). A dynamic added to no
  world yet changes nothing.
