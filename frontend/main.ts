import "./ocean";
import "./trainer";
import { safeInvoke } from "./api";

const tabOcean = document.getElementById("tab-ocean") as HTMLButtonElement;
const tabTrainer = document.getElementById("tab-trainer") as HTMLButtonElement;

const oceanView = document.getElementById("ocean-view-container") as HTMLDivElement;
const trainerView = document.getElementById("trainer-view-container") as HTMLDivElement;

const navIndicator = document.getElementById("nav-indicator") as HTMLSpanElement;
const navBadge = document.getElementById("nav-badge") as HTMLSpanElement;
const navSubtitle = document.getElementById("nav-subtitle") as HTMLParagraphElement;
const navHelp = document.getElementById("nav-help") as HTMLDivElement;

function pauseAllSimulations() {
  // Fundamentally suspend all background simulations (Ocean and Trainer) upon tab/view changes
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "TOGGLE_SIMULATION", running: false }) }).catch(() => {});
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "PAUSE_TRAINING" }) }).catch(() => {});
}

function switchToOcean() {
  pauseAllSimulations();

  if (oceanView) oceanView.style.display = "block";
  if (trainerView) trainerView.style.display = "none";

  if (tabOcean) {
    tabOcean.style.color = "#00f2fe";
    tabOcean.style.borderBottom = "2px solid #00f2fe";
    tabOcean.style.fontWeight = "bold";
    tabOcean.style.textShadow = "0 0 8px rgba(0, 242, 254, 0.4)";
  }

  if (tabTrainer) {
    tabTrainer.style.color = "#94a3b8";
    tabTrainer.style.borderBottom = "";
    tabTrainer.style.fontWeight = "normal";
    tabTrainer.style.textShadow = "";
  }

  if (navIndicator) navIndicator.style.background = "";
  if (navBadge) {
    navBadge.innerText = "TAURI SPA";
    navBadge.style.background = "var(--blue-glow)";
  }
  if (navSubtitle) navSubtitle.innerText = "Highly Reactive Cybernetic Substrate";
  if (navHelp) navHelp.innerHTML = "<span>🔍 Scroll: Zoom | 🖱️ Drag: Move Camera | ⌨️ R: Reset View</span>";

  // Set mode to ocean
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "ocean" }) }).catch(() => {});
}

function switchToTrainer() {
  pauseAllSimulations();

  if (oceanView) oceanView.style.display = "none";
  if (trainerView) trainerView.style.display = "block";

  if (tabTrainer) {
    tabTrainer.style.color = "#00f2fe";
    tabTrainer.style.borderBottom = "2px solid #00f2fe";
    tabTrainer.style.fontWeight = "bold";
    tabTrainer.style.textShadow = "0 0 8px rgba(0, 242, 254, 0.4)";
  }

  if (tabOcean) {
    tabOcean.style.color = "#94a3b8";
    tabOcean.style.borderBottom = "";
    tabOcean.style.fontWeight = "normal";
    tabOcean.style.textShadow = "";
  }

  if (navIndicator) navIndicator.style.background = "var(--primary-cyan)";
  if (navBadge) {
    navBadge.innerText = "RL TRAINER";
    navBadge.style.background = "";
  }
  if (navSubtitle) navSubtitle.innerText = "Evolutionary Reinforcement Learning Playground";
  if (navHelp) navHelp.innerHTML = "<span>🎯 Click a sandbox to inspect brain live | ⌨️ WASD controls active on selected</span>";

  // Set mode to trainer
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "trainer" }) }).catch(() => {});
}

if (tabOcean) {
  tabOcean.addEventListener("click", () => {
    switchToOcean();
  });
}

if (tabTrainer) {
  tabTrainer.addEventListener("click", () => {
    switchToTrainer();
  });
}

// Default starting view on launch
switchToOcean();
