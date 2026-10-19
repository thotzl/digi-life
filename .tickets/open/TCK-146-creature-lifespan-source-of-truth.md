---
id: TCK-146
title: Creature Lifespan Source of Truth
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B08
---

# TCK-146: Creature Lifespan Source of Truth

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-B08).

## Description
Code uses `mature_age * 12`, config defines `creatureMaxAgeTicks` (36000). Both claim to be the intended lifespan.

## Proposed Scope
Decide which model is intended; remove the other.
