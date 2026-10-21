---
sources: [.tickets/closed/TCK-107-refactoring-dry-and-shared-modularization.md, .tickets/closed/TCK-108-headless-backend-evolutionary-trainer-migration.md, .tickets/closed/TCK-109-tauri-desktop-shell.md, .tickets/closed/TCK-110-rust-simulation-core.md, .tickets/closed/TCK-114-tauri-v2-github-actions-workflow-desktop-releases.md, .tickets/closed/TCK-115-master-architectural-refactoring.md, .tickets/closed/TCK-124-tauri-exclusive-consolidation.md, .tickets/closed/TCK-125-dry-kiss-cleanup.md, .tickets/closed/FIX-126-rebuild-loop.md, .tickets/closed/FIX-126-B-taurignore.md, .tickets/closed/TCK-137-persistence-appdata-shared-connection.md, .tickets/closed/TCK-138-embedded-config.md, .tickets/closed/TCK-139-ocean-hot-loop-performance.md, .tickets/closed/TCK-140-trainer-shared-performance.md]
---

# Architecture History (implemented)

How the platform got from a browser/Node.js prototype to the Tauri/Rust app. Tickets stay in `.tickets/closed/`; release notes in `CHANGELOG.md`.

## Prototype era: TypeScript + Node.js (Aug 2026)

- **TCK-107** — Shared isomorphic TS modules (`physics.ts`, `sensory.ts`, `brain.ts`, `configHelper.ts`) to remove 4× duplicated physics/sensing between Ocean server, balance sim, trainer and preview. Documented divergences: `halfCone` 1.5 vs 2.5, rock restitution 0.45 vs 0.50, sensor bandwidth divisor `bandwidth*1.8+0.12` vs `*2.0+0.1`.
- **TCK-108** — Trainer moved from the browser into the Node.js backend with WebSocket replication (25 Hz), server-side SQLite writes. Superseded by the Rust core.

## Migration to Tauri + Rust (Aug 2026)

- **TCK-109** — Tauri v2 desktop shell; portless IPC (`invoke` + event stream) instead of HTTP/WebSocket; standalone installers.
- **TCK-110** — Headless Rust core: spatial grid, CTRNN Euler solver, DNA compiler, SQLite, 25 Hz telemetry.
- **TCK-114** — `.github/workflows/release.yml`: matrix build (macOS, Windows, Ubuntu 22.04) with `tauri-action`, draft release with `.dmg/.exe/.msi/.deb/.AppImage`. Trigger later restricted to `v*` tags (TCK-134).
- **TCK-115** — SPA migration (single `index.html`), ts-rs type export, `InteractiveCamera`, shared `BrainRenderer`, ~3,500 lines of Express server and duplicate TS parsers removed; `better-sqlite3` and browser DB removed. (Claim "frontend re-exports the bindings" did not hold, see RV-A03.)
- **TCK-124** — Root convergence: `Cargo.toml`, `tauri.conf.json`, `package.json` at repo root; `src/` = Rust, `frontend/` = UI; `src-tauri/` removed; `ws` dependency and legacy TS simulation deleted.
- **TCK-125** — Frontend `mapGenerator.ts` deleted (violated Rust-as-SSOT); world generation restored in Rust by FIX-127 ([[simulation/history]]).
- **FIX-126** — Vite dev port fixed to `8765` with `strictPort: true`; `devUrl` aligned; DB files excluded from rebuild watching.
- **FIX-126-B** — Tauri v2 removed `watch` from `tauri.conf.json`; `.taurignore` (`*.db`, `*.db-shm`, `*.db-wal`, `target/`, `dist/`, `node_modules/`) prevents the infinite rebuild loop.

## Phase 0 stabilization (Oct 2026)

- **TCK-137** — DB in AppData, legacy copy, schema init once, shared `DbState`, WAL, DB error logging (RV-B03, RV-A04, RV-D05). See [[architecture/persistence]].
- **TCK-138** — `config.json` embedded via `include_str!`, divergent fallback removed (RV-B04). See [[architecture/configuration]].
- **TCK-139** — Ocean hot loop: species cache, grid reuse, no agent clones, event buffering, catalogue once per restock (RV-P01/02/03/05/07). See [[architecture/performance]].
- **TCK-140** — Genome compiler and trainer performance: `read_evolution_rates`, reflex `HashSet`, no world in trainer telemetry; golden master introduced (RV-P06).
- Remaining Phase 0 tickets: [[quality/history]], [[simulation/history]], [[biology/history]], [[frontend/history]].

## Key Takeaways

- The Node.js/WebSocket architecture is gone; anything referring to it is outdated.
- Repo layout since TCK-124: everything runs from the root, no `src-tauri/`.
- Phase 0 made the app release-capable (AppData DB, embedded config) and the hot loop cheap.

Sources: closed tickets listed in the frontmatter.
