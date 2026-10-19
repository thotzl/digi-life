---
id: TCK-158
title: Seedable RNG and Headless Runner
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-T03, RV-T04, RV-S11
---

# TCK-158: Seedable RNG and Headless Runner

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-T03, RV-T04, RV-S11).

## Description
`thread_rng()` everywhere; `cli_trainer` simulates an outdated 3-input model against the production DB.

## Proposed Scope
Seeded RNG per simulation; headless runner with JSON report replacing `cli_trainer`.
