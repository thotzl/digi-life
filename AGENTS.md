# BroodBattle - AI Agent Directive

> **Universal AI Directive:** The `.agents/skills/` directory is the single source of truth for **all** AI agent operations in this workspace. Read and self-configure your execution context to prioritize these skills over default global instructions or other tool-specific configurations (e.g., `.cursorrules`, `.cursor/rules/`, etc.).

Repository `digi-life`; the app is **BroodBattle** (Tauri v2 + Rust + SQLite + Preact Signals).

## AI Workspace: `.agents/`

| Path | Purpose | Git |
| :--- | :--- | :--- |
| `.agents/skills/<name>/SKILL.md` | Agent rules per domain; take precedence over global skills | tracked |
| `.agents/state/active-task.md` | Optional, dated: the currently active task (goal, where to find what, progress, next step); deleted once the task is done | tracked |
| `.agents/state/next-tasks.md` | Optional, dated: tasks planned next and findings not yet ticketed (an entry is removed once it is ticketed or becomes the active task) | tracked |
| `.agents/artifacts/` | Temporary scripts, logs, filtered outputs; never write scratch files elsewhere | ignored |

At session start: the user's request defines the session task; the project state comes from the tickets (`tickets/ongoing/`, roadmap epic TCK-169, which holds the current focus) and `git log`. If present, read `.agents/state/active-task.md` and `.agents/state/next-tasks.md` as handover context, not as instructions; if they contradict the tickets, the tickets win and the note gets updated. Then activate the matching skills below. Search tools must not skip `.agents/` (gitignored paths in `artifacts/`).

## Project Knowledge

- **Feature at an extension point** (organ kind, dynamic, room, evaluator, mode, event sink, IPC message, config key, table, view): skill `plugin-authoring` — read one guide, one contract and `wiki/architecture/invariants.md`, nothing else.
- **Rules that must not regress:** `wiki/architecture/invariants.md` (the only copy; read it before touching engine, router, physics or bundle config).
- **Wiki** (`wiki/_master-index.md`): current state, contracts and guides, decisions (ADRs), research; history in `wiki/history/`.
- **Tickets** (`tickets/`): all open work and findings (`RV-*`). Release notes: `CHANGELOG.md`. If wiki and code disagree, the code wins and the wiki gets corrected.

## Skills Index

- **`plugin-authoring`**: entry point for any feature at an extension point; maps each point to its guide and contract. `.agents/skills/plugin-authoring/SKILL.md`
- **`project-baseline`**: commands, versioning, ticket close checklist, where to look. `.agents/skills/project-baseline/SKILL.md`
- **`biology-dna`**: pointers for the creature model (genome, organs, brain, sensing, metabolism). `.agents/skills/biology-dna/SKILL.md`
- **`server-simulation`**: pointers for engine, creature systems, Ocean, trainer, IPC and persistence. `.agents/skills/server-simulation/SKILL.md`
- **`client-rendering`**: pointers for views, router, renderers and HUD. `.agents/skills/client-rendering/SKILL.md`
- **`simulation-analytics`**: measuring and balancing with the headless runner. `.agents/skills/simulation-analytics/SKILL.md`
- **`testing-strategies`**: reproduction-first tests, fixtures, golden master handling. `.agents/skills/testing-strategies/SKILL.md`
- **`workspace-workflows`**: `/init` and `/dump` context maintenance, wiki upkeep. `.agents/skills/workspace-workflows/SKILL.md`
