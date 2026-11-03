// Boot (TCK-160): create every registered view once, route to the main menu, subscribe the views to backend events,
// then ask the engine for its state (CLIENT_READY → INIT_STATE).
import "./styles.css";
import { call, onUiEvent, send } from "./core/ipc";
import { dispatchUiEvent, startRouter } from "./core/router";
import { selectionStore } from "./state/selection";
import { INITIAL_VIEW, VIEWS } from "./views";

function bootstrap() {
  const app = document.getElementById("app");
  if (!app) {
    console.error("[Boot] Fatal error: Mount element '#app' not found.");
    return;
  }
  startRouter(VIEWS, { ipc: { call, send }, selection: selectionStore }, { initial: INITIAL_VIEW, root: app });
  onUiEvent(dispatchUiEvent);
  void send({ type: "CLIENT_READY" });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrap);
} else {
  bootstrap();
}
