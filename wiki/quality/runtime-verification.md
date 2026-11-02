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

## Installed Windows build (TCK-195, 2026-11-02, v1.12.27)

- `npm run release:win` → NSIS setup (3.3 MB) and portable ZIP; the setup contains only `BroodBattle.exe` and `uninstall.exe` (`trainer_runner` is feature-gated).
- Installed to `C:\BroodBattle` while the dev app ran on its own data: page `http://tauri.localhost/`, title and heading "BroodBattle", footer "BROODBATTLE ALPHA · v1.12.27"; Ocean 25 creatures; trainer pop 100 reached generation 3; 0 console errors; fresh alpha DB in `%APPDATA%\de.pixlich.broodbattle\`, dev DB in the `.dev` folder written at the same time.
- Update: a 1.12.99 test setup installed silently over 1.12.27 → footer v1.12.99, Windows lists 1.12.99, trainer run restored (generation 2), save folder unchanged. The portable 1.12.27 ZIP then started on the same saves (generation 2).
- Silent uninstall removed the install folder, the Windows uninstall entry and the Start-menu link; the save folder stayed. `HKCU\Software\pixlich\BroodBattle` (remembered install folder and language) stays, as the Tauri NSIS uninstaller does by default.
- Agent tooling: `installed-check.sh` / `installed.cjs` (CDP port 9223, local, not in the repo). In Git Bash pass installer switches with `MSYS_NO_PATHCONV=1` (`/S`, `/D=...`), otherwise Bash rewrites them into paths and the installer opens its dialogs.

## Not verified at runtime

- Catalogue UI flows; trainer flows beyond the smoke checklist.
- Linux and macOS release builds (TCK-195 section Later).

## Key Takeaways

- App start, persistence location and state restore are verified at runtime.
- Ocean and trainer core flows are smoke-checked per phase (TCK-186); the installed Windows build, update and uninstall are verified (TCK-195); catalogue flows and Linux/macOS builds are not.

Sources: `HANDOFF.md`, TCK-134, TCK-142, TCK-186, `docs/revisit/00-executive-summary.md`.
