import { signal } from "@preact/signals-core";
import { send, setMode } from "./ipc";

export type ActiveView = "main-menu" | "ocean" | "trainer" | "catalogue" | "creator";

/**
 * SSOT Navigation Signal governing the active visible SPA container.
 */
export const currentView = signal<ActiveView>("main-menu");

/**
 * Performs a highly disciplined, thread-safe view transition.
 * Automatically halts any active background simulation loops and synchronizes
 * the active mode of the Rust backend.
 */
export function navigateTo(view: ActiveView) {
  // 1. Pause both simulations, then 2. switch the backend mode (the only `set_mode` caller besides the trainer's
  // unload reset, invariant 1). The calls run in this order; errors are reported by the IPC client (toast). The main
  // menu and the catalogue idle in Ocean mode.
  const mode = view === "trainer" ? "trainer" : "ocean";
  (async () => {
    await send({ type: "TOGGLE_SIMULATION", running: false });
    await send({ type: "PAUSE_TRAINING" });
    await setMode(mode);
  })();

  // 3. Update view signal
  currentView.value = view;
  console.log(`[Router] Clean state shift. Switched active view to: '${view}'`);
}
