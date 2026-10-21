---
name: client-rendering
description: Rules for changing the Pixel DNA Life webview UI - views, router, Canvas renderers, camera and Preact Signals HUD (frontend/).
---
# client-rendering: Frontend Rules

## I. Knowledge (read first)
- Frontend structure and open findings: `wiki/frontend/structure.md`
- IPC contract: `wiki/architecture/ipc-protocol.md`

## II. Rules
- Only `navigateTo` in `frontend/core/router.ts` sends `SET_MODE`; `trainer.ts` must never send `SET_MODE: trainer` on load.
- The client is a renderer: never mutate authoritative simulation state, only send actions.
- Untrusted strings (names, genomes, logs) go through `textContent` or `escapeHtml` (`frontend/core/escape.ts`), never raw `innerHTML`.
- World data comes from the backend (`INIT_STATE`); do not generate or hardcode world geometry in the frontend.
- When touching IPC types, prefer types from `bindings/` over adding to `frontend/shared/types.ts` (RV-A03).
- Verify with `npm run build` (tsc + Vite).
