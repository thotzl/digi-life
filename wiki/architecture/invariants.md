---
sources: [.agents/state/active-task.md, .agents/skills/server-simulation/SKILL.md, .agents/skills/project-baseline/SKILL.md, .agents/skills/testing-strategies/SKILL.md]
---

# Invariants (do not regress)

Rules verified against the code. Breaking one has caused real defects before.

1. **Mode is owned by the router.** `SET_MODE` is sent only by `navigateTo` in `frontend/core/router.ts`. `frontend/trainer.ts` must never send `SET_MODE: trainer` on load (it only resets to `ocean` on `unload`). Otherwise the Ocean is frozen on startup.
2. **Warp cap of 35 steps per frame.** `src/server/engine.rs` clamps both `ocean_warp_factor` and `trainer_warp_speed` to 35 steps per 60 Hz frame. Unbounded steps block the action channel; Pause stops responding.
3. **Reset clears the newborn queue.** `RESET_EVOLUTION` calls `newly_spawned_creatures.clear()`; otherwise the client re-adds stale creatures. Revisit when the handling of mitosis children changes (RV-B01).
4. **Thrust is scaled, never raw.** The motor output `out_thrust` (activation-dependent, typically `tanh` in `[-1, 1]`) goes through `step_creature_kinematics` in `src/shared/physics.rs`, which scales it by phenotype stiffness, pulse and `rules.thrustBaseMultiplier`. Do not pass raw activation as force. See [[wiki/simulation/physics]].
5. **Bundle icons stay registered.** `tauri.conf.json` → `bundle.icon` must list `icons/icon.ico` and `icons/icon.icns` (valid files, generated via `npx tauri icon`), or the Windows/macOS packagers fail.
6. **No DB work in the hot loop.** Keep SQLite access out of the per-step physics path; batch writes on timers or events (RV-P01). See [[wiki/architecture/simulation-engine]].
7. **Bindings are committed.** After changing a `#[derive(TS)]` struct run `cargo test` and commit `bindings/`; CI fails on drift. See [[wiki/architecture/ipc-protocol]].
8. **Golden master is intentional.** Update `GOLDEN_FINGERPRINT` only for intended phenotype/brain model changes and say so in the ticket. See [[wiki/quality/testing]].

## Key Takeaways

- Rules 1–3 protect the engine/UI coupling, 4 the physics model, 5 the release build, 6–8 performance and contracts.
- Rules 2 and 3 are not covered by tests (RV-X02).

Sources: `.agents/state/active-task.md`, `.agents/skills/server-simulation/SKILL.md`, `.agents/skills/project-baseline/SKILL.md`, `.agents/skills/testing-strategies/SKILL.md`.
