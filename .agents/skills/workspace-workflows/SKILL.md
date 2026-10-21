---
name: workspace-workflows
description: Standardized workflows for maintaining and syncing this project's AI context and wiki. Includes portability documentation for initializations and status updates.
---
# workspace-workflows: Portable AI Workspace Synchronization

## I. Portable Context
This repository carries its own agent context (`AGENTS.md` -> `.agents/skills/`, `.agents/state/active-task.md`) and its own knowledge base (`wiki/`). Any agent can bootstrap from it without global configuration.

## II. Workflows
- **`/init`:** bootstrap or extend the `.agents/skills/` layout from the tech stack. Spec: `.agents/skills/workspace-workflows/references/init.md`.
- **`/dump`:** scan `src/`, `frontend/`, `bindings/` and root configs for structural changes and sync findings into the wiki and skills. Spec: `.agents/skills/workspace-workflows/references/dump.md`.
- **Wiki:** compiled and audited with the `wiki-builder` skill; sources are registered in `wiki/_sources.md`.

## III. Maintenance Rules
- **Separation:** skills hold agent instructions; facts about the code live in `wiki/`; skills link to wiki articles instead of repeating them.
- Verify every path and claim against `src/` and `frontend/` before writing it.
- Planned work and known defects are tracked in `.tickets/` and `wiki/quality/review-findings.md` (IDs `RV-*`); reference IDs instead of duplicating analysis.
- Update `.agents/state/active-task.md` when the work focus changes.
