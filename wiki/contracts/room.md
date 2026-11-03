---
sources: [tickets/closed/TCK-201-r3-rooms-evaluator-breeder.md, src/trainer/room/mod.rs, src/trainer/room/registry.rs, src/trainer/room/level.rs, src/trainer/chamber.rs, src/trainer/sandbox.rs, src/trainer/mode.rs, src/trainer/sim.rs]
---

# Contract: room

## Purpose

A room is one kind of training trial: chamber size, episode length, the rules of its world for the shared creature
tick, the start layout of creatures and food, the end of an episode and the score of a creature (TCK-201). The trainer
(`TrainerSim`), the headless runner and the diagnosis probes find rooms in one registry by id. Registered rooms:
`LevelRoom` (`src/trainer/room/level.rs`) as `standard` (level 0) and `exploration` (level 1), plus the curriculum's
levels in between (built directly, not registered); the current list is the Room row of "Registered plugins" in
[[wiki/architecture/module-map]]. Planned: an arena (several creatures per chamber,
opponents), a reproduction room (births during a trial), an obstacle course. A room holds no per-trial state; the
trainer owns a `Chamber` per sandbox and lends it to the room's hooks.

## Signature

```rust
// src/trainer/room/mod.rs
pub type EntityId = usize;                          // index into Chamber::creatures; 0 = the slot's creature
pub struct RoomTelemetry { pub level: f32 }
pub trait Room: Send + Sync + Debug {               // object-safe, held as Arc<dyn Room>
    fn id(&self) -> &str;                           // registry id
    fn chamber_size(&self) -> f32;                  // square chamber edge (px)
    fn episode_ticks(&self) -> u32;                 // ticks of one trial
    fn entities(&self) -> usize { 1 }               // creatures per chamber
    fn world_settings(&self, config: &AppConfig) -> WorldSettings;  // rules of the creature tick (contract creature-system)
    fn obstacles(&self) -> &[ProceduralObstacle] { &[] }
    fn layout(&self, chamber: &mut Chamber, rng: &mut SimRng);       // start pose of every creature + food
    fn is_done(&self, chamber: &Chamber) -> bool { chamber.tick >= self.episode_ticks() }
    fn score(&self, chamber: &Chamber, entity: EntityId) -> f32;     // fitness of one creature in this trial
    fn level(&self) -> f32 { 0.0 }                  // difficulty level (report `level`, UI)
    fn telemetry(&self, chamber: &Chamber) -> RoomTelemetry;         // default: { level }
}

// src/trainer/room/registry.rs
pub struct RoomEntry { pub id: &'static str, pub build: fn(&AppConfig) -> Arc<dyn Room>, pub summary: &'static str }
pub const ROOMS: &[RoomEntry];
pub fn build(id: &str, config: &AppConfig) -> Option<Arc<dyn Room>>;  // also entry, static_id, ids

// src/trainer/chamber.rs: what a room reads and writes
pub struct Chamber {
    pub size: f32, pub creatures: Vec<CreatureAgent>, pub trials: Vec<EntityTrial>,  // aligned per entity
    pub foods: Vec<FoodSpore>, pub tick: u32, pub catches: u32, pub respawn_seed: u64,
}
pub struct EntityTrial {   // reset at every trial start
    finished, finish_tick: Option<u32>, distance_traveled, wall_collisions, consumed_spore_type,
    sight_distance, approach_min_distance, accumulated_yield, consumed_count, cumulative_rotation,
    coverage_mask, starved, step: CreatureStep,
}

// src/trainer/mode.rs: how a room is selected
pub enum Scenario { Standard, Exploration, Curriculum }       // UI mode (bindings), room_id() -> registry id
pub enum TrainingMode { Room(&'static str), Curriculum }      // any registry room; parse("<id>"|"curriculum")
```

Obstacle model: `ProceduralObstacle` (`src/shared/map_generator.rs`) is used as a circle; only `x`, `y` and `radius`
are read (`vertices`, `obstacle_type`, `color` are ignored by the simulation). Three systems read `obstacles()`:
movement collision (`shared::physics::step_creature_kinematics`, push-out and bounce), the contact sense
(`organs::contact`, reefs as hard walls) and food drift (`step_food_spore_physics`, push-out). A rectangle or wall can
only be approximated by a chain of overlapping circles; a real polygon collision is a host change (stop and report).
Food respawn (`RespawnPolicy::MinDistance`, `sim::systems::eat::spawn_at_min_distance`) ignores obstacles and can
place food inside one (drift pushes it out next tick); obstacles are not part of the trainer telemetry
(`TrainerTelemetrySandbox`), so the trainer UI does not draw them.

Reusing the level rooms: wrap a `LevelRoom::at(level, config)` in the new room's struct and delegate (`chamber_size`,
`episode_ticks`, `world_settings`, `layout`, `score`); a new struct is needed because `LevelRoom`'s fields (`id`,
`spawn_margin`, `food_amount`, ...) are private to `level.rs`. Public: `food_per_type()`, `spawn_min_distance()`,
`metabolic_scale()`, `with_metabolic_scale`; the private ones come from config (`trainer.trainerSpawnMargin`,
`metabolism.foodSporeEnergy`), so read those keys directly. A wrapping `layout` may call `self.base.layout(chamber, rng)`
and then move or add things, drawing further numbers from the same `rng`.

## Lifecycle / call order

1. Selection: `TrainerParams::mode()` (UI `scenario`, or `room` for a registry room without a UI mode) →
   `registry::build(id, config)` once per trial room; curriculum trials build `LevelRoom::at(level)`.
   `TrainerSim::set_mode` / `set_scenario` switch at the next generation boundary.
2. Trial start (`Chamber::start_trial`, also `TrainerSandbox::spawn` for trial 0): the trainer resets every creature
   (at rest, start energy, age 0, neuron state cleared, learned weights kept) and every `EntityTrial`, sets
   `size = chamber_size()`, `tick = catches = 0`, then calls `layout(chamber, rng)` and draws `respawn_seed` from the
   same `rng`. In the trainer `rng` is the trial's shared layout stream, so every sandbox gets the same task.
3. Per tick (`TrainerSandbox::step` → `Chamber::step`): `tick += 1`; per entity in order the shared creature systems
   with `world_settings(config)` and `obstacles()`, then the trainer's bookkeeping (age, coverage, wall counter,
   approach tracking) and the catches (yield, first catch, respawn with `derive(respawn_seed, [catches])`).
4. Per tick for the UI and at the trial end: `score(chamber, 0)` (through the evaluator, [[wiki/contracts/evaluator]]).
5. `is_done(chamber)` true for every sandbox → the trial ends; the next trial starts at step 2.

## Invariants

- No state in the room; everything that changes during a trial lives in `Chamber`. Rooms are shared by all sandboxes
  of a trial (`Arc`), from parallel threads.
- `layout` draws only from its `rng`, in a fixed order; nothing in a room reads a global RNG, the clock or IO
  (epic principle 5). `score` is a pure function of the chamber.
- `layout` must place every creature and the food (the trainer clears neither positions nor food).
- Entity 0 is the creature of the sandbox's breeding slot; `score(chamber, 0)` is its fitness. More entities (prey,
  opponents) are added by the room in `layout` (`Chamber::creatures` / `trials` must stay aligned).
- New RNG streams of a room (e.g. births) use the named ids in `src/trainer/streams.rs` (`BIRTHS`, `PAIRING`).
- Room geometry and timing (chamber size, episode ticks, start positions, distances, score scales) are tunables:
  they come from config keys, never literals in Rust ([[wiki/architecture/invariants]] 11).
- Changing `LevelRoom` values, its layout order or `calculate_fitness` is a model change: trainer references and
  probe JSON change ([[wiki/architecture/invariants]] 8).

## Registration point

One entry in `ROOMS` (`src/trainer/room/registry.rs`) plus the room's module under `src/trainer/room/`
(`pub mod <name>;` in `room/mod.rs`). The runner flags `--scenario`, `--schedule` and `--eval-final` accept every
registered id; `TrainerSim::set_mode(TrainingMode::Room(id))` trains in it. A UI mode for the room is a separate,
frontend step (a `Scenario` variant and its select option) and not needed for headless use.

## Config keys

Rooms read their parameters from the config at `build`. `LevelRoom`: `trainer.trainerCurriculum` (`chamberSize`,
`epochTicks`, `foodPerType` per end), `trainer.trainerSpawnMinDistance`, `trainer.trainerFitness`, and in
`world_settings` `metabolism.trainerMetabolicScale` ([[wiki/contracts/config-section]]). A new room adds its keys to
section `trainer`.

## Events / IPC

None of its own. The UI telemetry (`TrainerTelemetrySandbox`) shows `chamber.size`, `telemetry(chamber).level`, the
creature and `chamber.foods`; the report carries `level()` per generation and in `final_eval`
([[wiki/contracts/ipc-message]]).

## Test recipe

Targeted subset (the guide's "Done when" is the gate): `cargo test trainer::` covers the registry
(`test_every_registered_room_builds_under_its_id_and_ids_are_unique` builds every entry), the level room and the
chamber tick. A new room adds tests in its module for its layout (deterministic for a seed, every creature placed),
`is_done` and `score`, and runs headless:
`cargo run --release --features runner --bin trainer_runner -- run --seed s --generations 3 --scenario <id> --population 16`.
A new room keeps the four CI references (commands: `.agents/skills/plugin-authoring/SKILL.md`, Gate) and the probe JSON
identical; only a change to `LevelRoom` or `calculate_fitness` changes them.

Layout test: build a sandbox as the helper `sandbox()` in the tests of `src/trainer/room/level.rs` does (copy it;
`TrainerSandbox::spawn` runs `layout` for trial 0) and inspect `sb.chamber`:

```rust
use std::sync::Arc;
use crate::biology::genome::WILDTYPE_GENOME;
use crate::shared::rng::seeded;
use crate::trainer::sandbox::{SandboxOrigin, TrainerSandbox};

fn sandbox(seed: u64) -> TrainerSandbox {
    let config = crate::config::test_config();
    TrainerSandbox::spawn(1, WILDTYPE_GENOME, 1, SandboxOrigin::Elite, sprint(config), None, None, &mut seeded(seed), config)
}
// same seed → same sb.chamber.creatures[i].px/py and sb.chamber.foods; sb.chamber.size == room.chamber_size()
```

`TrainerSim` run (pattern: `src/trainer/sim/tests.rs`). `TrainerParams::room` selects a registry room without a UI
mode from generation 1; `set_mode(TrainingMode::Room(id))` applies at once before the first tick and otherwise at the
next generation boundary:

```rust
use crate::trainer::sim::{TrainerEvent, TrainerParams, TrainerSim};

let config = crate::config::test_config();
let params = TrainerParams { population: 4, room: Some("sprint"), ..TrainerParams::from_config(config) };
let mut sim = TrainerSim::new(params, 7, 200, config);   // params, seed, hall-of-fame retention, config
assert_eq!(sim.room().id(), "sprint");
while !matches!(sim.step(false), Some(TrainerEvent::GenerationCompleted { .. })) {}
```

## Minimal example

```rust
// src/trainer/room/sprint.rs: one creature, no food, scored by distance travelled. Size, episode length and world
// rules are the small room's (config `trainer.trainerCurriculum`), so no tunable is a literal (invariant 11); own
// values would be new keys in section `trainer` (wiki/guides/add-config-key.md).
use std::sync::Arc;
use rand::Rng;
use crate::config::AppConfig;
use crate::shared::rng::SimRng;
use crate::sim::WorldSettings;
use crate::trainer::chamber::Chamber;
use super::{EntityId, LevelRoom, Room};

#[derive(Debug)]
pub struct SprintRoom {
    base: LevelRoom,
}

pub fn sprint(config: &AppConfig) -> Arc<dyn Room> {
    Arc::new(SprintRoom { base: LevelRoom::at(0.0, config) })
}

impl Room for SprintRoom {
    fn id(&self) -> &str { "sprint" }
    fn chamber_size(&self) -> f32 { self.base.chamber_size() }
    fn episode_ticks(&self) -> u32 { self.base.episode_ticks() }
    fn world_settings(&self, config: &AppConfig) -> WorldSettings {
        self.base.world_settings(config)   // the trainer's creature rules
    }
    fn layout(&self, chamber: &mut Chamber, rng: &mut SimRng) {
        let centre = self.chamber_size() / 2.0;
        for agent in &mut chamber.creatures {
            (agent.px, agent.py) = (centre, centre);
            agent.heading_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        }
        chamber.foods.clear();
    }
    fn score(&self, chamber: &Chamber, entity: EntityId) -> f32 {
        chamber.trials[entity].distance_traveled / self.chamber_size()
    }
}
// room/mod.rs: `pub mod sprint;`; registry.rs: RoomEntry { id: "sprint", build: super::sprint::sprint, summary: "..." }
```
