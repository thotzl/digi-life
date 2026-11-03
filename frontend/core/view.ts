// View seam (TCK-160): one module per screen, registered in `views/index.ts`. The router creates every view once,
// mounts the active one with a fresh `Scope` and disposes that scope when the view is left. Contract:
// wiki/contracts/frontend-view.md.
import type { SimMode } from "../../bindings/SimMode";
import type { UiEvent } from "../../bindings/UiEvent";
import type { Action, Command, Commands } from "./ipc";
import type { Scope } from "./scope";
import type { SelectionStore } from "../state/selection";

/** The IPC surface a view uses (a subset of `core/ipc.ts`; tests pass a fake). */
export interface ViewIpc {
  call<C extends Command>(command: C, ...args: Commands[C]["args"] extends void ? [] : [Commands[C]["args"]]): Promise<Commands[C]["result"]>;
  send(action: Action): Promise<boolean>;
}

export interface ViewContext {
  ipc: ViewIpc;
  selection: SelectionStore;
  /** In `create`: the app-lifetime scope (never disposed). In `mount`: the scope of this visit, disposed on leave. */
  scope: Scope;
  /** Switches the view (the router's `navigateTo`). */
  navigate(id: string): void;
}

export interface ViewModule<Id extends string = string> {
  readonly id: Id;
  /** Document title while the view is shown */
  readonly title: string;
  /** Backend mode the router sets when the view is entered (`set_mode`, invariant 1) */
  readonly backendMode: SimMode;
  /** Builds the DOM once at boot and keeps the element references; returns the root element. */
  create(ctx: ViewContext): HTMLElement;
  /** The view is shown: register global listeners, effects, animation loops on `ctx.scope`. */
  mount(ctx: ViewContext): void;
  /** The view is hidden; `ctx.scope` of the mount is already disposed. Optional extra cleanup. */
  unmount?(): void;
  /** Every backend event, whether the view is shown or not (keep model state; draw only while mounted). */
  onEvent?(event: UiEvent): void;
  /** Pause this view's simulation. The router calls it on every navigation for every view except the target,
   * before `set_mode`. */
  suspend?(): Promise<unknown>;
  /** The view was entered and the backend mode is set: resume what `suspend` paused, if the user had it running. */
  resume?(): Promise<unknown>;
}
