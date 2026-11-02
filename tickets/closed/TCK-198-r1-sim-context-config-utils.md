---
id: TCK-198
title: "R1 Explicit SimContext and Config Sections, Shared Utilities, Logging"
status: closed
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R1
depends_on: TCK-197
---

# TCK-198: Explicit SimContext and Config Sections, Shared Utilities, Logging

Foundation every plugin builds on: a plugin gets its config and helpers passed in, it never reaches into globals. Touches many files, so it runs alone (no parallel slice).

## Findings (2026-11-02 review)
- `AppConfig::global()` (`OnceLock`, `src/shared/types.rs:341-347`) is read at ~60 sites, inside the model too: `dna.rs:1664` (`parse_genome` reads `thalamus_sum_activation`, so the golden master depends on config.json), `physics.rs:244` (ignores the `app_config` passed in), `creature_factory.rs:52`, `trainer_engine.rs:129/162/330/551`, `scenarios.rs:53/189/419/487`, `trainer_sim.rs:213/566/635/752/831/957/1028`. Runner and tests cannot inject a config.
- `types.rs` mixes IPC DTOs (7-114), config schema (116-338), loader (340-368).
- The read-site test (`types.rs:411-441`) is a substring search; generic keys (`step`, `upper`, `enabled`, `standard`) pass trivially. `TrainerSpawnMinDistance` duplicates `RoomPair<f32>`.
- The release build reads a `config.json` override from the working directory (`types.rs:352`).
- Repeated helpers: `SystemTime…UNIX_EPOCH` 18× with `now_millis` holding seconds (api.rs:153/193, engine.rs:260/390/538/593/678/715/1183/1311/1398; real ms at engine.rs:1489, api.rs:371; RV-A06); angle wrap loops (`brain.rs:294-298`, `scenarios.rs:364`, `probes.rs:229`); distance ~14×; `spatial_grid.rs:53-97` two copied query fns allocating a `Vec`; ring-spawn code 3× (`engine.rs:283/747/1065`); JSON vector columns decoded with `unwrap_or_default` (`database.rs:412`, `api.rs:233/326`); a second RNG/hash family (`map_generator.rs:129-156` `Mulberry32`, `hash_string_to_int`).
- 35 `println!`/`eprintln!` with hand-written tags; in the Windows release build (`windows_subsystem = "windows"`) all logs are lost.

## Scope
1. Split `types.rs` into `shared/dto.rs` (IPC structs), `config/` (schema per section + loader).
2. Config sections in `config.json` (single source of truth stays): `core`, `ocean`, `trainer`, `genome`, `brain`, `metabolism`, `physics`, ... Each module owns its `XConfig` (`Deserialize + Default`, `deny_unknown_fields`) next to its code; `AppConfig` is a plain struct with one field per section. The JSON layout changes once; keys keep their names inside the sections.
3. `SimContext { config: &AppConfig, ... }` (or the section refs) passed explicitly into `parse_genome`, physics, factory, trainer, scenarios; `AppConfig::global()` remains only at entry points (Tauri setup, engine thread start, runner). Working-directory override only under `cfg(debug_assertions)`.
4. Key test via serde: deserialize → serialize → compare key sets, plus "every section key is read" through the struct (no substring search).
5. `shared/geom.rs` (`dist`, `dist_sq`, `wrap_angle`, `polar_offset`), `shared/time.rs` (`now_secs`, `now_ms`, names match units), `shared/json_col.rs` (encode/decode with logged failure), generic `SpatialGrid::for_each_nearby(bucket, ..)` without allocation.
6. Logging: `log` facade + a file logger in the app data dir (rotating, small) for release builds, console in dev; replace the tagged prints.
7. Keep `Mulberry32` only if replacing it would change world generation; otherwise route map generation through `SimRng` (world layout change → state it; Ocean is not reference-tested yet).

## Gate
Golden master and both trainer references unchanged (config values identical, only access paths change). `cargo test`, vitest, build green.

## Acceptance
- `grep AppConfig::global` only in entry-point files; `parse_genome` and every system take config explicitly.
- Contract doc `wiki/contracts/config-section.md` (how a plugin adds its section and keys) written in this ticket.
- Release build writes a log file; UI smoke + installed check green.

## Verification (2026-11-02, v1.12.30)
- Module map: `src/shared/types.rs` removed; IPC structs in `src/shared/dto.rs`; loader, `AppConfig` (one field per section) and config tests in `src/config/mod.rs`; section structs next to their code (`OceanConfig` `src/server/ocean_config.rs`, `TrainerConfig` + fitness/curriculum/minimal-criterion/enums `src/biology/trainer_config.rs` with `RoomPair<f32>` replacing `TrainerSpawnMinDistance`, `GenomeConfig` in `dna.rs`, `BrainConfig` in `brain.rs`, `PhysicsConfig` in `physics.rs`, `MetabolismConfig` with food/bite/start-energy keys).
- Config passed explicitly: `parse_genome(.., &BrainConfig)`, kinematics/physics `&PhysicsConfig`, factory, trainer sandbox, breeding, scenarios, `TrainerSim` (stores a clone), report/runner, Tauri commands via `State<AppConfig>`, engine thread. `AppConfig::global()` only in `src/main.rs` (Tauri setup) and `src/bin/trainer_runner.rs` (main). Working-directory override only under `cfg(debug_assertions)`.
- `config.json` in 7 sections (`core`, `ocean`, `trainer`, `genome`, `brain`, `metabolism`, `physics`), key names unchanged: 94/94 leaf values identical. Read-site test reads key names via serde and checks token-level field accesses (limitation for generic names documented in the contract). Section structs have no value `Default` on purpose (no second value source); tests use `test_config()`.
- Helpers: `shared/geom.rs` (`dist`, `dist_sq`, `wrap_angle`, `polar_offset`, bit-identical expressions), `shared/time.rs` (`now_secs`, `now_ms`, `now_nanos`; RV-A06 names), `shared/json_col.rs` (logged decode failures), `SpatialGrid::for_each_nearby` (no allocation, same order), Ocean ring-spawn helper. `Mulberry32` kept (routing through `SimRng` would change every world layout; note in `map_generator.rs`).
- Logging: `log` facade + `src/logging.rs` (stderr in dev/runner; release app writes `<app_data_dir>/logs/broodbattle.log`, rotation at 1 MiB, one backup); 31 tagged prints replaced. Release exe check: log file written (config source, DB path, handshake, mode switch, unknown action warning).
- Gate on master (`.agents/artifacts/gate.sh`): `cargo test` 188 passed, 1 ignored; golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum and rehearsal references match; `npm test` 13 passed; build ok; doc links OK (82 files); no bindings drift. Dev and release builds without warnings.
- UI smoke (dev) OK: Ocean, inject, trainer generation (RTT 8 ms at max warp), metrics, curriculum, room switch, pause, reset; 0 console errors. Release exe (`installed.cjs`): menu v1.12.29, Ocean, trainer to generation 3, 0 console errors. Smoke scripts now match the new log line "generation N completed".
- Docs: `wiki/contracts/_template.md`, `wiki/guides/_template.md` (TCK-203 skeleton), `wiki/contracts/config-section.md`, `wiki/guides/add-config-key.md`; `configuration.md` rewritten to current state; invariants #4, persistence, system-overview, repository-map, testing, physics, trainer, master index, project-baseline updated.
