---
sources: [HANDOFF.md, tickets/closed/TCK-142-docs-skills-ticket-sync.md, tickets/closed/TCK-134-rust-toolchain-baseline-and-ci.md, docs/revisit/00-executive-summary.md]
---

# Runtime Verification

What has been verified by actually running the app, as opposed to tests or static reading.

## Baselines

- Static review (2026-10-19, commit `998244e`, v1.12.2): `npm ci` OK, `tsc --noEmit` 0 errors, `vite build` OK (88 kB main bundle).
- Toolchain baseline (TCK-134): `rustc 1.99.1` stable-msvc; `cargo test` 34 passed; `cargo clippy --all-targets` 69 warnings, 0 errors.

## Smoke tests (2026-10-19, TCK-142)

- `npm run tauri:dev` starts; DB in `%APPDATA%\com.pixeldna.life\` with WAL; config loaded; handshake OK; 0 errors in the log.
- Second start restored 15 creatures from the AppData DB; trainer grid built (16 chambers); log still free of errors.

## Not verified at runtime

- Ocean, Trainer and Catalogue UI flows clicked through manually.
- An installed Windows release build.

## Key Takeaways

- App start, persistence location and state restore are verified at runtime.
- Manual UI flows and an installed release build are not.

Sources: `HANDOFF.md`, TCK-134, TCK-142, `docs/revisit/00-executive-summary.md`.
