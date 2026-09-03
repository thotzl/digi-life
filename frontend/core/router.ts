import { signal } from "@preact/signals-core";
import { safeInvoke } from "../api";

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
  // 1. Absolute State Clean-up: Pause all client-side rendering loops
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "TOGGLE_SIMULATION", running: false }) }).catch(() => {});
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "PAUSE_TRAINING" }) }).catch(() => {});

  // 2. Synchronize active simulation mode in the Rust backend
  if (view === "ocean") {
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "ocean" }) }).catch(() => {});
  } else if (view === "trainer") {
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "trainer" }) }).catch(() => {});
  } else {
    // Default back to ocean mode (idle) when returning to the main menu
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "ocean" }) }).catch(() => {});
  }

  // 3. Update view signal
  currentView.value = view;
  console.log(`[Router] Clean state shift. Switched active view to: '${view}'`);
}
