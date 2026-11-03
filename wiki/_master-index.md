# Pixel DNA Life Wiki

Project knowledge for Pixel DNA Life (Tauri v2 + Rust + SQLite + Preact Signals). Obsidian vault = repo root. Tickets: `tickets/`; release notes: `CHANGELOG.md`; agent rules: `.agents/skills/`.

- [[wiki/overview/_index|Overview]] — vision (whitepaper) and glossary
- [[wiki/architecture/_index|Architecture]] — runtime structure, engine, IPC, persistence, config, invariants, performance
- [[wiki/biology/_index|Biology]] — genome compiler, CTRNN brain, sensing, metabolism and species
- [[wiki/simulation/_index|Simulation]] — Ocean, Trainer, physics, analytics and balancing
- [[wiki/frontend/_index|Frontend]] — webview structure and rendering
- [[wiki/quality/_index|Quality]] — testing, CI and tooling, runtime verification
- [[wiki/project-map/_index|Project Map]] — module-level code map
- Contracts and guides (TCK-203, filled per extension point): [[wiki/contracts/config-section|config section]] / [[wiki/guides/add-config-key|add a config key]], [[wiki/contracts/persistence-repo|persistence repository]] / [[wiki/guides/add-table|add a table]]; [[wiki/contracts/organ-kind|organ kind]] / [[wiki/guides/add-organ-kind|add an organ kind]]; [[wiki/contracts/creature-system|creature system]] / [[wiki/guides/add-dynamic|add a dynamic]]; [[wiki/contracts/ipc-message|IPC message]] / [[wiki/guides/add-ipc-message|add an IPC message]]; [[wiki/contracts/frontend-view|frontend view]] / [[wiki/guides/add-view|add a view]]; templates [[wiki/contracts/_template]], [[wiki/guides/_template]]
- [[wiki/research/_index|Research]] — Alife references and porting ideas (from `ideas.md`), neuroevolution and training literature
- [[wiki/decisions/_index|Decisions]] — ADRs (taken decisions) and template; latest: [[wiki/decisions/adr-002-brain-v2|ADR-002 Brain v2]]

The wiki documents the current state and the results of closed tickets. Open work lives only in `tickets/`; finished work is listed in `CHANGELOG.md`.
