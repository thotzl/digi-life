---
sources: [HANDOFF.md, tickets/closed/TCK-142-docs-skills-ticket-sync.md, tickets/closed/TCK-134-rust-toolchain-baseline-and-ci.md, docs/revisit/00-executive-summary.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md]
---

# Runtime Verification

What has been verified by actually running the app, as opposed to tests or static reading.

## Baselines

- Static review (2026-10-19, commit `998244e`, v1.12.2): `npm ci` OK, `tsc --noEmit` 0 errors, `vite build` OK (88 kB main bundle).
- Toolchain baseline (TCK-134): `rustc 1.99.1` stable-msvc; `cargo test` 34 passed; `cargo clippy --all-targets` 69 warnings, 0 errors.

## Smoke tests (2026-10-19, TCK-142)

- `npm run tauri:dev` starts; DB in `%APPDATA%\com.pixeldna.life\` with WAL; config loaded; handshake OK; 0 errors in the log.
- Second start restored 15 creatures from the AppData DB; trainer grid built (16 chambers); log still free of errors.

## Phase smoke checks (TCK-186)

- Checklist per phase of the training epic: [[wiki/quality/testing]]. The UI part runs in the dev app (`tauri dev`) driven via WebView2 remote debugging (local agent tooling, not in the repo), with the app DB backed up and restored.
- P2 of TCK-171 (2026-10-22): Ocean 15 → 16 creatures after inject, no NaN; trainer generations completed, pause gave 0 generation completions in 8 s, reset returned to generation 1; 0 console errors. Found and fixed on the way: `styled` `class` props were never applied ([[wiki/frontend/structure]]); `CLIENT_READY` switched the trainer off after a fast navigation ([[wiki/architecture/simulation-engine]]).

## Not verified at runtime

- Catalogue UI flows; trainer flows beyond the smoke checklist.
- An installed Windows release build.

## Key Takeaways

- App start, persistence location and state restore are verified at runtime.
- Ocean and trainer core flows are smoke-checked per phase (TCK-186); catalogue flows and an installed release build are not.

Sources: `HANDOFF.md`, TCK-134, TCK-142, TCK-186, `docs/revisit/00-executive-summary.md`.
