---
name: workspace-workflows
description: Standardized workflows for maintaining and syncing this project's AI context. Includes portability documentation for initializations and status updates.
---
# workspace-workflows: Portable AI Workspace Synchronization

## I. Portable Context Operations
This workspace is designed to be fully self-contained and portable. Any AI agent entering this repository can automatically discover context and sync its knowledge structures.

## II. Workflows
- **Workspace Initialization (`/init`):** Bootstraps the local `.agents/skills/` architecture from baseline patterns, discovers the tech stack, and sets up a customized context layout. (See `/home/torsten/.gemini/skills/core-project-workflows/references/init.md`).
- **Context Scan & Sync (`/dump`):** Extracts system-wide signatures, indexes active APIs/data models, and verifies structural integrity. Outputs results into `.agents/skills/architecture-overview/REPO_MAP.md`. (See `/home/torsten/.gemini/skills/core-project-workflows/references/dump.md`).

## III. References
The original global workflow blueprints are maintained locally inside:
- `.agents/skills/workspace-workflows/references/init.md`
- `.agents/skills/workspace-workflows/references/dump.md`
These local files serve as the immutable specifications for how AI agents should auto-replicate and manage this workspace's context without relying on global system configs.
