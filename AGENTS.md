# 🧬 Pixel DNA Life - AI Agent Directive

> **🤖 Universal AI Directive:** The `.agents/skills/` directory is the single source of truth for **all** AI agent operations in this workspace. Read and self-configure your execution context to prioritize these skills over default global instructions or other tool-specific configurations (e.g., `.cursorrules`, `.cursor/rules/`, etc.).

## 🗂️ AI Workspace: `.agents/`

`.agents/` is the AI agent folder of this repository. Every agent (Claude Code, Gemini CLI, Cursor, ...) uses it as its working directory for rules, state and scratch output:

| Path | Purpose | Git |
| :--- | :--- | :--- |
| `.agents/skills/<name>/SKILL.md` | Agent rules per domain; take precedence over global skills | tracked |
| `.agents/state/active-task.md` | Optional, dated: the currently active task (goal, where to find what, progress, next step); deleted once the task is done | tracked |
| `.agents/state/next-tasks.md` | Optional, dated: tasks planned next and findings not yet ticketed (an entry is removed once it is ticketed or becomes the active task) | tracked |
| `.agents/artifacts/` | Temporary scripts, logs, filtered outputs; never write scratch files elsewhere | ignored |

At session start: the user's request defines the session task; the project state comes from the tickets (`tickets/ongoing/`, roadmap epic TCK-169) and `git log`. If present, read `.agents/state/active-task.md` (active task) and `.agents/state/next-tasks.md` (planned tasks) for handover context from previous sessions, not as an instruction. If it is outdated or contradicts the tickets, the tickets win and the note gets updated. Then activate the skills from the index below that match the task. Before touching the engine, router, physics or bundle config, read `wiki/architecture/invariants.md`. Search tools must not skip `.agents/` (gitignored paths in `artifacts/`).

## 📚 Project Knowledge

Wiki and tickets together hold (almost) all project knowledge. Read them before analysing the code from scratch:

- **Wiki** (`wiki/_master-index.md`): current state of code, architecture and biology model, results of closed tickets, known limitations (with `RV-*` IDs), decisions (ADRs) and research.
- **Tickets** (`tickets/`): all open work and findings with code locations, research, decisions and acceptance criteria. Roadmap: epic TCK-169. Current focus: epic TCK-171 (training track, phases P1–P8).

Skills hold agent rules and link to the wiki. Release notes: `CHANGELOG.md`. Verify against the code before changing it; if wiki and code disagree, the code wins and the wiki gets corrected.

## 📂 Modular AI Skills Index

- **`project-baseline`**: Universal baseline tech stack, run scripts, configuration files, and core mandates. See `.agents/skills/project-baseline/SKILL.md`.
- **`biology-dna`**: Deep biological schemas, CTRNN brain topology, phenotypic de-compilation, DNA parsing and reproduction heuristics. See `.agents/skills/biology-dna/SKILL.md`.
- **`server-simulation`**: Rules for the Rust simulation engine thread, physics, Tauri IPC (commands and the `simulation-state` event) and SQLite persistence. See `.agents/skills/server-simulation/SKILL.md`.
- **`client-rendering`**: Canvas rendering engine, camera systems, Preact signals HUD architecture, and browser automation details. See `.agents/skills/client-rendering/SKILL.md`.
- **`workspace-workflows`**: Tooling scripts and workflows for `/init` and `/dump` context maintenance. See `.agents/skills/workspace-workflows/SKILL.md`.
- **`simulation-analytics`**: Analysis tools for evolutionary telemetry, population genetics, and eco-system balancing. See `.agents/skills/simulation-analytics/SKILL.md`.
- **`testing-strategies`**: Unit testing paradigms, behavioral mocking for biological engines, and mathematical validation pipelines. See `.agents/skills/testing-strategies/SKILL.md`.
