import { CreatureRenderer } from './render/creatureRenderer';
import { safeInvoke, getPhenotype, phenotypeCache } from './api';
import {
  selectedId, selectedName, selectedTaxa, selectedStatus, selectedEnergy,
  selectedMaxEnergy, selectedAdrenaline, selectedAge, selectedGenome,
  selectedMethylations, selectedPhenotype, selectedBrainActivations
} from "./signals";

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
const chkLamarck = document.getElementById("chk-lamarck-syn") as HTMLInputElement;
const selectTrainerMode = document.getElementById("select-trainer-mode") as HTMLSelectElement;
const sliderZoomTiles = document.getElementById("slider-zoom-tiles") as HTMLInputElement;

const lblGridSize = document.getElementById("lbl-grid-size") as HTMLSpanElement;
const lblSpeedup = document.getElementById("lbl-speedup") as HTMLSpanElement;
const lblEliteRatio = document.getElementById("lbl-elite-ratio") as HTMLSpanElement;
const lblMutation = document.getElementById("lbl-mutation-rate") as HTMLSpanElement;
const lblInflow = document.getElementById("lbl-inflow-rate") as HTMLSpanElement;
const lblHof = document.getElementById("lbl-hof-rate") as HTMLSpanElement;
const lblTrainerMode = document.getElementById("lbl-trainer-mode") as HTMLSpanElement;
const lblZoomTiles = document.getElementById("lbl-zoom-tiles") as HTMLSpanElement;

const txtDna = document.getElementById("txt-dna") as HTMLTextAreaElement;

// State Management
interface Sandbox {
  id: number;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  renderer: CreatureRenderer;
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
  if (data.lamarckian !== undefined) {
    isLamarckian = data.lamarckian;
    chkLamarck.checked = isLamarckian;
  }
  if (data.runId !== undefined) {
    runId = data.runId;
  }
}

let N = 16;
let warpSpeed = 1;
let eliteRatio = 0.15;
let genomeMutationRate = 0.06;
let inflowRate = 0.15;
let hofRate = 0.15;
let isMultiTrial = true;
let isLamarckian = false;
let runId = "default_run";
let currentGeneration = 1;

function getSandboxTaxaString(id: number, gen: number, pheno: any): string {
  const carn = pheno.carnivory;
  const dietLabel = carn >= 0.60 ? "Predator" : (carn < 0.40 ? "Herbivore" : "Omnivore");
  return `${pheno.latinName} (${dietLabel}) - Sandbox #${id} (Gen. ${gen})`;
}

function drawSandbox(sb: Sandbox) {
  const tele = sb.lastTelemetry;
  if (!tele) return;

  const chamberSize = tele.chamberSize || 1000.0;
  const scale = 1000.0 / chamberSize;

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

  tele.foods.forEach((spore: any) => {
    ctx.beginPath();
    ctx.arc(spore.x, spore.y, 8, 0, Math.PI * 2);
    const isMeat = spore.typeId === 2;
    ctx.fillStyle = isMeat ? '#ef4444' : '#10b981';
    ctx.shadowBlur = 4;
    ctx.shadowColor = isMeat ? '#ef4444' : '#10b981';
    ctx.fill();
    ctx.shadowBlur = 0;
  });

  let pheno = phenotypeCache.get(tele.genome);
  if (!pheno) {
    getPhenotype(tele.genome);
    ctx.restore();
    return;
  }

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
      
      safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SELECT_TRAINER_SANDBOX", id }) }).catch(() => {});
      
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
          selectedAdrenaline.value = lastTele.adrenaline || 1.0;
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
              selectedAdrenaline.value = lastTele.adrenaline || 1.0;
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
      runId,
      lamarckian: isLamarckian
    })
  }).catch(() => {});
}

btnStart.addEventListener("click", () => {
  isRunning = !isRunning;
  updateStartButtonUI();
  
  if (isRunning) {
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "START_TRAINING" }) }).catch(() => {});
  } else {
    safeInvoke("handle_client_action", { action: JSON.stringify({ type: "PAUSE_TRAINING" }) }).catch(() => {});
  }
});

btnReset.addEventListener("click", async () => {
  const confirmReset = confirm("Are you sure you want to completely wipe the evolutionary trainer database and start from Gen 1?");
  if (confirmReset) {
    isRunning = false;
    updateStartButtonUI();
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
      runId = item.id;
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

chkLamarck.addEventListener("change", () => {
  isLamarckian = chkLamarck.checked;
  pushHyperparamsToRust();
});

selectTrainerMode?.addEventListener("change", () => {
  const m = selectTrainerMode.value;
  if (lblTrainerMode) {
    lblTrainerMode.innerText = m === "exploration" ? "Erkundung" : "Standard";
  }
  safeInvoke("handle_client_action", { action: JSON.stringify({ type: "SET_TRAINER_MODE", mode: m }) })
    .catch(err => console.error("Tauri invoke error SET_TRAINER_MODE:", err));
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
    
    await listen("simulation-state", (event: any) => {
      const data = event.payload;

      if (data.type === "TRAINER_TELEMETRY_TICK") {
        currentGeneration = data.generation;
        statGen.innerText = currentGeneration.toString();
        statTimer.innerText = data.timeStr;

        data.sandboxes.forEach((tele: any) => {
          const sb = sandboxes.find(s => s.id === tele.id);
          if (sb) {
            sb.lastTelemetry = tele;
            drawSandbox(sb);

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
            selectedAdrenaline.value = tele.adrenaline || 1.0;
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
        statBestFit.innerText = data.bestFitness.toFixed(1);
        statAvgFit.innerText = data.avgFitness.toFixed(1);
        txtDna.value = data.bestGenome;
        populateRunSelector();
      }
      else if (data.type === "TRAINER_RESET_COMPLETED") {
        statGen.innerText = "1";
        statBestFit.innerText = "0.0";
        statAvgFit.innerText = "0.0";
        statTimer.innerText = "5.0s";
        txtDna.value = "";
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
