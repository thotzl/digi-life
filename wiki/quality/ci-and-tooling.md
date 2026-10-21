---
sources: [README.md, .agents/skills/project-baseline/SKILL.md, docs/revisit/07-testing-ci-tooling.md, .tickets/closed/TCK-134-rust-toolchain-baseline-and-ci.md, .github/workflows/sync-tickets.yml, package.json]
---

# CI, Toolchain and Releases

## Commands (repo root)

| Command | Purpose |
| :--- | :--- |
| `npm run tauri:dev` | launch the desktop app (Vite dev server on port 8765 + Rust build) |
| `npm run build` | `tsc` + Vite build into `dist/` |
| `cargo test` / `npm run test:rust` / `npm run types:generate` | Rust tests, regenerates `bindings/` |
| `cargo clippy --all-targets` | lint (baseline 69 warnings, TCK-134) |
| `npm run test` | Vitest (no test files yet) |
| `npm run test:all` | Vitest + cargo test + `cargo check --bin cli_trainer` |
| `npm run sim:balance` | outdated `cli_trainer` (RV-S11, do not trust) |

## Windows setup

1. rustup with `stable-msvc` toolchain.
2. Visual Studio C++ Build Tools (MSVC + Windows SDK).
3. Microsoft Edge WebView2 (preinstalled on Windows 10/11).
4. Node.js, then `npm ci` and `npm run tauri:dev`.

## Linux setup and troubleshooting

- Missing headers: `sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.
- Stale webview assets: WebKitGTK caches under `~/.cache/webkitgtk-4.1`; enable "Disable Cache" in DevTools (`Ctrl+Shift+I`) or delete the cache.
- DB problems: [[architecture/persistence]].

## Workflows (`.github/workflows/`)

- `ci.yml` — push to master / PR / manual: `npm ci`, `npm run build`, `cargo test`, `git diff --exit-code -- bindings/` (TCK-134).
- `release.yml` — only `v*` tags and manual dispatch; matrix macOS / Windows / Ubuntu 22.04 via `tauri-action`; draft release.
- `sync-tickets.yml` — on push to master/main: `.github/scripts/sync_github.cjs` mirrors `.tickets/` to GitHub issues (`[TCK-…] title`, labels `priority:*`, `status:*`; closed folder = closed issue). Needs valid ticket frontmatter.
- `.gitattributes` pins `bindings/**` and `gen/**` to LF so the drift check is stable on Windows.

## Installers (end users)

- macOS `.dmg`/`.app` (WKWebView); Windows `.exe`/`.msi` (WebView2, downloaded if missing); Linux `.deb`/`.AppImage` (WebKitGTK; `chmod +x` for AppImage).

## Versioning

- One patch bump per closed ticket, synchronized in `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`/`Cargo.lock`, with a `CHANGELOG.md` entry referencing the ticket. Release notes: `CHANGELOG.md` (kept outside the wiki).

## Toolchain debt (RV-T05, rest in TCK-161)

- `Cargo.toml` package name `src-tauri`, version mismatch history; `rusqlite 0.29` and `rand 0.8` outdated — update only after a green baseline.
- `typescript ^5`, `vite ^5`, `vitest ^1.6`; transitive deprecations, no urgent action.
- `types:generate` depends on all tests passing.

## Key Takeaways

- CI verifies build, tests and binding drift; releases only on tags.
- Tickets are mirrored to GitHub issues on every push to master.
- Every closed ticket = patch bump + changelog entry.

Sources: `README.md`, `.agents/skills/project-baseline/SKILL.md`, `docs/revisit/07-testing-ci-tooling.md`, TCK-134, `.github/workflows/`, `package.json`.
