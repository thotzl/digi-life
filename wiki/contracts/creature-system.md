---
sources: [tickets/closed/TCK-200-r3-world-and-creature-systems.md, tickets/closed/TCK-156-engine-decomposition.md, src/sim/ocean/mod.rs, src/sim/mod.rs, src/sim/world.rs, src/sim/systems/mod.rs, src/trainer/chamber.rs, src/trainer/room/level.rs]
---

# Contract: creature system

## Purpose

A creature system is one dynamic of the per-creature tick: sensing, brain, metabolism, death, movement, food drift,
eating, biting, reproduction. All simulations run the same system functions over a `World` (`src/sim/`, TCK-200):
the trainer rooms (`trainer::chamber::Chamber::step`, [[wiki/contracts/room]]), the diagnosis probes
(`diagnostics::probes`, brain-only world), the metabolism calibration (`probes::mean_trial_activity_and_thrust`) and the Ocean
(`sim::ocean::OceanSim::step`, TCK-156). A planned dynamic (digestion,
attack variants) is one new module in `src/sim/systems/`. Where Ocean and trainer behave differently, the difference
is a named field of `WorldSettings`, never a branch on the mode. Since TCK-202 these fields are only real world
features (sensor targets, metabolic scale, environment energy terms, what happens to a corpse, food respawn); the
creature model (clock, eat radius, food drift order, death threshold, water temperature) is one rule for every world.

## Signature

```rust
// src/sim/mod.rs
pub enum System { Sense, Think, Metabolize, Death, Move, Eat }
pub fn step_creature(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep);
pub enum CreatureEvent { Ate { food: usize, is_meat: bool, x: f32, y: f32 }, Decomposed { x: f32, y: f32, spores: i32 } }
pub struct CreatureStep { inputs, visible, motors: MotorOutputs, drain, old_heading, hit_wall, died, events: Vec<CreatureEvent>, nearby }

// src/sim/world.rs
pub struct World<'a> {
    creatures: &'a mut [CreatureAgent], foods: &'a mut [FoodSpore], obstacles: &'a [ProceduralObstacle],
    peers: &'a [PeerSnapshot], grid: Option<&'a SpatialGrid>, width: f32, height: f32, settings: &'a WorldSettings,
}
impl World { pub fn single(agent, foods, size, settings) -> World }      // one creature, square chamber, no obstacles
pub struct WorldSettings {
    systems: &'static [System],          // CREATURE_SYSTEMS (trainer rooms, Ocean), BRAIN_ONLY_SYSTEMS (probes)
    targets: TargetSelector,             // All | NearestPerType (trainer) | NearestPerTypeWithPrey (Ocean)
    metabolic_scale: f32,                // trainer: trainerMetabolicScale · room factor; Ocean 1
    energy_modifiers: Vec<EnergyModifier>, // Ocean: Photosynthesis, ThermalStress
    death: DeathPolicy,                  // Freeze (trainer) | Decompose { lifespan_multiplier, spores } (Ocean)
    respawn: RespawnPolicy,              // MinDistance { min_distance, margin } (trainer) | Patchy { nutrient_centers, recycle_meat } (Ocean)
}
impl WorldSettings { pub fn ocean(&OceanConfig, nutrient_centers) -> Self }  // trainer: Room::world_settings
pub struct TickCtx<'a> { brain: &BrainConfig, metabolism: &MetabolismConfig, physics: &PhysicsConfig, tick: u64 }

// src/sim/systems/<name>.rs: a system in the list is `fn(world: &mut World, idx: usize, [ctx: &TickCtx,] step: &mut CreatureStep)`
sense::sense, think::think, metabolize::metabolize, death::death, r#move::move_creature, eat::eat
// Shared rules every world uses:
sense::clock_input(age) -> f32                                     // 0.5 + 0.5 · fmath::sin(age · 0.1)
eat::eat_radius(mean_radius, &MetabolismConfig) -> f32             // r · grazingRadiusMultiplier + grazingRadiusOffset
death::is_dead_energy(energy) -> bool                              // energy <= 0, in the step and after bite damage
shared::temperature::TemperatureField::WATER                       // depth field: sensing and thermal stress
// Owner-called helpers (state the world does not hold):
food_drift::drift_before_creatures(world, nearby, pushers)        // every world, once per tick before the creature loop
eat::respawn(food, &RespawnPolicy, from, width, height, rng)       // apply CreatureEvent::Ate
death::decompose(foods, x, y, spores, rng)                         // apply CreatureEvent::Decomposed
bite::bite(attacker, candidates, &BiteRule, metabolism, damage_map, bitten_recently) -> Option<BiteEvent>
reproduce::{ready, split, child}(.., &MitosisRule, ..)
```

## Lifecycle / call order

1. Per tick the owner builds a `World` over its storage (cheap: borrows only; the trainer builds its
   `WorldSettings` from the room each tick without allocation).
2. Every world: `food_drift::drift_before_creatures` once before the creature loop (spores pushed by the creatures at
   their tick-start positions).
3. Per creature: `step_creature` runs `settings.systems` in order and stops after a system sets `step.died`.
   `CREATURE_SYSTEMS` (trainer rooms and Ocean): Sense → Think → Metabolize → Death → Move → Eat.
4. After the step the owner reads `step` (motors, inputs, `hit_wall`, `old_heading`, `died`) and applies
   `step.events`: respawn eaten food (`eat::respawn` with its own RNG stream), decompose corpses. Trainer bookkeeping
   (age, coverage, wall counter, approach tracking, yield, first catch) runs here, before the respawn.
5. Ocean only (`OceanSim::step`, TCK-156): eat respawns (`eat::respawn`, also sets `has_eaten`) and decompositions
   are applied right after each creature's step; after all creatures' steps, per creature that survived its step:
   age + 1, bite (candidates from the grid, read from the peer snapshot of the tick start, without creatures that
   died in their own step this tick) and mitosis (TCK-202); bite damage after all creatures, then
   `death::is_dead_energy`, then the species census. RNG: `derive(seed, [STREAM_FOOD | STREAM_MITOSIS, tick])`.

The owner advances `agent.age` (trainer: every tick, also when starved; Ocean: after the move of a living creature).

## Invariants

- Systems draw no random numbers; randomness enters only through the explicit `rng` of the owner-called helpers
  (trainer respawn: `derive(respawn_seed, [catch])`). Determinism and parallel == sequential (epic principle 5).
- Events are applied after the step, never inside a system (the trainer's respawn and the Ocean's bite damage).
- One creature model in every world: clock, eat radius, food drift order, death threshold and water temperature are
  shared rules, not settings (TCK-202). A new difference between worlds needs a reason that is a feature of the
  world (a room without light zone, a trial that freezes a starved creature), not of the creature.
- A system writes only creature `idx`, `step`, and (food drift) the food positions. No IO, no SQL, no allocation in
  the hot loop beyond reused `CreatureStep` buffers ([[wiki/architecture/invariants]] 6).
- Thrust reaches the body only through `step_creature_kinematics` ([[wiki/architecture/invariants]] 4).
- Sensor layout: every selector but `All` yields `[nearest plant, nearest meat]`; an absent target sits at
  `sense::ABSENT_POS` (organ contract [[wiki/contracts/organ-kind]]).
- Changing a system's arithmetic or a world's settings is a model change: golden master, trainer references and
  `MODEL_VERSION` follow invariants 8 and 9. A refactor keeps the probe JSON identical for fixed seeds.

## Registration point

A system run by `step_creature`: one variant in `sim::System`, one arm in the `match` of `step_creature` (the compiler
reports a missing arm), and an entry in each world's list (`CREATURE_SYSTEMS`, `BRAIN_ONLY_SYSTEMS`) at
its place in the order. A world rule: one field of `WorldSettings`, set by `Room::world_settings`
(trainer rooms), `WorldSettings::ocean` and the probes' `brain_world`.

## Config keys

None of its own. Systems read `brain.hebbianLearningRateBase`, section `metabolism` (incl. the eat radius
`grazingRadiusMultiplier` / `grazingRadiusOffset`, shared since TCK-202) and section `physics` through `TickCtx`;
world settings are built from `metabolism.trainerMetabolicScale`, `trainerSpawnMinDistance` / `trainerSpawnMargin` /
`trainerCurriculum` (rooms) and section `ocean` (photosynthesis, thermal stress, lifespan, decomposition, biting,
mitosis) ([[wiki/contracts/config-section]]). Who bites and who photosynthesises is the diet class
(`biology::diet::DietClass`, model constants).

## Events / IPC

`CreatureEvent` and `bite::BiteEvent` are in-process results for the owner. `OceanSim` turns them into
`OceanEvent`s, which `OceanMode` maps onto UI events (`EAT_EVENT`, `BITE_EVENT`, log lines; [[wiki/contracts/mode]]);
the systems emit nothing.

## Test recipe

`cargo test sim::` (selectors, eat radius, respawn, decomposition, bite, mitosis, the TCK-202 regression suite
`sim::model_tests`) and
`cargo test trainer::` (trainer tick on the systems). A refactor must also keep the probe JSON identical:
`trainer_runner probe all --seed <s> --samples <n>` before and after, then diff the directories. Full gate: `cargo test`
(golden master), the three trainer reference runs of `.github/workflows/ci.yml` compared with
`node tests/reference/compare-trainer-report.mjs`, `npm test`, `npm run build`.

## Minimal example

```rust
// src/sim/systems/fatigue.rs: a creature that moved fast this tick loses extra energy
use crate::sim::{CreatureStep, TickCtx, World};

pub fn fatigue(world: &mut World, idx: usize, _ctx: &TickCtx, _step: &mut CreatureStep) {
    let agent = &mut world.creatures[idx];
    let speed = (agent.vx * agent.vx + agent.vy * agent.vy).sqrt();
    if speed > 4.0 {
        agent.energy -= 0.01 * (speed - 4.0);
    }
}
// sim/mod.rs: `Fatigue` in `System`, arm `System::Fatigue => systems::fatigue::fatigue(world, idx, ctx, step)`,
// systems/mod.rs: `pub mod fatigue;`, and `System::Fatigue` after `System::Move` in the lists that use it.
```
