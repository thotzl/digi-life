---
sources: [tickets/closed/TCK-195-windows-release-build.md, tickets/open/TCK-193-unified-product-naming.md, tickets/closed/TCK-194-trainer-tiles-show-all-origins-and-reset-reseeds.md, wiki/quality/ci-and-tooling.md]
---

# ADR-007: Release Identity, Separate Dev and Installed Data, Windows-First Builds

- Status: accepted
- Date: 2026-11-03
- Tickets: TCK-195, TCK-193, TCK-194

## Context

The user wanted to test a release version while development continued in the dev app. Both used the identifier `com.pixeldna.life` and therefore one data folder: two engines on one database, and a dev build with a new model version would reset the test version's save data. The product had four names (Pixel DNA Life, Pixel DNA, pixel_life, digi-life). Builds should follow the conventions of other games, Windows first, macOS and Linux later.

## Decision

1. **Product name BroodBattle** for the app, exe (`BroodBattle.exe`), window and page titles. The remaining technical names (crate, package, DB file, repo, whitepaper) follow in TCK-193.
2. **Identifier `de.pixlich.broodbattle` for the alpha only.** Before the beta (other users), a final domain is chosen and the identifier changed once, with a one-time copy of the alpha save folder; the alpha is then uninstalled by hand.
3. **Dev and installed app never share data.** `npm run tauri:dev` merges `tauri.dev.conf.json` with the identifier `<id>.dev` and the title "BroodBattle [DEV]" (invariant 10).
4. **Windows release: per-user NSIS installer and a portable ZIP** (`npm run release:win`), no MSI, no admin rights; a newer installer replaces the program and keeps the save data; uninstall keeps it too. No automatic updates. Platform-specific settings only under `bundle.windows`; the bundle target list stays platform-neutral so `release.yml` can build Linux (AppImage, `.deb`) and macOS (universal `.dmg`) later.
5. **Release builds log to a file** in the app data folder; the dev build logs to the console. A `config.json` next to the release exe is ignored.
6. **A trainer reset starts a new run** with a new seed stored in the run state, instead of replaying the same run (TCK-194).

## Consequences

- Testers install and update the alpha like any other game; save data survives updates (a model change resets trained creatures with a backup, ADR-005).
- The beta needs the identifier switch and a trademark search before release; code signing (SmartScreen warning) and macOS notarization are later steps.
- Local UI checks run against the dev data folder and back it up; checks of the installed app use their own debug port.
