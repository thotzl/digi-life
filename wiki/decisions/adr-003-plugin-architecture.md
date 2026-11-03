---
sources: [tickets/closed/TCK-196-plugin-architecture-epic.md, tickets/closed/TCK-199-r2-genome-and-organ-registry.md, tickets/closed/TCK-200-r3-world-and-creature-systems.md, tickets/closed/TCK-201-r3-rooms-evaluator-breeder.md, tickets/closed/TCK-156-engine-decomposition.md, tickets/closed/TCK-160-frontend-structure.md, tickets/closed/TCK-198-r1-sim-context-config-utils.md, tickets/closed/TCK-203-r6-docs-contracts-agent-setup.md, tickets/closed/TCK-204-r7-contract-proof.md, .agents/skills/plugin-authoring/SKILL.md]
---

# ADR-003: Plugin Architecture with Registries, Contracts and Guides

- Status: accepted
- Date: 2026-11-03
- Tickets: TCK-196 (epic), TCK-198, TCK-199, TCK-200, TCK-201, TCK-156, TCK-160, TCK-203, TCK-204

## Context

The goal after the training track (TCK-171) is the game dynamics of the whitepaper: active traits (claws, teeth, armour), dynamics (attack, digestion, reproduction), rooms and modes (arena, creator). The review of 2026-11-02 (six parallel read-only reviews, base `f34a301`) showed that the code could not take these as additions:

- Adding an organ kind touched about ten places (genome lists, sensing chain, physics, metabolism, engine, trainer, probes, renderer).
- The per-creature tick existed four times (Ocean inline, trainer, two probe copies) and diverged.
- The engine was one ~1,465-line closure; the room "plugin" was a closed enum with one creature per chamber; the UI had no view lifecycle.
- `AppConfig::global()` was read in about 60 places, inside the model too.
- Rules were restated in four to five documents, and current-state articles carried phase logs, so an agent had to read much of the wiki and code to add anything.

## Decision

1. **Extension points with registries.** Each kind of addition has one seam and one registration point: organ kind (`OrganKind::spec()`, `src/biology/organs/`), creature system / dynamic (ordered system list, `src/sim/systems/`), room (`ROOMS`, `src/trainer/room/registry.rs`), evaluator, engine mode (`Mode`), event sink, IPC message (ts-rs types), config section, persistence table / repository, UI view (`VIEWS`, `frontend/views/index.ts`). A new feature is its own module plus one registry entry plus what its contract names (config section, regenerated bindings, references for a model change).
2. **Enum dispatch and plain functions before `dyn`.** A trait object only where a second implementation exists or is planned (`Room`, `Evaluator`, `Mode`, `EventSink`); exhaustive `match` on enums lets the compiler find every site of a new variant.
3. **Explicit context instead of globals.** Every module owns its config section and receives it explicitly; `AppConfig::global()` is read only at the two entry points (Tauri setup, runner `main`).
4. **One shared creature tick** for Ocean, trainer rooms and probes; differences between worlds are explicit world settings, not copies.
5. **Contracts and guides as the interface for feature agents.** Every extension point has a contract (`wiki/contracts/`, nine fixed sections incl. a runnable test recipe) and a short guide (`wiki/guides/`, at most ~60 lines). `wiki/architecture/invariants.md` is the only copy of the project rules. A feature agent starts from `AGENTS.md` and the skill `plugin-authoring` and reads its guide, its contract and the invariants, nothing else of the wiki.
6. **Structure before behaviour.** The rebuild changed structure only, gated on an unchanged golden master and unchanged reference runs; all behaviour changes were bundled in one model change (TCK-202, model version 7).
7. **Proof with fresh agents.** The goal "a feature needs only its interfaces" is checked by agents that add an organ, a room and a view from guide + contract alone (TCK-204); gaps they find are fixed in the docs.

## Consequences

- A new organ kind touches two files, a new room its module and one registry entry, a new view its module, one registry entry and a launcher. The second proof round needed 17 files read, 2 of them not named by the docs.
- Every new seam ships with its contract in the same commit; the close checklist (`project-baseline`) and the doc link check in CI keep contracts, module map and links current.
- Some changes stay host changes by design and are listed as current limits in the contracts: polygon obstacles, a room-specific respawn rule, per-trial obstacles, obstacles in trainer telemetry, views without a backend mode (`.agents/state/next-tasks.md`).
- More files than before, each smaller (largest engine module 330 lines, largest biology module 483 lines incl. tests).
