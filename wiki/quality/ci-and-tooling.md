---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, tickets/closed/TCK-134-rust-toolchain-baseline-and-ci.md, tickets/closed/TCK-114-tauri-v2-github-actions-workflow-desktop-releases.md, tickets/closed/FIX-126-rebuild-loop.md, tickets/closed/FIX-126-B-taurignore.md, .github/workflows/sync-tickets.yml, .github/workflows/ci.yml, package.json, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-187-gene-spans-from-backend.md, tickets/closed/TCK-189-dependency-security-alerts.md, tickets/closed/TCK-158-seedable-rng-headless-runner.md]
---

# CI, Toolchain and Releases

## Commands (repo root)

| Command | Purpose |
| :--- | :--- |
| `npm run tauri:dev` | launch the dev app (Vite dev server + Rust build) with `tauri.dev.conf.json`: identifier `de.pixlich.broodbattle.dev`, title "BroodBattle [DEV]", own data folder (TCK-195) |
| `npm run release:win` | Windows release build (TCK-195): `tauri build --bundles nsis` → `target/release/bundle/nsis/BroodBattle_<version>_x64-setup.exe`, then `scripts/package-portable.mjs` → `target/release/bundle/portable/BroodBattle_<version>_x64-portable.zip` (exe + `scripts/portable/LIESMICH.txt`) |
| `npm run build` | `tsc` + Vite build into `dist/` |
| `cargo test` / `npm run test:rust` / `npm run types:generate` | Rust tests, regenerates `bindings/` |
| `cargo clippy --all-targets` | lint (baseline 69 warnings, TCK-134) |
| `npm run test` | Vitest (`frontend/shared/genome.test.ts`, TCK-187; not part of CI) |
| `npm run test:all` | Vitest + cargo test + `cargo check --features runner --bin trainer_runner` |
| `npm run sim:balance` | headless trainer run (release): `--seed baseline --generations 20 --scenario standard --population 16 --parallel true`, report to `.agents/artifacts/runner/balance.json` |
| `cargo run --release --features runner --bin trainer_runner -- run ...` | headless trainer run with JSON report ([[wiki/simulation/analytics-and-balancing]]) |
| `cargo run --release --features runner --bin trainer_runner -- ocean --seed <s> --ticks <N>` | headless Ocean run with a time-series JSON report (TCK-158, [[wiki/simulation/analytics-and-balancing]]) |
| `cargo run --release --features runner --bin trainer_runner -- probe <name>\|all --seed <s>` | mechanics probes (TCK-172), JSON to `.agents/artifacts/probes/` |

## Dev server

- Vite on port `8765` with `strictPort: true`; `tauri.conf.json` `devUrl` matches (FIX-126).
- `.taurignore` (`*.db`, `*.db-shm`, `*.db-wal`, `target/`, `dist/`, `node_modules/`) prevents the rebuild loop caused by the running app writing the DB (FIX-126-B; Tauri v2 has no `watch` key).

## Windows setup

1. rustup with `stable-msvc` toolchain.
2. Visual Studio C++ Build Tools (MSVC + Windows SDK).
3. Microsoft Edge WebView2 (preinstalled on Windows 10/11).
4. Node.js, then `npm ci` and `npm run tauri:dev`.

## Release builds (TCK-195)

- Identity in `tauri.conf.json`: `productName` / `mainBinaryName` "BroodBattle" (`BroodBattle.exe`), identifier `de.pixlich.broodbattle`. **Alpha only:** before the beta (other users) a final domain is chosen and the identifier changed once, with a one-time copy of the alpha save folder (TCK-195 section Identifier). The dev app overrides the identifier (`tauri.dev.conf.json`, `.dev` suffix), so dev and installed app never share a data folder and can run at the same time.
- Bundle targets are a platform-neutral list (`nsis`, `deb`, `appimage`, `app`, `dmg`; Tauri builds only those of the current platform); Windows settings only under `bundle.windows` (`nsis.installMode` `currentUser`: per-user install to `%LOCALAPPDATA%\BroodBattle`, no admin, Start-menu entry, uninstall via Windows settings).
- A newer setup replaces the installed version in place; save data in `%APPDATA%\de.pixlich.broodbattle\` is untouched by install, update and uninstall. No automatic updates. The portable ZIP uses the same save folder.
- Unsigned: SmartScreen shows "unknown publisher" ("More info → Run anyway"). Code signing, Linux (AppImage + `.deb`) and macOS (universal `.dmg`) builds via `release.yml` are later steps (TCK-195 section Later).
- Release builds are optimised: trainer and Ocean run much faster than in the dev app; do not compare timings across the two.
- The headless runner is a dev tool behind the cargo feature `runner` (`[[bin]] trainer_runner`, `required-features = ["runner"]`), so `tauri build` neither builds nor bundles it; every runner call passes `--features runner`.
- Silent install/uninstall for scripted checks: `BroodBattle_<v>_x64-setup.exe /S /D=C:\BroodBattle`, `uninstall.exe /S` (from Git Bash with `MSYS_NO_PATHCONV=1`). The uninstaller keeps `HKCU\Software\pixlich\BroodBattle` (remembered install folder).

## Linux setup and troubleshooting

- Missing headers: `sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
- Stale webview assets: WebKitGTK caches under `~/.cache/webkitgtk-4.1`; enable "Disable Cache" in DevTools (`Ctrl+Shift+I`) or delete the cache.
- DB problems: [[wiki/architecture/persistence]].

## Workflows (`.github/workflows/`)

- `ci.yml` — push to master / PR / manual: `npm ci`, `npm run build`, `cargo test`, `git diff --exit-code -- bindings/` (TCK-134), then step "trainer reference run": `cargo run --release --features runner --bin trainer_runner -- run --seed ci-reference --generations 3 --scenario standard --population 16 --parallel true --out target/trainer-ci.json` and `node tests/reference/compare-trainer-report.mjs tests/reference/trainer-ci.json target/trainer-ci.json` (TCK-186). The comparator ignores `perf` and `meta.version`, prints the first 20 differing values and fails on any difference; an intended model change updates `tests/reference/trainer-ci.json` in the same commit. Step "trainer curriculum reference run" does the same for the curriculum room with final evaluation against `tests/reference/trainer-ci-curriculum; step "trainer rehearsal reference run" (TCK-197, `--seed ci-rehearsal --generations 4 --scenario curriculum --population 16 --rehearsal 0.25 --curriculum-upper 0.05 --curriculum-lower 0.02`) against `tests/reference/trainer-ci-rehearsal.json`.json` (flags in [[wiki/quality/testing]]); step "ocean reference run" runs `trainer_runner -- ocean --seed ci-ocean-2 --ticks 12000 --sample-every 500 --out target/ocean-ci.json` and compares it with the same comparator against `tests/reference/ocean-ci.json` (TCK-158; ≈ 3 s in release, 14.6 KB, 25 samples); step "doc link check" runs `node scripts/check-links.mjs`: wikilinks and frontmatter `sources:` in `wiki/`, skills and `tickets/open|ongoing`, plus backtick doc paths (`wiki/`, `tickets/`, `.agents/` `.md`) in the wiki and skills must resolve; optional `.agents/state/` notes are exempt (TCK-197).
- `release.yml` — only `v*` tags and manual dispatch; matrix macOS / Windows / Ubuntu 22.04 via `tauri-action`; draft release (TCK-114, TCK-134).
- `sync-tickets.yml` — on push to master/main: `.github/scripts/sync_github.cjs` mirrors `tickets/` to GitHub issues (`[TCK-…] title`, labels `priority:*`, `status:*`; closed folder = closed issue). Needs valid ticket frontmatter.
- `.gitattributes` pins `bindings/**` and `gen/**` to LF so the drift check is stable on Windows.

## Installers (end users)

- macOS `.dmg`/`.app` (WKWebView); Windows `.exe`/`.msi` (WebView2, downloaded if missing); Linux `.deb`/`.AppImage` (WebKitGTK; `chmod +x` for AppImage).

## Versioning

- One patch bump per closed ticket, synchronized in `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`/`Cargo.lock`, with a `CHANGELOG.md` entry referencing the ticket. `CHANGELOG.md` is the list of finished work.

## Toolchain state

- `rustc 1.99.1` stable-msvc (TCK-134).
- `Cargo.toml` package name `src-tauri`, binaries `src-tauri` (default) and `trainer_runner`; dependencies added in TCK-173: `rand_chacha 0.3` (seeded `ChaCha8Rng`), `rayon 1` (parallel trainer), `libm 0.2` (platform-identical math). `rusqlite 0.29` and `rand 0.8` are old; `typescript ^5`, `vite ^6.4`, `vitest` / `@vitest/coverage-v8` `^4.1` (TCK-189).
- `types:generate` depends on all tests passing.

## Dependencies and audits (TCK-189)

- Dev dependencies are only `@tauri-apps/cli`, `@vitest/coverage-v8`, `typescript`, `vite`, `vitest`; the unused `concurrently`, `tsx` and Stryker (and the `test:mutation` script) were removed. `npm audit`: 0 vulnerabilities (2026-11-02).
- Vite stays on major 6: Vite 7/8 need Node ≥ 20.19, the local dev machine runs Node 20.13.1 (CI uses `lts/*`). Vitest 4 supports Vite 6–8.
- npm 10.5.2 (bundled with Node 20.13) crashes (`Cannot read properties of null (reading 'edgesOut')`) when resolving a new Vitest 4 tree; `npm ci` from the existing lock file works. To change the lock file, use a newer npm, e.g. `npx -y npm@10.9.9 install`.
- `cargo audit` (cargo-audit 0.22): 0 vulnerabilities, 8 warnings: unmaintained `proc-macro-error` and `unic-*` (Tauri build tooling), unsound `glib 0.18.5` (RUSTSEC-2024-0429, Linux GTK stack via Tauri/wry, not used directly) and `tokio 1.53.1` (RUSTSEC-2026-0386, `AsyncFd`, not used directly). Accepted; no crate upgrades.

## Key Takeaways

- CI verifies build, tests, binding drift, the three trainer reference reports and the Ocean reference report; releases only on tags.
- Tickets are mirrored to GitHub issues on every push to master.
- Every closed ticket = patch bump + changelog entry.

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, TCK-114, TCK-134, TCK-158, TCK-173, TCK-186, TCK-187, TCK-189, FIX-126, FIX-126-B, `.github/workflows/`, `package.json`.
