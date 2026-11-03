---
sources: [.agents/skills/server-simulation/SKILL.md, .agents/skills/project-baseline/SKILL.md, .agents/skills/testing-strategies/SKILL.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md]
---

# Invariants (do not regress)

Rules verified against the code. Breaking one has caused real defects before.

1. **Mode is owned by the router.** The mode changes only through the `set_mode` command (`setMode` in `frontend/core/ipc.ts`), called only by `navigateTo` in `frontend/core/router.ts`; `handle_client_action` rejects a `SET_MODE` action and `send` does not accept one (TCK-157). The trainer view (`frontend/views/trainer/index.ts`) must never set the trainer mode on load (it only resets to `ocean` on `unload`). Otherwise the Ocean is frozen on startup. The backend does not change the mode on its own either: `CLIENT_READY` only answers with `INIT_STATE` (forcing Ocean there switched the trainer off after a fast navigation, TCK-186).
2. **Warp cap of 35 steps per frame.** `MAX_STEPS_PER_FRAME` (`src/server/engine/mode.rs`, TCK-156): `clamp_warp` caps the Ocean warp (`SET_SIMULATION_SPEED`) and the trainer warp (`warpSpeed`) when they are set, and the clamped value is echoed (`SIM_STATE.speed`, `TRAINER_STATE_CHANGED.warpSpeed`). Unbounded steps block the action channel; Pause stops responding. Test `test_warp_is_clamped_and_echoed_for_ocean_and_trainer`.
3. **Reset clears the newborn queue.** `RESET_EVOLUTION` clears `OceanMode::newborns`; otherwise the client re-adds stale creatures. Revisit when the handling of mitosis children changes (RV-B01). Test `test_reset_repopulates_from_the_catalogue_and_clears_newborns_and_snapshot`.
4. **Thrust is scaled, never raw.** The motor output `out_thrust` (activation-dependent, typically `tanh` in `[-1, 1]`) goes through `step_creature_kinematics` in `src/shared/physics.rs`, which multiplies it by the thrust that makes full thrust settle at the genome's target speed in `[physics.locomotionSpeedMin, physics.locomotionSpeedMax]` (`target_terminal_speed`, `thrust_for_terminal_speed`, TCK-150). Do not pass raw activation as force. See [[wiki/simulation/physics]].
5. **Bundle icons stay registered.** `tauri.conf.json` → `bundle.icon` must list `icons/icon.ico` and `icons/icon.icns` (valid files, generated via `npx tauri icon`), or the Windows/macOS packagers fail.
6. **No DB work in the hot loop.** Keep SQLite access out of the per-step physics path; batch writes on timers or events (RV-P01). See [[wiki/architecture/simulation-engine]].
7. **Bindings are committed.** After changing a `#[derive(TS)]` struct run `cargo test` and commit `bindings/`; CI fails on drift. See [[wiki/architecture/ipc-protocol]].
8. **Golden master is intentional.** Update `GOLDEN_FINGERPRINT` only for intended phenotype/brain model changes and say so in the ticket. The same holds for the reference reports checked in CI: `tests/reference/trainer-ci.json` (TCK-186), `trainer-ci-curriculum.json` and `trainer-ci-rehearsal.json` (TCK-197) and the Ocean reference `ocean-ci.json` (TCK-158). The simulation path uses `shared::fmath` for every transcendental function (no std `sin`/`cos`/`exp`/`powf`, TCK-202), so the references are platform-independent. See [[wiki/quality/testing]].
9. **Model change bumps `MODEL_VERSION`.** A change that makes stored genomes' phenotypes or synapse weights invalid (normally one that changes the golden master) bumps `MODEL_VERSION` in `src/persistence/schema.rs` in the same commit; the next start then resets the model tables instead of loading weights that no longer match (TCK-183). See [[wiki/architecture/persistence]].

10. **Dev and installed app never share a data folder.** `npm run tauri:dev` merges `tauri.dev.conf.json`, whose identifier is the release identifier plus `.dev`; the data folder (DB) follows the identifier. A dev build with a new `MODEL_VERSION` would otherwise reset the save data of the installed test version, and two engines would write one DB (TCK-195). See [[wiki/quality/ci-and-tooling]].

## Key Takeaways

- Rules 1–3 protect the engine/UI coupling, 4 the physics model, 5 the release build, 6–8 performance and contracts, 9 stored data across model changes, 10 the save data of an installed test version.
- Rules 2 and 3 are covered by the headless engine tests since TCK-156 (`server::engine::tests`).

Sources: `.agents/skills/server-simulation/SKILL.md`, `.agents/skills/project-baseline/SKILL.md`, `.agents/skills/testing-strategies/SKILL.md`, TCK-150, TCK-183, TCK-186.
