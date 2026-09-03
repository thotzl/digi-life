import "./styles.css";
import { effect } from "@preact/signals-core";
import { currentView } from "./core/router";
import { createMainMenu } from "./views/MainMenu";
import { createOceanView } from "./views/OceanView";
import { createTrainerView } from "./views/TrainerView";
import { createCatalogueView } from "./views/CatalogueView";

async function bootstrap() {
  const app = document.getElementById("app");
  if (!app) {
    console.error("[Boot] Fatal error: Mount element '#app' not found.");
    return;
  }

  // 1. Compile and mount all core SPA views once on startup
  const mainMenuEl = createMainMenu();
  const oceanEl = createOceanView();
  const trainerEl = createTrainerView();
  const catalogueEl = createCatalogueView();

  app.appendChild(mainMenuEl);
  app.appendChild(oceanEl);
  app.appendChild(trainerEl);
  app.appendChild(catalogueEl);

  // 2. Dynamically import simulation logics AFTER DOM is fully initialized
  // This guarantees that all document.getElementById queries succeed with zero null errors!
  await import("./ocean");
  await import("./trainer");

  // 3. Reactively manage the visible SPA container (safeguarding GPU canvas contexts)
  effect(() => {
    const activeView = currentView.value;
    
    mainMenuEl.style.display = activeView === "main-menu" ? "flex" : "none";
    oceanEl.style.display = activeView === "ocean" ? "block" : "none";
    trainerEl.style.display = activeView === "trainer" ? "block" : "none";
    catalogueEl.style.display = activeView === "catalogue" ? "block" : "none";

    // Set page tab titles matching the active state
    if (activeView === "main-menu") {
      document.title = "Pixel DNA - Laboratory Main Hub";
    } else if (activeView === "ocean") {
      document.title = "Ozean-Labor Substrate Simulation";
    } else if (activeView === "trainer") {
      document.title = "RL Evolutionary Training Chamber";
    } else if (activeView === "catalogue") {
      document.title = "Persistent Cryo-Frozen Species Library";
    }
  });

  console.log("[Boot] Pure TypeScript SPA bootstrapped successfully with lazy module initialization.");
}

// Start booting when DOM is ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", bootstrap);
} else {
  bootstrap();
}
