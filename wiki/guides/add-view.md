---
sources: [tickets/closed/TCK-160-frontend-structure.md]
---

# Guide: add a view

Read first: [[wiki/contracts/frontend-view]] and [[wiki/architecture/invariants]].

## Steps

1. Create `frontend/views/<name>/index.ts` exporting a `ViewModule<"<name>">` (`id`, `title`, `backendMode`,
   `create`, `mount`). Large views split the DOM into `views/<name>/layout.ts` (returns element references) and
   the pure state into `views/<name>/model.ts`. `backendMode` is required and only `"ocean" | "trainer"`: a view
   without its own simulation uses `"ocean"` like the main menu and the catalogue; entering it still suspends the
   other views (training pauses) (contract: Lifecycle).
2. `create(ctx)`: build the DOM once, keep references, bind element-local listeners. Use `ctx.ipc`, never Tauri.
   Styling: `styled("tag")` from `core/styled.ts`; icons are `html(...)` constants in `components/Icons.ts` (add one
   there when the view needs a new SVG, e.g. its launcher icon). There is no shared back button: copy
   `BackToMenuButton` of `views/catalogue/index.ts` (`[IconBack, "Main Menu"]`, `ctx.navigate("main-menu")`).
3. `mount(ctx)`: register window/document listeners, effects, `scope.frame` loops and dialogs (`openModal(scope, …)`)
   on `ctx.scope`. Mount `DiagnosticsPanel` there if the view has an inspector.
4. Backend events: `onEvent(event)` (switch on `event.type`). If the view runs a simulation, add `suspend` (pause it)
   and, if useful, `resume` (restore only what the user had running). Do not call `setMode`.
5. Selection: build it with a builder in `state/selection.ts` (add a `SelectionRef` variant for a new source) and
   read ids only through the accessor of your source.
6. Register: add the module to `VIEWS` in `frontend/views/index.ts`. Launcher (expected host edit in
   `views/main-menu/index.ts`): a `LauncherCard` calling `ctx.navigate("<name>")`, its own `&.card-<name>:hover`
   block in the `LauncherCard` CSS, and `LaunchersGrid`'s `repeat(3, 320px)` columns widened for a fourth card.
7. Tests: a DOM case of its own in `views/views.dom.test.ts` (contract: Test recipe); `views/<name>/model.test.ts`
   only if the view has pure logic; extend `core/router.test.ts` only if the lifecycle changes.
8. Docs, same commit: the contract (Purpose list), the Frontend view row of "Registered plugins" in
   `wiki/architecture/module-map.md`, `wiki/frontend/structure.md` (file table, the `VIEWS = [...]` row, the view
   count in "Views and lifecycle") and the view list in `frontend/views/README.md`.

## Done when

- The contract's test recipe passes (`npm test`, incl. the new DOM case).
- The project gate is green: `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`,
  `cargo test`, `npm test`, `npm run lint`, `npm run build`, `node scripts/check-links.mjs`, and the four CI
  reference runs match their committed reports (commands: `.agents/skills/plugin-authoring/SKILL.md`, Gate).
- Leaving the view stops its loops and listeners (no work while hidden).
