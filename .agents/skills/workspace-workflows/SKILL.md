---
name: workspace-workflows
description: Standardized workflows for maintaining and syncing this project's AI context. Includes portability documentation for initializations and status updates.
---
# workspace-workflows: Portable AI Workspace Synchronization

## I. Portable Context
This repository carries its own agent context (`AGENTS.md` -> `.agents/skills/`, `.agents/state/active-task.md`). Any agent can bootstrap from it without global configuration.

## II. Workflows
- **`/init`:** bootstrap or extend the `.agents/skills/` layout from the tech stack. Spec: `.agents/skills/workspace-workflows/references/init.md`.
- **`/dump`:** scan `src/`, `frontend/`, `bindings/` and root configs for structural changes and sync findings into the skills. Spec: `.agents/skills/workspace-workflows/references/dump.md`.

## III. Maintenance Rules
- Skills must describe the current code. Verify every path and claim against `src/` and `frontend/` before writing it.
- Planned work and known defects are tracked in `docs/revisit/` (IDs `RV-*`) and `.tickets/`; reference IDs instead of duplicating analysis.
- Update `.agents/state/active-task.md` when the work focus changes.
