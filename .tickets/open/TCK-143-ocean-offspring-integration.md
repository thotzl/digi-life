---
id: TCK-143
title: Ocean Offspring Integration
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B01, RV-D03
---

# TCK-143: Ocean Offspring Integration

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-B01, RV-D03).

## Description
Mitosis children are only pushed to `newly_spawned_creatures` and never simulated; the client drops them again. The Ocean has no heredity.

## Proposed Scope
Add children to the simulated population with a `maxCreatures` cap; requires in-memory species tracking (done in TCK-139) and species retention. Changes ecosystem dynamics substantially.
