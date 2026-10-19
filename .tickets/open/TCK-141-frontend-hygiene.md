---
id: TCK-141
title: Frontend Hygiene (safe DOM output, unused deps, unknown actions)
status: open
assigned: Claude Code
created_at: 2026-10-19
revisit: RV-F02, RV-F07, RV-B09
---

# TCK-141: Frontend Hygiene

## Description
- `innerHTML` with interpolated DB/user data (latin names, catalogue names, logs) while `csp` is `null`.
- The `zustand` dependency is unused.
- The engine silently drops unknown actions (`SPAWN_FOOD`, `CLIENT_LOG`).

## Scope
- Escape interpolated data in `innerHTML` templates (visual output identical).
- Remove the unused dependency.
- The engine logs unknown action types.

## Acceptance
- `npm run build` green, `cargo test` green, UI renders identically.
