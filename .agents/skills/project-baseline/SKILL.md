---
name: project-baseline
description: Baseline for working in the BroodBattle repository (Tauri v2 + Rust + SQLite + Preact Signals) - commands, versioning, ticket close checklist and where to look. Read this before running builds or scripts, bumping versions or closing a ticket.
---
# project-baseline: Commands, Versioning, Close Checklist

## I. Where to Look
- Feature at an extension point: `.agents/skills/plugin-authoring/SKILL.md` (guide + contract + invariants only).
- Rules that must not regress: `wiki/architecture/invariants.md` (the one copy; refer to rules by number).
- Code layout and registered plugins: `wiki/architecture/module-map.md`; runtime: `wiki/architecture/system-overview.md`.
- Everything else: `wiki/_master-index.md`. Work plan: `tickets/ongoing/TCK-169-roadmap-epic.md`; open findings
  (`RV-*`) live in their tickets (`revisit:` frontmatter + `## Findings`), tickets in `tickets/{open,ongoing,closed}/`.
- The wiki holds the current state and results of closed tickets only; open work never goes into the wiki.

## II. Commands (repo root)
| Command | Purpose |
| :--- | :--- |
| `npm run tauri:dev` | desktop app in dev (own `.dev` identifier and data folder) |
| `cargo test` | Rust tests incl. golden master; regenerates `bindings/` |
| `npm test` / `npm run lint` / `npm run build` | Vitest, ESLint (zero warnings), tsc + Vite |
| `cargo fmt --check` / `cargo clippy --all-targets --features runner -- -D warnings` | Rust format / lints (CI gates) |
| `cargo run --release --features runner --bin trainer_runner -- run\|probe\|ocean ...` | headless measurements (skill `simulation-analytics`) |
| `node scripts/check-links.mjs` | doc link check (CI step) |
Toolchain, CI and releases: `wiki/quality/ci-and-tooling.md`. Long outputs go to `.agents/artifacts/`.

## III. Versioning and Close Checklist
- **Versioning:** one patch bump per closed ticket in `package.json`, `package-lock.json`, `tauri.conf.json`,
  `Cargo.toml`/`Cargo.lock`, plus a `CHANGELOG.md` entry referencing the ticket.
- **Close checklist:** closing a ticket (move to `tickets/closed/`) updates in the same commit: the affected wiki
  articles and contracts (result-oriented, cite the ticket; measurements and replaced models go to `wiki/history/`),
  skill links that point to the ticket's old path, the parent epic's status/phase table, the `CHANGELOG.md` entry and
  the version bump. Then run `node scripts/check-links.mjs`; it fails on wikilinks, `sources:` entries and wiki/skill
  doc paths that no longer resolve.
- Wikilinks are vault-root relative (`[[wiki/topic/article]]`; vault = repo root).
