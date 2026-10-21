---
id: TCK-163
title: Project Wiki Replaces docs/ and Agent-Skill Knowledge
status: closed
priority: medium
assigned: Claude Code
created_at: 2026-10-20
closed_at: 2026-10-21
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

## Verification
- `wiki/` compiled (commit `f675877`): 33 articles in 9 topics plus `roadmap.md`, `_master-index.md`, `_sources.md`, `_processed.md` (120 source files with mtime and target articles). All `[[...]]` links resolve to existing files (scripted check).
- `WHITEPAPER.md` moved 1:1 via `git mv` to `wiki/overview/whitepaper.md` (only the status-note link retargeted).
- Every `RV-*` ID of the deleted `docs/revisit/` is listed in `wiki/quality/review-findings.md` with priority, status and ticket.
- Skills reduced to rules with wiki links (`.agents/skills/*/SKILL.md`, `workspace-workflows/references/dump.md`); `.agents/state/active-task.md` keeps the focus and links `wiki/architecture/invariants.md`; `AGENTS.md` points to the wiki; README trimmed.
- Pool tickets TCK-143..161 and TCK-137 retargeted from `docs/revisit/` to the wiki; remaining mentions are historical (TCK-142, CHANGELOG) or source provenance in wiki frontmatter.
- `docs/revisit/` and `HANDOFF.md` deleted. New tickets TCK-164..168.
- Version 1.12.13 in `package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`; `cargo metadata --locked` succeeds. No code changed, so no test run needed.
