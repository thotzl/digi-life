import { signal, effect } from "@preact/signals-core";
import { 
  parseGenome
} from "./biology/dna";
import { CreatureRenderer } from "./render/creatureRenderer";
import { getAllSpecies, SpeciesRecord } from "./biology/speciesDb";
import { CreatureAgent, FoodSpore } from "./shared/types";

// ============================================================================
// 📊 STATE 1: Pure Mutable Game-Engine Arrays (0% Memory Allocation, 60 FPS Locked)
// ============================================================================
let creatures: CreatureAgent[] = [];
let foodPellets: FoodSpore[] = [];
let highestGeneration = 1;

// ============================================================================
// 🧬 STATE 2: Fine-Grained Preact Signals for HUD Overlays
// ============================================================================
const selectedId = signal<number | null>(null);
const selectedName = signal("Namenloses Wesen");
const selectedTaxa = signal("Clonal strain - Gen. 1");
const selectedStatus = signal("Lebend");
const selectedEnergy = signal(0);
const selectedMaxEnergy = signal(100);
const selectedAdrenaline = signal(1.0);
const selectedAge = signal(0);

const selectedGenome = signal("");
const selectedMethylations = signal<number[]>([]);

const speciesRosterSignal = signal<SpeciesRecord[]>([]);

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

// Re-render Left Roster of recorded species only when speciesRosterSignal changes
effect(() => {
  const list = speciesRosterSignal.value;
  if (list.length === 0) {
    speciesRoster.innerHTML = `<div class="loading-state">Keine Spezies registriert.</div>`;
    return;
  }

  let html = "";
  list.forEach(rec => {
    const isSelected = selectedId.value !== null && 
      creatures.find(c => Number(c.id) === Number(selectedId.value))?.speciesId === rec.id;

    const statusClass = rec.status === "alive" ? "alive" : "fossil";
    const statusText = rec.status === "alive" ? "Lebend" : "Fossil";

    html += `
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
  });
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
  specimenStatus.style.background = selectedStatus.value === "Lebend" ? "rgba(16, 185, 129, 0.12)" : "rgba(77, 89, 116, 0.12)";
  specimenStatus.style.color = selectedStatus.value === "Lebend" ? "var(--secondary-green)" : "var(--text-muted)";
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
           title="Locus ${i}: ${char}${isMethylated ? ' (Methyliert +' + m[i] + ')' : ''}">
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

  // Cache element references for sub-millisecond glows
  brain.neurons.forEach((n: any) => {
    const id = `beta-node-${n.id}`;
    const el = document.getElementById(id) as any;
    if (el) brainSvgCache.set(id, el);
  });
}

function updateBetaBrainLiveGlows(activations: number[], brain: any): void {
  if (!brain || !activations) return;

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
    logToTerminal("Verbindung zum Evolutions-Substrat hergestellt.", "system");
    fetchRosterRecords();
  };

  socket.onmessage = (event: MessageEvent) => {
    try {
      const data = JSON.parse(event.data);

      if (data.type === "INIT_STATE") {
        highestGeneration = data.highestGeneration;
        foodPellets = data.foodPellets;
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
            selectedStatus.value = "Lebend";
            selectedEnergy.value = active.energy;
            selectedAdrenaline.value = active.adrenaline || 1.0;
            selectedAge.value = active.age;

            if (data.selectedBrain && Number(data.selectedBrain.id) === Number(id)) {
              updateBetaBrainLiveGlows(data.selectedBrain.activations, active.phenotype.brain);
            }
          } else {
            selectedStatus.value = "Ausgestorben (Fossil)";
            selectedEnergy.value = 0;
            selectedAge.value = 2700;
          }
        }

        // 4. Update global stats DOM
        statPopulation.innerText = `${creatures.length} / 15`;
        statGeneration.innerText = `${highestGeneration}. Gen`;
        statSpores.innerText = `${foodPellets.length} Sporen`;
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
    logToTerminal("Schnittstelle abgebrochen. Reaktivierung läuft...", "mutation");
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

  compileBetaBrainSVG(agent.phenotype);

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
  tag.innerText = logType === "repair" ? "heilt" : (logType === "mutation" ? "biologie" : "system");

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

function resizeBetaCanvas() {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = window.innerWidth * dpr;
  canvas.height = window.innerHeight * dpr;
}

function drawBetaSimulationFrame(timestamp: number) {
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const cw = canvas.width / dpr;
  const ch = canvas.height / dpr;

  ctx.clearRect(0, 0, cw, ch);

  // Calculate the visible, unobstructed viewport (excluding left and right panels)
  // Left: 290px + 15px gap = 305px. Right: 340px + 15px gap = 355px. Total margins = 660px
  // Top: 60px + 15px gap + 15px pad = 90px. Bottom: 140px + 15px gap = 155px. Total margins = 245px
  const visibleWidth = window.innerWidth - 680; 
  const visibleHeight = window.innerHeight - 260; 

  const zoom = Math.min(visibleWidth / 19200, visibleHeight / 10800);

  // Calculate centering offsets to position the tank perfectly in the visible gap
  const offsetX = 305 + (visibleWidth - 19200 * zoom) / 2;
  const offsetY = 90 + (visibleHeight - 10800 * zoom) / 2;

  ctx.save();
  ctx.translate(offsetX * dpr, offsetY * dpr);
  ctx.scale(dpr * zoom, dpr * zoom);

  // 1. Draw glowing space deep gradient over the entire logical boundary
  ctx.save();
  const grad = ctx.createLinearGradient(19200 / 2, 0, 19200 / 2, 10800);
  grad.addColorStop(0, "#080c18");
  grad.addColorStop(0.35, "#060812");
  grad.addColorStop(1, "#030409");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, 19200, 10800);
  ctx.restore();

  // 2. Draw algae/food spores
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

  // 3. Draw swimming creatures
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

  // 4. Render shockwave crimson rings (BITE impact)
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

  // 5. Draw high-tech glowing glass tank boundary frame!
  ctx.strokeStyle = "rgba(0, 242, 254, 0.4)";
  ctx.lineWidth = 15;
  ctx.strokeRect(0, 0, 19200, 10800);

  ctx.restore(); // Restore high-level visible-space transform

  requestAnimationFrame(drawBetaSimulationFrame);
}

// Click selection on canvas
canvas.addEventListener("click", (e) => {
  const rect = canvas.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;

  const visibleWidth = window.innerWidth - 680;
  const visibleHeight = window.innerHeight - 260;
  const zoom = Math.min(visibleWidth / 19200, visibleHeight / 10800);

  const offsetX = 305 + (visibleWidth - 19200 * zoom) / 2;
  const offsetY = 90 + (visibleHeight - 10800 * zoom) / 2;
  
  const mx = (((e.clientX - rect.left) * (canvas.width / dpr / rect.width)) - offsetX) / zoom;
  const my = (((e.clientY - rect.top) * (canvas.height / dpr / rect.height)) - offsetY) / zoom;

  let closestAgent: any = null;
  let minDist = 80;

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
    // Click on empty water spawns food
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: "SPAWN_FOOD", x: mx, y: my }));
    }
  }
});

// Click list delegation for roster card select
speciesRoster.addEventListener("click", (e) => {
  const item = (e.target as HTMLElement).closest(".roster-card") as HTMLDivElement | null;
  if (!item) return;

  const recId = item.getAttribute("data-id");
  if (recId) {
    const agent = creatures.find(c => c.speciesId === recId);
    if (agent) {
      selectSpecimen(agent);
    } else {
      // Fetch fossil stats
      const record = speciesRosterSignal.value.find(r => r.id === recId);
      if (record) {
        selectedId.value = 99999; // temporary virtual ID for fossils
        selectedName.value = record.name;
        selectedTaxa.value = `${record.name} [FOSSIL - AUSGESTORBEN]`;
        selectedGenome.value = record.genome;
        selectedMethylations.value = Array(256).fill(0); // empty methylations for fossil view
        selectedMaxEnergy.value = 100;
        selectedStatus.value = "Ausgestorben (Fossil)";
        selectedEnergy.value = 0;
        selectedAge.value = 2700;
        brainContainer.innerHTML = `<div class="fallback-state">Spezies ist fossilisiert. Neuronale Aktivität erloschen.</div>`;
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
  
  // Establish web socket stream
  initBetaWebSocket();

  // Run graphic frame loop
  requestAnimationFrame(drawBetaSimulationFrame);
});