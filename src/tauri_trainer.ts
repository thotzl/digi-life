import { invoke } from "@tauri-apps/api/core";

function safeInvoke(cmd: string, args?: any): Promise<any> {
  try {
    if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__ && typeof invoke !== "undefined") {
      return invoke(cmd, args);
    }
  } catch (e) {
    console.error(`Error invoking command ${cmd}:`, e);
  }
  return Promise.resolve(null);
}

import { generateWorld, ProceduralWorld } from './shared/mapGenerator';
import { parseGenome } from './biology/dna';
import { CreatureRenderer } from './render/creatureRenderer';

// Dimensions for the mini-canvases
const canvasWidth = 1000;
const canvasHeight = 1000;

// DOM Elements
const gridContainer = document.getElementById("sandbox-grid") as HTMLDivElement;
const statGen = document.getElementById("stat-gen") as HTMLSpanElement;
const statBestFit = document.getElementById("stat-best-fit") as HTMLSpanElement;
const statAvgFit = document.getElementById("stat-avg-fit") as HTMLSpanElement;
const statTimer = document.getElementById("stat-timer") as HTMLSpanElement;

const btnStart = document.getElementById("btn-start") as HTMLButtonElement;
const btnReset = document.getElementById("btn-reset-train") as HTMLButtonElement;
const btnCopyDna = document.getElementById("btn-copy-dna") as HTMLButtonElement;

const sliderGridSize = document.getElementById("slider-grid-size") as HTMLInputElement;
const sliderSpeedup = document.getElementById("slider-speedup") as HTMLInputElement;
const sliderEliteRatio = document.getElementById("slider-elite-ratio") as HTMLInputElement;
const sliderMutation = document.getElementById("slider-mutation-rate") as HTMLInputElement;
const sliderInflow = document.getElementById("slider-inflow-rate") as HTMLInputElement;
const sliderHof = document.getElementById("slider-hof-rate") as HTMLInputElement;
const chkMultiTrial = document.getElementById("chk-multi-trial") as HTMLInputElement;

const lblGridSize = document.getElementById("lbl-grid-size") as HTMLSpanElement;
const lblSpeedup = document.getElementById("lbl-speedup") as HTMLSpanElement;
const lblEliteRatio = document.getElementById("lbl-elite-ratio") as HTMLSpanElement;
const lblMutation = document.getElementById("lbl-mutation-rate") as HTMLSpanElement;
const lblInflow = document.getElementById("lbl-inflow-rate") as HTMLSpanElement;
const lblHof = document.getElementById("lbl-hof-rate") as HTMLSpanElement;

const txtDna = document.getElementById("txt-dna") as HTMLTextAreaElement;

// Diagnostics inspector DOM targets
const focusMeta = document.getElementById("focus-meta") as HTMLParagraphElement;
const focusGenome = document.getElementById("focus-genome") as HTMLTextAreaElement;
const neuronMeta = document.getElementById("neuron-meta") as HTMLDivElement;
const inspectBrainContainer = document.getElementById("inspect-brain-container") as HTMLDivElement;

const diagCanvas = document.getElementById("diagnostics-preview-canvas") as HTMLCanvasElement;
const diagCtx = diagCanvas?.getContext('2d');
let diagRenderer: CreatureRenderer | null = null;
if (diagCanvas) {
  diagRenderer = new CreatureRenderer(diagCanvas);
}

// --------------------------------------------------------------------------
// State Management
// --------------------------------------------------------------------------
interface Sandbox {
  id: number;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  renderer: CreatureRenderer;
  world: ProceduralWorld;
  lastTelemetry: any | null;
}

let isRunning = false;
let sandboxes: Sandbox[] = [];
let selectedSandboxIdx = 0; // index in the sandboxes list (0-based)

function updateStartButtonUI() {
  if (isRunning) {
    btnStart.innerText = "Pause Training";
    btnStart.classList.add("btn-danger");
    btnStart.classList.remove("btn-primary");
  } else {
    btnStart.innerText = "Start Training";
    btnStart.classList.add("btn-primary");
    btnStart.classList.remove("btn-danger");
  }
}

function syncSlidersFromBackend(data: any) {
  if (data.N !== undefined) {
    N = data.N;
    sliderGridSize.value = N.toString();
    lblGridSize.innerText = N.toString();
  }
  if (data.warpSpeed !== undefined) {
    warpSpeed = data.warpSpeed;
    if (warpSpeed <= 10) {
      sliderSpeedup.value = warpSpeed.toString();
      lblSpeedup.innerText = `${warpSpeed}x`;
    } else if (warpSpeed === 50) {
      sliderSpeedup.value = "11";
      lblSpeedup.innerText = `50x`;
    } else {
      sliderSpeedup.value = "12";
      lblSpeedup.innerText = `🏎️ Unlimited`;
    }
  }
  if (data.eliteRatio !== undefined) {
    eliteRatio = data.eliteRatio;
    sliderEliteRatio.value = Math.round(eliteRatio * 100).toString();
    lblEliteRatio.innerText = `${Math.round(eliteRatio * 100)}%`;
  }
  if (data.mutationRate !== undefined) {
    genomeMutationRate = data.mutationRate;
    sliderMutation.value = Math.round(genomeMutationRate * 100).toString();
    lblMutation.innerText = `${Math.round(genomeMutationRate * 100)}%`;
  }
  if (data.inflowRate !== undefined) {
    inflowRate = data.inflowRate;
    sliderInflow.value = Math.round(inflowRate * 100).toString();
    lblInflow.innerText = `${Math.round(inflowRate * 100)}%`;
  }
  if (data.hofRate !== undefined) {
    hofRate = data.hofRate;
    sliderHof.value = Math.round(hofRate * 100).toString();
    lblHof.innerText = `${Math.round(hofRate * 100)}%`;
  }
  if (data.multiTrial !== undefined) {
    isMultiTrial = data.multiTrial;
    chkMultiTrial.checked = isMultiTrial;
  }
  if (data.runId !== undefined) {
    runId = data.runId;
  }
}

let N = 16;
let warpSpeed = 1;
let eliteRatio = 0.25;
let genomeMutationRate = 0.15;
let inflowRate = 0.10;
let hofRate = 0.10;
let isMultiTrial = false;
let runId = "default_run";
let currentGeneration = 1;

let hoveredNeuronId: number | null = null;
const brainSvgCache = new Map<string, SVGElement>();

// --------------------------------------------------------------------------
// Core Telemetry & Drawing Steps
// --------------------------------------------------------------------------
function drawSandbox(sb: Sandbox) {
  const tele = sb.lastTelemetry;
  if (!tele) return;

  const ctx = sb.ctx;
  ctx.fillStyle = '#020617';
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);

  // Draw obstacles
  sb.world.obstacles.forEach(obs => {
    ctx.beginPath();
    const sx = (obs.x / 19200) * canvasWidth;
    const sy = (obs.y / 10800) * canvasHeight;
    const sr = (obs.radius / 19200) * canvasWidth * 1.5;

    ctx.arc(sx, sy, sr, 0, Math.PI * 2);
    ctx.fillStyle = "#334155";
    ctx.fill();
  });

  // Draw Spores (green for plants, red/crimson for meat)
  tele.foods.forEach((spore: any) => {
    const isEaten = tele.finished && tele.consumed_spore_type === (spore.id === 9999 ? "meat" : "plant");
    if (!isEaten) {
      ctx.beginPath();
      ctx.arc(spore.x, spore.y, 8, 0, Math.PI * 2);
      ctx.fillStyle = spore.id === 9999 ? '#ef4444' : '#10b981';
      ctx.shadowBlur = 4;
      ctx.shadowColor = spore.id === 9999 ? '#ef4444' : '#10b981';
      ctx.fill();
      ctx.shadowBlur = 0;
    }
  });

  // Compile full phenotype locally on demand to support rendering
  const pheno = parseGenome(tele.genome);

  // Draw Agent
  sb.renderer.render(
    pheno,
    Date.now() * 0.05,
    tele.px,
    tele.py,
    tele.heading_angle,
    tele.omega_rot
  );

  // Draw centered high-zoom live preview in the diagnostics sidebar if selected
  if (tele.id === (selectedSandboxIdx + 1) && diagCtx && diagRenderer) {
    diagCtx.fillStyle = '#020617';
    diagCtx.fillRect(0, 0, 100, 100);
    
    // Zoomed-in preview render (scale canvas context manually to fit nicely)
    diagCtx.save();
    diagCtx.scale(1.2, 1.2); // zoom in slightly
    diagRenderer.render(
      pheno,
      Date.now() * 0.03,
      42, // adjusted center X
      42, // adjusted center Y
      tele.heading_angle,
      tele.omega_rot
    );
    diagCtx.restore();
  }
}

// --------------------------------------------------------------------------
// Sandbox Initializations
// --------------------------------------------------------------------------
async function rebuildSandboxGrid() {
  gridContainer.innerHTML = "";
  sandboxes = [];

  for (let idx = 0; idx < N; idx++) {
    const id = idx + 1;
    const box = document.createElement("div");
    box.className = `sandbox-card ${idx === selectedSandboxIdx ? "selected" : ""}`;
    box.id = `sandbox-card-${id}`;

    const label = document.createElement("div");
    label.className = "sandbox-meta";
    label.innerHTML = `<span>#${id}</span> <span id="sb-origin-lbl-${id}">🌱 Random</span>`;

    const canvas = document.createElement("canvas");
    canvas.width = canvasWidth;
    canvas.height = canvasHeight;

    const fitEl = document.createElement("div");
    fitEl.className = "sandbox-meta";
    fitEl.innerHTML = `<span id="sb-fit-lbl-${id}">F: 0.0</span> <span id="sb-status-lbl-${id}">🏃 T1</span>`;

    box.appendChild(label);
    box.appendChild(canvas);
    box.appendChild(fitEl);
    gridContainer.appendChild(box);

    const ctx = canvas.getContext('2d')!;
    const renderer = new CreatureRenderer(canvas);
    const world = generateWorld("SANDBOX_SEED_" + id + "_GEN_" + currentGeneration);

    sandboxes.push({
      id,
      canvas,
      ctx,
      renderer,
      world,
      lastTelemetry: null
    });

    // Make canvas selectable on click
    canvas.addEventListener("click", () => {
      document.querySelectorAll(".sandbox-card").forEach(c => c.classList.remove("selected"));
      box.classList.add("selected");
      selectedSandboxIdx = idx;
      
      // Notify Rust of selected diagnostic sandbox
      safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SELECT_TRAINER_SANDBOX", id }) }).catch(() => {});
      
      // Render brain SVG directed graph
      if (sandboxes[selectedSandboxIdx]?.lastTelemetry) {
        const fullPheno = parseGenome(sandboxes[selectedSandboxIdx].lastTelemetry.genome);
        compileBrainSVG(fullPheno);
      }
    });
  }

  // Push updated hyperparameters to Rust core
  pushHyperparamsToRust();
}

function pushHyperparamsToRust() {
  safeInvoke("handle_client_action", {
    action: JSON.stringify({
      type: "UPDATE_TRAINER_HYPERPARAMS",
      N,
      warpSpeed,
      eliteRatio,
      mutationRate: genomeMutationRate,
      inflowRate,
      hofRate,
      multiTrial: isMultiTrial,
      isHeadless: false,
      runId
    })
  }).catch(() => {});
}

// --------------------------------------------------------------------------
// Brain SVG Directed Graph Rendering
// --------------------------------------------------------------------------
function compileBrainSVG(pheno: any) {
  if (!inspectBrainContainer) return;

  const brain = pheno.brain;
  if (!brain || brain.neurons.length === 0) {
    inspectBrainContainer.innerHTML = `<div class="fallback-state">No genetically encoded CTRNN brain found.</div>`;
    return;
  }

  const K = pheno.organelles.length;
  brainSvgCache.clear();

  let svgContent = `<svg width="100%" height="100%" viewBox="0 0 310 240" style="background:#020617;">`;

  // Draw synapses
  brain.synapses.forEach((syn: any) => {
    const fromId = syn.fromNode;
    const toId = syn.toNode;

    const fromX = getNeuronX(fromId, K);
    const fromY = getNeuronY(fromId, K, brain.neurons.length);
    const toX = getNeuronX(toId, K);
    const toY = getNeuronY(toId, K, brain.neurons.length);

    const isExcitatory = syn.weight > 0;
    const strokeColor = isExcitatory ? "rgba(16, 185, 129, 0.28)" : "rgba(239, 68, 68, 0.28)";

    svgContent += `
      <line id="trainer-syn-${fromId}-${toId}" x1="${fromX}" y1="${fromY}" x2="${toX}" y2="${toY}"
            stroke="${strokeColor}" stroke-width="1.2" marker-end="url(#arrow)" />
    `;
  });

  // Draw neurons
  brain.neurons.forEach((n: any) => {
    const nx = getNeuronX(n.id, K);
    const ny = getNeuronY(n.id, K, brain.neurons.length);

    const isInput = n.type === "input";
    const isOutput = n.type === "output";
    const fill = isInput ? "#0ea5e9" : (isOutput ? "#c084fc" : "#94a3b8");
    const radius = isInput || isOutput ? 4.5 : 3.2;

    svgContent += `
      <circle id="trainer-node-${n.id}" cx="${nx}" cy="${ny}" r="${radius}" fill="${fill}"
              style="cursor:pointer; filter:drop-shadow(0 0 2px ${fill});" />
    `;
  });

  // Arrow markers definition
  svgContent += `
    <defs>
      <marker id="arrow" viewBox="0 0 10 10" refX="16" refY="5" markerWidth="4" markerHeight="4" orient="auto-start-reverse">
        <path d="M 0 1 L 10 5 L 0 9 z" fill="rgba(148, 163, 184, 0.4)" />
      </marker>
    </defs>
  </svg>`;

  inspectBrainContainer.innerHTML = svgContent;

  // Add event listeners for hover tooltips on neuron nodes
  brain.neurons.forEach((n: any) => {
    const id = `trainer-node-${n.id}`;
    const el = document.getElementById(id) as any;
    if (el) {
      brainSvgCache.set(id, el);
      
      el.addEventListener("mouseenter", () => {
        hoveredNeuronId = n.id;
        el.setAttribute("stroke", "#ffffff");
        el.setAttribute("stroke-width", "1.5");
      });
      el.addEventListener("mouseleave", () => {
        hoveredNeuronId = null;
        el.removeAttribute("stroke");
        el.removeAttribute("stroke-width");
        if (neuronMeta) {
          neuronMeta.innerHTML = `Hover a neuron node to see live telemetry...`;
        }
      });
    }
  });

  brain.synapses.forEach((syn: any) => {
    const id = `trainer-syn-${syn.fromNode}-${syn.toNode}`;
    const el = document.getElementById(id) as any;
    if (el) brainSvgCache.set(id, el);
  });
}

function getNeuronX(id: number, K: number): number {
  if (id <= K) return 25; // Left Column: Sensors
  if (id >= K + 1 && id <= K + 4) return 280; // Right Column: Motors
  return 150; // Center Column: Interneurons
}

function getNeuronY(id: number, K: number, totalNeurons: number): number {
  if (id <= K) {
    // Distribute sensors evenly
    const step = 200 / (K + 1);
    return 20 + (id + 1) * step;
  }
  if (id >= K + 1 && id <= K + 4) {
    // Distribute motors evenly
    const motorIdx = id - (K + 1);
    const step = 200 / 5;
    return 20 + (motorIdx + 1) * step;
  }
  // Distribute interneurons evenly
  const interIdx = id - K - 5;
  const numInter = totalNeurons - K - 5;
  const step = 200 / (numInter + 1);
  return 20 + (interIdx + 1) * step;
}

// --------------------------------------------------------------------------
// UI Interactivity Bindings
// --------------------------------------------------------------------------
btnStart.addEventListener("click", () => {
  isRunning = !isRunning; // Toggle state instantly!
  updateStartButtonUI();   // Redraw button visually instantly (0ms)!
  
  if (isRunning) {
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "START_TRAINING" }) }).catch(() => {});
  } else {
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "PAUSE_TRAINING" }) }).catch(() => {});
  }
});

btnReset.addEventListener("click", async () => {
  const confirmReset = confirm("Are you sure you want to completely wipe the evolutionary trainer database and start from Gen 1?");
  if (confirmReset) {
    isRunning = false;      // Stop visually instantly!
    updateStartButtonUI();   // Redraw button visually instantly!
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "TRAINER_RESET" }) }).catch(() => {});
  }
});

const trainingListContainer = document.getElementById("training-list-container") as HTMLDivElement;
const dropdownTrigger = document.getElementById("training-dropdown-trigger") as HTMLButtonElement;
const dropdownCurrentLabel = document.getElementById("training-dropdown-current") as HTMLSpanElement;
const txtNewTraining = document.getElementById("txt-new-training") as HTMLInputElement;
const btnCreateTraining = document.getElementById("btn-create-training") as HTMLButtonElement;

async function populateRunSelector() {
  const runs = await safeInvoke("get_trainer_runs");
  if (!runs) return;

  if (trainingListContainer) {
    trainingListContainer.innerHTML = "";
  }
  
  let defaultExists = false;
  let activeExistsInDb = false;

  const listItems: { id: string; label: string; isNew?: boolean }[] = [];

  runs.forEach((run: any) => {
    if (run.run_id === "default_run") defaultExists = true;
    if (run.run_id === runId) activeExistsInDb = true;

    listItems.push({
      id: run.run_id,
      label: `${run.run_id} (Gen ${run.max_gen}, F: ${run.max_fit.toFixed(0)})`
    });
  });

  if (!activeExistsInDb && runId !== "default_run") {
    listItems.push({
      id: runId,
      label: `${runId} (New)`,
      isNew: true
    });
  }

  if (!defaultExists) {
    listItems.unshift({
      id: "default_run",
      label: "default_run (New)"
    });
  }

  // Update trigger button label to reflect active selection
  const activeItem = listItems.find(item => item.id === runId);
  if (dropdownCurrentLabel && activeItem) {
    dropdownCurrentLabel.innerText = activeItem.label;
  }

  // Render each item as a clickable div with a delete button
  listItems.forEach(item => {
    const row = document.createElement("div");
    row.className = `training-item ${item.id === runId ? "active" : ""}`;
    
    // Label span (selects session on click)
    const labelSpan = document.createElement("span");
    labelSpan.innerText = item.label;
    labelSpan.style.flex = "1";
    labelSpan.style.overflow = "hidden";
    labelSpan.style.textOverflow = "ellipsis";
    labelSpan.style.whiteSpace = "nowrap";

    labelSpan.addEventListener("click", () => {
      runId = item.id;
      dropdownTrigger.click(); // Close list dropdown
      rebuildSandboxGrid();
      populateRunSelector();
    });

    row.appendChild(labelSpan);

    // Delete span (✕)
    if (item.id !== "default_run") {
      const deleteSpan = document.createElement("span");
      deleteSpan.className = "training-delete-btn";
      deleteSpan.innerText = "✕";
      deleteSpan.title = "Delete this training session and clear history";

      deleteSpan.addEventListener("click", async (e) => {
        e.stopPropagation(); // prevent select action
        const wasActive = item.id === runId;
        const confirmDelete = confirm(`Are you sure you want to permanently delete run '${item.id}'?`);
        if (confirmDelete) {
          try {
            await safeInvoke("clear_trainer_history", { runId: item.id });
          } catch (err) {
            console.error("[Trainer API] Error deleting training:", err);
          }

          if (wasActive) {
            runId = "default_run";
          }

          await rebuildSandboxGrid();
          await populateRunSelector();
        }
      });
      row.appendChild(deleteSpan);
    }

    if (trainingListContainer) {
      trainingListContainer.appendChild(row);
    }
  });
}

// Interactivity triggers for dropdown
dropdownTrigger.addEventListener("click", (e) => {
  e.stopPropagation();
  const isOpen = trainingListContainer.style.display === "flex";
  trainingListContainer.style.display = isOpen ? "none" : "flex";
});

document.addEventListener("click", () => {
  if (trainingListContainer) {
    trainingListContainer.style.display = "none";
  }
});

btnCreateTraining.addEventListener("click", () => {
  const name = txtNewTraining.value.trim().replace(/[^a-zA-Z0-9_-]/g, "_");
  if (name.length > 0) {
    runId = name;
    txtNewTraining.value = "";
    rebuildSandboxGrid();
    populateRunSelector();
  }
});

btnCopyDna.addEventListener("click", () => {
  if (txtDna.value.trim()) {
    navigator.clipboard.writeText(txtDna.value);
    btnCopyDna.innerText = "📋 DNA Copied!";
    btnCopyDna.style.background = "#10b981";
    setTimeout(() => {
      btnCopyDna.innerText = "Copy Champion DNA";
      btnCopyDna.style.background = "";
    }, 1500);
  }
});

// Slider inputs sync listeners (Buttery-smooth 60FPS: separation of 'input' and 'change'!)
sliderGridSize.addEventListener("input", () => {
  lblGridSize.innerText = sliderGridSize.value;
});
sliderGridSize.addEventListener("change", () => {
  N = parseInt(sliderGridSize.value);
  rebuildSandboxGrid();
});

sliderSpeedup.addEventListener("input", () => {
  const val = parseInt(sliderSpeedup.value);
  if (val <= 10) {
    lblSpeedup.innerText = `${val}x`;
  } else if (val === 11) {
    lblSpeedup.innerText = `50x`;
  } else {
    lblSpeedup.innerText = `🏎️ Unlimited`;
  }
});
sliderSpeedup.addEventListener("change", () => {
  const val = parseInt(sliderSpeedup.value);
  if (val <= 10) {
    warpSpeed = val;
  } else if (val === 11) {
    warpSpeed = 50;
  } else {
    warpSpeed = 9999;
  }
  pushHyperparamsToRust();
});

sliderEliteRatio.addEventListener("input", () => {
  lblEliteRatio.innerText = `${sliderEliteRatio.value}%`;
});
sliderEliteRatio.addEventListener("change", () => {
  eliteRatio = parseFloat(sliderEliteRatio.value) / 100;
  pushHyperparamsToRust();
});

sliderMutation.addEventListener("input", () => {
  lblMutation.innerText = `${sliderMutation.value}%`;
});
sliderMutation.addEventListener("change", () => {
  genomeMutationRate = parseFloat(sliderMutation.value) / 100;
  pushHyperparamsToRust();
});

sliderInflow.addEventListener("input", () => {
  lblInflow.innerText = `${sliderInflow.value}%`;
});
sliderInflow.addEventListener("change", () => {
  inflowRate = parseFloat(sliderInflow.value) / 100;
  pushHyperparamsToRust();
});

sliderHof.addEventListener("input", () => {
  lblHof.innerText = `${sliderHof.value}%`;
});
sliderHof.addEventListener("change", () => {
  hofRate = parseFloat(sliderHof.value) / 100;
  pushHyperparamsToRust();
});

chkMultiTrial.addEventListener("change", () => {
  isMultiTrial = chkMultiTrial.checked;
  pushHyperparamsToRust();
});

// --------------------------------------------------------------------------
// Webview Event Listeners (Receiving native telemetry ticks from Rust core)
// --------------------------------------------------------------------------
async function setupTauriListeners() {
  if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__) {
    const { listen } = await import("@tauri-apps/api/event");
    
    // Listen to real-time physical telemetry updates sent by Rust
    await listen("simulation-state", (event: any) => {
      const data = event.payload;

      if (data.type === "TRAINER_TELEMETRY_TICK") {
        currentGeneration = data.generation;
        statGen.innerText = currentGeneration.toString();
        statTimer.innerText = data.timeStr;

        // Render each sandbox canvas with its corresponding telemetry frame
        data.sandboxes.forEach((tele: any) => {
          const sb = sandboxes.find(s => s.id === tele.id);
          if (sb) {
            sb.lastTelemetry = tele;
            
            // Render sandbox visual output
            drawSandbox(sb);

            // Sync card label details
            const labelOrigin = document.getElementById(`sb-origin-lbl-${tele.id}`);
            if (labelOrigin) {
              if (tele.origin_type === "elite") {
                labelOrigin.innerHTML = "👑 Elite Parent";
                labelOrigin.style.color = "#fbbf24";
              } else if (tele.origin_type === "hof") {
                labelOrigin.innerHTML = "🏆 Hall of Fame";
                labelOrigin.style.color = "#60a5fa";
              } else if (tele.origin_type === "mutant") {
                labelOrigin.innerHTML = "🧬 Mutant Clone";
                labelOrigin.style.color = "#c084fc";
              } else {
                labelOrigin.innerHTML = "🌱 Fresh Random";
                labelOrigin.style.color = "#34d399";
              }
            }

            const labelFit = document.getElementById(`sb-fit-lbl-${tele.id}`);
            if (labelFit) {
              labelFit.innerText = `F: ${tele.current_fitness.toFixed(1)}`;
            }

            const labelStatus = document.getElementById(`sb-status-lbl-${tele.id}`);
            if (labelStatus) {
              labelStatus.innerText = tele.finished ? "🏁 SUCCESS" : `🏃 Active`;
              labelStatus.style.color = tele.finished ? "#10b981" : "var(--text-muted)";
            }
          }
        });

        // 3. Update focused directed brain directed graph in real-time
        if (data.selectedBrain) {
          const activations = data.selectedBrain.activations;
          const states = data.selectedBrain.states;
          const sb = sandboxes[selectedSandboxIdx];

          if (sb && sb.lastTelemetry && activations) {
            const pheno = parseGenome(sb.lastTelemetry.genome);
            const brain = pheno.brain;

            // Update node elements glows
            brain.neurons.forEach((n: any) => {
              const id = `trainer-node-${n.id}`;
              const el = brainSvgCache.get(id);
              if (el) {
                const rawAct = Math.max(0.0, Math.min(1.0, Math.abs(activations[n.id] || 0.0)));
                const isInput = n.type === "input";
                const isOutput = n.type === "output";
                const exponent = isInput ? 1.5 : 4.0;
                const act = Math.pow(rawAct, exponent);

                const colorGlow = isInput ? "#0ea5e9" : (isOutput ? "#c084fc" : "#e2e8f0");
                const fill = act > 0.35 ? colorGlow : "#111827";
                const radius = isInput || isOutput ? (act > 0.45 ? 6.5 : 4.5) : (act > 0.45 ? 5.0 : 3.2);

                el.setAttribute("fill", fill);
                el.setAttribute("r", radius.toString());
              }
            });

            // Update synapse elements strokes
            brain.synapses.forEach((syn: any) => {
              const id = `trainer-syn-${syn.fromNode}-${syn.toNode}`;
              const el = brainSvgCache.get(id);
              if (el) {
                const preVal = Math.max(0.0, Math.min(1.0, Math.abs(activations[syn.fromNode] || 0.0)));
                const act = Math.pow(preVal, 4.0);
                const isExcitatory = syn.weight > 0;
                const baseColor = isExcitatory ? "16, 185, 129" : "239, 68, 68";
                const opacity = act > 0.35 ? 0.95 : 0.28;
                el.setAttribute("stroke", `rgba(${baseColor}, ${opacity})`);
              }
            });

            // Update focus text metadata
            if (focusGenome) {
              focusGenome.value = sb.lastTelemetry.genome;
            }

            const seedStr = `SANDBOX_SEED_${sb.id}_GEN_${currentGeneration}`;
            focusMeta.innerHTML = `
              Sandbox: #${sb.id}<br/>
              Status: ${sb.lastTelemetry.finished ? "🏁 SUCCESS" : "🏃 TRAINING"}<br/>
              Fitness: ${sb.lastTelemetry.current_fitness.toFixed(1)}<br/>
              Diet: ${sb.lastTelemetry.consumed_spore_type === "meat" ? "Carnivore (meat)" : "Herbivore (plant)"}<br/>
              Seed: <span style="color: var(--primary-cyan); font-size: 0.58rem; word-break: break-all;">${seedStr}</span>
            `;

            // Hover tooltip mapping
            if (hoveredNeuronId !== null) {
              const K = pheno.organelles.length;
              const isInput = hoveredNeuronId <= K;
              const isOutput = hoveredNeuronId >= K + 1 && hoveredNeuronId <= K + 4;
              
              let baseDesc = `Neuron #${hoveredNeuronId}`;
              let mathFormula = "";
              let liveValues = "";

              if (isInput) {
                mathFormula = "f(x) = Identity (Bounded [0, 1])";
                const act = activations[hoveredNeuronId] || 0.0;
                liveValues = `<b>Activation (a):</b> ${act.toFixed(3)}`;
                if (hoveredNeuronId === K) {
                  baseDesc = `Input #${hoveredNeuronId}: Internal Clock`;
                } else {
                  baseDesc = `Input #${hoveredNeuronId}: Organelle #${hoveredNeuronId + 1}`;
                }
              } else {
                const neuron = brain.neurons[hoveredNeuronId];
                const state = states[hoveredNeuronId] || 0.0;
                const act = activations[hoveredNeuronId] || 0.0;
                
                const actType = (neuron && neuron.activationType) || "tanh";
                if (actType === "relu") mathFormula = "f(s) = max(0, s) [ReLU]";
                else if (actType === "sigmoid") mathFormula = "f(s) = 1 / (1 + e^-s) [Sigmoid]";
                else if (actType === "sin") mathFormula = "f(s) = sin(s) [-1.0 to 1.0] [Oscillatory]";
                else mathFormula = "f(s) = tanh(s) [-1.0 to 1.0] [Hyperbolic]";

                liveValues = `<b>Potential (s):</b> ${state.toFixed(3)}<br/><b>Activation (a):</b> ${act.toFixed(3)}`;
                
                if (isOutput) {
                  const outputIndex = hoveredNeuronId - (K + 1);
                  if (outputIndex === 0) baseDesc = `Output #${hoveredNeuronId}: Thrust`;
                  else if (outputIndex === 1) baseDesc = `Output #${hoveredNeuronId}: Flexion Steering`;
                  else if (outputIndex === 2) baseDesc = `Output #${hoveredNeuronId}: Biolum Flash`;
                  else baseDesc = `Output #${hoveredNeuronId}: Reserved Motor`;
                } else {
                  baseDesc = `Neuron #${hoveredNeuronId} (Interneuron #${hoveredNeuronId - K - 4})`;
                }
              }

              if (neuronMeta) {
                neuronMeta.innerHTML = `
                  <span style="color: #00f2fe; font-weight: bold;">${baseDesc}</span><br/>
                  <span style="color: var(--text-muted); font-size: 0.53rem;">Formula: ${mathFormula}</span><br/>
                  ${liveValues}
                `;
              }
            }
          }
        }
      }
      else if (data.type === "TRAINER_STATE_CHANGED") {
        isRunning = data.isRunning;
        updateStartButtonUI();
        syncSlidersFromBackend(data); // Sync sliders to absolute backend truth!
      }
      else if (data.type === "TRAINER_GENERATION_COMPLETED") {
        statBestFit.innerText = data.bestFitness.toFixed(1);
        statAvgFit.innerText = data.avgFitness.toFixed(1);
        txtDna.value = data.bestGenome;
        populateRunSelector(); // Refresh dropdown metadata on generation completion!
      }
      else if (data.type === "TRAINER_RESET_COMPLETED") {
        statGen.innerText = "1";
        statBestFit.innerText = "0.0";
        statAvgFit.innerText = "0.0";
        statTimer.innerText = "5.0s";
        txtDna.value = "";
        isRunning = false;
        updateStartButtonUI();
        syncSlidersFromBackend(data); // Sync sliders to reset backend truth!
        rebuildSandboxGrid(); // Rebuild DOM cards immediately on reset!
        populateRunSelector(); // Refresh dropdown metadata on reset!
      }
      else if (data.type === "DATABASE_CHANGED") {
        populateRunSelector(); // Refresh dropdown when database changes!
      }
    });
  }
}

// --------------------------------------------------------------------------
// Startup Boot Actions (Registering with Rust)
// --------------------------------------------------------------------------
setupTauriListeners().then(() => {
  // 1. Tell Rust to activate the trainer simulator
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "trainer" }) }).then(() => {
    rebuildSandboxGrid().then(() => {
      populateRunSelector();
    });
  });
});

// Pause training and restore background simulation on page leave
window.addEventListener("unload", () => {
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "PAUSE_TRAINING" }) }).catch(() => {});
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "ocean" }) }).catch(() => {});
});
