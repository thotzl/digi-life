---
name: client-rendering
description: Rules for changing the Pixel DNA Life webview UI - views, router, Canvas renderers, camera and Preact Signals HUD (frontend/).
---
# client-rendering: Frontend Rules

## I. Knowledge (read first)
- Frontend structure and open findings: `wiki/frontend/structure.md`
- IPC contract: `wiki/architecture/ipc-protocol.md`

## II. Rules
- Only `navigateTo` in `frontend/core/router.ts` switches the backend mode (`setMode` → command `set_mode`); views never call `setMode` (the trainer view only resets to Ocean on unload). New screens are `ViewModule`s: contract `wiki/contracts/frontend-view.md`, guide `wiki/guides/add-view.md`; global listeners, effects and loops only on the mount `Scope`.
- Talk to the backend only through `frontend/core/ipc.ts` (`call`, `send`, `onUiEvent`), never `invoke`/`listen` directly; errors are reported by its toast hook. New messages: guide `wiki/guides/add-ipc-message.md`, contract `wiki/contracts/ipc-message.md`.
- The client is a renderer: never mutate authoritative simulation state, only send actions.
- Untrusted strings (names, genomes, logs) go through `textContent` or `escapeHtml` (`frontend/core/escape.ts`), never raw `innerHTML`.
- `styled` children: strings are always text; static markup (icon SVG) only through `html()` from `frontend/core/styled.ts`. Never wrap user or backend data in `html()`. The CSP in `tauri.conf.json` forbids inline scripts and remote origins.
- World data comes from the backend (`INIT_STATE`); do not generate or hardcode world geometry in the frontend.
- Types only from `bindings/`; no hand-written model types in the frontend.
- Verify with `npm run build` (tsc + Vite).
