# Active Task & Safety Ledger

## Current Focus
Revisit **Phase 0 — Stabilize** is implemented (TCK-134..142, TCK-162); next: user decisions on pool tickets TCK-143..161. Plan, step order and Definition of Done: `docs/revisit/roadmap.md`. Findings by ID (`RV-*`): `docs/revisit/01-critical-bugs.md` ff. Tickets: `.tickets/open/TCK-134` .. `TCK-142`.

No feature ticket (TCK-111, TCK-113, TCK-118, TCK-123) starts before Phase 0 is complete. Each phase ends with a hard stop and explicit user approval.

---

## Safety Rules (verified against code; do not regress)

1. **Mode is owned by the router.**
   `SET_MODE` is sent only by `navigateTo` in `frontend/core/router.ts`. `frontend/trainer.ts` must never send `SET_MODE: trainer` on load (it only resets to `ocean` on `unload`). Otherwise the Ocean is frozen on startup.

2. **Warp cap of 35 steps per frame.**
   `src/server/engine.rs` clamps both `ocean_warp_factor` and `trainer_warp_speed` to 35 steps per 60 Hz frame. Unbounded steps block the action channel; Pause stops responding. Target: constant + test (RV-X02).

3. **Reset clears the newborn queue.**
   `RESET_EVOLUTION` in `engine.rs` calls `newly_spawned_creatures.clear()`; otherwise the client re-adds stale creatures. Revisit when RV-B01 changes how children are added.

4. **Thrust is scaled, never raw.**
   The motor output `out_thrust` (activation-dependent, typically `tanh` in `[-1, 1]`) goes through `step_creature_kinematics` in `src/shared/physics.rs`, which scales it by phenotype stiffness, pulse and `rules.thrustBaseMultiplier` from `config.json`. Do not pass raw activation as force.

5. **Bundle icons stay registered.**
   `tauri.conf.json` -> `bundle.icon` must list `icons/icon.ico` and `icons/icon.icns` (valid files, generated via `npx tauri icon`), or the Windows/macOS packagers fail.
