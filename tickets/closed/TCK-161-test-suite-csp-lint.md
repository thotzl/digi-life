---
id: TCK-161
title: Behavioural Test Suite, CSP and Lint Baseline
status: closed
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-T02, RV-F02 (CSP), RV-T05
---

# TCK-161: Behavioural Test Suite, CSP and Lint Baseline

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R4 (after TCK-160). CSP moved to TCK-197. RV-T02 partly resolved (property, config, brain-determinism tests). Scope:
- Vitest with `happy-dom` for view mount/unmount; controllers as pure event->state functions with tests; renderer pure helpers (`columnYs`), labels, warp mapping, IPC client.
- `cargo fmt --check`, `cargo clippy -D warnings` (baseline 69 -> 0), ESLint for `frontend/`; all in CI.
- CI job on Windows (shipping target) next to Ubuntu; `release.yml` uses `npm ci`, release name BroodBattle.
- Dead dev dependencies handled in TCK-189.


> Pool ticket: requires a user decision before implementation. Findings (RV-T02, RV-F02 (CSP), RV-T05) below.

## Description
Few behavioural tests; `csp: null`; 69 clippy warnings; outdated rusqlite/rand.

## Proposed Scope
Behavioural tests per RV-T02 list, restrictive CSP (verify inline styles), clippy clean-up, dependency updates after green baseline.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-T02 · P1** — tests check implementation details: `test_execute_brain_determinism` fed 4 inputs into a ≥ 7-input brain; DB tests test SQLite itself. Behavioural tests to add: (1) Ocean step: after mitosis `creatures.len()` +1 and the child moves next step; (2) brain: two independent runs bit-identical, output range per activation; (3) config: every key deserializes, no unknown keys; (4) `proptest`: `parse_genome` never panics on ASCII 0..1024 chars, `mutate_genome` keeps length in [128, 512]; (5) warp cap 35 as constant + test. Stryker only once frontend tests exist.
- **RV-F02 (CSP) · P1** — `tauri.conf.json` `"csp": null`; escaping done in TCK-141. Target: `default-src 'self'`. Relevant once genomes/catalogue entries are imported or shared.
- **RV-F07 (rest) · P3** — (`concurrently`, `tsx`, Stryker removed in TCK-189) `ActiveView` contains `"creator"` without a view.
- **RV-T05 (rest) · P2** — `Cargo.toml` package name `src-tauri`; `rusqlite 0.29`, `rand 0.8` outdated (update only after a green baseline); clippy baseline 69 warnings (TCK-134).

## Verification (2026-11-03, v1.12.39)
- Formatting: `rustfmt.toml` (`max_width = 120`, the width with the fewest changed lines), `cargo fmt` as a separate formatting-only commit (93 files).
- Clippy: 26 warnings (15 lib/test duplicates) -> 0 with `cargo clippy --all-targets --features runner -- -D warnings`; 10 auto-fixes, hand fixes in `mutate.rs`, `brain.rs`, `map_generator.rs` (`Mulberry32::next` -> `next_f32`), `metabolism.rs` docs, a `Tally` alias in the phenotype tests; one `#[allow(clippy::too_many_arguments)]` with its reason on `apply_creature_physics`. Behaviour unchanged (golden master and all references identical).
- ESLint: `eslint.config.js` (flat, typescript-eslint recommended), `npm run lint` = `eslint frontend --max-warnings 0`; 36 errors -> 0 (production `any` replaced by `unknown` or narrow casts; `_`-prefixed unused allowed; `no-explicit-any` off in tests).
- Tests: `frontend/views/views.dom.test.ts` (happy-dom, 8 tests) boots the real `VIEWS` through `startRouter` with a fake IPC: one creation per view, listeners and frame loops back to baseline after leaving Ocean/trainer/catalogue (fails if `Scope.listen` stops removing), Ocean dialog removed on leave, only the router sets the mode (invariant 1), Ocean and trainer controllers react to their events, no hyperparameters before the backend state; `core/ipc.test.ts` +1 (`onUiEvent`). Vitest 41 -> 50 tests (12 files); default environment node, DOM tests opt in per file.
- CI: job `verify` (Ubuntu) adds `cargo fmt --check`, `npm run lint`, clippy `-D warnings`; new job `verify-windows` (`npm ci`, build, `npm test`, `cargo test`, bindings drift); reference runs stay Ubuntu-only. `release.yml` uses `npm ci`, release name "BroodBattle <tag>".
- Gate on master (`.agents/artifacts/gate.sh`, now incl. fmt, clippy, lint): fmt ok, clippy ok, `cargo test` 279 passed, 1 ignored, golden master `0x6b78c31a4d19b0e8` unchanged, standard/curriculum/rehearsal/Ocean references match, `npm test` 50 passed, lint ok, build ok, doc links OK, no bindings drift.
- **Correction (post-fix v1.12.40):** the close of v1.12.39 stated "UI smoke OK", but the smoke run before the close had failed (no trainer generation within 90 s at max warp; the close script did not stop on it). It was flaky (about 1 of 3 runs). Root cause addressed: trainer controls sent the full hyperparameter set and, since TCK-160, nothing before the backend state arrived, so a speed change made early was dropped or overwritten by stale values. Now every control sends only its own field (`TrainerSettings.update`, partial `UPDATE_TRAINER_HYPERPARAMS`), and the trainer logs start, pause and warp changes. After the fix 10 consecutive smoke runs passed (one failure right after the change could not be reproduced again); the new log lines make a recurrence diagnosable. Gate after the fix: fmt, clippy, `cargo test` 279/1 ignored, 4 references, `npm test` 49 (two settings tests merged into one partial-update test), lint, build, links OK.
- Open: `npm audit` shows moderate dev-only findings (`minimatch` < 11.0.1 inside ESLint 9 / typescript-eslint; `npm audit --omit=dev` 0; documented in ci-and-tooling); happy-dom has no 2D canvas (renderer draw paths untested); the Windows CI job runs on the next push.
- Docs: testing, ci-and-tooling (commands, both CI jobs, release workflow, audit note), repository-map, README commands, skill `testing-strategies`.
