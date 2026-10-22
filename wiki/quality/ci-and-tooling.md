---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, docs/revisit/07-testing-ci-tooling.md, tickets/closed/TCK-134-rust-toolchain-baseline-and-ci.md, tickets/closed/TCK-114-tauri-v2-github-actions-workflow-desktop-releases.md, tickets/closed/FIX-126-rebuild-loop.md, tickets/closed/FIX-126-B-taurignore.md, .github/workflows/sync-tickets.yml, .github/workflows/ci.yml, package.json, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-187-gene-spans-from-backend.md]
---

# CI, Toolchain and Releases

## Commands (repo root)

| Command | Purpose |
| :--- | :--- |
| `npm run tauri:dev` | launch the desktop app (Vite dev server + Rust build) |
| `npm run build` | `tsc` + Vite build into `dist/` |
| `cargo test` / `npm run test:rust` / `npm run types:generate` | Rust tests, regenerates `bindings/` |
| `cargo clippy --all-targets` | lint (baseline 69 warnings, TCK-134) |
| `npm run test` | Vitest (`frontend/shared/genome.test.ts`, TCK-187; not part of CI) |
| `npm run test:all` | Vitest + cargo test + `cargo check --bin trainer_runner` |
| `npm run sim:balance` | headless trainer run (release): `--seed baseline --generations 20 --scenario standard --population 16 --parallel true`, report to `.agents/artifacts/runner/balance.json` |
| `cargo run --release --bin trainer_runner -- run ...` | headless trainer run with JSON report ([[wiki/simulation/analytics-and-balancing]]) |
| `cargo run --release --bin trainer_runner -- probe <name>\|all --seed <s>` | mechanics probes (TCK-172), JSON to `.agents/artifacts/probes/` |

## Dev server

- Vite on port `8765` with `strictPort: true`; `tauri.conf.json` `devUrl` matches (FIX-126).
- `.taurignore` (`*.db`, `*.db-shm`, `*.db-wal`, `target/`, `dist/`, `node_modules/`) prevents the rebuild loop caused by the running app writing the DB (FIX-126-B; Tauri v2 has no `watch` key).

## Windows setup

1. rustup with `stable-msvc` toolchain.
2. Visual Studio C++ Build Tools (MSVC + Windows SDK).
3. Microsoft Edge WebView2 (preinstalled on Windows 10/11).
4. Node.js, then `npm ci` and `npm run tauri:dev`.

## Linux setup and troubleshooting

- Missing headers: `sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
- Stale webview assets: WebKitGTK caches under `~/.cache/webkitgtk-4.1`; enable "Disable Cache" in DevTools (`Ctrl+Shift+I`) or delete the cache.
- DB problems: [[wiki/architecture/persistence]].

## Workflows (`.github/workflows/`)

- `ci.yml` — push to master / PR / manual: `npm ci`, `npm run build`, `cargo test`, `git diff --exit-code -- bindings/` (TCK-134), then step "trainer reference run": `cargo run --release --bin trainer_runner -- run --seed ci-reference --generations 3 --scenario standard --population 16 --parallel true --out target/trainer-ci.json` and `node tests/reference/compare-trainer-report.mjs tests/reference/trainer-ci.json target/trainer-ci.json` (TCK-186). The comparator ignores `perf` and `meta.version`, prints the first 20 differing values and fails on any difference; an intended model change updates `tests/reference/trainer-ci.json` in the same commit.
- `release.yml` — only `v*` tags and manual dispatch; matrix macOS / Windows / Ubuntu 22.04 via `tauri-action`; draft release (TCK-114, TCK-134).
- `sync-tickets.yml` — on push to master/main: `.github/scripts/sync_github.cjs` mirrors `tickets/` to GitHub issues (`[TCK-…] title`, labels `priority:*`, `status:*`; closed folder = closed issue). Needs valid ticket frontmatter.
- `.gitattributes` pins `bindings/**` and `gen/**` to LF so the drift check is stable on Windows.

## Installers (end users)

- macOS `.dmg`/`.app` (WKWebView); Windows `.exe`/`.msi` (WebView2, downloaded if missing); Linux `.deb`/`.AppImage` (WebKitGTK; `chmod +x` for AppImage).

## Versioning

- One patch bump per closed ticket, synchronized in `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`/`Cargo.lock`, with a `CHANGELOG.md` entry referencing the ticket. `CHANGELOG.md` is the list of finished work.

## Toolchain state

- `rustc 1.99.1` stable-msvc (TCK-134).
- `Cargo.toml` package name `src-tauri`, binaries `src-tauri` (default) and `trainer_runner`; dependencies added in TCK-173: `rand_chacha 0.3` (seeded `ChaCha8Rng`), `rayon 1` (parallel trainer), `libm 0.2` (platform-identical math). `rusqlite 0.29` and `rand 0.8` are old; `typescript ^5`, `vite ^5`, `vitest ^1.6` with transitive deprecations (RV-T05).
- `types:generate` depends on all tests passing.

## Key Takeaways

- CI verifies build, tests, binding drift and the trainer reference report; releases only on tags.
- Tickets are mirrored to GitHub issues on every push to master.
- Every closed ticket = patch bump + changelog entry.

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/07-testing-ci-tooling.md`, TCK-114, TCK-134, TCK-173, TCK-186, TCK-187, FIX-126, FIX-126-B, `.github/workflows/`, `package.json`.
