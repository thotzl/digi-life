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

> Pool ticket: requires a user decision before implementation. Findings (RV-B01, RV-D03) below.

## Description
Mitosis children are only pushed to `newly_spawned_creatures` and never simulated; the client drops them again. The Ocean has no heredity.

## Proposed Scope
Add children to the simulated population with a `maxCreatures` cap; requires in-memory species tracking (done in TCK-139) and species retention. Changes ecosystem dynamics substantially.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B01 · P0 · static** — `src/server/engine.rs` (mitosis block): the child is pushed only to `newly_spawned_creatures`, never to `next_creatures`/`creatures`. The client (`frontend/ocean.ts`) adds it from `newCreatures` and removes it in the same handler because the ID is not in the server list. Consequence: no heredity, no mutation across generations in the Ocean; `highestGeneration` and species rows still grow, so telemetry suggests evolution that does not happen. Population comes only from restocking.
- Fix sketch: collect children in a `births` list, append to `creatures` after the loop, cap at `maxCreatures` (currently unread). Coupling: once children live, `species_records` grows fast — fix together with RV-D03 retention (TCK-159). In-memory species tracking (RV-P01) is already done (TCK-139).
- Regression test target: after mitosis `creatures.len()` +1 and the child moves in the next step.
