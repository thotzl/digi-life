---
id: TCK-154
title: Trainer Scenario Parameters into Plugins
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S08, RV-S09
---

# TCK-154: Trainer Scenario Parameters into Plugins

> Pool ticket: requires a user decision before implementation. Details in `wiki/quality/review-findings.md` (RV-S08, RV-S09).

## Description
Exploration spawn distances differ (650/1200/550); re-trial logic duplicates `initialize`; rejection sampling has no iteration limit; Gen-1 ignores `base_dna`.

## Proposed Scope
Move chamber size, epoch length and spawn distances into the scenario plugin; decide the intended distances.
