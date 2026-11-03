---
sources: [tickets/closed/TCK-160-frontend-structure.md]
---

# Guide: add a view

Read first: [[wiki/contracts/frontend-view]] and [[wiki/architecture/invariants]].

## Steps

1. Create `frontend/views/<name>/index.ts` exporting a `ViewModule<"<name>">` (`id`, `title`, `backendMode`,
   `create`, `mount`). Large views split the DOM into `views/<name>/layout.ts` (returns element references) and
   the pure state into `views/<name>/model.ts`.
2. `create(ctx)`: build the DOM once, keep references, bind element-local listeners. Use `ctx.ipc`, never Tauri.
3. `mount(ctx)`: register window/document listeners, effects, `scope.frame` loops and dialogs (`openModal(scope, …)`)
   on `ctx.scope`. Mount `DiagnosticsPanel` there if the view has an inspector.
4. Backend events: `onEvent(event)` (switch on `event.type`). If the view runs a simulation, add `suspend` (pause it)
   and, if useful, `resume` (restore only what the user had running). Do not call `setMode`.
5. Selection: build it with a builder in `state/selection.ts` (add a `SelectionRef` variant for a new source) and
   read ids only through the accessor of your source.
6. Register: add the module to `VIEWS` in `frontend/views/index.ts`; add a launcher calling `ctx.navigate("<name>")`.
7. Tests: `views/<name>/model.test.ts` for the pure logic; extend `core/router.test.ts` only if the lifecycle changes.
8. Docs: list the view in the frontend structure article (`wiki/frontend/structure.md`).

## Done when

- The contract's test recipe passes.
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green.
- Leaving the view stops its loops and listeners (no work while hidden).
