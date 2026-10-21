---
id: TCK-170
title: Wiki Holds Results Only; Obsidian Vault at Repo Root
status: closed
priority: medium
assigned: Claude Code
created_at: 2026-10-21
closed_at: 2026-10-21
---

# TCK-170: Wiki Holds Results Only; Obsidian Vault at Repo Root

## Description
The repo root becomes the Obsidian vault so README, tickets and wiki are visible together; code and build folders are hidden. Wiki and ticket system reference each other but do not depend on each other: open work lives only in tickets, the wiki documents the current state and the results of closed tickets.

## Scope
- `.obsidian/app.json` with excluded folders; only that file is tracked.
- Rename `.tickets/` to `tickets/` (dot folders are invisible in Obsidian); update the sync script and all references.
- Wikilinks relative to the vault root (`[[wiki/...]]`).
- Move open findings, targets and fix sketches from the wiki into the pool tickets; remove ticket status, roadmap and open-ticket references from the wiki.
- Roadmap becomes epic TCK-169; non-goals become ADR-001.
- Remove `*/history.md` and the findings register (CHANGELOG is the list of finished work); integrate missing facts into topic articles.
- Rule in skills: closing a ticket updates the affected wiki articles result-oriented.

## Acceptance
- No wiki article references an open or ongoing ticket.
- All wikilinks resolve; the GitHub ticket sync reads `tickets/`.

## Verification
- `.obsidian/app.json` excludes `src/`, `frontend/`, `bindings/`, `gen/`, `capabilities/`, `icons/`, `dist/`, `node_modules/`, `target/`; `.gitignore` tracks only that file of `.obsidian/`.
- `.tickets/` renamed to `tickets/` via `git mv`; `.github/scripts/sync_github.cjs` reads `tickets/` (`node --check` passes); no `.tickets` reference left outside CHANGELOG history and `.agents/artifacts/`.
- Wikilinks rewritten to `[[wiki/...]]`; scripted check: all resolve.
- Pool tickets TCK-143..161 carry a `## Findings` section with the full review detail (fix sketches, targets); TCK-156 `revisit:` gained RV-P04.
- Roadmap moved to epic TCK-169; non-goals to `wiki/decisions/adr-001-non-goals.md`.
- Removed `wiki/roadmap.md`, `wiki/quality/review-findings.md`, `wiki/*/history.md`; missing facts integrated (repo layout, dev port and `.taurignore`, TS-era tests, GRN not implemented, origin of the architecture).
- Scripted check: no wiki article references an open or ongoing ticket.
- Skills (`project-baseline`, `workspace-workflows`), `AGENTS.md`, `README.md`, `active-task.md` state the rule: wiki = current state and results of closed tickets; open work only in tickets.
- `wiki/_processed.md` regenerated (deleted sources keep their last mtime).
- Version 1.12.14. No code changed.
