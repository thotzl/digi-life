---
id: TCK-147
title: SPAWN_FOOD Action: Implement or Remove
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B09
---

# TCK-147: SPAWN_FOOD Action: Implement or Remove

> Pool ticket: requires a user decision before implementation. Findings (RV-B09) below.

## Description
The UI sends `SPAWN_FOOD` on canvas interaction; the engine ignores it (now logged as unknown action).

## Proposed Scope
Decide: implement food spawning or remove the UI trigger.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B09 · P2 · static** — `frontend/ocean.ts` sends `SPAWN_FOOD` (right-click) and `CLIENT_LOG`; the engine match has no arm for them. Unknown actions are logged since TCK-141, but right-click food spawn is still a silent no-op. Implement the action or remove the UI. With the typed `Action` enum (TCK-157) this class of errors becomes a compile error.
