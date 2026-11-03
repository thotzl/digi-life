# BroodBattle Wiki

Project knowledge for BroodBattle (repository `digi-life`; Tauri v2 + Rust + SQLite + Preact Signals). Obsidian vault =
repo root. The wiki documents the current state and the results of closed tickets; open work and the current focus
live only in `tickets/` (roadmap TCK-169); finished work is listed in `CHANGELOG.md`; agent rules in `.agents/skills/`.

## Add a feature

Read the guide, its contract and [[wiki/architecture/invariants]] — nothing else.

| Extension point | Guide | Contract |
| :--- | :--- | :--- |
| Organ kind | [[wiki/guides/add-organ-kind]] | [[wiki/contracts/organ-kind]] |
| Creature system (dynamic) | [[wiki/guides/add-dynamic]] | [[wiki/contracts/creature-system]] |
| Room | [[wiki/guides/add-room-or-mode]] | [[wiki/contracts/room]] |
| Evaluator | [[wiki/guides/add-evaluator]] | [[wiki/contracts/evaluator]] |
| Engine mode | [[wiki/guides/add-room-or-mode]] | [[wiki/contracts/mode]] |
| Event sink | [[wiki/guides/add-event-sink]] | [[wiki/contracts/event-sink]] |
| IPC message | [[wiki/guides/add-ipc-message]] | [[wiki/contracts/ipc-message]] |
| Config key / section | [[wiki/guides/add-config-key]] | [[wiki/contracts/config-section]] |
| Persistence table / repository | [[wiki/guides/add-table]] | [[wiki/contracts/persistence-repo]] |
| Frontend view | [[wiki/guides/add-view]] | [[wiki/contracts/frontend-view]] |

Templates: [[wiki/contracts/_template]], [[wiki/guides/_template]].

## Topics

- [[wiki/overview/_index|Overview]] — vision (whitepaper) and glossary
- [[wiki/architecture/_index|Architecture]] — runtime and plugin host, module map, engine, IPC, persistence, config, invariants, performance
- [[wiki/biology/_index|Biology]] — genome compiler, CTRNN brain, sensing, metabolism and species
- [[wiki/simulation/_index|Simulation]] — Ocean, trainer, physics, analytics and balancing
- [[wiki/frontend/_index|Frontend]] — webview structure and rendering
- [[wiki/quality/_index|Quality]] — testing, CI and tooling, runtime verification
- [[wiki/research/_index|Research]] — Alife references, porting ideas, neuroevolution literature
- [[wiki/decisions/_index|Decisions]] — ADRs
- [[wiki/history/_index|History]] — measurement logs and replaced models
