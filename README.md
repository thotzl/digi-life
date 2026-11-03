# BroodBattle

A native 2D biomorphic ecosystem and neuroevolution trainer: creatures defined by DNA, driven by a CTRNN brain,
bred in training rooms and an open Ocean. Desktop app built with **Tauri v2**, **Rust 2024**, **SQLite** and
**Preact Signals** (repository `digi-life`).

Status and work plan: [roadmap epic TCK-169](tickets/ongoing/TCK-169-roadmap-epic.md).

## Commands

All commands run from the repository root (`Cargo.toml` and `package.json` both live there).

| Command | Action |
| :--- | :--- |
| `npm run tauri:dev` | launch the desktop app (Vite dev server + Rust build) |
| `npm run build` | type-check and build the frontend (`tsc` + Vite into `dist/`) |
| `cargo test` | Rust tests (genome compiler, brain, engine, persistence, ...); regenerates `bindings/` |
| `npm test` | Vitest frontend tests (`frontend/**/*.test.ts`) |
| `npm run lint` | ESLint on `frontend/` (zero warnings) |
| `cargo fmt --check` / `cargo clippy --all-targets --features runner -- -D warnings` | Rust format check / lints |

### Windows developer setup
1. Install Rust via [rustup](https://rustup.rs) with the `stable-msvc` toolchain.
2. Install the **Visual Studio C++ Build Tools** (MSVC + Windows SDK).
3. Ensure **Microsoft Edge WebView2** is present (preinstalled on Windows 10/11).
4. Install Node.js, then `npm ci` and `npm run tauri:dev`.

Linux setup, troubleshooting, CI and releases: [`wiki/quality/ci-and-tooling.md`](wiki/quality/ci-and-tooling.md).

## Installers

Installers are built by GitHub Actions when a `v*` tag is pushed; `npm run release:win` builds the Windows installer
and a portable ZIP locally. End users need no developer dependencies.

## Documentation

- Wiki: [`wiki/_master-index.md`](wiki/_master-index.md) (open the repo root as an Obsidian vault to browse wiki, README and tickets together).
- Adding a feature: one guide and one contract per extension point, listed in the wiki index; rules in [`wiki/architecture/invariants.md`](wiki/architecture/invariants.md).
- Tickets: [`tickets/`](tickets/). Release notes: [`CHANGELOG.md`](CHANGELOG.md). Agent rules: [`AGENTS.md`](AGENTS.md).
