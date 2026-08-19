import { signal, effect } from "@preact/signals-core";
import { 
  parseGenome
} from "./biology/dna";
import { CreatureRenderer } from "./render/creatureRenderer";
import { getAllSpecies, SpeciesRecord } from "./biology/speciesDb";
import { CreatureAgent, FoodSpore } from "./shared/types";
import { generateWorld, ProceduralWorld } from "./shared/mapGenerator";

let world: ProceduralWorld | null = null;
let offscreenCanvas: HTMLCanvasElement | null = null;

function createBiomeCache(world: ProceduralWorld) {
  offscreenCanvas = document.createElement('canvas');
  offscreenCanvas.width = 48;
  offscreenCanvas.height = 27;
  const oCtx = offscreenCanvas.getContext('2d')!;

  for (let c = 0; c < 48; c++) {
    for (let r = 0; r < 27; r++) {
      const idx = c * 27 + r;
      const biome = world.biomes[idx];
      if (biome) {
        oCtx.fillStyle = biome.color;
        oCtx.fillRect(c, r, 1, 1);
      }
    }
  }
}

// ============================================================================
// 📊 STATE 1: Pure Mutable Game-Engine Arrays (0% Memory Allocation, 60 FPS Locked)
// ============================================================================
let creatures: CreatureAgent[] = [];
let foodPellets: FoodSpore[] = [];
let highestGeneration = 1;

// ============================================================================
// 📷 STATE 2: Interactive Camera State (Zoom, Pan, Scroll)
// ============================================================================
let camX = 19200 / 2; // Camera focus look-at X (starts at middle of the giant map)
let camY = 10800 / 2; // Camera focus look-at Y
let camZoom = 0.05;   // Scale magnifier (will be set responsively on boot)

// Mouse drag-panning state trackers
let isDragging = false;
let hasDragged = false;
let startX = 0;
let startY = 0;

// ============================================================================
// 🧬 STATE 3: Fine-Grained Preact Signals for HUD Overlays
// ============================================================================
const selectedId = signal<number | null>(null);
const selectedName = signal("Unnamed Creature");
const selectedTaxa = signal("Clonal strain - Gen. 1");
const selectedStatus = signal("Alive");
const selectedEnergy = signal(0);
const selectedMaxEnergy = signal(100);
const selectedAdrenaline = signal(1.0);
const selectedAge = signal(0);

const selectedGenome = signal("");
const selectedMethylations = signal<number[]>([]);

const speciesRosterSignal = signal<SpeciesRecord[]>([]);

// Accordion Toggles (Fine-grained UI folding states)
const isAliveExpanded = signal(true);       // Expanded by default
const isExtinctExpanded = signal(false);   // Collapsed by default to keep workspace clean

// ============================================================================
// 🎨 UI 1: High-Speed Direct DOM Signal Binders
// ============================================================================
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

// Re-render Left Roster of recorded species only when speciesRosterSignal or toggle signals change!
effect(() => {
  const list = speciesRosterSignal.value;
  if (list.length === 0) {
    speciesRoster.innerHTML = `<div class="loading-state">No species registered.</div>`;
    return;
  }

  const aliveList = list.filter(r => r.status === "alive");
  const extinctList = list.filter(r => r.status === "extinct");

  let html = "";

  // 1. Group 1: Living Species
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

  // 2. Group 2: Extinct Species (Fossils)
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

// Toggle Inspector view states based on active selection
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

// Bind Profile Info
effect(() => { specimenName.innerText = selectedName.value; });
effect(() => { specimenTaxa.innerText = selectedTaxa.value; });
effect(() => { 
  specimenStatus.innerText = selectedStatus.value; 
  specimenStatus.style.background = selectedStatus.value === "Alive" ? "rgba(16, 185, 129, 0.12)" : "rgba(77, 89, 116, 0.12)";
  specimenStatus.style.color = selectedStatus.value === "Alive" ? "var(--secondary-green)" : "var(--text-muted)";
});

// Bind Energy Progress Bar
effect(() => {
  const val = selectedEnergy.value;
  const max = selectedMaxEnergy.value;
  const pct = Math.max(0, Math.min(100, (val / max) * 100));
  energyBar.style.width = `${pct}%`;
  energyText.innerText = `${Math.round(val)} / ${Math.round(max)}nJ`;
});

// Bind Adrenaline Progress Bar
effect(() => {
  const val = selectedAdrenaline.value;
  const pct = Math.max(0, Math.min(100, ((val - 1.0) / 0.8) * 100));
  adrenalineBar.style.width = `${pct}%`;
  adrenalineText.innerText = `${val.toFixed(2)}x`;
});

// Bind Age Progress Bar (2700 frames absolute lifespan)
effect(() => {
  const val = selectedAge.value;
  const pct = Math.max(0, Math.min(100, (val / 2700) * 100));
  ageBar.style.width = `${pct}%`;
  ageText.innerText = `${Math.round(val / 60)}s`;
});

// Bind Watson-Crick DNA Helix Split Grid
effect(() => {
  const g = selectedGenome.value;
  const m = selectedMethylations.value;
  if (!g) {
    genomeGrid.innerHTML = "";
    return;
  }

  let html = "";
  for (let i = 0; i < g.length; i++) {
    const char = g[i];
    const isPromoter = i < 16;
    const isMethylated = m && m[i] !== 0;

    let bg = "rgba(255,255,255,0.02)";
    let border = "1px solid rgba(255,255,255,0.03)";

    if (isPromoter) {
      bg = "#ffffff";
    } else {
      const charVal = char.charCodeAt(0) - 65;
      bg = `hsla(${charVal * 13.8}, 75%, 45%, 0.15)`;
      border = `1.2px solid hsla(${charVal * 13.8}, 75%, 45%, 0.4)`;
    }

    html += `
      <div class="loci-node ${isPromoter ? 'promoter' : 'active'} ${isMethylated ? 'methylated' : ''}" 
           style="background: ${bg}; border: ${border};" 
           title="Locus ${i}: ${char}${isMethylated ? ' (Methylated +' + m[i] + ')' : ''}">
        ${char}
      </div>
    `;
  }
  genomeGrid.innerHTML = html;
});

// ============================================================================
// 🎨 UI 2: Brain Directed Graph Cache Renderer
// ============================================================================
const brainContainer = document.getElementById("inspect-brain-container") as HTMLDivElement;
const brainSvgCache = new Map<string, SVGElement>();

function compileBetaBrainSVG(brain: any): void {
  brainSvgCache.clear();
  if (!brain) {
    brainContainer.innerHTML = "";
    return;
  }

  let svgContent = `<svg viewBox="0 0 320 210">`;

  // 1. Draw Synapses
  brain.synapses.forEach((syn: any) => {
    const from = brain.neurons[syn.fromNode];
    const to = brain.neurons[syn.toNode];
    if (from && to) {
      const synId = `beta-syn-${syn.fromNode}-${syn.toNode}`;
      const color = syn.weight > 0 ? "rgba(16, 185, 129, 0.28)" : "rgba(239, 68, 68, 0.28)";
      svgContent += `
        <line id="${synId}" x1="${from.x * 320}" y1="${from.y * 210}" x2="${to.x * 320}" y2="${to.y * 210}" 
              stroke="${color}" stroke-width="${Math.max(0.5, Math.abs(syn.weight) * 1.5)}" />
      `;
    }
  });

  // 2. Draw Neuron Nodes
  brain.neurons.forEach((n: any) => {
    const nodeId = `beta-node-${n.id}`;
    const isInput = n.type === "input";
    const isOutput = n.type === "output";
    const color = isInput ? "var(--primary-cyan)" : (isOutput ? "var(--accent-purple)" : "var(--text-muted)");

    svgContent += `
      <circle id="${nodeId}" cx="${n.x * 320}" cy="${n.y * 210}" r="${isInput || isOutput ? 4.5 : 3.2}" 
              fill="#111827" stroke="${color}" stroke-width="1.5" />
    `;
  });

  svgContent += `</svg>`;
  brainContainer.innerHTML = svgContent;

  // Cache neuron circle element references for sub-millisecond glows
  brain.neurons.forEach((n: any) => {
    const id = `beta-node-${n.id}`;
    const el = document.getElementById(id) as any;
    if (el) brainSvgCache.set(id, el);
  });

  // Cache synapse line element references for sub-millisecond glows
  brain.synapses.forEach((syn: any) => {
    const id = `beta-syn-${syn.fromNode}-${syn.toNode}`;
    const el = document.getElementById(id) as any;
    if (el) brainSvgCache.set(id, el);
  });
}

function updateBetaBrainLiveGlows(activations: number[], brain: any): void {
  if (!brain || !activations) return;

  // 1. Update Neurons
  brain.neurons.forEach((n: any) => {
    const id = `beta-node-${n.id}`;
    const el = brainSvgCache.get(id);
    if (el) {
      const rawAct = Math.max(0.0, Math.min(1.0, Math.abs(activations[n.id] || 0.0)));
      const act = Math.pow(rawAct, 4.0); // clean contrast

      const isInput = n.type === "input";
      const isOutput = n.type === "output";
      const colorGlow = isInput ? "#00f2fe" : (isOutput ? "#c084fc" : "#e2e8f0");

      const fill = act > 0.35 ? colorGlow : "#111827";
      const radius = isInput || isOutput ? (act > 0.45 ? 6.5 : 4.5) : (act > 0.45 ? 5.0 : 3.2);

      el.setAttribute("fill", fill);
      el.setAttribute("r", radius.toString());
    }
  });

  // 2. Update Synapses (Highlighting active signals in-flight!)
  brain.synapses.forEach((syn: any) => {
    const id = `beta-syn-${syn.fromNode}-${syn.toNode}`;
    const el = brainSvgCache.get(id);
    if (el) {
      const preVal = Math.max(0.0, Math.min(1.0, Math.abs(activations[syn.fromNode] || 0.0)));
      const act = Math.pow(preVal, 4.0); // clean contrast

      const isExcitatory = syn.weight > 0;
      const baseColor = isExcitatory ? "16, 185, 129" : "239, 68, 68";
      const opacity = act > 0.35 ? 0.95 : 0.28;
      const strokeWidth = Math.max(0.5, Math.abs(syn.weight) * 1.5) * (act > 0.45 ? 2.2 : 1.0);

      el.setAttribute("stroke", `rgba(${baseColor}, ${opacity})`);
      el.setAttribute("stroke-width", strokeWidth.toString());
    }
  });
}

// ============================================================================
// 🔌 MULTIPLAYER ENGINE: Real-Time Sockets Pipeline
// ============================================================================
let socket: WebSocket | null = null;
let biteImpacts: { x: number; y: number; age: number }[] = [];

function fetchRosterRecords() {
  getAllSpecies().then(records => {
    speciesRosterSignal.value = records;
  }).catch(err => {
    console.error("Failed to load species ledger:", err);
  });
}

function initBetaWebSocket() {
  if (socket) return;

  socket = new WebSocket("ws://localhost:3002");

  socket.onopen = () => {
    logToTerminal("Connection to evolution substrate established.", "system");
    fetchRosterRecords();
  };

  socket.onmessage = (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data);

      if (data.type === "INIT_STATE") {
        highestGeneration = data.highestGeneration;
        foodPellets = data.foodPellets;

        if (data.seed) {
          world = generateWorld(data.seed);
          createBiomeCache(world);
        }

        creatures = data.creatures.map((c: any) => ({
          ...c,
          phenotype: parseGenome(c.genome, c.antisense)
        }));

        // Set default selection if none active
        if (creatures.length > 0 && selectedId.value === null) {
          selectSpecimen(creatures[0]);
        }
      }
      else if (data.type === "TELEMETRY_TICK") {
        highestGeneration = data.highestGeneration;
        foodPellets = data.foodPellets;
        const incoming = data.creatures;

        // 1. High-Performance, In-Place Mutable Update (0 Bytes allocated!)
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

        // 2. Filter dead agents locally
        const serverIds = new Set(incoming.map((c: any) => c.id));
        creatures = creatures.filter(c => serverIds.has(c.id));

        // 3. Highly-targeted selective signals update
        const id = selectedId.value;
        if (id !== null) {
          const active = creatures.find(c => Number(c.id) === Number(id));
          if (active) {
            selectedStatus.value = "Alive";
            selectedEnergy.value = active.energy;
            selectedAdrenaline.value = active.adrenaline || 1.0;
            selectedAge.value = active.age;

            if (data.selectedBrain && Number(data.selectedBrain.id) === Number(id)) {
              updateBetaBrainLiveGlows(data.selectedBrain.activations, active.phenotype.brain);
            }
          } else {
            selectedStatus.value = "Extinct (Fossil)";
            selectedEnergy.value = 0;
            selectedAge.value = 2700;
          }
        }

        // 4. Update global stats DOM
        statPopulation.innerText = `${creatures.length} / 25`;
        statGeneration.innerText = `Gen. ${highestGeneration}`;
        statSpores.innerText = `${foodPellets.length} Spores`;
      }
      else if (data.type === "CREATURE_SPAWNED") {
        const tele = data.creature;

        // Self-healing check to avoid duplicates
        if (!creatures.some(c => Number(c.id) === Number(tele.id))) {
          creatures.push({
            ...tele,
            phenotype: parseGenome(tele.genome, tele.antisense),
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

    } catch (err) {
      console.error("[Client Beta] Telemetry crash:", err);
    }
  };

  socket.onclose = () => {
    socket = null;
    logToTerminal("Interface severed. Reactivation in progress...", "mutation");
    setTimeout(initBetaWebSocket, 2000);
  };
}

function selectSpecimen(agent: any) {
  selectedId.value = agent.id;
  selectedName.value = agent.phenotype.latinName;
  selectedTaxa.value = `${agent.phenotype.latinName.substring(0, 16)} (Strain: #${agent.id}, Gen: ${agent.generation})`;
  selectedGenome.value = agent.genome;
  selectedMethylations.value = agent.phenotype.methylations;
  selectedMaxEnergy.value = agent.phenotype.stomachCapacity;

  compileBetaBrainSVG(agent.phenotype.brain);

  // Inform the server to start streaming brain activations
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "SELECT_AGENT", id: agent.id }));
  }
}

// ============================================================================
// 📜 LOGGER: Scrollbox terminal updates
// ============================================================================
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
  terminalLogs.scrollTop = terminalLogs.scrollHeight; // Auto-scroll
}

// ============================================================================
// 🌌 GRAPHICS: Render canvas guide
// ============================================================================
const canvas = document.getElementById("creature-canvas") as HTMLCanvasElement;
const ctx = canvas.getContext("2d") as CanvasRenderingContext2D;
let renderer: CreatureRenderer;

// Reset camera focal point to optimal fit on load or manual hot-key
function resetCameraView() {
  const visibleWidth = window.innerWidth - 680;
  const visibleHeight = window.innerHeight - 260;
  
  // Calculate dynamic fit zoom
  camZoom = Math.min(visibleWidth / 19200, visibleHeight / 10800);
  camX = 19200 / 2;
  camY = 10800 / 2;
  
  logToTerminal("Camera view centered on full-screen glass tank and reset.", "system");
}

function resizeBetaCanvas() {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
}

function drawWorldTerrain(ctx: CanvasRenderingContext2D) {
  if (!world) return;

  // 1. Draw Biome Areas (renders seamless organic smoothed coastlines)
  if (offscreenCanvas) {
    ctx.save();
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = "high";
    ctx.drawImage(offscreenCanvas, 0, 0, 19200, 10800);
    ctx.restore();
  }

  // 2. Draw Thermal Vents (dotted current zones)
  for (const vent of world.vents) {
    ctx.beginPath();
    ctx.arc(vent.x, vent.y, vent.radius, 0, Math.PI * 2);
    ctx.strokeStyle = "rgba(14, 165, 233, 0.05)";
    ctx.lineWidth = 6;
    ctx.stroke();

    // Center vent core
    ctx.beginPath();
    ctx.arc(vent.x, vent.y, 40, 0, Math.PI * 2);
    ctx.fillStyle = vent.forceType === "push" ? "rgba(56, 189, 248, 0.25)" : vent.forceType === "pull" ? "rgba(236, 72, 153, 0.25)" : "rgba(168, 85, 247, 0.25)";
    ctx.fill();
  }

  // 3. Draw Solid Obstacles (circular solid coral reefs / rocks)
  for (const obs of world.obstacles) {
    // Outer glow
    ctx.beginPath();
    ctx.arc(obs.x, obs.y, obs.radius + 15, 0, Math.PI * 2);
    ctx.fillStyle = obs.type === "rock" ? "rgba(51, 65, 85, 0.05)" : "rgba(244, 63, 94, 0.05)";
    ctx.fill();

    // Solid core (Organic Jagged Polygon)
    ctx.beginPath();
    ctx.moveTo(obs.vertices[0].x, obs.vertices[0].y);
    for (let j = 1; j < obs.vertices.length; j++) {
      ctx.lineTo(obs.vertices[j].x, obs.vertices[j].y);
    }
    ctx.closePath();
    ctx.fillStyle = obs.color;
    ctx.fill();

    ctx.strokeStyle = "#f1f5f9";
    ctx.lineWidth = 6;
    ctx.stroke();
  }
}

function drawBetaSimulationFrame(timestamp: number) {
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const cw = canvas.width / dpr;
  const ch = canvas.height / dpr;

  ctx.clearRect(0, 0, cw, ch);

  // Unobstructed center viewport bounds
  const visibleWidth = window.innerWidth - 680;
  const visibleHeight = window.innerHeight - 260;
  const viewCenterX = 305 + visibleWidth / 2;
  const viewCenterY = 90 + visibleHeight / 2;

  // 1. Apply 2D View Projection Matrix (Centered on look-at point)
  ctx.save();
  ctx.translate(viewCenterX * dpr, viewCenterY * dpr);
  ctx.scale(dpr * camZoom, dpr * camZoom);
  ctx.translate(-camX, -camY);

  // 2. Draw glowing space deep gradient over the entire logical boundary
  ctx.save();
  const grad = ctx.createLinearGradient(19200 / 2, 0, 19200 / 2, 10800);
  grad.addColorStop(0, "#080c18");
  grad.addColorStop(0.35, "#060812");
  grad.addColorStop(1, "#030409");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, 19200, 10800);
  ctx.restore();

  // Draw procedural Bio-Basin terrain (biomes, solid reefs, thermal current vents)
  drawWorldTerrain(ctx);

  // 3. Draw algae/food spores
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

  // 4. Draw swimming creatures
  creatures.forEach(agent => {
    renderer.render(agent.phenotype, timestamp, agent.px, agent.py, agent.headingAngle, agent.omegaRot);

    // Render selector guide ring if selected
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
  });

  // 5. Render shockwave crimson rings (BITE impact)
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

  // 6. Draw high-tech glowing glass tank boundary frame!
  ctx.strokeStyle = "rgba(0, 242, 254, 0.4)";
  ctx.lineWidth = 15;
  ctx.strokeRect(0, 0, 19200, 10800);

  ctx.restore(); // Restore camera look-at viewport transforms

  requestAnimationFrame(drawBetaSimulationFrame);
}

// ============================================================================
// 🖱️ MOUSE INTERACTION & DRAG-PAN CONTROLS
// ============================================================================
canvas.addEventListener("mousedown", (e) => {
  isDragging = true;
  hasDragged = false;
  startX = e.clientX;
  startY = e.clientY;
  canvas.style.cursor = "grabbing"; // Instantly show grabbing on click hold!
});

canvas.addEventListener("mousemove", (e) => {
  if (isDragging) {
    const dx = e.clientX - startX;
    const dy = e.clientY - startY;

    // Set drag flag only if mouse actually moves past buffer threshold
    if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
      hasDragged = true;
    }

    // Pan camera focused look-at position (scale velocity inversely with zoom scale!)
    camX -= dx / camZoom;
    camY -= dy / camZoom;

    // Hard clamp camera to keep viewport neatly bounded inside the giant tank
    camX = Math.max(0, Math.min(19200, camX));
    camY = Math.max(0, Math.min(10800, camY));

    startX = e.clientX;
    startY = e.clientY;
    canvas.style.cursor = "grabbing";
  } else {
    // If not dragging, check for hovers to toggle between "grab" and "pointer"!
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    
    const visibleWidth = window.innerWidth - 680;
    const visibleHeight = window.innerHeight - 260;
    const viewCenterX = 305 + visibleWidth / 2;
    const viewCenterY = 90 + visibleHeight / 2;

    const mx = camX + (((e.clientX - rect.left) * (canvas.width / dpr / rect.width)) - viewCenterX) / camZoom;
    const my = camY + (((e.clientY - rect.top) * (canvas.height / dpr / rect.height)) - viewCenterY) / camZoom;

    let hovered = false;
    creatures.forEach(agent => {
      const dx = agent.px - mx;
      const dy = agent.py - my;
      const dist = Math.sqrt(dx*dx + dy*dy);
      // If hovering near the creature body radius, trigger pointer hover!
      const bodyRadius = agent.phenotype.spinalHarmonics.meanRadius * 2.6 * 0.5 + 10;
      if (dist < bodyRadius) {
        hovered = true;
      }
    });

    canvas.style.cursor = hovered ? "pointer" : "grab";
  }
});

canvas.addEventListener("mouseup", () => {
  isDragging = false;
  canvas.style.cursor = "grab";
});

canvas.addEventListener("mouseleave", () => {
  isDragging = false;
  canvas.style.cursor = "default";
});

// Interactive Mouse Wheel Zoom centered on cursor
canvas.addEventListener("wheel", (e) => {
  e.preventDefault();

  const rect = canvas.getBoundingClientRect();
  const visibleWidth = window.innerWidth - 680;
  const visibleHeight = window.innerHeight - 260;
  const viewCenterX = 305 + visibleWidth / 2;
  const viewCenterY = 90 + visibleHeight / 2;

  // Get current mouse coordinate relative to screen center
  const mouseX = e.clientX - rect.left;
  const mouseY = e.clientY - rect.top;

  // Back-project screen mouse position to logical space coordinates before scaling
  const logMouseX = camX + (mouseX - viewCenterX) / camZoom;
  const logMouseY = camY + (mouseY - viewCenterY) / camZoom;

  // Define zoom intensity scaling multiplier
  const intensity = 0.12;
  const direction = e.deltaY < 0 ? 1 : -1;
  const factor = Math.exp(direction * intensity);

  // Clamp camera zoom values
  const minZoom = Math.min(visibleWidth / 19200, visibleHeight / 10800) * 0.8;
  const maxZoom = 2.0;

  camZoom = Math.max(minZoom, Math.min(maxZoom, camZoom * factor));

  // Translate camera focal points to keep mouse coordinates identical after scaling!
  camX = logMouseX - (mouseX - viewCenterX) / camZoom;
  camY = logMouseY - (mouseY - viewCenterY) / camZoom;

  // Hard boundary clamps
  camX = Math.max(0, Math.min(19200, camX));
  camY = Math.max(0, Math.min(10800, camY));
}, { passive: false });

// Click selection on canvas (Only triggers if user didn't drag/pan!)
canvas.addEventListener("click", (e) => {
  if (hasDragged) return; // Prevent selection triggers when mouse-up ends a camera slide!

  const rect = canvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;

  const visibleWidth = window.innerWidth - 680;
  const visibleHeight = window.innerHeight - 260;
  const viewCenterX = 305 + visibleWidth / 2;
  const viewCenterY = 90 + visibleHeight / 2;

  // Translate click screen coordinates to logical world space!
  const mx = camX + (((e.clientX - rect.left) * (canvas.width / dpr / rect.width)) - viewCenterX) / camZoom;
  const my = camY + (((e.clientY - rect.top) * (canvas.height / dpr / rect.height)) - viewCenterY) / camZoom;

  let closestAgent: any = null;
  let minDist = 120; // click proximity buffer

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
    // Click on empty water spawns food on logical server coordinates
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: "SPAWN_FOOD", x: mx, y: my }));
    }
  }
});

// Reset Camera View Hot-Key (Press R)
window.addEventListener("keydown", (e) => {
  if (e.key.toLowerCase() === "r") {
    resetCameraView();
  }
});

// Click list delegation for roster card select and accordion headers
speciesRoster.addEventListener("click", (e) => {
  const target = e.target as HTMLElement;

  // 1. Accordion Toggles
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

  // 2. Card Selection
  const item = target.closest(".roster-card") as HTMLDivElement | null;
  if (!item) return;

  const recId = item.getAttribute("data-id");
  if (recId) {
    const agent = creatures.find(c => c.speciesId === recId);
    if (agent) {
      selectSpecimen(agent);
      
      // Auto focus camera lock-on target on selection click!
      camX = agent.px;
      camY = agent.py;
      logToTerminal(`Camera focused on specimen #${agent.id}.`, "system");
    } else {
      // Fetch fossil stats
      const record = speciesRosterSignal.value.find(r => r.id === recId);
      if (record) {
        selectedId.value = 99999; // temporary virtual ID for fossils
        selectedName.value = record.name;
        selectedTaxa.value = `${record.name} [FOSSIL - EXTINCT]`;
        selectedGenome.value = record.genome;
        selectedMethylations.value = Array(256).fill(0); // empty methylations for fossil view
        selectedMaxEnergy.value = 100;
        selectedStatus.value = "Extinct (Fossil)";
        selectedEnergy.value = 0;
        selectedAge.value = 2700;
        brainContainer.innerHTML = `<div class="fallback-state">Species is fossilized. Neural activity extinguished.</div>`;
      }
    }
  }
});

// Action Buttons Event Handlers
document.getElementById("btn-inject")?.addEventListener("click", () => {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "INJECT_URZELLE" }));
  }
});

document.getElementById("btn-reset")?.addEventListener("click", () => {
  if (socket && socket.readyState === WebSocket.OPEN) {
    socket.send(JSON.stringify({ type: "RESET_EVOLUTION" }));
    selectedId.value = null;
  }
});

document.getElementById("btn-clear-console")?.addEventListener("click", () => {
  terminalLogs.innerHTML = "";
});

// ============================================================================
// 🚀 BOOTSTRAP: Window loader initializations
// ============================================================================
window.addEventListener("load", () => {
  resizeBetaCanvas();
  renderer = new CreatureRenderer(canvas);
  
  window.addEventListener("resize", resizeBetaCanvas);
  
  // Initialize camera responsive fits on boot
  resetCameraView();

  // Establish web socket stream
  initBetaWebSocket();

  // Run graphic frame loop
  requestAnimationFrame(drawBetaSimulationFrame);
});