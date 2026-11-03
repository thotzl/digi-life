---
name: client-rendering
description: Pointers for changing the BroodBattle webview UI - views, router, Canvas renderers, camera and Preact Signals HUD (frontend/).
---
# client-rendering: Frontend Pointers

## I. Read first
- New screen: `.agents/skills/plugin-authoring/SKILL.md` (guide `wiki/guides/add-view.md`, contract
  `wiki/contracts/frontend-view.md`); new message: guide `wiki/guides/add-ipc-message.md`.
- Rules: `wiki/architecture/invariants.md` 1 (router owns the mode), 7, 13, 14, 15.
- Current state: `wiki/frontend/structure.md`; IPC list `wiki/architecture/ipc-protocol.md`.

## II. Task rules
- Verify with `npm test`, `npm run lint` and `npm run build`; controller changes get a case in the happy-dom suite
  `frontend/views/views.dom.test.ts`.
- A new organ kind needs a painter in `ORGAN_PAINTERS` (`frontend/render/organs.ts`).
