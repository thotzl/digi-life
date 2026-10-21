---
id: TCK-163
title: Project Wiki Replaces docs/ and Agent-Skill Knowledge
status: ongoing
priority: medium
assigned: Claude Code
created_at: 2026-10-20
---

# TCK-163: Project Wiki Replaces docs/ and Agent-Skill Knowledge

## Description
Project knowledge is spread over `docs/revisit/`, `README.md`, `HANDOFF.md`, `.agents/state/active-task.md`, the agent skills and closed tickets, with overlap and drift risk. A single cross-linked wiki in `wiki/` becomes the place for project knowledge.

## Scope
- Compile `wiki/` (English, Obsidian-style links) from the sources in `wiki/_sources.md`.
- `WHITEPAPER.md` moved 1:1 into the wiki; `ideas.md` stays and gets a research area that links to it.
- Closed tickets documented per area (tickets stay).
- Project map at module level, derived from `src/` and `frontend/`.
- After review: skills keep only agent instructions and link to the wiki; README trimmed to intro, commands, setup and wiki link; references to `docs/revisit` retargeted; `docs/revisit/` and `HANDOFF.md` deleted.
- `CHANGELOG.md` stays separate and is linked.

## Acceptance
- Every `RV-*` finding and its status is findable in the wiki.
- No file references `docs/revisit/` or `HANDOFF.md` after the cleanup.
