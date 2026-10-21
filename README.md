# 🧬 Pixel DNA Life - Native Desktop Lab

A native 2D biomorphic ecosystem and neuroevolution trainer, built as a desktop application with **Tauri v2**, **Rust 2024**, **SQLite**, and **Preact Signals**.

Current status and work plan: [`wiki/roadmap.md`](wiki/roadmap.md).

---

## 🚀 Execution Commands

All commands are executed from the **repository root directory** (`Cargo.toml` and `package.json` both live there).

| Command | Action |
| :--- | :--- |
| `npm run tauri:dev` | **Launch the desktop application** (Vite dev server + Rust build) |
| `npm run build` | **Type-check and build the frontend** (`tsc` + Vite into `dist/`) |
| `cargo test` | **Run Rust unit tests** (DNA compiler, CTRNN brain, database schema); also regenerates `bindings/` |
| `npm run types:generate` | Alias for `cargo test` (ts-rs export) |
| `npm run test` | Vitest (configured; no frontend test files exist yet) |

### Windows developer setup
1. Install Rust via [rustup](https://rustup.rs) with the `stable-msvc` toolchain.
2. Install the **Visual Studio C++ Build Tools** (MSVC + Windows SDK).
3. Ensure **Microsoft Edge WebView2** is present (preinstalled on Windows 10/11).
4. Install Node.js, then `npm ci` and `npm run tauri:dev`.

Linux setup, troubleshooting, CI and releases: [`wiki/quality/ci-and-tooling.md`](wiki/quality/ci-and-tooling.md).

---

## 🖥️ Installers

Installers (`.dmg`, `.exe`/`.msi`, `.deb`/`.AppImage`) are built by GitHub Actions when a `v*` tag is pushed. End-users need no developer dependencies.

---

## 📚 Documentation

The project wiki in [`wiki/`](wiki/_master-index.md) holds all project knowledge (Obsidian-style links):

| Topic | Content |
| :--- | :--- |
| [Roadmap](wiki/roadmap.md) | phases, current state, ordered tickets, non-goals |
| [Overview](wiki/overview/_index.md) | product vision (whitepaper), glossary |
| [Architecture](wiki/architecture/_index.md) | runtime structure, engine, IPC, persistence, config, invariants, performance |
| [Biology](wiki/biology/_index.md) | genome compiler, CTRNN brain, sensing, metabolism and species |
| [Simulation](wiki/simulation/_index.md) | Ocean, Trainer, physics, analytics and balancing |
| [Frontend](wiki/frontend/_index.md) | webview structure and rendering |
| [Quality](wiki/quality/_index.md) | review findings (`RV-*`), testing, CI and tooling, runtime verification |
| [Project Map](wiki/project-map/_index.md) | module-level code map |
| [Research](wiki/research/_index.md) | Alife references and porting ideas (from [`ideas.md`](ideas.md)) |
| [Decisions](wiki/decisions/_index.md) | ADRs |

Tickets: [`.tickets/`](.tickets/). Release notes: [`CHANGELOG.md`](CHANGELOG.md). Agent rules: [`AGENTS.md`](AGENTS.md).
