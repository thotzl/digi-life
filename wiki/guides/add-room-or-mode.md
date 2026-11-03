---
sources: [wiki/contracts/room.md, wiki/contracts/evaluator.md, src/trainer/room/registry.rs]
---

# Guide: add a room or a mode

A room is one kind of training trial (arena, reproduction room, obstacle course); a mode is a top-level engine mode
(Ocean, trainer, creator). Read first: [[wiki/contracts/room]] (rooms), [[wiki/contracts/evaluator]] (if the room
needs a new scoring scheme) and [[wiki/architecture/invariants]].

## Add a room

1. Add `src/trainer/room/<name>.rs` with a struct implementing `Room` and a constructor
   `pub fn <name>(config: &AppConfig) -> Arc<dyn Room>`; declare it in `src/trainer/room/mod.rs` (`pub mod <name>;`).
2. `layout`: place every creature (`chamber.creatures`) and the food, drawing only from the given `rng`. More than
   one creature per chamber: return the count from `entities()` and push the extra creatures and their
   `EntityTrial::default()` in `layout`, keeping both vectors aligned.
3. `world_settings`: start from `LevelRoom::at(0.0, config).world_settings(config)` and change only the fields the
   room needs ([[wiki/contracts/creature-system]] lists them); obstacles via `obstacles()`.
4. `score(chamber, entity)`: the creature's fitness from `chamber.trials[entity]` (catch units if it should compare
   with the level rooms); `is_done` only if the episode can end early.
5. Registration: one `RoomEntry` in `ROOMS` (`src/trainer/room/registry.rs`).
6. Tunables: keys in config section `trainer` (guide [[wiki/guides/add-config-key]]); no literals for balance values.
7. RNG: a stream beyond the layout `rng` uses a named id from `src/trainer/streams.rs`.
8. Tests in the room's module: layout deterministic per seed and complete, `score` on a hand-built chamber, and one
   short `TrainerSim` run with `set_mode(TrainingMode::Room("<name>"))`.
9. Docs: the room list in [[wiki/contracts/room]] (Purpose) and [[wiki/simulation/trainer]] (Rooms).

A UI entry for the room (training-mode select) is a frontend change and a `Scenario` variant; headless runs need
neither (`trainer_runner run --scenario <name>`).

## Add a mode

> Placeholder: the engine `Mode` seam (Ocean, trainer, creator) is defined by TCK-156 (engine decomposition). That
> ticket fills this section with its steps and contract.

## Done when

- `cargo test trainer::` passes, including `test_every_registered_room_builds_under_its_id_and_ids_are_unique`.
- `cargo run --release --features runner --bin trainer_runner -- run --seed s --generations 3 --scenario <name> --population 16`
  writes a report.
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green.
- Trainer references and probe JSON unchanged (a new room changes neither; changing `LevelRoom` does, invariant 8).
