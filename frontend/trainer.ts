import { effect } from "@preact/signals-core";
import { CreatureRenderer } from './render/creatureRenderer';
import { currentView } from "./core/router";
import { safeInvoke, getPhenotype, phenotypeCache } from './api';
import { IconElite, IconTrophy, IconMutant, IconPlant, IconSuccess, IconActive } from './components/Icons';
import {
  selectedId, selectedName, selectedTaxa, selectedStatus, selectedEnergy,
  selectedMaxEnergy, selectedAge, selectedGenome,
  selectedMethylations, selectedPhenotype, selectedBrainActivations
} from "./signals";
import { drawProgressChart } from "./render/progressChart";
import {
  ProgressHistory, TRAINING_MODE_LABELS, formatFirstCatch, formatLevel, formatRate, modeChoice, pendingSwitchLabel,
  type TrainingModeChoice,
} from "./trainerMetrics";
import type { TrainerAction } from "../bindings/TrainerAction";
import type { TrainerGenerationCompleted } from "../bindings/TrainerGenerationCompleted";
import type { TrainerState } from "../bindings/TrainerState";
import type { TrainerTelemetrySandbox } from "../bindings/TrainerTelemetrySandbox";
import type { TrainerUiEvent } from "../bindings/TrainerUiEvent";

// Backing-store size of the mini-canvases: tiles show at ~130-160 CSS px, so 360 px covers DPR 2;
// a 1000 px store per tile made 16 tiles at 25 Hz GPU-bound and backed up the webview
const canvasWidth = 360;
const canvasHeight = 360;
// Minimum drawn size of spores and creatures in canvas px per world unit (the small room draws at 0.36)
const MIN_PX_PER_UNIT = 0.3;

// DOM Elements
const gridContainer = document.getElementById("sandbox-grid") as HTMLDivElement;
const statGen = document.getElementById("stat-gen") as HTMLSpanElement;
const statBestFit = document.getElementById("stat-best-fit") as HTMLSpanElement;
const statAvgFit = document.getElementById("stat-avg-fit") as HTMLSpanElement;
const statTimer = document.getElementById("stat-timer") as HTMLSpanElement;
const statSuccess = document.getElementById("stat-success") as HTMLSpanElement;
const statMedianFit = document.getElementById("stat-median-fit") as HTMLSpanElement;
const statFirstCatch = document.getElementById("stat-first-catch") as HTMLSpanElement;
const statLevel = document.getElementById("stat-level") as HTMLSpanElement;
const canvasProgress = document.getElementById("canvas-progress") as HTMLCanvasElement;

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
const chkLamarck = document.getElementById("chk-lamarck-syn") as HTMLInputElement;
const selectTrainerMode = document.getElementById("select-trainer-mode") as HTMLSelectElement;
const sliderZoomTiles = document.getElementById("slider-zoom-tiles") as HTMLInputElement;
const sliderLevel = document.getElementById("slider-level") as HTMLInputElement;

const lblGridSize = document.getElementById("lbl-grid-size") as HTMLSpanElement;
const lblSpeedup = document.getElementById("lbl-speedup") as HTMLSpanElement;
const lblEliteRatio = document.getElementById("lbl-elite-ratio") as HTMLSpanElement;
const lblMutation = document.getElementById("lbl-mutation-rate") as HTMLSpanElement;
const lblInflow = document.getElementById("lbl-inflow-rate") as HTMLSpanElement;
const lblHof = document.getElementById("lbl-hof-rate") as HTMLSpanElement;
const lblTrainerMode = document.getElementById("lbl-trainer-mode") as HTMLSpanElement;
const lblZoomTiles = document.getElementById("lbl-zoom-tiles") as HTMLSpanElement;
const lblLevel = document.getElementById("lbl-level") as HTMLSpanElement;
const lblPendingSwitch = document.getElementById("lbl-pending-switch") as HTMLSpanElement;

const txtDna = document.getElementById("txt-dna") as HTMLTextAreaElement;

// State Management
interface Sandbox {
  id: number;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  renderer: CreatureRenderer;
  lastTelemetry: TrainerTelemetrySandbox | null;
}

// Trainer actions are typed by the generated bindings (TCK-177)
function sendTrainerAction(action: TrainerAction) {
  return safeInvoke("handle_client_action", { action: JSON.stringify(action) }).catch((err) => {
    console.error(`Tauri invoke error ${action.type}:`, err);
  });
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

function syncSlidersFromBackend(data: TrainerState) {
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
  // `mutationRate` carries the max mutation operator calls per child since TCK-174 (field name kept)
  if (data.mutationOpsMin !== undefined) {
    mutationOpsMin = data.mutationOpsMin;
    sliderMutation.min = mutationOpsMin.toString();
  }
  if (data.mutationOpsLimit !== undefined) {
    sliderMutation.max = data.mutationOpsLimit.toString();
  }
  if (data.mutationRate !== undefined) {
    mutationOpsMax = data.mutationRate;
    sliderMutation.value = mutationOpsMax.toString();
    lblMutation.innerText = mutationOpsLabel(mutationOpsMax);
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
  if (data.lamarckian !== undefined) {
    isLamarckian = data.lamarckian;
    chkLamarck.checked = isLamarckian;
  }
  if (data.runId !== undefined) {
    if (data.runId !== runId) progress.clear();
    runId = data.runId;
  }
  syncTrainingMode(data);
}

// Training mode, difficulty level and a pending room switch (TCK-176); switches apply at the next generation
function syncTrainingMode(data: TrainerState) {
  const choice = modeChoice(data.trainingMode, data.levelAuto);
  selectTrainerMode.value = choice;
  lblTrainerMode.innerText = TRAINING_MODE_LABELS[choice];
  statLevel.innerText = formatLevel(data.level, data.trainingMode, data.levelAuto);
  statLevel.title = String(data.level);
  if (document.activeElement !== sliderLevel) {
    sliderLevel.value = Math.round(data.level * 100).toString();
    lblLevel.innerText = `${sliderLevel.value}%`;
  }
  lblPendingSwitch.innerText = pendingSwitchLabel(data.pendingMode);
}

// Progress curve of the current run; cleared on reset and run switch
const progress = new ProgressHistory();
function redrawProgress() {
  drawProgressChart(canvasProgress, progress.points);
}

function resetGenerationStats() {
  for (const el of [statSuccess, statMedianFit, statFirstCatch]) {
    el.innerText = "–";
    el.title = "";
  }
  progress.clear();
  redrawProgress();
}

// Report stats of a completed generation (same values as the headless report's `generations[]`); exact values in `title`
function showGenerationStats(data: TrainerGenerationCompleted) {
  const s = data.stats;
  statSuccess.innerText = formatRate(s.successRate);
  statSuccess.title = String(s.successRate);
  statMedianFit.innerText = s.medianFitness.toFixed(2);
  statMedianFit.title = String(s.medianFitness);
  statFirstCatch.innerText = formatFirstCatch(s.medianTimeToFirstCatch);
  statFirstCatch.title = s.medianTimeToFirstCatch === null ? "" : `${s.medianTimeToFirstCatch} ticks (mean ${s.meanTimeToFirstCatch})`;
  progress.add(s);
  redrawProgress();
}

function switchRun(id: string) {
  if (id !== runId) resetGenerationStats();
  runId = id;
}

let N = 16;
let warpSpeed = 1;
let eliteRatio = 0.25;
// Mutation operator calls per mutant / hall-of-fame child: uniform in [mutationOpsMin, mutationOpsMax] (TCK-174)
let mutationOpsMin = 1;
let mutationOpsMax = 3;
const mutationOpsLabel = (max: number) => (max > mutationOpsMin ? `${mutationOpsMin}–${max}` : `${max}`);
let inflowRate = 0.15;
let hofRate = 0.15;
let isMultiTrial = true;
let isLamarckian = false;
let runId = "default_run";
let currentGeneration = 1;

redrawProgress();

function getSandboxTaxaString(id: number, gen: number, pheno: any): string {
  const carn = pheno.carnivory;
  const dietLabel = carn >= 0.60 ? "Predator" : (carn < 0.40 ? "Herbivore" : "Omnivore");
  return `${pheno.latinName} (${dietLabel}) - Sandbox #${id} (Gen. ${gen})`;
}

function drawSandbox(sb: Sandbox) {
  const tele = sb.lastTelemetry;
  if (!tele) return;

  const chamberSize = tele.chamberSize || 1000.0;
  const scale = canvasWidth / chamberSize;

  const ctx = sb.ctx;
  ctx.save();
  ctx.clearRect(0, 0, canvasWidth, canvasHeight);

  // Apply scaling for the current chamber mode size
  ctx.scale(scale, scale);

  // 1. Draw beautiful radial gradient background (matching MainMenu aesthetic)
  const cx = chamberSize / 2.0;
  const cy = chamberSize / 2.0;
  const radGrad = ctx.createRadialGradient(cx, cy, 0, cx, cy, chamberSize * 0.7);
  radGrad.addColorStop(0, '#0b1329'); // Subtle center glow
  radGrad.addColorStop(1, '#020617'); // Dark slate edge
  ctx.fillStyle = radGrad;
  ctx.fillRect(0, 0, chamberSize, chamberSize);

  // 2. Draw beautifully scaling digital grid matching MainMenu
  ctx.strokeStyle = 'rgba(0, 242, 254, 0.025)'; // Subtle cyber cyan lines
  ctx.lineWidth = 1.5;
  ctx.beginPath();
  const step = 100;
  for (let x = 0; x <= chamberSize; x += step) {
    ctx.moveTo(x, 0);
    ctx.lineTo(x, chamberSize);
  }
  for (let y = 0; y <= chamberSize; y += step) {
    ctx.moveTo(0, y);
    ctx.lineTo(chamberSize, y);
  }
  ctx.stroke();

  // Large rooms shrink the world to ~0.1 px per unit on a 360 px tile; spores and the creature keep a minimum
  // on-screen size (MIN_PX_PER_UNIT) so they stay visible, the positions stay true to scale
  const markerZoom = Math.max(1, MIN_PX_PER_UNIT / scale);

  // Glow as a translucent halo instead of shadowBlur (a blur pass per spore is expensive on the GPU)
  tele.foods.forEach((spore: any) => {
    const color = spore.typeId === 2 ? '#ef4444' : '#10b981';
    ctx.fillStyle = color;
    ctx.globalAlpha = 0.25;
    ctx.beginPath();
    ctx.arc(spore.x, spore.y, 12 * markerZoom, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalAlpha = 1;
    ctx.beginPath();
    ctx.arc(spore.x, spore.y, 8 * markerZoom, 0, Math.PI * 2);
    ctx.fill();
  });

  let pheno = phenotypeCache.get(tele.genome);
  if (!pheno) {
    getPhenotype(tele.genome);
    ctx.restore();
    return;
  }

  ctx.translate(tele.px, tele.py);
  ctx.scale(markerZoom, markerZoom);
  ctx.translate(-tele.px, -tele.py);
  sb.renderer.render(
    pheno,
    Date.now() * 0.05,
    tele.px,
    tele.py,
    tele.heading_angle,
    tele.omega_rot
  );

  ctx.restore();
}

// Render loop (TCK-177): telemetry (25 Hz) only stores the latest snapshot and marks the sandbox dirty; while the
// trainer view is active a requestAnimationFrame loop draws the dirty sandboxes once per frame, so a slow GPU drops
// frames instead of queueing draws. Unchanged snapshots are not redrawn.
const dirtySandboxes = new Set<Sandbox>();
let renderLoopId = 0;
function scheduleDraw(sb: Sandbox) {
  dirtySandboxes.add(sb);
}
function renderFrame() {
  dirtySandboxes.forEach(drawSandbox);
  dirtySandboxes.clear();
  renderLoopId = requestAnimationFrame(renderFrame);
}
effect(() => {
  if (currentView.value === "trainer") {
    if (!renderLoopId) {
      sandboxes.forEach((sb) => dirtySandboxes.add(sb));
      renderLoopId = requestAnimationFrame(renderFrame);
    }
  } else if (renderLoopId) {
    cancelAnimationFrame(renderLoopId);
    renderLoopId = 0;
  }
});

async function rebuildSandboxGrid() {
  dirtySandboxes.clear();
  gridContainer.innerHTML = "";
  sandboxes = [];

  for (let idx = 0; idx < N; idx++) {
    const id = idx + 1;
    const box = document.createElement("div");
    box.className = `sandbox-card ${idx === selectedSandboxIdx ? "selected" : ""}`;
    box.id = `sandbox-card-${id}`;

    const label = document.createElement("div");
    label.className = "sandbox-meta";
    label.innerHTML = `<span>#${id}</span> <span id="sb-origin-lbl-${id}">${IconPlant} Random</span>`;

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

    sandboxes.push({
      id,
      canvas,
      ctx,
      renderer,
      lastTelemetry: null
    });

    canvas.addEventListener("click", () => {
      document.querySelectorAll(".sandbox-card").forEach(c => c.classList.remove("selected"));
      box.classList.add("selected");
      selectedSandboxIdx = idx;
      
      sendTrainerAction({ type: "SELECT_TRAINER_SANDBOX", id });
      
      const lastTele = sandboxes[selectedSandboxIdx]?.lastTelemetry;
      if (lastTele) {
        const fullPheno = phenotypeCache.get(lastTele.genome);
        if (fullPheno) {
          selectedId.value = id;
          selectedName.value = fullPheno.latinName;
          selectedTaxa.value = getSandboxTaxaString(id, currentGeneration, fullPheno);
          selectedStatus.value = lastTele.finished ? "🏁 SUCCESS" : "Active";
          selectedEnergy.value = lastTele.energy;
          selectedMaxEnergy.value = fullPheno.stomachCapacity;
          selectedAge.value = lastTele.age;
          selectedGenome.value = lastTele.genome;
          selectedMethylations.value = fullPheno.methylations;
          selectedPhenotype.value = fullPheno;
        } else {
          getPhenotype(lastTele.genome).then((p) => {
            if (p) {
              selectedId.value = id;
              selectedName.value = p.latinName;
              selectedTaxa.value = getSandboxTaxaString(id, currentGeneration, p);
              selectedStatus.value = lastTele.finished ? "🏁 SUCCESS" : "Active";
              selectedEnergy.value = lastTele.energy;
              selectedMaxEnergy.value = p.stomachCapacity;
              selectedAge.value = lastTele.age;
              selectedGenome.value = lastTele.genome;
              selectedMethylations.value = p.methylations;
              selectedPhenotype.value = p;
            }
          });
        }
      }
    });
  }

  pushHyperparamsToRust();
}

// `mutationRate` is sent only when the mutation slider itself changed: with the 1/5 rule the backend adapts the
// max ops per generation, and other sliders must not overwrite it with the slider's stale value
function pushHyperparamsToRust(includeMutation = false) {
  sendTrainerAction({
    type: "UPDATE_TRAINER_HYPERPARAMS",
    N,
    warpSpeed,
    eliteRatio,
    ...(includeMutation ? { mutationRate: mutationOpsMax } : {}),
    inflowRate,
    hofRate,
    multiTrial: isMultiTrial,
    isHeadless: false,
    runId,
    lamarckian: isLamarckian
  });
}

btnStart.addEventListener("click", () => {
  isRunning = !isRunning;
  updateStartButtonUI();
  
  if (isRunning) {
    sendTrainerAction({ type: "START_TRAINING" });
  } else {
    sendTrainerAction({ type: "PAUSE_TRAINING" });
  }
});

btnReset.addEventListener("click", async () => {
  const confirmReset = confirm("Are you sure you want to completely wipe the evolutionary trainer database and start from Gen 1?");
  if (confirmReset) {
    isRunning = false;
    updateStartButtonUI();
    sendTrainerAction({ type: "TRAINER_RESET" });
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
      label: `${run.run_id} (Gen ${run.max_gen}, F: ${run.max_fit.toFixed(2)})`
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

  const activeItem = listItems.find(item => item.id === runId);
  if (dropdownCurrentLabel && activeItem) {
    dropdownCurrentLabel.innerText = activeItem.label;
  }

  listItems.forEach(item => {
    const row = document.createElement("div");
    row.className = `training-item ${item.id === runId ? "active" : ""}`;
    
    const labelSpan = document.createElement("span");
    labelSpan.innerText = item.label;
    labelSpan.style.flex = "1";
    labelSpan.style.overflow = "hidden";
    labelSpan.style.textOverflow = "ellipsis";
    labelSpan.style.whiteSpace = "nowrap";

    labelSpan.addEventListener("click", () => {
      switchRun(item.id);
      dropdownTrigger.click();
      rebuildSandboxGrid();
      populateRunSelector();
    });

    row.appendChild(labelSpan);

    if (item.id !== "default_run") {
      const deleteSpan = document.createElement("span");
      deleteSpan.className = "training-delete-btn";
      deleteSpan.innerText = "✕";
      deleteSpan.title = "Delete this training session and clear history";

      deleteSpan.addEventListener("click", async (e) => {
        e.stopPropagation();
        const wasActive = item.id === runId;
        const confirmDelete = confirm(`Are you sure you want to permanently delete run '${item.id}'?`);
        if (confirmDelete) {
          try {
            await safeInvoke("clear_trainer_history", { runId: item.id });
          } catch (err) {
            console.error("[Trainer API] Error deleting training:", err);
          }

          if (wasActive) {
            switchRun("default_run");
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
    switchRun(name);
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
  lblMutation.innerText = mutationOpsLabel(parseInt(sliderMutation.value));
});
sliderMutation.addEventListener("change", () => {
  mutationOpsMax = parseInt(sliderMutation.value);
  pushHyperparamsToRust(true);
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

chkLamarck.addEventListener("change", () => {
  isLamarckian = chkLamarck.checked;
  pushHyperparamsToRust();
});

// The backend answers with TRAINER_STATE_CHANGED (mode, level, pending switch); "manual" sets the slider's level
selectTrainerMode.addEventListener("change", () => {
  const choice = selectTrainerMode.value as TrainingModeChoice;
  lblTrainerMode.innerText = TRAINING_MODE_LABELS[choice];
  if (choice === "manual") {
    sendTrainerAction({ type: "SET_TRAINER_LEVEL", level: parseInt(sliderLevel.value) / 100 });
  } else {
    sendTrainerAction({ type: "SET_TRAINER_MODE", mode: choice });
  }
});

sliderLevel.addEventListener("input", () => {
  lblLevel.innerText = `${sliderLevel.value}%`;
});
sliderLevel.addEventListener("change", () => {
  sendTrainerAction({ type: "SET_TRAINER_LEVEL", level: parseInt(sliderLevel.value) / 100 });
  sliderLevel.blur();
});

sliderZoomTiles?.addEventListener("input", () => {
  const size = sliderZoomTiles.value;
  if (lblZoomTiles) {
    lblZoomTiles.innerText = `${size}px`;
  }
  if (gridContainer) {
    gridContainer.style.setProperty("--tile-size", `${size}px`);
  }
});

async function setupTauriListeners() {
  if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__) {
    const { listen } = await import("@tauri-apps/api/event");
    
    await listen<TrainerUiEvent | { type: "DATABASE_CHANGED" }>("simulation-state", (event) => {
      const data = event.payload;

      if (data.type === "TRAINER_TELEMETRY_TICK") {
        currentGeneration = data.generation;
        statGen.innerText = currentGeneration.toString();
        statTimer.innerText = data.timeStr;

        data.sandboxes.forEach((tele) => {
          const sb = sandboxes.find(s => s.id === tele.id);
          if (sb) {
            sb.lastTelemetry = tele;
            scheduleDraw(sb);

            const labelOrigin = document.getElementById(`sb-origin-lbl-${tele.id}`);
            if (labelOrigin) {
              if (tele.origin_type === "elite") {
                labelOrigin.innerHTML = `${IconElite} Elite Parent`;
                labelOrigin.style.color = "#fbbf24";
              } else if (tele.origin_type === "hof") {
                labelOrigin.innerHTML = `${IconTrophy} Hall of Fame`;
                labelOrigin.style.color = "#60a5fa";
              } else if (tele.origin_type === "mutant") {
                labelOrigin.innerHTML = `${IconMutant} Mutant Clone`;
                labelOrigin.style.color = "#c084fc";
              } else {
                labelOrigin.innerHTML = `${IconPlant} Fresh Random`;
                labelOrigin.style.color = "#34d399";
              }
            }

            const labelFit = document.getElementById(`sb-fit-lbl-${tele.id}`);
            if (labelFit) {
              labelFit.innerText = `F: ${tele.current_fitness.toFixed(2)}`;
            }

            const labelStatus = document.getElementById(`sb-status-lbl-${tele.id}`);
            if (labelStatus) {
              labelStatus.innerHTML = tele.finished ? `${IconSuccess} SUCCESS` : `${IconActive} Active`;
              labelStatus.style.color = tele.finished ? "#10b981" : "var(--text-muted)";
            }
          }
        });

        // Select the active sandbox and write its details to the global SSOT signals!
        const activeSb = sandboxes[selectedSandboxIdx];
        if (activeSb && activeSb.lastTelemetry) {
          const tele = activeSb.lastTelemetry;
          const pheno = phenotypeCache.get(tele.genome);
          if (pheno) {
            selectedId.value = activeSb.id;
            selectedName.value = pheno.latinName;
            selectedTaxa.value = getSandboxTaxaString(activeSb.id, currentGeneration, pheno);
            selectedStatus.value = tele.finished ? "🏁 SUCCESS" : "Active";
            selectedEnergy.value = tele.energy;
            selectedMaxEnergy.value = pheno.stomachCapacity;
            selectedAge.value = tele.age;
            selectedGenome.value = tele.genome;
            selectedMethylations.value = pheno.methylations;
            selectedPhenotype.value = pheno;

            if (data.selectedBrain && Number(data.selectedBrain.id) === Number(activeSb.id)) {
              selectedBrainActivations.value = data.selectedBrain.activations;
            }
          }
        }
      }
      else if (data.type === "TRAINER_STATE_CHANGED") {
        isRunning = data.isRunning;
        updateStartButtonUI();
        syncSlidersFromBackend(data);
      }
      else if (data.type === "TRAINER_GENERATION_COMPLETED") {
        statBestFit.innerText = data.bestFitness.toFixed(2);
        statAvgFit.innerText = data.avgFitness.toFixed(2);
        txtDna.value = data.bestGenome;
        mutationOpsMax = data.mutationOpsMax;
        sliderMutation.value = mutationOpsMax.toString();
        lblMutation.innerText = mutationOpsLabel(mutationOpsMax);
        showGenerationStats(data);
        populateRunSelector();
      }
      else if (data.type === "TRAINER_RESET_COMPLETED") {
        statGen.innerText = "1";
        statBestFit.innerText = "0.00";
        statAvgFit.innerText = "0.00";
        statTimer.innerText = "5.0s";
        txtDna.value = "";
        resetGenerationStats();
        isRunning = false;
        updateStartButtonUI();
        syncSlidersFromBackend(data);
        rebuildSandboxGrid();
        populateRunSelector();
      }
      else if (data.type === "DATABASE_CHANGED") {
        populateRunSelector();
      }
    });
  }
}

setupTauriListeners().then(() => {
  rebuildSandboxGrid().then(() => {
    populateRunSelector();
  });
});

window.addEventListener("unload", () => {
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "PAUSE_TRAINING" }) }).catch(() => {});
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_MODE", mode: "ocean" }) }).catch(() => {});
});
