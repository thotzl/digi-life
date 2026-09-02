import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { effect } from "@preact/signals-core";

import { CreatureRenderer } from "./render/creatureRenderer";
import { CreatureAgent, FoodSpore, GeneSpan, SpeciesRecord } from "./shared/types";

import { safeInvoke } from "./api";
import {
  selectedId, selectedName, selectedTaxa, selectedStatus, selectedEnergy,
  selectedMaxEnergy, selectedAdrenaline, selectedAge, selectedGenome,
  selectedMethylations, selectedPhenotype, speciesRosterSignal,
  isAliveExpanded, isExtinctExpanded, getLocusDescription, computeActiveGeneSpans
} from "./signals";
import { InteractiveCamera } from "./core/camera";
import { BrainRenderer } from "./render/brainRenderer";

// Pure Mutable Game-Engine Arrays
let creatures: CreatureAgent[] = [];
let foodPellets: FoodSpore[] = [];
let highestGeneration = 1;

// Interactive Camera Instance
let camera: InteractiveCamera;

// DOM Binders
const statPopulation = document.getElementById("stat-population") as HTMLSpanElement;
const statGeneration = document.getElementById("stat-generation") as HTMLSpanElement;
const statSpores = document.getElementById("stat-spores") as HTMLSpanElement;

const inspectFallback = document.getElementById("inspect-fallback") as HTMLDivElement;
const inspectActiveContent = document.getElementById("inspect-active-content") as HTMLDivElement;

const specimenName = document.getElementById("specimen-name") as HTMLHeadingElement;
const specimenTaxa = document.getElementById("specimen-taxa") as HTMLParagraphElement;
const specimenStatus = document.getElementById("specimen-status") as HTMLSpanElement;

const energyBar = document.getElementById("inspect-energy-bar") as HTMLDivElement;
const energyText = document.getElementById("inspect-energy-text") as HTMLSpanElement;

const adrenalineBar = document.getElementById("inspect-adrenaline-bar") as HTMLDivElement;
const adrenalineText = document.getElementById("inspect-adrenaline-text") as HTMLSpanElement;

const ageBar = document.getElementById("inspect-age-bar") as HTMLDivElement;
const ageText = document.getElementById("inspect-age-text") as HTMLSpanElement;

const genomeGrid = document.getElementById("inspect-genome-grid") as HTMLDivElement;
const speciesRoster = document.getElementById("species-roster") as HTMLDivElement;
const terminalLogs = document.getElementById("terminal-logs") as HTMLDivElement;

function renderRosterCardHTML(rec: SpeciesRecord, isSelected: boolean): string {
  const statusClass = rec.status === "alive" ? "alive" : "fossil";
  const statusText = rec.status === "alive" ? "Alive" : "Fossil";

  return `
    <div class="roster-card ${isSelected ? 'roster-card-active' : ''}" data-id="${rec.id}">
      <div class="roster-title-row">
        <span class="roster-name" style="color: ${rec.status === 'alive' ? 'var(--secondary-green)' : 'var(--text-muted)'}">${rec.name}</span>
        <span class="badge-fossil ${statusClass}">${statusText}</span>
      </div>
      <div class="roster-meta-row">
        <span>Gen: ${rec.generation} | Peak: ${rec.peakPopulation}</span>
        <span>${new Date(rec.birthTime).toLocaleTimeString()}</span>
      </div>
    </div>
  `;
}

// Effect Binders
effect(() => {
  const list = speciesRosterSignal.value;
  if (list.length === 0) {
    speciesRoster.innerHTML = `<div class="loading-state">No species registered.</div>`;
    return;
  }

  const aliveList = list.filter(r => r.status === "alive");
  const extinctList = list.filter(r => r.status === "extinct");

  let html = "";

  const aliveOpen = isAliveExpanded.value;
  html += `
    <div class="accordion-header ${aliveOpen ? 'expanded' : ''}" id="acc-alive-trigger" style="border-left: 3px solid var(--secondary-green);">
      <span>🟢 Living Species (${aliveList.length})</span>
      <span class="chevron">▼</span>
    </div>
  `;

  if (aliveOpen) {
    html += `<div class="accordion-content">`;
    if (aliveList.length === 0) {
      html += `<div class="loading-state" style="padding:15px; font-size:0.65rem;">No active species in the ocean.</div>`;
    } else {
      aliveList.forEach(rec => {
        const isSelected = selectedId.value !== null && 
          creatures.find(c => Number(c.id) === Number(selectedId.value))?.speciesId === rec.id;
        html += renderRosterCardHTML(rec, isSelected);
      });
    }
    html += `</div>`;
  }

  const extinctOpen = isExtinctExpanded.value;
  html += `
    <div class="accordion-header ${extinctOpen ? 'expanded' : ''}" id="acc-extinct-trigger" style="border-left: 3px solid var(--text-muted);">
      <span>💀 Fossil Relics (${extinctList.length})</span>
      <span class="chevron">▼</span>
    </div>
  `;

  if (extinctOpen) {
    html += `<div class="accordion-content">`;
    if (extinctList.length === 0) {
      html += `<div class="loading-state" style="padding:15px; font-size:0.65rem;">No fossil records registered.</div>`;
    } else {
      extinctList.forEach(rec => {
        const isSelected = selectedId.value !== null && 
          creatures.find(c => Number(c.id) === Number(selectedId.value))?.speciesId === rec.id;
        html += renderRosterCardHTML(rec, isSelected);
      });
    }
    html += `</div>`;
  }

  speciesRoster.innerHTML = html;
});

effect(() => {
  const id = selectedId.value;
  if (id === null) {
    inspectFallback.style.display = "flex";
    inspectActiveContent.style.display = "none";
  } else {
    inspectFallback.style.display = "none";
    inspectActiveContent.style.display = "block";
  }
});

effect(() => { specimenName.innerText = selectedName.value; });
effect(() => { specimenTaxa.innerText = selectedTaxa.value; });
effect(() => { 
  specimenStatus.innerText = selectedStatus.value; 
  specimenStatus.style.background = selectedStatus.value === "Alive" ? "rgba(16, 185, 129, 0.12)" : "rgba(77, 89, 116, 0.12)";
  specimenStatus.style.color = selectedStatus.value === "Alive" ? "var(--secondary-green)" : "var(--text-muted)";
});

effect(() => {
  const val = selectedEnergy.value;
  const max = selectedMaxEnergy.value;
  const pct = Math.max(0, Math.min(100, (val / max) * 100));
  energyBar.style.width = `${pct}%`;
  energyText.innerText = `${Math.round(val)} / ${Math.round(max)}nJ`;
});

effect(() => {
  const val = selectedAdrenaline.value;
  const pct = Math.max(0, Math.min(100, ((val - 1.0) / 0.8) * 100));
  adrenalineBar.style.width = `${pct}%`;
  adrenalineText.innerText = `${val.toFixed(2)}x`;
});

effect(() => {
  const val = selectedAge.value;
  const pct = Math.max(0, Math.min(100, (val / 2700) * 100));
  ageBar.style.width = `${pct}%`;
  ageText.innerText = `${Math.round(val / 60)}s`;
});

effect(() => {
  const g = selectedGenome.value;
  const m = selectedMethylations.value;
  const p = selectedPhenotype.value;
  if (!g) {
    genomeGrid.innerHTML = "";
    return;
  }

  let html = "";
  const activeGeneSpans = p ? computeActiveGeneSpans(g, p.chromatinState) : [];

  for (let i = 0; i < g.length; i++) {
    const char = g[i];
    const isPromoter = i < 16;
    const isMethylated = m && m[i] !== 0;

    let isActiveGene = false;
    if (activeGeneSpans) {
      isActiveGene = activeGeneSpans.some((span: GeneSpan) => i >= span.start && i <= span.end);
    }

    let bg = "rgba(255,255,255,0.02)";
    let border = "1px solid rgba(255,255,255,0.03)";
    let lociClass = "inactive";
    let colorStyle = "";

    if (isPromoter) {
      bg = "#ffffff";
      lociClass = "promoter";
    } else {
      const charVal = char.charCodeAt(0) - 65;
      if (isActiveGene) {
        lociClass = "active";
        bg = `hsla(${charVal * 13.8}, 75%, 45%, 0.35)`;
        border = `1.2px solid hsla(${charVal * 13.8}, 75%, 45%, 0.8)`;
      } else {
        lociClass = "inactive";
        bg = `hsla(${charVal * 13.8}, 35%, 15%, 0.03)`;
        border = `1.0px dashed hsla(${charVal * 13.8}, 35%, 15%, 0.15)`;
        colorStyle = "color: rgba(255,255,255,0.15);";
      }
    }

    const locusDesc = getLocusDescription(i);
    const activityText = isPromoter ? " [Promoter]" : (isActiveGene ? " [Active Gene]" : " [Inactive Junk DNA]");

    html += `
      <div class="loci-node ${lociClass} ${isMethylated ? 'methylated' : ''}" 
           style="background: ${bg}; border: ${border}; ${colorStyle}" 
           title="Locus ${i}: ${char} - ${locusDesc}${activityText}${isMethylated ? ' (Methylated +' + m[i] + ')' : ''}">
        ${char}
      </div>
    `;
  }
  genomeGrid.innerHTML = html;
});

// Brain Render Graph
const brainContainer = document.getElementById("inspect-brain-container-ocean") as HTMLDivElement;
const brainRenderer = new BrainRenderer(brainContainer, "beta");

let biteImpacts: { x: number; y: number; age: number }[] = [];

function fetchRosterRecords() {
  safeInvoke("get_registered_species").then(records => {
    if (records) {
      speciesRosterSignal.value = records;
    }
  }).catch(err => {
    console.error("Failed to load native species ledger:", err);
  });
}

async function initBetaWebSocket() {
  logToTerminal("Connection to Tauri evolution substrate established.", "system");
  fetchRosterRecords();

  if (typeof window === "undefined" || !(window as any).__TAURI_INTERNALS__) {
    logToTerminal("Running in Browser Diagnostic Mode. Sockets/Tauri offline.", "system");
    return;
  }

  await listen("simulation-state", (event) => {
    try {
      const data: any = event.payload;

      if (data.type === "INIT_STATE") {
        highestGeneration = data.highestGeneration;
        foodPellets = data.foodPellets;
        fetchRosterRecords();

        isSimRunning = data.running !== undefined ? data.running : true;
        if (lblToggleSim) {
          lblToggleSim.innerText = isSimRunning ? "Pause Substrate" : "Resume Substrate";
        }
        if (btnToggleSim) {
          if (isSimRunning) {
            btnToggleSim.classList.add("btn-primary");
            btnToggleSim.classList.remove("btn-danger");
          } else {
            btnToggleSim.classList.add("btn-danger");
            btnToggleSim.classList.remove("btn-primary");
          }
        }

        creatures = data.creatures.map((c: any) => ({
          ...c,
          phenotype: c.phenotype
        }));

        if (creatures.length > 0 && selectedId.value === null) {
          selectSpecimen(creatures[0]);
        }
      }
      else if (data.type === "SIM_STATE") {
        isSimRunning = data.running;
        if (lblToggleSim) {
          lblToggleSim.innerText = isSimRunning ? "Pause Substrate" : "Resume Substrate";
        }
        if (btnToggleSim) {
          if (isSimRunning) {
            btnToggleSim.classList.add("btn-primary");
            btnToggleSim.classList.remove("btn-danger");
          } else {
            btnToggleSim.classList.add("btn-danger");
            btnToggleSim.classList.remove("btn-primary");
          }
        }
      }
      else if (data.type === "TELEMETRY_TICK") {
        highestGeneration = data.highestGeneration;
        const incoming = data.creatures;

        if (data.newCreatures && data.newCreatures.length > 0) {
          data.newCreatures.forEach((tele: any) => {
            if (!creatures.some(c => Number(c.id) === Number(tele.id))) {
              const pheno = tele.phenotype;
              creatures.push({
                ...tele,
                phenotype: pheno,
                neuronStates: [],
                neuronActivations: []
              });
              safeInvoke("handle_client_action", { 
                action: JSON.stringify({ 
                  type: "CLIENT_LOG", 
                  message: `[CLIENT] Successfully registered newborn creature #${tele.id} (${pheno.latinName}) at px=${tele.px.toFixed(1)}, py=${tele.py.toFixed(1)}` 
                }) 
              }).catch(() => {});
            }
          });
        }

        incoming.forEach((tele: any) => {
          const local = creatures.find(c => Number(c.id) === Number(tele.id));
          if (local) {
            local.px = tele.px;
            local.py = tele.py;
            local.vx = tele.vx;
            local.vy = tele.vy;
            local.headingAngle = tele.headingAngle;
            local.omegaRot = tele.omegaRot;
            local.energy = tele.energy;
            local.adrenaline = tele.adrenaline;
            local.age = tele.age;
            local.generation = tele.generation;
            local.hasEaten = tele.hasEaten;
          }
        });

        const serverIds = new Set(incoming.map((c: any) => Number(c.id)));
        creatures = creatures.filter(c => serverIds.has(Number(c.id)));

        const id = selectedId.value;
        if (id !== null) {
          const active = creatures.find(c => Number(c.id) === Number(id));
          if (active) {
            selectedStatus.value = "Alive";
            selectedEnergy.value = active.energy;
            selectedAdrenaline.value = active.adrenaline || 1.0;
            selectedAge.value = active.age;

            if (data.selectedBrain && Number(data.selectedBrain.id) === Number(id)) {
              brainRenderer.updateLiveGlows(data.selectedBrain.activations, active.phenotype.brain);
            }
          } else {
            selectedStatus.value = "Extinct (Fossil)";
            selectedEnergy.value = 0;
            selectedAge.value = 2700;
          }
        }

        statPopulation.innerText = `${creatures.length} / 25`;
        statGeneration.innerText = `Gen. ${highestGeneration}`;
        statSpores.innerText = `${foodPellets.length} Spores`;
      }
      else if (data.type === "FOOD_TICK") {
        foodPellets = data.foodPellets;
        statSpores.innerText = `${foodPellets.length} Spores`;
      }
      else if (data.type === "CREATURE_SPAWNED") {
        const tele = data.creature;

        if (!creatures.some(c => Number(c.id) === Number(tele.id))) {
          creatures.push({
            ...tele,
            phenotype: tele.phenotype,
            neuronStates: [],
            neuronActivations: []
          });
        }
      }
      else if (data.type === "DATABASE_CHANGED") {
        fetchRosterRecords();
      }
      else if (data.type === "BITE_EVENT") {
        biteImpacts.push({ x: data.x, y: data.y, age: 0 });
      }
      else if (data.type === "LOG_EVENT") {
        logToTerminal(data.message, data.logType);
      }

    } catch (err: any) {
      console.error("[Tauri Beta] Telemetry crash:", err);
      safeInvoke("handle_client_action", { action: JSON.stringify({ type: "CLIENT_ERROR", error: "[TELEMETRY CRASH] " + (err.stack || err.message || String(err)) }) }).catch(e => {
        console.error("Failed to mirror telemetry crash error:", e);
      });
    }
  });
}

function selectSpecimen(agent: CreatureAgent) {
  selectedId.value = agent.id;
  selectedName.value = agent.phenotype.latinName;
  selectedTaxa.value = `${agent.phenotype.latinName.substring(0, 16)} (Strain: #${agent.id}, Gen: ${agent.generation})`;
  selectedGenome.value = agent.genome;
  selectedMethylations.value = agent.phenotype.methylations;
  selectedPhenotype.value = agent.phenotype;
  selectedMaxEnergy.value = agent.phenotype.stomachCapacity;

  brainRenderer.compile(agent.phenotype.brain, agent.phenotype.organelles.length * 5);

  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SELECT_AGENT", id: agent.id }) }).catch(err => {
    console.error("Tauri invoke error SELECT_AGENT:", err);
  });
}

function logToTerminal(message: string, logType: string = "system") {
  const row = document.createElement("div");
  row.className = "log-row";

  const time = document.createElement("span");
  time.className = "log-time";
  time.innerText = new Date().toLocaleTimeString();

  const tag = document.createElement("span");
  tag.className = `log-tag ${logType}`;
  tag.innerText = logType === "repair" ? "healing" : (logType === "mutation" ? "biology" : "system");

  const msg = document.createElement("span");
  msg.className = "log-msg";
  msg.innerText = message;

  row.appendChild(time);
  row.appendChild(tag);
  row.appendChild(msg);

  terminalLogs.appendChild(row);
  terminalLogs.scrollTop = terminalLogs.scrollHeight;
}

const canvas = document.getElementById("creature-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
let renderer: CreatureRenderer;

function resizeBetaCanvas() {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
}

function drawBetaSimulationFrame(timestamp: number) {
  if (!ctx) return;

  try {
    const dpr = window.devicePixelRatio || 1;
    const cw = canvas.width / dpr;
    const ch = canvas.height / dpr;

    ctx.clearRect(0, 0, cw, ch);

    const { viewCenterX, viewCenterY } = camera.getViewportBounds();

    ctx.save();
    ctx.translate(viewCenterX * dpr, viewCenterY * dpr);
    ctx.scale(dpr * camera.camZoom, dpr * camera.camZoom);
    ctx.translate(-camera.camX, -camera.camY);

    ctx.save();
    const grad = ctx.createLinearGradient(19200 / 2, 0, 19200 / 2, 10800);
    grad.addColorStop(0, "#080c18");
    grad.addColorStop(0.35, "#060812");
    grad.addColorStop(1, "#030409");
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, 19200, 10800);
    ctx.restore();

    ctx.save();
    for (const pellet of foodPellets) {
      ctx.beginPath();
      ctx.arc(pellet.x, pellet.y, 4, 0, Math.PI * 2);
      ctx.fillStyle = "rgba(16, 185, 129, 0.82)";
      ctx.shadowColor = "#10b981";
      ctx.shadowBlur = 8;
      ctx.fill();
    }
    ctx.restore();

    creatures.forEach(agent => {
      if (agent.phenotype) {
        renderer.render(agent.phenotype, timestamp, agent.px, agent.py, agent.headingAngle, agent.omegaRot);

        if (Number(agent.id) === Number(selectedId.value)) {
          ctx.save();
          const isPred = agent.phenotype.carnivory >= 0.55;
          ctx.strokeStyle = isPred ? "rgba(239, 68, 68, 0.72)" : "rgba(0, 242, 254, 0.72)";
          ctx.lineWidth = 1.5;
          ctx.setLineDash([4, 4]);
          ctx.beginPath();
          ctx.arc(agent.px, agent.py, agent.phenotype.spinalHarmonics.meanRadius * 2.6 * 0.5, 0, Math.PI * 2);
          ctx.stroke();
          ctx.restore();
        }
      }
    });

    ctx.save();
    biteImpacts.forEach(impact => {
      impact.age++;
      const progress = impact.age / 24;
      const radius = 5 + progress * 28;
      const opacity = 1.0 - progress;

      ctx.strokeStyle = `rgba(255, 0, 127, ${opacity})`;
      ctx.lineWidth = 2.0;
      ctx.beginPath();
      ctx.arc(impact.x, impact.y, radius, 0, Math.PI * 2);
      ctx.stroke();
    });
    biteImpacts = biteImpacts.filter(i => i.age < 24);
    ctx.restore();

    ctx.strokeStyle = "rgba(0, 242, 254, 0.4)";
    ctx.lineWidth = 15;
    ctx.strokeRect(0, 0, 19200, 10800);

    ctx.restore();
  } catch (err: any) {
    console.error("[Tauri Beta] Render error caught:", err);
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "CLIENT_ERROR", error: err.stack || err.message || String(err) }) }).catch(e => {
      console.error("Failed to mirror client error:", e);
    });
  }

  requestAnimationFrame(drawBetaSimulationFrame);
}

// Bind Camera Mouse Panning & Zoom Events
camera = new InteractiveCamera(canvas);
camera.setupEventListeners(
  (mx, my) => {
    let hovered = false;
    creatures.forEach(agent => {
      const dx = agent.px - mx;
      const dy = agent.py - my;
      const dist = Math.sqrt(dx*dx + dy*dy);
      const bodyRadius = agent.phenotype.spinalHarmonics.meanRadius * 2.6 * 0.5 + 10;
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

    if (closestAgent) {
      selectSpecimen(closestAgent);
    } else {
      invoke("handle_client_action", { action: JSON.stringify({ type: "SPAWN_FOOD", x: mx, y: my }) }).catch(err => {
        console.error("Tauri invoke error SPAWN_FOOD:", err);
      });
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
        safeInvoke("get_fossil_phenotype", { genome: record.genome }).then((pheno) => {
          if (pheno) {
            selectedPhenotype.value = pheno;
          }
        });
        selectedMaxEnergy.value = 100;
        selectedStatus.value = "Extinct (Fossil)";
        selectedEnergy.value = 0;
        selectedAge.value = 2700;
        brainContainer.innerHTML = `<div class="fallback-state">Species is fossilized. Neural activity extinguished.</div>`;
      }
    }
  }
});

const btnToggleSim = document.getElementById("btn-toggle-sim") as HTMLButtonElement;
const lblToggleSim = document.getElementById("lbl-toggle-sim") as HTMLSpanElement;
let isSimRunning = true;

btnToggleSim?.addEventListener("click", () => {
  const nextState = !isSimRunning;
  invoke("handle_client_action", { action: JSON.stringify({ type: "TOGGLE_SIMULATION", running: nextState }) }).catch(err => {
    console.error("Tauri invoke error TOGGLE_SIMULATION:", err);
  });
});

document.getElementById("btn-inject")?.addEventListener("click", () => {
  invoke("handle_client_action", { action: JSON.stringify({ type: "INJECT_URZELLE" }) }).catch(err => {
    console.error("Tauri invoke error INJECT_URZELLE:", err);
  });
});

document.getElementById("btn-reset")?.addEventListener("click", () => {
  invoke("handle_client_action", { action: JSON.stringify({ type: "RESET_EVOLUTION" }) }).then(() => {
    selectedId.value = null;
  }).catch(err => {
    console.error("Tauri invoke error RESET_EVOLUTION:", err);
  });
});

document.getElementById("btn-clear-console")?.addEventListener("click", () => {
  terminalLogs.innerHTML = "";
});

window.addEventListener("load", async () => {
  resizeBetaCanvas();
  renderer = new CreatureRenderer(canvas);
  
  window.addEventListener("resize", resizeBetaCanvas);
  
  const { visibleWidth, visibleHeight } = camera.getViewportBounds();
  camera.reset(visibleWidth, visibleHeight);

  await initBetaWebSocket();

  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "CLIENT_READY" }) }).catch(err => {
    console.error("Failed to send CLIENT_READY handshake:", err);
  });

  requestAnimationFrame(drawBetaSimulationFrame);
});
