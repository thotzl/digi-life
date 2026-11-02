import { effect } from "@preact/signals-core";

import { CreatureRenderer } from "./render/creatureRenderer";
import { CreatureAgent, FoodSpore, SpeciesRecord, ProceduralWorld } from "./shared/types";

import { call, onUiEvent, send } from "./core/ipc";
import { phenotypeCache } from "./core/phenotypes";
import type { CreatureAgent as BindingCreatureAgent } from "../bindings/CreatureAgent";
import { currentView } from "./core/router";
import {
  selectedId, selectedName, selectedTaxa, selectedStatus, selectedEnergy,
  selectedMaxEnergy, selectedAge, selectedGenome,
  selectedMethylations, selectedPhenotype, selectedBrainActivations, speciesRosterSignal,
  isAliveExpanded, isExtinctExpanded
} from "./signals";
import { InteractiveCamera } from "./core/camera";
import { escapeHtml } from "./core/escape";

// Local copy of a creature from an event (binding type): the client keeps no brain state of its own
function toLocalCreature(c: BindingCreatureAgent): CreatureAgent {
  return { ...c, neuronStates: [], neuronActivations: [] };
}

let world: ProceduralWorld | null = null;
let offscreenCanvas: HTMLCanvasElement | null = null;
let loopRunning = false;

function createBiomeCache(worldData: ProceduralWorld) {
  offscreenCanvas = document.createElement('canvas');
  offscreenCanvas.width = 240;
  offscreenCanvas.height = 135;
  const oCtx = offscreenCanvas.getContext('2d')!;

  for (let c = 0; c < 240; c++) {
    for (let r = 0; r < 135; r++) {
      const idx = c * 135 + r;
      const biome = worldData.biomes[idx];
      if (biome) {
        oCtx.fillStyle = biome.color;
        oCtx.fillRect(c, r, 1, 1);
      }
    }
  }
}

// Global active Ocean Substrate Canvas
const canvas = document.getElementById("creature-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d")!;
let renderer: CreatureRenderer;

// Interactive pan & zoom camera
const camera = new InteractiveCamera(canvas);

// Local client-side caches (Pure Mutable Game-Engine Arrays for high-fps in-place updates)
let foodPellets: FoodSpore[] = [];
let creatures: CreatureAgent[] = [];
let highestGeneration = 1;

// DOM Binders
const statPopulation = document.getElementById("stat-population") as HTMLSpanElement;
const statGeneration = document.getElementById("stat-generation") as HTMLSpanElement;
const statSpores = document.getElementById("stat-spores") as HTMLSpanElement;

const speciesRoster = document.getElementById("species-roster") as HTMLDivElement;
const terminalLogs = document.getElementById("terminal-logs") as HTMLDivElement;

function renderRosterCardHTML(rec: SpeciesRecord, isSelected: boolean): string {
  const statusClass = rec.status === "alive" ? "alive" : "fossil";
  const statusText = rec.status === "alive" ? "Alive" : "Fossil";

  let dietClass = "Omnivore";
  if (rec.carnivory >= 0.55) dietClass = "Predator";
  else if (rec.carnivory < 0.3) dietClass = "Herbivore";

  const charVal = rec.genome.charCodeAt(17) || 65; // Use color locus
  const primaryH = (charVal - 65) * 13.8;
  const colorStyle = `border-left: 4px solid hsla(${primaryH}, 85%, 55%, 1);`;

  return `
    <div class="roster-card ${statusClass} ${isSelected ? 'selected' : ''}" 
         style="${colorStyle}"
         data-id="${escapeHtml(rec.id)}"
         id="roster-card-${escapeHtml(rec.id)}">
      <div class="roster-title-row">
        <span class="roster-name" style="color: ${rec.status === 'alive' ? 'var(--secondary-green)' : 'var(--text-muted)'}">${escapeHtml(rec.name)}</span>
        <span class="badge-fossil ${statusClass}">${statusText}</span>
      </div>
      <div class="roster-meta-row">
        <span>Diet: ${dietClass}</span>
        <span>Gen: ${rec.generation}</span>
      </div>
    </div>
  `;
}

effect(() => {
  const liveList = speciesRosterSignal.value;
  const selectedIdVal = selectedId.value;

  if (liveList.length === 0) {
    speciesRoster.innerHTML = `<div class="loading-state">No species registered.</div>`;
    return;
  }

  let html = "";
  
  // A. Living Species Section (Collapsible)
  const aliveList = liveList.filter(r => r.status === "alive");
  const aliveOpen = isAliveExpanded.value;
  html += `
    <div class="accordion-header ${aliveOpen ? 'expanded' : ''}" id="acc-alive-trigger" style="border-left: 3px solid var(--secondary-green); border-bottom: 1px solid rgba(148, 163, 184, 0.08);">
      <span>🟢 Living Species (${aliveList.length})</span>
      <span class="chevron">▼</span>
    </div>
  `;
  if (aliveOpen) {
    html += `<div class="accordion-content">`;
    if (aliveList.length === 0) {
      html += `<div class="roster-empty-state">No living species in substrate. Inject cell or wait...</div>`;
    } else {
      aliveList.forEach(rec => {
        const isSelected = selectedIdVal !== null && 
          creatures.find(c => Number(c.id) === Number(selectedIdVal))?.speciesId === rec.id;
        html += renderRosterCardHTML(rec, isSelected);
      });
    }
    html += `</div>`;
  }

  // B. Extinct Species Section (Collapsible)
  const extinctList = liveList.filter(r => r.status === "extinct");
  const extinctOpen = isExtinctExpanded.value;
  html += `
    <div class="accordion-header ${extinctOpen ? 'expanded' : ''}" id="acc-extinct-trigger" style="border-left: 3px solid var(--text-muted); border-top: 1px solid rgba(148, 163, 184, 0.08); border-bottom: 1px solid rgba(148, 163, 184, 0.08);">
      <span>💀 Extinct Species (${extinctList.length})</span>
      <span class="chevron">▼</span>
    </div>
  `;
  if (extinctOpen) {
    html += `<div class="accordion-content">`;
    if (extinctList.length === 0) {
      html += `<div class="roster-empty-state">No extinctions recorded in database history.</div>`;
    } else {
      extinctList.forEach(rec => {
        const isSelected = selectedIdVal !== null && 
          creatures.find(c => Number(c.id) === Number(selectedIdVal))?.speciesId === rec.id;
        html += renderRosterCardHTML(rec, isSelected);
      });
    }
    html += `</div>`;
  }

  speciesRoster.innerHTML = html;
});

let biteImpacts: { x: number; y: number; age: number }[] = [];

function fetchRosterRecords() {
  // Errors are reported by the IPC client (toast)
  call("get_registered_species").then(records => {
    speciesRosterSignal.value = records;
  }).catch(() => {});
}

async function initBetaWebSocket() {
  logToTerminal("Connection to Tauri evolution substrate established.", "system");
  fetchRosterRecords();
}

/**
 * Renders the ocean "Inject Species" pick-and-add cyberpunk modal with live wiggling previews.
 */
function openOceanCatalogueDialog() {
  call("get_catalogue_creatures").then((catalogueList) => {
    if (catalogueList.length === 0) {
      alert("Your creature catalogue is empty! Save elite creatures from the ocean or the trainer first.");
      return;
    }

    let selectedIdx = 0;
    let animTime = 0;
    let animId: number;

    const backdrop = document.createElement("div");
    backdrop.style.cssText = "position:absolute; top:0; left:0; right:0; bottom:0; background:rgba(2,6,23,0.85); z-index:1000; display:flex; align-items:center; justify-content:center; backdrop-filter:blur(8px);";

    const modalBox = document.createElement("div");
    modalBox.style.cssText = "background:rgba(15,23,42,0.9); border:1px solid rgba(245,158,11,0.35); border-radius:8px; width:410px; padding:24px; box-sizing:border-box; display:flex; flex-direction:column; gap:14px; box-shadow:0 0 35px rgba(245,158,11,0.12); animation:slide-fade-in 0.2s ease-out; font-family:monospace; color:#f1f5f9;";

    const title = document.createElement("h3");
    title.style.cssText = "margin:0; font-size:1.05rem; font-weight:bold; color:#fff; text-transform:uppercase; letter-spacing:1px; border-bottom:1px solid rgba(148,163,184,0.1); padding-bottom:8px; display:flex; align-items:center; gap:6px;";
    title.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#f59e0b" stroke-width="2" style="margin-top:-1px;"><path d="M12 22C17.5228 22 22 19.7614 22 17C22 14.2386 17.5228 12 12 12C6.47715 12 2 14.2386 2 17C2 19.7614 6.47715 22 12 22Z"/><path d="M22 7C22 9.76142 17.5228 12 12 12C6.47715 12 2 9.76142 2 7C2 4.23858 6.47715 2 12 2C17.5228 2 22 4.23858 22 7Z"/><path d="M2 7V17M22 7V17"/></svg> Inject Species`;

    const desc = document.createElement("p");
    desc.style.cssText = "margin:0; font-size:0.68rem; color:#94a3b8; line-height:1.45;";
    desc.innerText = "Select a clone from your catalogue to release it live into the ocean:";

    // Centered, glowing aggregated local preview circle
    const previewContainer = document.createElement("div");
    previewContainer.style.cssText = "display:flex; justify-content:center; align-items:center; background:rgba(2,6,23,0.5); border:1px solid rgba(148,163,184,0.08); border-radius:6px; padding:12px; gap:16px;";

    const pCanvas = document.createElement("canvas");
    pCanvas.width = 72;
    pCanvas.height = 72;
    pCanvas.style.cssText = "background:#020617; border:1.5px solid rgba(245,158,11,0.25); border-radius:50%; box-shadow:0 0 15px rgba(245,158,11,0.12); flex-shrink:0;";
    const pCtx = pCanvas.getContext("2d")!;
    const pRenderer = new CreatureRenderer(pCanvas);

    const pMeta = document.createElement("div");
    pMeta.style.cssText = "flex:1; display:flex; flex-direction:column; gap:4px; min-width:0;";
    const pName = document.createElement("b");
    pName.style.cssText = "font-size:0.8rem; color:#fff;";
    const pDetails = document.createElement("span");
    pDetails.style.cssText = "font-size:0.58rem; color:#64748b; line-height:1.4;";

    previewContainer.appendChild(pCanvas);
    previewContainer.appendChild(pMeta);
    pMeta.appendChild(pName);
    pMeta.appendChild(pDetails);

    // High-tech scrollable list replacing the ugly dropdown select
    const listDeck = document.createElement("div");
    listDeck.style.cssText = "height:140px; overflow-y:auto; display:flex; flex-direction:column; gap:6px; padding-right:4px; border:1px solid rgba(148,163,184,0.1); border-radius:6px; background:rgba(15,23,42,0.3); padding:6px; box-sizing:border-box;";

    // Style scrollbar of modal list
    const styleTag = document.createElement("style");
    styleTag.innerText = `
      .modal-scroll-deck::-webkit-scrollbar { width: 3px; }
      .modal-scroll-deck::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.15); border-radius: 2px; }
      .modal-card { transition: all 0.15s ease-in-out; }
      .modal-card:hover { border-color:#f59e0b !important; background:rgba(245,158,11,0.02) !important; }
      .modal-card.active { border-color:#f59e0b !important; background:rgba(245,158,11,0.06) !important; box-shadow:0 0 10px rgba(245,158,11,0.05); }
    `;
    listDeck.classList.add("modal-scroll-deck");
    document.head.appendChild(styleTag);

    let activePhenotype: any = null;

    // Load detailed phenotype for the active selection
    const loadSelectedPreview = async (creature: any) => {
      pName.innerText = creature.name;
      let dietLabel = "Omnivore";
      if (creature.carnivory >= 0.55) dietLabel = "Predator";
      else if (creature.carnivory < 0.3) dietLabel = "Herbivore";
      pDetails.innerHTML = `Diet Class: <span style="color:#f59e0b;">${dietLabel}</span><br/>Fitness rating: ${creature.fitness.toFixed(2)}<br/>Origin: ${escapeHtml(creature.source)}`;
      
      try {
        activePhenotype = await call("get_fossil_phenotype", { genome: creature.genome });
      } catch {
        // reported by the IPC client
      }
    };

    // Render list cards deck
    const renderCards = () => {
      listDeck.innerHTML = "";
      catalogueList.forEach((c: any, index: number) => {
        const charVal = c.genome.charCodeAt(17) || 65;
        const colorH = (charVal - 65) * 13.8;

        const card = document.createElement("div");
        card.className = `modal-card ${index === selectedIdx ? "active" : ""}`;
        card.style.cssText = `border:1px solid rgba(148,163,184,0.1); border-radius:4px; padding:8px 10px; cursor:pointer; display:flex; align-items:center; justify-content:space-between; font-size:0.65rem; background:rgba(30,41,59,0.15);`;
        
        let dietLabel = "Omnivore";
        if (c.carnivory >= 0.55) dietLabel = "Predator";
        else if (c.carnivory < 0.3) dietLabel = "Herbivore";

        card.innerHTML = `
          <div style="display:flex; align-items:center; gap:8px;">
            <div style="width:6px; height:6px; border-radius:50%; background:hsla(${colorH},85%,55%,1); box-shadow:0 0 6px hsla(${colorH},85%,55%,0.8);"></div>
            <b style="color:#fff;">${escapeHtml(c.name)}</b>
          </div>
          <span style="color:#64748b; font-size:0.58rem; text-transform:uppercase;">${dietLabel} // F: ${c.fitness.toFixed(2)}</span>
        `;

        card.addEventListener("click", () => {
          selectedIdx = index;
          renderCards();
          loadSelectedPreview(c);
        });

        listDeck.appendChild(card);
      });
    };

    // Initial load first item preview
    loadSelectedPreview(catalogueList[0]);
    renderCards();

    // High-fps Preview loop inside modal
    const previewLoop = () => {
      animTime += 0.045;
      if (pCtx && activePhenotype) {
        pCtx.fillStyle = '#020617';
        pCtx.fillRect(0, 0, 72, 72);

        const baseLength = activePhenotype.spinalHarmonics?.baseLength || 130;
        const dynamicScale = 52.0 / (baseLength * 0.5);

        pCtx.save();
        pCtx.translate(36, 35);
        pCtx.scale(dynamicScale, dynamicScale);
        pRenderer.render(activePhenotype, animTime, 0, 0, -Math.PI / 2, 0);
        pCtx.restore();
      }
      animId = requestAnimationFrame(previewLoop);
    };
    previewLoop();

    // Checkbox + Actions
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.checked = false;
    checkbox.id = "ocean-chk-load-syn";
    checkbox.style.cssText = "cursor:pointer; width:15px; height:15px; margin:0;";

    const label = document.createElement("label");
    label.htmlFor = "ocean-chk-load-syn";
    label.style.cssText = "display:flex; align-items:center; gap:8px; font-size:0.65rem; cursor:pointer; user-select:none; color:#94a3b8;";
    label.appendChild(checkbox);
    const labelSpan = document.createElement("span");
    labelSpan.innerText = "Load with learned synaptic weights (imprinting)";
    label.appendChild(labelSpan);

    const qtyInput = document.createElement("input");
    qtyInput.type = "number";
    qtyInput.min = "1";
    qtyInput.max = "50";
    qtyInput.value = "1";
    qtyInput.style.cssText = "width:50px; background:rgba(15,23,42,0.85); color:#00f2fe; font-family:monospace; font-size:0.75rem; font-weight:bold; border:1px solid rgba(0,242,254,0.3); border-radius:4px; padding:4px 6px; outline:none; text-align:center;";

    const qtyLabel = document.createElement("label");
    qtyLabel.style.cssText = "display:flex; align-items:center; gap:6px; font-size:0.65rem; color:#94a3b8; margin:0;";
    const qtySpan = document.createElement("span");
    qtySpan.innerText = "Quantity:";
    qtyLabel.appendChild(qtySpan);
    qtyLabel.appendChild(qtyInput);

    const configRow = document.createElement("div");
    configRow.style.cssText = "display:flex; align-items:center; justify-content:between; gap:12px; padding-top:4px;";
    configRow.appendChild(label);
    configRow.appendChild(qtyLabel);

    const btnCancel = document.createElement("button");
    btnCancel.style.cssText = "background:rgba(148,163,184,0.06); border:1px solid rgba(148,163,184,0.15); color:#94a3b8; border-radius:4px; font-family:monospace; font-size:0.7rem; cursor:pointer; padding:8px 16px; font-weight:bold;";
    btnCancel.innerText = "Cancel";
    btnCancel.addEventListener("click", () => {
      cancelAnimationFrame(animId);
      backdrop.remove();
    });

    const btnConfirm = document.createElement("button");
    btnConfirm.style.cssText = "background:#10b981; color:#020617; border:none; border-radius:4px; font-family:monospace; font-weight:bold; font-size:0.7rem; cursor:pointer; padding:8px 16px;";
    btnConfirm.innerText = "Release";
    btnConfirm.addEventListener("click", async () => {
      const selectedCreature = catalogueList[selectedIdx];
      const loadSyn = checkbox.checked;
      const qty = Math.max(1, Math.min(50, parseInt(qtyInput.value) || 1));
      try {
        for (let i = 0; i < qty; i++) {
          await call("spawn_catalogue_creature_to_ocean", {
            id: selectedCreature.id,
            loadLearnedSynapses: loadSyn
          });
        }
        cancelAnimationFrame(animId);
        backdrop.remove();
      } catch {
        // reported by the IPC client (toast); the dialog stays open
      }
    });

    const buttonsRow = document.createElement("div");
    buttonsRow.style.cssText = "display:flex; justify-content:flex-end; gap:8px; border-top:1px solid rgba(148,163,184,0.1); padding-top:12px; margin-top:12px;";
    buttonsRow.appendChild(btnCancel);
    buttonsRow.appendChild(btnConfirm);

    modalBox.appendChild(title);
    modalBox.appendChild(desc);
    modalBox.appendChild(previewContainer);
    modalBox.appendChild(listDeck);
    modalBox.appendChild(configRow);
    modalBox.appendChild(buttonsRow);
    backdrop.appendChild(modalBox);
    document.getElementById("app")?.appendChild(backdrop);
  }).catch(() => {});
}

const btnToggleSim = document.getElementById("btn-toggle-sim") as HTMLButtonElement;
let isSimRunning = true;

btnToggleSim?.addEventListener("click", () => {
  const nextState = !isSimRunning;
  send({ type: "TOGGLE_SIMULATION", running: nextState });
});

const sliderOceanSpeedup = document.getElementById("slider-ocean-speedup") as HTMLInputElement;
const lblOceanSpeedup = document.getElementById("lbl-ocean-speedup") as HTMLSpanElement;
let oceanWarpSpeed = 1;

sliderOceanSpeedup?.addEventListener("input", () => {
  const val = parseInt(sliderOceanSpeedup.value);
  if (val <= 10) {
    lblOceanSpeedup.innerText = `${val}x`;
  } else if (val === 11) {
    lblOceanSpeedup.innerText = `50x`;
  } else {
    lblOceanSpeedup.innerText = `🏎️ Unlimited`;
  }
});

sliderOceanSpeedup?.addEventListener("change", () => {
  const val = parseInt(sliderOceanSpeedup.value);
  if (val <= 10) {
    oceanWarpSpeed = val;
  } else if (val === 11) {
    oceanWarpSpeed = 50;
  } else {
    oceanWarpSpeed = 9999;
  }

  send({ type: "SET_SIMULATION_SPEED", speed: oceanWarpSpeed });
});

document.getElementById("btn-inject")?.addEventListener("click", () => {
  send({ type: "INJECT_URZELLE" });
});

document.getElementById("btn-load-catalogue")?.addEventListener("click", () => {
  openOceanCatalogueDialog();
});

document.getElementById("btn-reset")?.addEventListener("click", () => {
  send({ type: "RESET_EVOLUTION" }).then((accepted) => {
    if (accepted) selectedId.value = null;
  });
});

document.getElementById("btn-clear-console")?.addEventListener("click", () => {
  terminalLogs.innerHTML = "";
});

async function startOcean() {
  resizeBetaCanvas();
  renderer = new CreatureRenderer(canvas);
  
  window.addEventListener("resize", resizeBetaCanvas);
  
  const { visibleWidth, visibleHeight } = camera.getViewportBounds();
  camera.reset(visibleWidth, visibleHeight);

  await initBetaWebSocket();

  send({ type: "CLIENT_READY" });

  requestAnimationFrame(drawBetaSimulationFrame);
}

if (document.readyState === "complete" || document.readyState === "interactive") {
  startOcean();
} else {
  window.addEventListener("load", startOcean);
}

// Global active Ocean simulation thread-step receiver (emitted from Rust background core)
onUiEvent((data) => {
  if (data.type === "INIT_STATE") {
    highestGeneration = data.highestGeneration;
    foodPellets = data.foodPellets;
    fetchRosterRecords();

    isSimRunning = data.running;
    if (btnToggleSim) {
      btnToggleSim.innerText = isSimRunning ? "Pause Substrate" : "Resume Substrate";
      btnToggleSim.style.background = isSimRunning ? "rgba(0, 242, 254, 0.05)" : "rgba(16, 185, 129, 0.05)";
      btnToggleSim.style.color = isSimRunning ? "#00f2fe" : "#10b981";
      btnToggleSim.style.borderColor = isSimRunning ? "rgba(0, 242, 254, 0.2)" : "rgba(16, 185, 129, 0.2)";
    }

    world = data.world;
    createBiomeCache(world);

    creatures = data.creatures.map(toLocalCreature);

    if (creatures.length > 0 && selectedId.value === null) {
      selectSpecimen(creatures[0]);
    }
  }
  else if (data.type === "SIM_STATE") {
    isSimRunning = data.running;
    if (btnToggleSim) {
      btnToggleSim.innerText = isSimRunning ? "Pause Substrate" : "Resume Substrate";
      btnToggleSim.style.background = isSimRunning ? "rgba(0, 242, 254, 0.05)" : "rgba(16, 185, 129, 0.05)";
      btnToggleSim.style.color = isSimRunning ? "#00f2fe" : "#10b981";
      btnToggleSim.style.borderColor = isSimRunning ? "rgba(0, 242, 254, 0.2)" : "rgba(16, 185, 129, 0.2)";
    }
  }
  else if (data.type === "TELEMETRY_TICK") {
    highestGeneration = data.highestGeneration;
    const incoming = data.creatures;

    // Handle newborn creatures in-place
    data.newCreatures.forEach((tele) => {
      if (!creatures.some(c => Number(c.id) === Number(tele.id))) {
        creatures.push(toLocalCreature(tele));
      }
    });

    // In-place positional update of existing local creatures to KEEP their precompiled .phenotype!
    incoming.forEach((tele) => {
      const local = creatures.find(c => Number(c.id) === Number(tele.id));
      if (local) {
        local.px = tele.px;
        local.py = tele.py;
        local.vx = tele.vx;
        local.vy = tele.vy;
        local.headingAngle = tele.headingAngle;
        local.omegaRot = tele.omegaRot;
        local.energy = tele.energy;
        local.age = tele.age;
        local.generation = tele.generation;
        local.hasEaten = tele.hasEaten;
      }
    });

    const serverIds = new Set(incoming.map((c) => Number(c.id)));
    creatures = creatures.filter(c => serverIds.has(Number(c.id)));

    // Connect real-time selected vitals to Preact Signals
    const id = selectedId.value;
    if (id !== null) {
      const active = creatures.find(c => Number(c.id) === Number(id));
      if (active) {
        selectedStatus.value = "Alive";
        selectedEnergy.value = active.energy;
        selectedAge.value = active.age;

        if (data.selectedBrain && Number(data.selectedBrain.id) === Number(id)) {
          selectedBrainActivations.value = data.selectedBrain.activations;
        }
      } else {
        selectedStatus.value = "Extinct (Fossil)";
        selectedEnergy.value = 0;
        selectedAge.value = 2700;
      }
    }

    statPopulation.innerText = `${creatures.length} / 25`;
    statGeneration.innerText = `${highestGeneration}`;
    statSpores.innerText = `${foodPellets.length} Spores`;
  }
  else if (data.type === "FOOD_TICK") {
    foodPellets = data.foodPellets;
    statSpores.innerText = `${foodPellets.length} Spores`;
  }
  else if (data.type === "BITE_EVENT") {
    const bx = (data.x / 19200.0) * canvas.width;
    const by = (data.y / 10800.0) * canvas.height;
    biteImpacts.push({ x: bx, y: by, age: 0 });
  }
  else if (data.type === "DATABASE_CHANGED") {
    fetchRosterRecords();
  }
  else if (data.type === "LOG_EVENT") {
    logToTerminal(data.message, data.logType);
  }
});

// Primary Render Engine Frame-Step loop (updates viewport canvas at 60Hz/120Hz/144Hz)
function drawBetaSimulationFrame(_timestamp = performance.now()) {
  if (currentView.value !== "ocean") {
    loopRunning = false;
    return; // Conserve clock cycles when backgrounded
  }

  loopRunning = true;
  ctx.fillStyle = '#020617';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  const { viewCenterX, viewCenterY } = camera.getViewportBounds();
  ctx.translate(viewCenterX, viewCenterY);
  ctx.scale(camera.camZoom, camera.camZoom);
  ctx.translate(-camera.camX, -camera.camY);

  if (offscreenCanvas) {
    ctx.drawImage(offscreenCanvas, 0, 0, 19200, 10800);
  }

  // Draw current vents circles
  if (world && world.vents) {
    world.vents.forEach((vent: any) => {
      ctx.beginPath();
      ctx.arc(vent.x, vent.y, vent.radius, 0, Math.PI * 2);
      ctx.fillStyle = "rgba(239, 68, 68, 0.035)";
      ctx.fill();
    });
  }

  // Draw procedural rock obstacles
  if (world && world.obstacles) {
    world.obstacles.forEach((obs: any) => {
      ctx.beginPath();
      ctx.moveTo(obs.vertices[0].x, obs.vertices[0].y);
      for (let j = 1; j < obs.vertices.length; j++) {
        ctx.lineTo(obs.vertices[j].x, obs.vertices[j].y);
      }
      ctx.closePath();
      ctx.fillStyle = obs.color;
      ctx.fill();

      ctx.strokeStyle = "#0b1329";
      ctx.lineWidth = 14;
      ctx.stroke();
    });
  }

  // Draw food spores
  foodPellets.forEach(spore => {
    ctx.beginPath();
    ctx.arc(spore.x, spore.y, 16.0, 0, Math.PI * 2);
    const isMeat = spore.typeId === 2;
    ctx.fillStyle = isMeat ? '#ef4444' : '#10b981';
    ctx.shadowBlur = 8;
    ctx.shadowColor = isMeat ? '#ef4444' : '#10b981';
    ctx.fill();
    ctx.shadowBlur = 0;
  });

  // Render living creature agents
  creatures.forEach(agent => {
    if (agent.phenotype) {
      renderer.render(
        agent.phenotype,
        Date.now() * 0.05, // Smooth system time animation driver, identical to Trainer!
        agent.px,
        agent.py,
        agent.headingAngle,
        agent.omegaRot
      );

      // Draw visual focus ring around selected creature
      if (selectedId.value !== null && Number(agent.id) === Number(selectedId.value)) {
        ctx.beginPath();
        ctx.arc(agent.px, agent.py, agent.phenotype.spinalHarmonics.baseLength * 0.65, 0, Math.PI * 2);
        ctx.strokeStyle = '#00f2fe';
        ctx.lineWidth = 4.0;
        ctx.setLineDash([12, 12]);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }
  });

  // Draw biting impact flashes
  for (let i = biteImpacts.length - 1; i >= 0; i--) {
    const flash = biteImpacts[i];
    flash.age += 1;
    if (flash.age > 15) {
      biteImpacts.splice(i, 1);
      continue;
    }
    ctx.beginPath();
    ctx.arc(flash.x, flash.y, flash.age * 5.0, 0, Math.PI * 2);
    ctx.strokeStyle = `rgba(239, 68, 68, ${1.0 - flash.age / 15.0})`;
    ctx.lineWidth = 3;
    ctx.stroke();
  }

  ctx.restore();

  // Render camera overlay HUD hints (for panning/zooming)
  ctx.fillStyle = 'rgba(255, 255, 255, 0.15)';
  ctx.font = '10px monospace';
  ctx.fillText('🖱️ Drag to pan // Scroll to zoom', 20, canvas.height - 20);

  requestAnimationFrame(drawBetaSimulationFrame);
}

function resizeBetaCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  camera.reset(canvas.width, canvas.height);
}

// Global active Ocean click inspector element
camera.setupEventListeners(
  (mx, my) => {
    let hovered = false;
    creatures.forEach(agent => {
      const dx = agent.px - mx;
      const dy = agent.py - my;
      const dist = Math.sqrt(dx*dx + dy*dy);
      let p = phenotypeCache.get(agent.genome);
      const bodyRadius = p ? (p.spinalHarmonics.meanRadius * 2.6 * 0.5 + 10) : 100;
      if (dist < bodyRadius) {
        hovered = true;
      }
    });
    return hovered;
  },
  (mx, my) => {
    let closestAgent: any = null;
    let minDist = 120;

    creatures.forEach(agent => {
      const dx = agent.px - mx;
      const dy = agent.py - my;
      const dist = Math.sqrt(dx*dx + dy*dy);
      if (dist < minDist) {
        minDist = dist;
        closestAgent = agent;
      }
    });

    // A click on empty water does nothing: the former `SPAWN_FOOD` action was never handled by the engine (RV-B09)
    // and was removed with the typed action set (TCK-157)
    if (closestAgent) {
      selectSpecimen(closestAgent);
    }
  }
);

window.addEventListener("keydown", (e) => {
  if (e.key.toLowerCase() === "r") {
    const { visibleWidth, visibleHeight } = camera.getViewportBounds();
    camera.reset(visibleWidth, visibleHeight);
    logToTerminal("Camera view centered on full-screen glass tank and reset.", "system");
  }
});

speciesRoster.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;

  const aliveHeader = target.closest("#acc-alive-trigger");
  if (aliveHeader) {
    isAliveExpanded.value = !isAliveExpanded.value;
    return;
  }

  const extinctHeader = target.closest("#acc-extinct-trigger");
  if (extinctHeader) {
    isExtinctExpanded.value = !isExtinctExpanded.value;
    return;
  }

  const item = target.closest(".roster-card") as HTMLDivElement | null;
  if (!item) return;

  const recId = item.getAttribute("data-id");
  if (recId) {
    const agent = creatures.find(c => c.speciesId === recId);
    if (agent) {
      selectSpecimen(agent);
      camera.camX = agent.px;
      camera.camY = agent.py;
      logToTerminal(`Camera focused on specimen #${agent.id}.`, "system");
    } else {
      const record = speciesRosterSignal.value.find(r => r.id === recId);
      if (record) {
        selectedId.value = 99999;
        selectedName.value = record.name;
        selectedTaxa.value = `${record.name} [FOSSIL - EXTINCT]`;
        selectedGenome.value = record.genome;
        selectedMethylations.value = Array(256).fill(0);
        selectedPhenotype.value = null;
        call("get_fossil_phenotype", { genome: record.genome }).then((pheno) => {
          selectedPhenotype.value = pheno;
        }).catch(() => {});
        selectedMaxEnergy.value = 100;
        selectedStatus.value = "Extinct (Fossil)";
        selectedEnergy.value = 0;
        selectedAge.value = 2700;
      }
    }
  }
});

function selectSpecimen(agent: CreatureAgent) {
  selectedId.value = agent.id;
  selectedName.value = agent.phenotype.latinName;
  selectedTaxa.value = `${agent.phenotype.latinName.substring(0, 16)} (Strain: #${agent.id}, Gen: ${agent.generation})`;
  selectedGenome.value = agent.genome;
  selectedMethylations.value = agent.phenotype.methylations;
  selectedPhenotype.value = agent.phenotype;
  selectedMaxEnergy.value = agent.phenotype.stomachCapacity;

  send({ type: "SELECT_AGENT", id: agent.id });
}

function logToTerminal(message: string, logType: string = "system") {
  const row = document.createElement("div");
  row.className = "log-row";

  const time = document.createElement("span");
  time.className = "log-time";
  time.innerText = new Date().toLocaleTimeString();

  const msg = document.createElement("span");
  msg.className = "log-msg";
  msg.innerText = message;

  row.appendChild(time);
  row.appendChild(msg);

  if (logType === "mutation") {
    row.style.color = "#00f2fe";
  } else if (logType === "extinction") {
    row.style.color = "#64748b";
  } else {
    row.style.color = "#10b981";
  }

  terminalLogs.appendChild(row);
  terminalLogs.scrollTop = terminalLogs.scrollHeight;
}

// Reactively restart rendering loop when user returns to Ocean View!
effect(() => {
  if (currentView.value === "ocean" && !loopRunning) {
    drawBetaSimulationFrame();
  }
});
