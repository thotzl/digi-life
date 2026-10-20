# Handoff: Pixel DNA Life

## Where to start
- **Plan:** `docs/revisit/roadmap.md` — Phase 0 (stabilize) is active; hard stop and user approval before Phase 1.
- **Critical bugs:** `docs/revisit/01-critical-bugs.md` (IDs `RV-B*`). Further findings in `docs/revisit/02`..`08`.
- **Tickets:** Phase 0 (TCK-134..142, TCK-162) is closed. Out-of-scope items are pool tickets TCK-143..161 (`priority: pool`, decision pending).
- **Agent context:** `AGENTS.md` -> `.agents/skills/`, current focus and safety rules in `.agents/state/active-task.md`.

## Open diagnosis: motionless Ocean
The earlier hypothesis (render time scale in `frontend/ocean.ts`) is superseded. Two findings explain the symptom better:
- **RV-B01 (P0, deliberately deferred):** mitosis children are pushed only to `newly_spawned_creatures` in `src/server/engine.rs` and never enter the simulated creature list. The client adds and immediately removes them. The Ocean has no surviving offspring; population comes only from restocking.
- **RV-S02 (P1, hypothesis):** thrust in `src/shared/physics.rs` scales with `stiffness * (pulse * 1000)^2 * thrustBaseMultiplier`; for low-pulse/low-stiffness genomes terminal speed is near zero. Measure the speed distribution over random genomes headlessly before changing constants.

## Not yet verified at runtime
Baseline recorded in TCK-134. `tauri dev` smoke test (2026-10-19): app starts, DB in `%APPDATA%m.pixeldna.life` with WAL, handshake OK, no errors in the log. Ocean/Trainer UI flows still to be clicked through by the user.
