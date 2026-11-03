// @vitest-environment happy-dom
// DOM tests of the real views (TCK-161): every registered view is created and routed in happy-dom with a fake IPC.
// Leaving a view must remove what its mount scope registered (window/document listeners, frame loops, dialogs);
// the Ocean and trainer controllers turn backend events into DOM state and user input into actions.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const invokeMock = vi.fn((..._args: unknown[]) => Promise.resolve(null));
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invokeMock(...args) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

import type { CatalogueCreature } from "../../bindings/CatalogueCreature";
import type { UiEvent } from "../../bindings/UiEvent";
import type { Action } from "../core/ipc";
import { currentView, dispatchUiEvent, navigateTo, startRouter } from "../core/router";
import type { ViewIpc } from "../core/view";
import { selectionStore } from "../state/selection";
import { INITIAL_VIEW, VIEWS } from "./index";

const CREATURE: CatalogueCreature = {
  id: "c1", name: "Testus", genome: "ATG", source: "Ocean Substrate", fitness: 1, carnivory: 0, methylations: [],
  synapseWeights: [], createdAt: "2026-11-03 10:00:00",
};

const TRAINER_STATE = {
  isRunning: false, population: 16, N: 4, warpSpeed: 1, eliteRatio: 0.25, mutationRate: 2, mutationOpsMin: 1,
  mutationOpsLimit: 3, inflowRate: 0.1, hofRate: 0.1, multiTrial: true, isHeadless: false, runId: "run_a",
  lamarckian: false, trainingMode: "standard", level: 0, levelAuto: false, pendingMode: null, generation: 1,
} as const;

/** Window/document listeners currently registered (add minus remove), by target. */
function trackListeners() {
  const active = new Map<EventTarget, Set<string>>();
  const key = (type: string, handler: unknown, options: unknown) => {
    const capture = typeof options === "boolean" ? options : !!(options as AddEventListenerOptions | undefined)?.capture;
    return `${type}|${capture}|${ids.get(handler) ?? ids.set(handler, ids.size).get(handler)}`;
  };
  const ids = new Map<unknown, number>();
  for (const target of [window, document] as EventTarget[]) {
    const set = new Set<string>();
    active.set(target, set);
    const add = target.addEventListener.bind(target);
    const remove = target.removeEventListener.bind(target);
    vi.spyOn(target, "addEventListener").mockImplementation((type, handler, options) => {
      set.add(key(type, handler, options));
      add(type, handler, options);
    });
    vi.spyOn(target, "removeEventListener").mockImplementation((type, handler, options) => {
      set.delete(key(type, handler, options));
      remove(type, handler, options);
    });
  }
  return { count: () => [...active.values()].reduce((n, s) => n + s.size, 0) };
}

/** Manual animation frames: loops only advance on `step()`, pending callbacks are observable. */
function trackFrames() {
  const pending = new Map<number, FrameRequestCallback>();
  let next = 1;
  vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => {
    pending.set(next, cb);
    return next++;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => pending.delete(id));
  return {
    pending: () => pending.size,
    step() {
      const due = [...pending.values()];
      pending.clear();
      due.forEach((cb) => cb(performance.now()));
    },
  };
}

function boot() {
  const sent: Action[] = [];
  const calls: string[] = [];
  const modes: string[] = [];
  const results: Record<string, unknown> = {
    get_registered_species: [],
    get_trainer_runs: [],
    get_catalogue_creatures: [CREATURE],
  };
  const ipc: ViewIpc = {
    call: ((command: string) => {
      calls.push(command);
      return Promise.resolve(results[command] ?? null);
    }) as ViewIpc["call"],
    send: (action) => {
      sent.push(action);
      return Promise.resolve(true);
    },
  };
  const app = document.createElement("div");
  app.id = "app";
  document.body.appendChild(app);
  const listeners = trackListeners();
  const frames = trackFrames();
  const baseline = { listeners: 0 };
  const elements = startRouter(VIEWS, { ipc, selection: selectionStore }, {
    initial: INITIAL_VIEW,
    root: app,
    setMode: async (mode) => {
      modes.push(mode);
      return mode;
    },
  });
  baseline.listeners = listeners.count();
  const event = (e: UiEvent) => dispatchUiEvent(e);
  return { app, elements, sent, calls, modes, listeners, frames, baseline, event };
}

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("views in the DOM (TCK-161)", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    selectionStore.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("creates every registered view once and shows only the initial one", () => {
    const { app, elements } = boot();
    expect([...elements.keys()]).toEqual(VIEWS.map((v) => v.id));
    expect(app.children).toHaveLength(VIEWS.length);
    for (const [id, el] of elements) expect(el.style.display).toBe(id === INITIAL_VIEW ? "" : "none");
    expect(currentView.value).toBe(INITIAL_VIEW);
    expect(document.title).toBe(VIEWS.find((v) => v.id === INITIAL_VIEW)!.title);
  });

  it("leaving a view removes its global listeners and stops its frame loops", async () => {
    const { listeners, frames, baseline, elements } = boot();
    expect(frames.pending()).toBe(0);
    for (const id of ["ocean", "trainer", "catalogue"]) {
      await navigateTo(id);
      expect(elements.get(id)!.style.display).toBe("");
      if (id !== "catalogue") {
        expect(listeners.count()).toBeGreaterThan(baseline.listeners);
        expect(frames.pending()).toBeGreaterThan(0);
        frames.step(); // a frame runs and re-arms the loop
        expect(frames.pending()).toBeGreaterThan(0);
      }
      await navigateTo("main-menu");
      expect(elements.get(id)!.style.display).toBe("none");
      expect(listeners.count()).toBe(baseline.listeners);
      expect(frames.pending()).toBe(0);
    }
  });

  it("only the router switches the backend mode; the trainer view never sets it on its own (invariant 1)", async () => {
    const { modes, calls } = boot();
    await navigateTo("trainer");
    await navigateTo("ocean");
    expect(modes).toEqual(["trainer", "ocean"]);
    expect(calls).not.toContain("set_mode");
    expect(invokeMock).not.toHaveBeenCalledWith("set_mode", expect.anything());
  });

  it("an open dialog is removed when its view is left", async () => {
    const { app, frames } = boot();
    await navigateTo("ocean");
    const ocean = app.children[VIEWS.findIndex((v) => v.id === "ocean")] as HTMLElement;
    const before = app.children.length;
    const loadButton = [...ocean.querySelectorAll("button")].find((b) => b.textContent?.includes("Catalog"));
    expect(loadButton).toBeDefined();
    loadButton!.click();
    await flush();
    expect(app.children.length).toBe(before + 1);
    expect(app.textContent).toContain("Inject Species");
    await navigateTo("main-menu");
    expect(app.children.length).toBe(before);
    expect(app.textContent).not.toContain("Inject Species");
    expect(frames.pending()).toBe(0);
  });

  it("Ocean: backend events drive the controls and the log; the toggle sends the opposite state", async () => {
    const { sent, event, elements } = boot();
    const root = elements.get("ocean")!;
    const toggle = root.querySelector<HTMLButtonElement>("#btn-toggle-sim")!;
    event({
      type: "INIT_STATE", highestGeneration: 3, running: false, seed: "s", foodPellets: [], creatures: [],
      world: { seed: "s", width: 1000, height: 800, obstacles: [], vents: [], biomes: [] },
      rules: {} as never,
    });
    expect(toggle.textContent).toBe("Resume Substrate");
    event({ type: "SIM_STATE", running: true, speed: 1 });
    expect(toggle.textContent).toBe("Pause Substrate");
    event({ type: "LOG_EVENT", message: "<b>mutation</b> happened", logType: "mutation" });
    const logs = root.querySelector("#terminal-logs")!;
    expect(logs.lastElementChild?.textContent).toContain("<b>mutation</b> happened"); // text, not markup
    event({ type: "TELEMETRY_TICK", highestGeneration: 4, creatures: [], newCreatures: [], selectedBrain: null });
    expect(root.querySelector("#stat-population")!.textContent).toBe("0");

    toggle.click();
    expect(sent).toContainEqual({ type: "TOGGLE_SIMULATION", running: false });
  });

  it("Ocean: the router's pause does not change the user's choice, so re-entering resumes only a running Ocean", async () => {
    const { sent, event, elements } = boot();
    event({
      type: "INIT_STATE", highestGeneration: 1, running: true, seed: "s", foodPellets: [], creatures: [],
      world: { seed: "s", width: 1000, height: 800, obstacles: [], vents: [], biomes: [] }, rules: {} as never,
    });
    await navigateTo("ocean");
    await navigateTo("trainer");
    expect(sent).toContainEqual({ type: "TOGGLE_SIMULATION", running: false });
    event({ type: "SIM_STATE", running: false, speed: 1 });
    sent.length = 0;
    await navigateTo("ocean");
    expect(sent).toContainEqual({ type: "TOGGLE_SIMULATION", running: true });
    event({ type: "SIM_STATE", running: true, speed: 1 }); // backend echo

    // User pauses: leaving and re-entering does not resume
    elements.get("ocean")!.querySelector<HTMLButtonElement>("#btn-toggle-sim")!.click();
    expect(sent).toContainEqual({ type: "TOGGLE_SIMULATION", running: false });
    event({ type: "SIM_STATE", running: false, speed: 1 });
    await navigateTo("main-menu");
    sent.length = 0;
    await navigateTo("ocean");
    expect(sent).not.toContainEqual({ type: "TOGGLE_SIMULATION", running: true });
  });

  it("trainer: state events set the controls, start/pause sends the action, leaving pauses training", async () => {
    const { sent, event, elements } = boot();
    const root = elements.get("trainer")!;
    const start = root.querySelector<HTMLButtonElement>("#btn-start")!;
    await navigateTo("trainer");
    event({ type: "TRAINER_STATE_CHANGED", ...TRAINER_STATE, isRunning: true });
    expect(start.textContent).toBe("Pause Training");
    event({ type: "TRAINER_RESET_COMPLETED", ...TRAINER_STATE });
    expect(start.textContent).toBe("Start Training");
    expect(root.querySelector("#stat-gen")!.textContent).toBe("1");

    sent.length = 0;
    start.click();
    expect(sent).toEqual([{ type: "START_TRAINING" }]);
    start.click();
    expect(sent).toEqual([{ type: "START_TRAINING" }, { type: "PAUSE_TRAINING" }]);

    sent.length = 0;
    await navigateTo("ocean");
    expect(sent[0]).toEqual({ type: "PAUSE_TRAINING" });
  });

  it("trainer: no hyperparameters are sent before the backend state arrived", async () => {
    const { sent } = boot();
    await navigateTo("trainer");
    expect(sent.some((a) => a.type === "UPDATE_TRAINER_HYPERPARAMS")).toBe(false);
  });
});
