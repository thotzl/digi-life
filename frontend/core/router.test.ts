// Router and view lifecycle (TCK-160): mount/unmount with a per-visit scope, backend order pause → set_mode → resume,
// and entering the Ocean no longer pauses it.
import { describe, expect, it, vi } from "vitest";

vi.mock("./ipc", () => ({ setMode: vi.fn(() => Promise.resolve(null)) }));

import { currentView, dispatchUiEvent, navigateTo, startRouter } from "./router";
import type { Scope } from "./scope";
import type { ViewModule } from "./view";

const fakeElement = () => ({ style: { display: "" } }) as unknown as HTMLElement;

function setup() {
  const log: string[] = [];
  const sent: unknown[] = [];
  let oceanRunning = true;
  let wantsRunning = true;
  const scopes: Record<string, Scope> = {};
  const view = (id: string, backendMode: "ocean" | "trainer", extra: Partial<ViewModule> = {}): ViewModule => ({
    id, title: id, backendMode,
    create: () => fakeElement(),
    mount: (ctx) => { log.push(`mount ${id}`); scopes[id] = ctx.scope; },
    unmount: () => log.push(`unmount ${id}`),
    ...extra,
  });
  const ocean = view("ocean", "ocean", {
    suspend: async () => { if (oceanRunning) { sent.push({ type: "TOGGLE_SIMULATION", running: false }); oceanRunning = false; } },
    resume: async () => { if (wantsRunning && !oceanRunning) { sent.push({ type: "TOGGLE_SIMULATION", running: true }); oceanRunning = true; } },
    onEvent: (e) => log.push(`event ${e.type}`),
  });
  const trainer = view("trainer", "trainer", { suspend: async () => { sent.push({ type: "PAUSE_TRAINING" }); } });
  const views = [view("main-menu", "ocean"), ocean, trainer, view("catalogue", "ocean")];
  const setMode = vi.fn(async (mode: string) => { sent.push({ setMode: mode }); return mode; });
  startRouter(views, { ipc: {} as any, selection: {} as any }, { initial: "main-menu", setMode });
  return { log, sent, scopes, setMode, pauseByUser: () => { wantsRunning = false; } };
}

describe("router (TCK-160)", () => {
  it("entering the Ocean from the menu does not pause it", async () => {
    const { sent } = setup();
    await navigateTo("ocean");
    expect(sent).not.toContainEqual({ type: "TOGGLE_SIMULATION", running: false });
    expect(sent).toEqual([{ type: "PAUSE_TRAINING" }, { setMode: "ocean" }]);
    expect(currentView.value).toBe("ocean");
  });

  it("pauses the other simulations before set_mode and resumes the Ocean the user had running", async () => {
    const { sent } = setup();
    await navigateTo("ocean");
    sent.length = 0;
    await navigateTo("trainer");
    expect(sent).toEqual([{ type: "TOGGLE_SIMULATION", running: false }, { setMode: "trainer" }]);
    sent.length = 0;
    await navigateTo("ocean");
    expect(sent).toEqual([{ type: "PAUSE_TRAINING" }, { setMode: "ocean" }, { type: "TOGGLE_SIMULATION", running: true }]);
  });

  it("does not resume an Ocean the user paused", async () => {
    const { sent, pauseByUser } = setup();
    await navigateTo("ocean");
    pauseByUser();
    await navigateTo("main-menu");
    sent.length = 0;
    await navigateTo("ocean");
    expect(sent).not.toContainEqual({ type: "TOGGLE_SIMULATION", running: true });
  });

  it("disposes the scope of the left view and mounts the target with a fresh one", async () => {
    const { log, scopes } = setup();
    await navigateTo("ocean");
    const first = scopes.ocean;
    let disposed = false;
    first.add(() => { disposed = true; });
    await navigateTo("catalogue");
    expect(disposed).toBe(true);
    expect(first.disposed).toBe(true);
    await navigateTo("ocean");
    expect(scopes.ocean).not.toBe(first);
    expect(log.filter((l) => l.startsWith("mount") || l.startsWith("unmount"))).toEqual([
      "mount main-menu", "unmount main-menu", "mount ocean", "unmount ocean", "mount catalogue", "unmount catalogue", "mount ocean",
    ]);
  });

  it("navigating to the shown view or an unknown id changes nothing", async () => {
    const { setMode } = setup();
    await navigateTo("main-menu");
    await navigateTo("creator");
    expect(setMode).not.toHaveBeenCalled();
    expect(currentView.value).toBe("main-menu");
  });

  it("delivers events to views that are not shown", () => {
    const { log } = setup();
    dispatchUiEvent({ type: "DATABASE_CHANGED" } as any);
    expect(log).toContain("event DATABASE_CHANGED");
  });
});
