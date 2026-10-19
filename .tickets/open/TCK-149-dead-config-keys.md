---
id: TCK-149
title: Dead Config Keys: Implement or Remove
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S01
---

# TCK-149: Dead Config Keys: Implement or Remove

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-S01).

## Description
16 of 44 `SimulationRules` are never read (adrenaline, biolum, biomes, predator savage, lateral drag, ...); `maxCreatures`/`seed` unused; 9 keys in config.json unknown to the struct.

## Proposed Scope
Decide per key; afterwards `deny_unknown_fields`.
