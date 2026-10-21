---
name: project-baseline
description: Baseline rules for working in the Pixel DNA Life repository (Tauri v2 + Rust + SQLite + Preact Signals). Read this before running builds, scripts, or managing dependencies.
---
# project-baseline: Core Directives & Knowledge Pointers

## I. Universal Directives
- **Agent rules** live in `.agents/skills/` and supersede tool-specific configs (e.g. `.cursorrules`).
- **Project knowledge** lives in the wiki: `wiki/_master-index.md`. Read the relevant article before changing a subsystem; do not duplicate wiki content into skills.
- **Work plan:** epic `tickets/ongoing/TCK-169-roadmap-epic.md`. Open findings (`RV-*`) live in their tickets (`revisit:` frontmatter + `## Findings`). Tickets: `tickets/{open,ongoing,closed}/` (no leading dot, so Obsidian shows them).
- **Wiki vs. tickets:** the wiki documents the current state and the results of closed tickets only; open work never goes into the wiki. When closing a ticket, update the affected wiki articles to the new state (result-oriented, cite the ticket) in the same commit. `CHANGELOG.md` is the list of finished work.

## II. Where to Look
| Need | Wiki article |
| :--- | :--- |
| Code layout | `wiki/project-map/repository-map.md` |
| Runtime structure | `wiki/architecture/system-overview.md` |
| Commands, toolchain, CI, versioning | `wiki/quality/ci-and-tooling.md` |
| Invariants that must not regress | `wiki/architecture/invariants.md` |

## III. Mandates
- **config.json is SSOT:** do not hardcode new tunables in Rust; add them to `config.json` + `SimulationRules`. Verify a key is read in `src/` before tuning it (`wiki/architecture/configuration.md`).
- **No DB work in the hot loop:** keep SQLite out of the per-step physics path; batch writes on timers or events.
- **Bindings:** after changing a `#[derive(TS)]` struct run `cargo test` and commit the regenerated `bindings/`.
- **Versioning:** one patch bump per closed ticket in `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`/`Cargo.lock`, plus a `CHANGELOG.md` entry referencing the ticket.
- **Do not trust `src/bin/cli_trainer.rs` output** (outdated model, RV-S11).
- **Keep the wiki current:** when a change invalidates a wiki statement, update the article in the same commit. Wikilinks are vault-root relative (`[[wiki/topic/article]]`; vault = repo root).
