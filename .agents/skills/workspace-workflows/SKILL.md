---
name: workspace-workflows
description: Standardized workflows for maintaining and syncing this project's AI context and wiki. Includes portability documentation for initializations and status updates.
---
# workspace-workflows: Portable AI Workspace Synchronization

## I. Portable Context
This repository carries its own agent context (`AGENTS.md` -> `.agents/skills/`, optional `.agents/state/next-tasks.md`) and its own knowledge base (`wiki/`). Any agent can bootstrap from it without global configuration.

## II. Workflows
- **`/init`:** bootstrap or extend the `.agents/skills/` layout from the tech stack. Spec: `.agents/skills/workspace-workflows/references/init.md`.
- **`/dump`:** scan `src/`, `frontend/`, `bindings/` and root configs for structural changes and sync findings into the wiki and skills. Spec: `.agents/skills/workspace-workflows/references/dump.md`.
- **Wiki:** compiled and audited with the `wiki-builder` skill; sources are registered in `wiki/_sources.md`.

## III. Maintenance Rules
- **Separation:** skills hold agent instructions; facts about the code live in `wiki/`; skills link to wiki articles instead of repeating them.
- Verify every path and claim against `src/` and `frontend/` before writing it.
- Planned work and open defects are tracked only in `tickets/` (IDs `RV-*` in the `revisit:` frontmatter); the wiki holds the current state and results of closed tickets, never open work. Reference IDs instead of duplicating analysis.
- Record findings and candidate tasks that are not yet tickets in `.agents/state/next-tasks.md`; remove them there once they are ticketed.
