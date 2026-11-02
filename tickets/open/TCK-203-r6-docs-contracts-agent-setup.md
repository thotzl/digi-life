---
id: TCK-203
title: "R6 Docs: Contracts and Guides per Extension Point, Agent Setup"
status: open
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R6 (skeleton in R1, filled per sub-ticket, consolidated at the end)
depends_on: TCK-197
---

# TCK-203: Contracts, Guides and Agent Setup

A feature agent reads one guide, one contract and `invariants.md` — nothing else.

## Findings (2026-11-02 review)
- Wiki 2,504 lines; skills (17–27 lines each) are organised by layer, not by extension point; `server-simulation` and `biology-dna` name concrete files/functions that the refactor invalidates.
- Invariants are restated in 4–5 places (`invariants.md`, skills `server-simulation`, `project-baseline`, `client-rendering`, `simulation-engine.md`); the topic table in README, `_master-index.md`, AGENTS.md, project-baseline.
- Current-state articles grew into phase logs: `wiki/simulation/trainer.md` (293 lines, ~60 % measurements P2–P8, 37 sources), measurement sections in `ctrnn-brain.md`, `genome-compiler.md`, `sensing.md`, `metabolism-and-reproduction.md`, history in `frontend/structure.md`.
- `repository-map.md` outdated (largest files), `_processed.md` last updated 2026-10-21, pool tickets TCK-156/157/158 and TCK-169 not updated when slices landed.

## Scope
1. **Skeleton (R1):** `wiki/contracts/_template.md` (sections: Purpose, Signature, Lifecycle/call order, Invariants, Registration point, Config keys, Events/IPC, Test recipe, Minimal example) and `wiki/guides/_template.md` (≤ 60 lines, links only to its contract + invariants).
2. **Contracts** (each written by the ticket that creates the seam): `config-section` (TCK-198), `persistence-repo` (TCK-159), `ipc-message` (TCK-157), `organ-kind` (TCK-199), `creature-system` (TCK-200), `room` + `evaluator` (TCK-201), `mode` + `event-sink` (TCK-156), `frontend-view` (TCK-160).
3. **Guides:** `add-organ-kind`, `add-dynamic`, `add-room-or-mode`, `add-view`, `add-ipc-message`, `add-config-key`, `add-table`.
4. **Architecture:** `system-overview.md` (plugin host + registries diagram), `invariants.md` as the only copy, `module-map.md` replaces `repository-map.md` (one row per module/plugin with its contract link).
5. **History split:** `wiki/history/` takes measurements and replaced models (`trainer-measurements-p2-p8.md`, `brain-measurements.md`, `replaced-models.md`); current-state articles keep the current model and latest numbers with a link.
6. **Skills:** new `.agents/skills/plugin-authoring/SKILL.md` ("read the guide for your extension point + its contract + invariants; do not read the rest of the wiki"); layer skills reduced to pointers; no restated invariants; `project-baseline` keeps commands, versioning, close checklist.
7. **AGENTS.md / README / `_master-index.md`:** short index, current focus only in TCK-169 / `next-tasks.md`; product name BroodBattle where it describes the app.
8. `src/<module>/README.md` stubs linking to the contract, so code-local agents find it.
9. TCK-169 roadmap updated: Phase 1 = TCK-196; pool tickets closed or re-scoped.

## Acceptance
- Link check (TCK-197) green; no invariant text outside `invariants.md`.
- Every extension point has contract + guide; each contract has a runnable test recipe.
- `trainer.md` ≤ ~120 lines current state; history in `wiki/history/`.
