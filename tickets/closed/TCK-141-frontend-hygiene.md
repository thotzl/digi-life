---
id: TCK-141
title: Frontend Hygiene (safe DOM output, unused deps, unknown actions)
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
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

## Verification
- `frontend/core/escape.ts`: new `escapeHtml` helper; applied to DB/user-derived interpolations in `frontend/ocean.ts` (roster card id/name, catalogue modal source/name), `frontend/views/UnifiedDiagnosticsPanel.ts` (genome loci, catalogue modal), `frontend/views/CatalogueView.ts` (DB error text). Code-generated numbers, labels and SVG icons intentionally unescaped.
- `zustand` removed from `package.json`/`package-lock.json` (no imports existed).
- `src/server/engine.rs`: unknown client action types are logged instead of silently dropped.
- `npx tsc --noEmit`: 0 errors; `npm run build`: success; `cargo test`: 34 passed.
- Remaining risk moved to pool ticket TCK-160: `styled.ts applyChildren` treats strings starting with `<` as HTML.
