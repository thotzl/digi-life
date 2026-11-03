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
is a named field of `WorldSettings`, never a branch on the mode.

## Signature

```rust
// src/sim/mod.rs
pub enum System { Sense, Think, Metabolize, Death, Move, FoodDrift, Eat }
pub fn step_creature(world: &mut World, idx: usize, ctx: &TickCtx, step: &mut CreatureStep);
pub enum CreatureEvent { Ate { food: usize, is_meat: bool, x: f32, y: f32 }, Decomposed { x: f32, y: f32, spores: i32 } }
pub struct CreatureStep { inputs, visible, motors: MotorOutputs, drain, old_heading, hit_wall, died, events: Vec<CreatureEvent>, nearby }

// src/sim/world.rs
pub struct World<'a> {
    creatures: &'a mut [CreatureAgent], foods: &'a mut [FoodSpore], obstacles: &'a [ProceduralObstacle],
    peers: &'a [PeerSnapshot], grid: Option<&'a SpatialGrid>, width: f32, height: f32,
    temperature: Option<TemperatureField>, settings: &'a WorldSettings,
}
impl World { pub fn single(agent, foods, size, settings) -> World }      // one creature, square chamber, no obstacles
pub struct WorldSettings {
    systems: &'static [System],          // TRAINER_SYSTEMS, OCEAN_SYSTEMS, BRAIN_ONLY_SYSTEMS
    clock: ClockVariant,                 // Fmath (trainer) | Std (Ocean)
    targets: TargetSelector,             // All | NearestPerType (trainer) | NearestPerTypeWithPrey (Ocean)
    metabolic_scale: f32,                // trainer: trainerMetabolicScale · room factor; Ocean 1
    energy_modifiers: Vec<EnergyModifier>, // Ocean: Photosynthesis, ThermalStress
    death: DeathPolicy,                  // Freeze (trainer) | Decompose { lifespan, spores } (Ocean)
    food_drift: FoodDriftOrder,          // AfterEachCreature (trainer) | BeforeAllCreatures (Ocean)
    eat_radius: EatRadiusRule,           // TrainerDiet | Linear { multiplier, offset } (Ocean config)
    respawn: RespawnPolicy,              // MinDistance { min_distance } (trainer) | Patchy { nutrient_centers, recycle_meat } (Ocean)
}
impl WorldSettings { pub fn ocean(&OceanConfig, nutrient_centers) -> Self }  // trainer: Room::world_settings
pub struct TickCtx<'a> { brain: &BrainConfig, metabolism: &MetabolismConfig, physics: &PhysicsConfig, tick: u64 }

// src/sim/systems/<name>.rs: a system in the list is `fn(world: &mut World, idx: usize, [ctx: &TickCtx,] step: &mut CreatureStep)`
sense::sense, think::think, metabolize::metabolize, death::death, r#move::move_creature,
food_drift::drift_after_creature, eat::eat
// Owner-called helpers (state the world does not hold):
food_drift::drift_before_creatures(world, nearby, pushers)        // Ocean order, before the creature loop
eat::respawn(food, &RespawnPolicy, from, width, height, rng)       // apply CreatureEvent::Ate
death::decompose(foods, x, y, spores, rng)                         // apply CreatureEvent::Decomposed
bite::bite(attacker, candidates, &BiteRule, metabolism, damage_map, bitten_recently) -> Option<BiteEvent>
reproduce::{ready, split, child}(.., &MitosisRule, ..)
```

## Lifecycle / call order

1. Per tick the owner builds a `World` over its storage (cheap: borrows only; the trainer builds its
   `WorldSettings` from the room each tick without allocation).
2. Ocean order only: `food_drift::drift_before_creatures` once before the creature loop.
3. Per creature: `step_creature` runs `settings.systems` in order and stops after a system sets `step.died`.
   Trainer list: Sense → Think → Metabolize → Death → Move → FoodDrift → Eat. Ocean list: the same without FoodDrift.
4. After the step the owner reads `step` (motors, inputs, `hit_wall`, `old_heading`, `died`) and applies
   `step.events`: respawn eaten food (`eat::respawn` with its own RNG stream), decompose corpses. Trainer bookkeeping
   (age, coverage, wall counter, approach tracking, yield, first catch) runs here, before the respawn.
5. Ocean only (`OceanSim::step`, TCK-156): eat respawns (`eat::respawn`, also sets `has_eaten`) and decompositions
   are applied right after each creature's step; then, for a living creature, age + 1, bite (candidates from the grid,
   read from the peer snapshot of the tick start) and mitosis; bite damage after all creatures, then
   `death::POST_DAMAGE_MIN_ENERGY`, then the species census. RNG: `derive(seed, [STREAM_FOOD | STREAM_MITOSIS, tick])`.

The owner advances `agent.age` (trainer: every tick, also when starved; Ocean: after the move of a living creature).

## Invariants

- Systems draw no random numbers; randomness enters only through the explicit `rng` of the owner-called helpers
  (trainer respawn: `derive(respawn_seed, [catch])`). Determinism and parallel == sequential (epic principle 5).
- Events are applied after the step, never inside a system (the trainer's respawn and the Ocean's bite damage).
- A system writes only creature `idx`, `step`, and (food drift) the food positions. No IO, no SQL, no allocation in
  the hot loop beyond reused `CreatureStep` buffers ([[wiki/architecture/invariants]] 6).
- Thrust reaches the body only through `step_creature_kinematics` ([[wiki/architecture/invariants]] 4).
- Sensor layout: every selector but `All` yields `[nearest plant, nearest meat]`; an absent target sits at
  `sense::ABSENT_POS` (organ contract [[wiki/contracts/organ-kind]]).
- Changing a system's arithmetic or a world's settings is a model change: golden master, trainer references and
  `MODEL_VERSION` follow invariants 8 and 9. A refactor keeps the probe JSON identical for fixed seeds.

## Registration point

A system run by `step_creature`: one variant in `sim::System`, one arm in the `match` of `step_creature` (the compiler
reports a missing arm), and an entry in each world's list (`TRAINER_SYSTEMS`, `OCEAN_SYSTEMS`, `BRAIN_ONLY_SYSTEMS`) at
its place in the order. A world rule: one field of `WorldSettings`, set by `Room::world_settings`
(trainer rooms), `WorldSettings::ocean` and the probes' `brain_world`.

## Config keys

None of its own. Systems read `brain.hebbianLearningRateBase`, section `metabolism` and section `physics` through
`TickCtx`; world settings are built from `metabolism.trainerMetabolicScale`, `trainerSpawnMinDistance` /
`trainerCurriculum` (rooms) and section `ocean` (grazing radius, photosynthesis, thermal stress, lifespan,
decomposition, biting, mitosis) ([[wiki/contracts/config-section]]).

## Events / IPC

`CreatureEvent` and `bite::BiteEvent` are in-process results for the owner. `OceanSim` turns them into
`OceanEvent`s, which `OceanMode` maps onto UI events (`EAT_EVENT`, `BITE_EVENT`, log lines; [[wiki/contracts/mode]]);
the systems emit nothing.

## Test recipe

`cargo test sim::` (selectors, eat radius, respawn, decomposition, bite, mitosis) and
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
// systems/mod.rs: `pub mod fatigue;`, and `System::Fatigue` after `System::Move` in the worlds that use it.
```
