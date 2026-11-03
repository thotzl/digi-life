---
sources: [wiki/contracts/room.md, wiki/contracts/mode.md, src/trainer/room/registry.rs, src/server/engine/mod.rs]
---

# Guide: add a room or a mode

A room is one kind of training trial (arena, reproduction room, obstacle course); a mode is a top-level engine mode
(Ocean, trainer, creator). Read first: [[wiki/contracts/room]] (rooms) or [[wiki/contracts/mode]] (modes), and
[[wiki/architecture/invariants]]. A new scoring scheme is an evaluator: `wiki/guides/add-evaluator.md`.

## Add a room
1. `src/trainer/room/<name>.rs`: a struct implementing `Room` and `pub fn <name>(config: &AppConfig) -> Arc<dyn Room>`;
   `pub mod <name>;` in `src/trainer/room/mod.rs`. To reuse the small/standard room, wrap `LevelRoom::at(0.0, config)`
   and delegate (contract: Signature, "Reusing the level rooms").
2. `layout`: place every creature (`chamber.creatures`) and the food, drawing only from the given `rng`. More creatures
   per chamber: return the count from `entities()`, push the creatures and their `EntityTrial::default()` in `layout`.
3. `world_settings`: start from `LevelRoom::at(0.0, config).world_settings(config)`, change only the fields the room
   needs; obstacles via `obstacles()`, circles only (contract: "Obstacle model").
4. `score(chamber, entity)` from `chamber.trials[entity]` (catch units to compare with the level rooms); `is_done` only
   if the episode can end early.
5. Registration: one `RoomEntry` in `ROOMS` (`src/trainer/room/registry.rs`).
6. Tunables, including room geometry (size, ticks, positions, distances): keys in config section `trainer`
   (`wiki/guides/add-config-key.md`, invariant 11); extra RNG streams: a named id in `src/trainer/streams.rs`.
7. Tests in the room's module (copyable recipes: contract, Test recipe): layout deterministic per seed and complete,
   `score` on a hand-built chamber, one short `TrainerSim` run in the room.
8. Docs, same commit: the room in the contract (Purpose), the Room row of "Registered plugins" in
   `wiki/architecture/module-map.md`, `src/trainer/room/README.md` and the Rooms section of `wiki/simulation/trainer.md`.
A UI entry (training-mode select) is a separate frontend step and a `Scenario` variant; headless runs need neither.

## Add a mode
1. The simulation: an IO-free, seeded struct (own module, e.g. `src/sim/<name>/`) with a `step()` that returns what
   happened; no SQL, Tauri or clock inside (invariant 16).
2. IPC: `src/server/<name>_ipc.rs` with `<Name>Action` and `<Name>UiEvent` (`wiki/guides/add-ipc-message.md`); a
   variant in `ClientAction` and `UiEvent`, a `SimMode` variant (`src/server/ipc.rs`).
3. `src/server/engine/<name>_mode.rs`: a struct implementing `Mode` (`type Action = <Name>Action`), emitting only
   through `ctx.emit`, writing the store only per action, event or timer (invariant 6), warp through `clamp_warp`.
4. Registration (`src/server/engine/mod.rs`): a field in `Engine` built in `Engine::new`, one arm each in
   `Engine::handle` (actions, `SET_MODE` exit/enter) and `Engine::frame`; the compiler lists every missing arm.
5. Frontend: the view that shows the mode sets `backendMode: "<name>"` (`wiki/guides/add-view.md`).
6. Tests: an engine test with `VecSink` + `MemoryDb` in `src/server/engine/tests.rs` (enter/exit, an action, one frame,
   warp echo) and determinism tests of the simulation struct (same seed, same result).
7. Docs, same commit: the mode list in the contract (Purpose) and the Engine mode row of "Registered plugins" in
   `wiki/architecture/module-map.md`.

## Done when
- Room: the contract's test recipe passes (`cargo test trainer::`, incl.
  `test_every_registered_room_builds_under_its_id_and_ids_are_unique`), and
  `cargo run --release --features runner --bin trainer_runner -- run --seed s --generations 3 --scenario <name> --population 16`
  writes a report. Mode: `cargo test server::engine::` passes, incl. the new tests.
- The project gate is green, also for a Rust-only room (`npm test`, `npm run lint` and `npm run build` are part of
  it): `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`, `cargo test`, `npm test`,
  `npm run lint`, `npm run build`, `node scripts/check-links.mjs`, and the four CI reference runs match their
  committed reports (commands: `.agents/skills/plugin-authoring/SKILL.md`, Gate).
- Golden master and CI references unchanged (a new room or mode changes neither; changing `LevelRoom` does, invariant 8).
