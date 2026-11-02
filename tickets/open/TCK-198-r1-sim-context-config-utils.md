---
id: TCK-198
title: "R1 Explicit SimContext and Config Sections, Shared Utilities, Logging"
status: open
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
