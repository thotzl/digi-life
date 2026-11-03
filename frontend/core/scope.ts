// Teardown scope (TCK-160): everything a view starts while it is shown (global listeners, signal effects, animation
// loops, timers) is registered here and stopped by one `dispose()` when the view is left. Contract:
// wiki/contracts/frontend-view.md.
import { effect } from "@preact/signals-core";

type FrameRequest = (cb: FrameRequestCallback) => number;
type FrameCancel = (id: number) => void;

export class Scope {
  private disposers: (() => void)[] = [];
  private closed = false;

  constructor(
    private readonly raf: FrameRequest = (cb) => requestAnimationFrame(cb),
    private readonly caf: FrameCancel = (id) => cancelAnimationFrame(id),
  ) {}

  get disposed(): boolean {
    return this.closed;
  }

  /** Runs `dispose` when the scope is disposed (at once if it already is). */
  add(dispose: () => void): void {
    if (this.closed) {
      dispose();
      return;
    }
    this.disposers.push(dispose);
  }

  /** `addEventListener` that is removed on dispose. */
  listen<K extends keyof WindowEventMap>(target: Window, type: K, handler: (e: WindowEventMap[K]) => void, options?: AddEventListenerOptions): void;
  listen<K extends keyof DocumentEventMap>(target: Document, type: K, handler: (e: DocumentEventMap[K]) => void, options?: AddEventListenerOptions): void;
  listen(target: EventTarget, type: string, handler: (e: Event) => void, options?: AddEventListenerOptions): void;
  listen(target: EventTarget, type: string, handler: (e: never) => void, options?: AddEventListenerOptions): void {
    if (this.closed) return;
    const listener = handler as EventListener;
    target.addEventListener(type, listener, options);
    this.add(() => target.removeEventListener(type, listener, options));
  }

  /** Signal `effect` that is disposed with the scope. */
  effect(fn: () => void): void {
    if (this.closed) return;
    this.add(effect(fn));
  }

  /** `requestAnimationFrame` loop calling `draw` every frame until the scope is disposed (or `stop` is called). */
  frame(draw: (time: number) => void): () => void {
    let id = 0;
    let running = !this.closed;
    const loop = (time: number) => {
      if (!running) return;
      draw(time);
      if (running) id = this.raf(loop);
    };
    const stop = () => {
      running = false;
      if (id) this.caf(id);
      id = 0;
    };
    if (running) {
      id = this.raf(loop);
      this.add(stop);
    }
    return stop;
  }

  /** `setTimeout` cleared on dispose. */
  timeout(fn: () => void, ms: number): void {
    if (this.closed) return;
    const id = setTimeout(fn, ms);
    this.add(() => clearTimeout(id));
  }

  /** Child scope, disposed with this one (or earlier on its own, e.g. a closed modal). */
  child(): Scope {
    const child = new Scope(this.raf, this.caf);
    this.add(() => child.dispose());
    return child;
  }

  /** Stops everything registered, newest first. Idempotent. */
  dispose(): void {
    if (this.closed) return;
    this.closed = true;
    const disposers = this.disposers;
    this.disposers = [];
    for (let i = disposers.length - 1; i >= 0; i--) {
      try {
        disposers[i]();
      } catch (e) {
        console.error("[Scope] dispose failed:", e);
      }
    }
  }
}
