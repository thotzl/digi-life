---
id: TCK-195
title: Windows Release Build (Installer and Portable), BroodBattle Alpha Identity
status: closed
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: Roadmap TCK-169 (tooling)
depends_on: TCK-114
---

# TCK-195: Windows Release Build (Installer and Portable), BroodBattle Alpha Identity

The user wants to test a release version while development continues in the dev app. Today both would share one data folder (`identifier` `com.pixeldna.life` → `%APPDATA%\com.pixeldna.life\pixel_life_local.db`): two engines on one DB, a dev build with a new `MODEL_VERSION` would wipe the test version's model tables, and the smoke scripts back up and overwrite that DB.

## Decisions (user, 2026-11-02)
- Windows first; macOS and Linux later without blocking the path (no Windows-only settings outside `bundle.windows`, data paths only via `app_data_dir`, `release.yml` unchanged).
- NSIS installer (per user, no admin) and a portable ZIP; no MSI. A newer installer only replaces the old version; save games stay. No automatic updates.
- Fresh install starts with an empty DB; the existing dev DB moves to the dev data folder.
- Product name and exe: **BroodBattle** (quick web check 2026-11-02: no game/app of that exact name; similar titles Brood, BroodStar on Steam, StarCraft: Brood War; no trademark search yet; do one before a public release). The rest of TCK-193 (crate, `package.json`, DB file name, docs, repo) stays in TCK-193.
- Backup of the DB before a model-version reset.

## Identifier (alpha only)
- Installed app: `de.pixlich.broodbattle`; dev app: `de.pixlich.broodbattle.dev`.
- **The domain is for the alpha only** (testing by the team). Before the beta (other people use it; the team decides at short notice when that is), a final domain must be chosen and the identifier changed once. Mandatory beta step: new identifier (dev = `<id>.dev`); on first start the beta copies the save data once from the alpha folder `%APPDATA%\de.pixlich.broodbattle\` (pattern `migrate_legacy_db`); beta notes tell testers to uninstall the alpha by hand (the installer does not see it as a predecessor under a new identifier).

## Scope
- **A. Separate dev and installed app:** `tauri.conf.json` `productName` / `mainBinaryName` / window title "BroodBattle", identifier `de.pixlich.broodbattle`; `tauri.dev.conf.json` (only `npm run tauri:dev`) with `de.pixlich.broodbattle.dev` and title "BroodBattle [DEV]"; one-time manual copy of the existing DB into the dev folder (old folder kept); smoke scripts use the dev folder; app version in the main menu; page titles in `index.html` / `frontend/main.ts` / main-menu heading.
- **B. Windows build:** bundle targets as a platform-neutral list (nsis + Linux/macOS formats for later), `bundle.windows.nsis.installMode` `currentUser`; `npm run release:win` builds the installer and a portable ZIP (exe + `LIESMICH.txt`, shares the save folder with the installed app).
- **C. Backup before a model reset:** `VACUUM INTO <db>.model-v<old>.bak.db` before the model tables are dropped; one per old version, never overwritten; a failed backup is logged and does not block the start.
- **D. Verification:** tests and builds green, golden master / CI reference unchanged; install, start, Ocean and trainer in the installed app while the dev app runs on its own data; update from v1.12.27 to a higher test build keeps a trainer run; portable ZIP starts on the same saves; uninstall keeps the saves.
- **E.** Wiki, CHANGELOG, v1.12.27.

## Later (not in this ticket)
- Linux (AppImage + `.deb`) and macOS (universal `.dmg`, later notarized) via the existing `.github/workflows/release.yml`.
- Code signing (SmartScreen "unknown publisher" until then).
- Beta identifier switch (section Identifier).

## Acceptance
- `npm run release:win` produces `BroodBattle_<version>_x64-setup.exe` and a portable ZIP; both start without a dev environment.
- Installed and dev app use separate data folders and can run at the same time.
- Installing a newer version over an older one keeps the save data; uninstalling keeps it too.
- A model-version reset leaves a readable backup of the old data.

## Verification (2026-11-02, v1.12.27)
- **Identity and separation:** `tauri.conf.json` `productName` / `mainBinaryName` "BroodBattle", identifier `de.pixlich.broodbattle`, window title "BroodBattle", bundle targets `["nsis", "deb", "appimage", "app", "dmg"]`, `bundle.windows.nsis.installMode` `currentUser`; `tauri.dev.conf.json` (identifier `.dev`, title "BroodBattle [DEV]") merged by `npm run tauri:dev`; page titles in `index.html` / `frontend/main.ts`, main-menu heading and version footer (`#app-version`, `getVersion()`) in `frontend/views/MainMenu.ts`. The dev DB (`%APPDATA%\com.pixeldna.life\`, incl. WAL) was copied once by hand to `%APPDATA%\de.pixlich.broodbattle.dev\`; the old folder is untouched. New invariant 10 (`wiki/architecture/invariants.md`).
- **Backup before a model reset:** `backup_before_model_reset` in `src/database.rs` (`VACUUM INTO`). Reproduction first: `test_model_reset_backs_up_the_old_data_first` failed before the fix with `SqliteFailure(... "no such table: trainer_genomes")` (no backup file; opening the path created an empty DB); green after the fix.
- **Build:** `npm run release:win` (`tauri build --bundles nsis` + `scripts/package-portable.mjs`) → `BroodBattle_1.12.27_x64-setup.exe` (3.3 MB), `BroodBattle_1.12.27_x64-portable.zip` (4.1 MB, exe + `scripts/portable/LIESMICH.txt`). Found during verification: Tauri bundled `trainer_runner.exe` too; it is now a `[[bin]]` with `required-features = ["runner"]` (callers in `package.json`, `ci.yml`, wiki, skill pass `--features runner`); the installed folder then holds only `BroodBattle.exe` and `uninstall.exe`.
- **Tests:** `cargo test` 182 passed, 1 ignored (existing manual measurement); `cargo check --features runner --bin trainer_runner` OK; CI trainer reference matches; golden master unchanged; `npm test` 10 passed; `npm run build` green.
- **Installed app** (`.agents/artifacts/uicheck/installed-check.sh` / `installed.cjs`, CDP port 9223, outputs `out-tck195*`): installed to `C:\BroodBattle` (user choice) while the dev app ran: title/heading "BroodBattle", footer "BROODBATTLE ALPHA · v1.12.27", Ocean 25 creatures, trainer pop 100 to generation 3, 0 console errors; fresh alpha DB in `%APPDATA%\de.pixlich.broodbattle\`, dev DB written in parallel in its own folder.
- **Update:** 1.12.99 test setup (`--config '{"version":"1.12.99"}'`, not committed) installed silently over 1.12.27: footer v1.12.99, uninstall entry 1.12.99, trainer run restored (generation 2), save folder unchanged. **Portable:** 1.12.27 ZIP started on the same saves (generation 2). **Uninstall** (silent): install folder, uninstall entry and Start-menu link removed, saves kept; `HKCU\Software\pixlich\BroodBattle` (remembered folder) stays (Tauri default).
- Notes: from Git Bash the installer switches need `MSYS_NO_PATHCONV=1`, otherwise `/S` is rewritten into a path and the installer runs interactively (cause of the first manual install/uninstall dialogs). The first manual install by the user went to `C:\BoodBattle`; its files remain (removal blocked by the agent's safety check, left to the user).
- Decision record: ADR-007 (`wiki/decisions/adr-007-release-identity.md`, 2026-11-03).
