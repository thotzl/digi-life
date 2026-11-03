import { signal } from "@preact/signals-core";
import { describe, expect, it } from "vitest";

import { Scope } from "./scope";

function fakeFrames() {
  const pending = new Map<number, FrameRequestCallback>();
  let next = 1;
  return {
    raf: (cb: FrameRequestCallback) => { pending.set(next, cb); return next++; },
    caf: (id: number) => { pending.delete(id); },
    tick() { const cbs = [...pending.values()]; pending.clear(); cbs.forEach((cb) => cb(0)); },
    get count() { return pending.size; },
  };
}

describe("Scope (TCK-160)", () => {
  it("removes listeners, effects and frame loops on dispose", () => {
    const frames = fakeFrames();
    const scope = new Scope(frames.raf, frames.caf);
    const target = new EventTarget();
    let events = 0;
    scope.listen(target, "ping", () => events++);
    const s = signal(0);
    let runs = 0;
    scope.effect(() => { s.value; runs++; });
    let draws = 0;
    scope.frame(() => draws++);

    target.dispatchEvent(new Event("ping"));
    s.value = 1;
    frames.tick();
    frames.tick();
    expect([events, runs, draws]).toEqual([1, 2, 2]);

    scope.dispose();
    target.dispatchEvent(new Event("ping"));
    s.value = 2;
    frames.tick();
    expect([events, runs, draws]).toEqual([1, 2, 2]);
    expect(frames.count).toBe(0);
  });

  it("child scopes die with the parent; registering on a disposed scope runs nothing", () => {
    const parent = new Scope(() => 0, () => {});
    const child = parent.child();
    let closed = false;
    child.add(() => { closed = true; });
    parent.dispose();
    expect(closed).toBe(true);
    let late = false;
    parent.add(() => { late = true; });
    expect(late).toBe(true); // disposed immediately
  });
});
