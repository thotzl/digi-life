// View router (TCK-160): creates every registered `ViewModule` once, shows one at a time with a per-visit `Scope`,
// and owns the backend mode: `navigateTo` is the only caller of `setMode` besides the trainer's unload reset
// (invariant 1). Contract: wiki/contracts/frontend-view.md.
import { signal } from "@preact/signals-core";

import type { SimMode } from "../../bindings/SimMode";
import type { UiEvent } from "../../bindings/UiEvent";
import { setMode as ipcSetMode } from "./ipc";
import { Scope } from "./scope";
import type { ViewContext, ViewModule } from "./view";

/** Id of the shown view ("" before `startRouter`). */
export const currentView = signal<string>("");

interface Entry {
  module: ViewModule;
  element: HTMLElement;
}

export interface RouterOptions {
  /** View shown at boot (no mode switch: the backend starts in Ocean mode) */
  initial: string;
  /** Parent of the view elements; omitted in tests */
  root?: HTMLElement;
  /** Mode switch; defaults to `setMode` of `core/ipc.ts` (tests pass a fake) */
  setMode?: (mode: SimMode) => Promise<unknown>;
}

const entries = new Map<string, Entry>();
let active: { entry: Entry; scope: Scope } | null = null;
let baseContext: Omit<ViewContext, "scope"> | null = null;
let switchMode: (mode: SimMode) => Promise<unknown> = ipcSetMode;
// Backend steps of successive navigations run in order (pause → set_mode → resume per navigation)
let backendQueue: Promise<void> = Promise.resolve();

function context(scope: Scope): ViewContext {
  return { ...baseContext!, scope };
}

function show(id: string, entry: Entry) {
  entry.element.style.display = "";
  currentView.value = id;
  if (typeof document !== "undefined") document.title = entry.module.title;
  const scope = new Scope();
  active = { entry, scope };
  entry.module.mount(context(scope));
}

/** Creates all views (once), appends them to `root` and shows `initial`. Returns the root elements by view id. */
export function startRouter(views: readonly ViewModule[], ctx: Omit<ViewContext, "scope" | "navigate">, options: RouterOptions): Map<string, HTMLElement> {
  entries.clear();
  active = null;
  backendQueue = Promise.resolve();
  switchMode = options.setMode ?? ipcSetMode;
  baseContext = { ...ctx, navigate: (id) => void navigateTo(id) };
  const appScope = new Scope();
  const elements = new Map<string, HTMLElement>();
  for (const module of views) {
    if (entries.has(module.id)) throw new Error(`duplicate view id '${module.id}'`);
    const element = module.create(context(appScope));
    element.style.display = "none";
    options.root?.appendChild(element);
    entries.set(module.id, { module, element });
    elements.set(module.id, element);
  }
  const initial = entries.get(options.initial);
  if (!initial) throw new Error(`unknown initial view '${options.initial}'`);
  show(options.initial, initial);
  return elements;
}

/**
 * Switches to view `id`: disposes the scope of the shown view, shows and mounts the target, then (in this order)
 * pauses every other view's simulation (`suspend`), sets the target's backend mode and lets the target `resume`.
 * Resolves when the backend steps are done.
 */
export function navigateTo(id: string): Promise<void> {
  const target = entries.get(id);
  if (!target) {
    console.error(`[Router] unknown view '${id}'`);
    return backendQueue;
  }
  if (active?.entry === target) return backendQueue;

  if (active) {
    active.scope.dispose();
    active.entry.module.unmount?.();
    active.entry.element.style.display = "none";
  }
  show(id, target);

  backendQueue = backendQueue.then(async () => {
    for (const entry of entries.values()) {
      if (entry !== target) await entry.module.suspend?.();
    }
    await switchMode(target.module.backendMode);
    await target.module.resume?.();
  }).catch((e) => console.error("[Router] backend switch failed:", e));
  return backendQueue;
}

/** Hands a backend event to every view (shown or not). One failing view does not stop the others. */
export function dispatchUiEvent(event: UiEvent) {
  for (const { module } of entries.values()) {
    try {
      module.onEvent?.(event);
    } catch (e) {
      console.error(`[Router] view '${module.id}' failed on ${event.type}:`, e);
    }
  }
}
