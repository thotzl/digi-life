---
sources: [tickets/closed/TCK-163-project-wiki.md, tickets/closed/TCK-170-wiki-results-policy-and-obsidian-vault.md, tickets/closed/TCK-203-r6-docs-contracts-agent-setup.md, CHANGELOG.md, AGENTS.md]
---

# ADR-015: Project Knowledge in Wiki, Tickets and CHANGELOG

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-21
- Tickets: TCK-163, TCK-170 (v1.12.13, v1.12.14); amended by TCK-203 (2026-11-03)

## Context

Project knowledge was spread over `docs/revisit/`, `README.md`, `HANDOFF.md`, `.agents/state/active-task.md`, the agent skills and closed tickets, with overlap and drift risk (TCK-163). Who took the decision is not named in the tickets; they are recorded as ticket scope and verified on close.

## Decision

1. **One cross-linked wiki** in `wiki/` (English, Obsidian-style links) for project knowledge; `docs/revisit/` and `HANDOFF.md` deleted, `WHITEPAPER.md` moved to `wiki/overview/whitepaper.md` (TCK-163).
2. **Skills hold agent rules only** and link to the wiki; README trimmed to intro, commands, setup and wiki link (TCK-163).
3. **Wiki = current state and results of closed tickets; open work only in tickets** (TCK-170). Open findings, targets and fix sketches moved into tickets; the roadmap became epic TCK-169; the non-goals became ADR-001. Closing a ticket updates the affected wiki articles result-oriented.
4. **CHANGELOG is the list of finished work**; per-topic history pages and the findings register were removed (TCK-170).
5. **Repo root is the Obsidian vault**: `.tickets/` renamed to `tickets/`, wikilinks relative to the vault root (`[[wiki/...]]`), only `.obsidian/app.json` tracked (TCK-170).
6. **Amendment (TCK-203):** current-state articles had grown into phase logs (`trainer.md` 293 lines, ~60 % measurements). Measurements and replaced models move to `wiki/history/`; current-state articles keep the current model and latest numbers with a link.

## Consequences

- Every `RV-*` finding stays findable (first in the wiki, after TCK-170 in the pool tickets' `## Findings`).
- No wiki article references an open or ongoing ticket (scripted check in TCK-170); links are checked in CI by `scripts/check-links.mjs` (TCK-197).
- Agent entry point: `AGENTS.md` names wiki and tickets as the project knowledge and `.agents/skills/` as the agent rules.
