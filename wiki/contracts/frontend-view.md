---
sources: [tickets/closed/TCK-160-frontend-structure.md, frontend/core/view.ts, frontend/core/scope.ts, frontend/core/router.ts, frontend/views/index.ts, frontend/state/selection.ts]
---

# Contract: frontend view

## Purpose

A view is one screen of the webview: main menu, Ocean Laboratory, Training Room, Creature Catalogue today (ids
`main-menu`, `ocean`, `trainer`, `catalogue`; current list: the Frontend view row of "Registered plugins" in
[[wiki/architecture/module-map]]); a creator
(TCK-111), an arena or a new trainer room's live view are planned. Each view is one `ViewModule` registered in
`frontend/views/index.ts`. The router creates every view once at boot, shows one at a time and owns the backend mode.
Everything a view starts while shown (global listeners, signal effects, animation loops, timers, dialogs) lives in a
`Scope` that the router disposes when the view is left (TCK-160). Current state: [[wiki/frontend/structure]].

## Signature

```ts
// frontend/core/view.ts
export interface ViewIpc { call(command, ...args): Promise<result>; send(action: Action): Promise<boolean> }
export interface ViewContext {
  ipc: ViewIpc;                 // core/ipc.ts call/send (tests pass a fake)
  selection: SelectionStore;    // state/selection.ts
  scope: Scope;                 // create: app lifetime; mount: this visit, disposed on leave
  navigate(id: string): void;   // router navigateTo
}
export interface ViewModule<Id extends string = string> {
  readonly id: Id;
  readonly title: string;           // document.title while shown
  readonly backendMode: SimMode;    // required, "ocean" | "trainer", set on entering (no view without a mode)
  create(ctx: ViewContext): HTMLElement;   // once at boot: build DOM, keep element references
  mount(ctx: ViewContext): void;           // shown: register listeners/effects/loops on ctx.scope
  unmount?(): void;                        // hidden (mount scope already disposed)
  onEvent?(event: UiEvent): void;          // every backend event, shown or not
  suspend?(): Promise<unknown>;            // pause own simulation (called for every view except the target)
  resume?(): Promise<unknown>;             // after set_mode, on the target
}

// frontend/core/scope.ts
class Scope { add(fn); listen(target, type, handler, options?); effect(fn); frame(draw): stop; timeout(fn, ms);
              child(): Scope; dispose(); readonly disposed }

// frontend/state/selection.ts
type SelectionRef = { source: "ocean"; creatureId: number } | { source: "fossil"; speciesId: string }
                  | { source: "trainer"; sandboxId: number } | { source: "catalogue"; catalogueId: string };
interface Selection { ref; name; taxa; genome; methylations; phenotype | null; fitness | null; synapseWeights }
interface Vitals { status; energy; maxEnergy; age: number | null }
interface SelectionStore { current; select(sel, vitals?); setPhenotype(ref, p); updateVitals(ref, v);
                           updateActivations(ref, a); rename(ref, name); clear() }
signals: selection, vitals, activations; accessors oceanCreatureId / trainerSandboxId / catalogueId;
builders fromOceanCreature / fromFossil / fromSandbox / fromCatalogue; catalogueEntry(sel, name)
```

Shared components a view composes: `components/Modal.ts` (`openModal(scope, options)`, `checkboxRow`),
`components/CreaturePreview.ts`, `components/CataloguePicker.ts`, `components/DiagnosticsPanel.ts` (inspector with
`sources`, `save`, `onAssign`; `mount(scope)`), `render/worldLayer.ts`, `shared/creatureLabels.ts`, `shared/warp.ts`.

## Lifecycle / call order

1. Boot (`main.ts`): `startRouter(VIEWS, ctx, { initial: "main-menu", root })` calls `create` of every view in
   registry order with the app scope, hides all, mounts the initial view. Then `onUiEvent(dispatchUiEvent)` and
   `CLIENT_READY`. No mode switch at boot (the backend starts in Ocean mode).
2. `navigateTo(id)`: dispose the shown view's scope → its `unmount` → hide it; show the target, new scope, target
   `mount`. Then, serialized across navigations: `suspend` of every other view → `setMode(target.backendMode)` →
   target `resume`. `setMode` is sent on every navigation to another view, also when the mode does not change (the
   engine then skips the old mode's `on_exit` but runs the target mode's `on_enter` again); navigating to the shown
   view or an unknown id does nothing (`router.test.ts`: "navigating to the shown view or an unknown id changes
   nothing"). A view without its own simulation (main menu, catalogue) sets `backendMode: "ocean"`. Entering it
   still suspends every other view: the trainer's `suspend` always sends `PAUSE_TRAINING`, the Ocean's pauses a
   running Ocean (its `resume` restarts it when the user returns to the Ocean).
3. Every backend event reaches every view's `onEvent`, shown or not (`INIT_STATE` arrives while the menu is shown).
   Keep model state there; draw only from mount-scoped loops.

## Invariants

- Only the router calls `setMode` (plus the trainer's unload reset) — [[wiki/architecture/invariants]] 1.
- No `document.getElementById` for a view's own elements: `create` builds the DOM and keeps references (layout
  builders `views/<name>/layout.ts` return them).
- No global listener, `effect`, `requestAnimationFrame` or `setTimeout` loop outside a scope; nothing is registered
  at module import.
- A view reads selection ids only through its source's accessor; a selection of another source is not its own.
- Types from `bindings/` only, world data from the backend ([[wiki/architecture/invariants]] 14); untrusted strings
  as text ([[wiki/architecture/invariants]] 15).
- `suspend` / `resume` change only the view's own simulation, and `resume` restores only what the user had running.

## Registration point

One entry in `VIEWS` in `frontend/views/index.ts` (`ViewId` is derived from it), plus a launcher or button that
calls `ctx.navigate(id)`.

## Config keys

None.

## Events / IPC

Consumes `UiEvent`s through `onEvent`; sends actions and calls commands through `ctx.ipc`
([[wiki/contracts/ipc-message]]). The mode switch (`set_mode`) is the router's.

## Test recipe

Targeted subset (the guide's "Done when" is the gate): `npm test` runs `frontend/core/router.test.ts` (mount/unmount
order, scope disposal, pause → set_mode → resume, Ocean not paused on entry), `core/scope.test.ts`,
`state/selection.test.ts` and the happy-dom suite `views/views.dom.test.ts`. Its first case ("creates every
registered view once") covers a new view automatically. The teardown case hard-codes `["ocean", "trainer",
"catalogue"]` and expects global listeners and a frame loop for every id except `catalogue`, so a new view adds an
`it(...)` case of its own inside the `describe` block (TCK-161) instead of joining that list:

```ts
it("<name>: leaving the view removes its listeners and loops", async () => {
  const { listeners, frames, baseline, elements, modes } = boot();
  await navigateTo("<name>");
  expect(elements.get("<name>")!.style.display).toBe("");
  // view-specific checks: buttons send the expected actions (`sent`), events update the DOM (`event(...)`)
  await navigateTo("main-menu");
  expect(listeners.count()).toBe(baseline.listeners);
  expect(frames.pending()).toBe(0);
  expect(modes).toEqual(["<backendMode>", "ocean"]);
});
```

A model test (`views/<name>/model.test.ts`, e.g. `views/ocean/model.test.ts`) is needed only when the view has pure
logic of its own.

## Minimal example

```ts
// frontend/views/arena/index.ts
import type { ViewModule } from "../../core/view";

export const arenaView: ViewModule<"arena"> = {
  id: "arena",
  title: "BroodBattle – Arena",
  backendMode: "ocean",
  create(ctx) {
    const root = document.createElement("div");
    const back = document.createElement("button");
    back.textContent = "Main Menu";
    back.addEventListener("click", () => ctx.navigate("main-menu"));
    root.appendChild(back);
    return root;
  },
  mount(ctx) {
    ctx.scope.listen(window, "keydown", (e) => { if (e.key === "Escape") ctx.navigate("main-menu"); });
  },
};
// frontend/views/index.ts: export const VIEWS = [mainMenuView, oceanView, trainerView, catalogueView, arenaView] as const;
```
