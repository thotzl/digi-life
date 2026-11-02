---
sources: [tickets/closed/TCK-198-r1-sim-context-config-utils.md, src/config/mod.rs, config.json]
---

# Contract: config section

## Purpose

`config.json` (repo root) is the single source of truth for every simulation tunable. It is split into one section
per subsystem; each section is a Rust struct that lives next to the code that reads it. A plugin (organ kind, room,
dynamic, mode) that needs tunables adds keys to an existing section or adds its own section. Current state and key
list: [[wiki/architecture/configuration]].

## Signature

```rust
// src/config/mod.rs
pub struct AppConfig {
    pub core: CoreConfig,             // src/config/mod.rs
    pub ocean: OceanConfig,           // src/server/ocean_config.rs
    pub trainer: TrainerConfig,       // src/biology/trainer_config.rs
    pub genome: GenomeConfig,         // src/biology/genome/mod.rs
    pub brain: BrainConfig,           // src/shared/brain.rs
    pub metabolism: MetabolismConfig, // src/biology/metabolism.rs
    pub physics: PhysicsConfig,       // src/shared/physics.rs
}
impl AppConfig {
    pub fn global() -> &'static Self; // entry points only
    pub fn load() -> Self;            // embedded config; debug builds: working-dir override
    pub fn embedded() -> Self;
}
#[cfg(test)] pub fn test_config() -> &'static AppConfig; // embedded config for unit tests
```

A section struct derives `Debug, Clone, PartialEq, Serialize, Deserialize` and carries
`#[serde(rename_all = "camelCase", deny_unknown_fields)]`. Fields are `pub`, snake_case in Rust, camelCase in JSON.
Nested blocks (`trainerCurriculum`, `genomeComposition`) are structs with the same attributes.

Code receives configuration as a parameter, never from a global:
- the whole `&AppConfig` where several sections are read (`init_rust_sandbox`, `step_trainer_sandbox_physics`, `sim::TickCtx::new`,
  `spawn_creature`, `Scenario::plugin`, `LevelScenario::at`, `TrainerParams::from_config`, `TrainerSim::new`);
- one section where only that one is read (`parse_genome(.., &BrainConfig)`, `step_creature_kinematics(..,
  &PhysicsConfig, ..)`, `start_energy(.., &MetabolismConfig)`, `food_energy(.., &MetabolismConfig)`).

## Lifecycle / call order

1. Entry point loads once: Tauri `setup` in `src/main.rs` (`AppConfig::global().clone()`, registered as managed state
   for the commands, moved into the engine thread by `spawn_simulation_thread`); the runner's `main` in
   `src/bin/trainer_runner.rs` (passed to `run_trainer`, `ProbeOptions::config`).
2. Debug builds read a `config.json` in the working directory instead of the embedded copy when it parses; an invalid
   one is logged and ignored. Release builds always use the embedded copy (`include_str!`).
3. The config is read-only for the whole process. `TrainerSim` stores a clone; probes clone and modify a copy (e.g.
   Hebbian rate 0) and pass the copy down.

## Invariants

- No `AppConfig::global()` below the entry points (`main.rs`, `trainer_runner.rs`); everything else gets the config
  passed in. Unit tests use `crate::config::test_config()`.
- No tunable is hardcoded in Rust; a section has no `Default` impl with values (a second source of values was RV-B04).
- Every key is read by production code and every key of `config.json` maps onto a field (tests below).
- A key changes its value only with an intended model change: values feed the golden master (`parse_genome` reads
  `brain`) and the trainer references (invariant 8 in [[wiki/architecture/invariants]]).

## Registration point

- New key: a `pub` field in the owning section struct (with a doc comment: meaning, unit, ticket) plus the key in the
  same section of `config.json`.
- New section: a struct next to its code, one field in `AppConfig`, one object in `config.json`, one row in the
  table of the `src/config/mod.rs` module doc and in [[wiki/architecture/configuration]].

## Config keys

This contract defines the mechanism; the keys per section are listed in [[wiki/architecture/configuration]].

## Events / IPC

None directly. `core.seed` is echoed in `INIT_STATE`; `trainer.mutationOpsLimit` in `TrainerState`.

## Test recipe

`cargo test config::tests` runs:
- `test_config_roundtrip_keeps_every_key`: `config.json` → `AppConfig` → JSON has exactly the key paths of the file.
- `test_every_config_key_has_read_site`: every field name of every section (taken from the struct via serde) appears
  as a field access `.field` in production code of `src/` (test modules, comments and strings stripped). A generic
  name (`standard`, `step`) can also match a same-named field of another type, so check new generic keys by hand.
- `test_unknown_config_key_is_rejected`: an unknown key in any section, or an unknown section, fails to parse.
- `test_embedded_config_matches_config_json`.
Then the full gate: `cargo test` (golden master) and the three trainer reference runs of `.github/workflows/ci.yml`.

## Minimal example

A room plugin with its own tunable in section `trainer` (key `arenaWallDamage`):

```rust
// src/biology/trainer_config.rs, struct TrainerConfig
/// Energy lost per wall contact in the arena room (TCK-xxx)
pub arena_wall_damage: f32,
```

```json
"trainer": { "...": "...", "arenaWallDamage": 0.5 }
```

```rust
// the reader gets the config passed in
fn wall_penalty(sb: &mut TrainerSandbox, config: &AppConfig) {
    sb.agent.energy -= config.trainer.arena_wall_damage;
}
```
