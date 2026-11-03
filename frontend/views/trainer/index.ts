// Training Room view (TCK-160; formerly frontend/trainer.ts + views/TrainerView.ts): run selector, hyperparameter
// controls, sandbox tile grid, progress chart and inspector. Element references come from `buildTrainerLayout`;
// the tile render loop, the dropdown's document listener and the inspector effects live in the mount scope.
import type { TrainerGenerationCompleted } from "../../../bindings/TrainerGenerationCompleted";
import type { TrainerState } from "../../../bindings/TrainerState";
import type { TrainerTelemetrySandbox } from "../../../bindings/TrainerTelemetrySandbox";
import type { UiEvent } from "../../../bindings/UiEvent";
import { CataloguePicker } from "../../components/CataloguePicker";
import { DiagnosticsPanel } from "../../components/DiagnosticsPanel";
import { IconActive, IconCatalogue, IconElite, IconMutant, IconPlant, IconSuccess, IconTrophy } from "../../components/Icons";
import { checkboxRow, openModal } from "../../components/Modal";
import { setMode } from "../../core/ipc";
import { peekPhenotype } from "../../core/phenotypes";
import type { Scope } from "../../core/scope";
import type { ViewContext, ViewModule } from "../../core/view";
import { CreatureRenderer } from "../../render/creatureRenderer";
import { drawProgressChart } from "../../render/progressChart";
import { drawChamber, drawSpores } from "../../render/worldLayer";
import { sliderToWarp, warpLabel, warpToSlider } from "../../shared/warp";
import { fromSandbox, sandboxVitals, sameRef, type Selection } from "../../state/selection";
import { buildTrainerLayout, type TrainerLayout } from "./layout";
import {
  POPULATION_STEPS, ProgressHistory, TRAINING_MODE_LABELS, formatFirstCatch, formatLevel, formatRate, modeChoice,
  pendingSwitchLabel, populationStepIndex, visibleSandboxRange, type TrainingModeChoice,
} from "./metrics";
import { TrainerSettings } from "./settings";

// Backing-store size of the tile canvases: tiles show at ~130-160 CSS px, so 360 px covers DPR 2; a 1000 px store
// per tile made 16 tiles at 25 Hz GPU-bound and backed up the webview
const TILE_SIZE = 360;
// Minimum drawn size of spores and creatures in canvas px per world unit (the small room draws at 0.36)
const MIN_PX_PER_UNIT = 0.3;

interface Tile {
  /** Sandbox id shown in this tile (known from the first telemetry frame; slot 0 is always id 1) */
  id: number;
  card: HTMLElement;
  ctx: CanvasRenderingContext2D | null;
  renderer: CreatureRenderer | null;
  idLabel: HTMLElement;
  originLabel: HTMLElement;
  fitLabel: HTMLElement;
  statusLabel: HTMLElement;
  telemetry: TrainerTelemetrySandbox | null;
}

const ORIGIN_LABELS: Record<TrainerTelemetrySandbox["origin_type"], { icon: string; text: string; color: string }> = {
  elite: { icon: String(IconElite), text: "Elite Parent", color: "#fbbf24" },
  hof: { icon: String(IconTrophy), text: "Hall of Fame", color: "#60a5fa" },
  mutant: { icon: String(IconMutant), text: "Mutant Clone", color: "#c084fc" },
  random: { icon: String(IconPlant), text: "Fresh Random", color: "#34d399" },
};

class TrainerController {
  private readonly layout: TrainerLayout;
  private readonly settings = new TrainerSettings();
  private readonly progress = new ProgressHistory();
  private readonly panel: DiagnosticsPanel;
  private tiles: Tile[] = [];
  private readonly dirty = new Set<Tile>();
  private selectedIdx = 0;
  // Sandbox id the backend currently selects (brain view); follows the selected tile's sandbox (TCK-194)
  private selectedSentId = 1;
  private isRunning = false;
  private generation = 1;
  private mountScope: Scope | null = null;

  constructor(private readonly ctx: ViewContext) {
    this.layout = buildTrainerLayout();
    this.panel = new DiagnosticsPanel({
      prefix: "trainer",
      sources: ["trainer"],
      ipc: ctx.ipc,
      save: true,
      onAssign: (sel) => void this.openAssignDialog(sel),
    });
    this.layout.inspector.appendChild(this.panel.element);
    this.bindControls();
    this.redrawProgress();
    // Boot: build the grid from the defaults without sending them; the backend state arrives when the trainer mode
    // is entered (TRAINER_STATE_CHANGED) and only then may the UI send hyperparameters (TCK-160)
    this.rebuildGrid(false);
    void this.populateRunSelector();

    // The trainer only resets the mode to Ocean on unload; it never sets the trainer mode itself (invariant 1)
    ctx.scope.listen(window, "unload", () => {
      void ctx.ipc.send({ type: "PAUSE_TRAINING" });
      void setMode("ocean");
    });
  }

  get root(): HTMLElement {
    return this.layout.root;
  }

  mount(scope: Scope) {
    this.mountScope = scope;
    const l = this.layout;
    // Close the run dropdown on any click outside its trigger
    scope.listen(document, "click", () => {
      l.runList.style.display = "none";
    });
    // Tile render loop (TCK-177): telemetry only stores the latest snapshot and marks the tile dirty; each frame
    // draws the dirty tiles once, so a slow GPU drops frames instead of queueing draws
    this.tiles.forEach((t) => this.dirty.add(t));
    scope.frame(() => {
      this.dirty.forEach((t) => this.drawTile(t));
      this.dirty.clear();
    });
    this.panel.mount(scope);
    scope.add(() => {
      this.mountScope = null;
    });
  }

  /** Router: pause training whenever another view is shown. */
  async suspend() {
    await this.ctx.ipc.send({ type: "PAUSE_TRAINING" });
  }

  onEvent(data: UiEvent) {
    const l = this.layout;
    switch (data.type) {
      case "TRAINER_TELEMETRY_TICK":
        this.generation = data.generation;
        l.statGen.innerText = String(data.generation);
        l.statTimer.innerText = data.timeStr;
        data.sandboxes.forEach((tele, idx) => {
          const tile = this.tiles[idx];
          if (tile) this.updateTile(tile, tele);
        });
        this.followSelectedTile(data.selectedBrain);
        break;
      case "TRAINER_STATE_CHANGED":
        this.setRunning(data.isRunning);
        this.applyState(data);
        break;
      case "TRAINER_GENERATION_COMPLETED":
        l.statBestFit.innerText = data.bestFitness.toFixed(2);
        l.statAvgFit.innerText = data.avgFitness.toFixed(2);
        l.txtDna.value = data.bestGenome;
        this.settings.mutationOpsMax = data.mutationOpsMax;
        l.sliderMutation.value = String(data.mutationOpsMax);
        l.lblMutation.innerText = this.settings.mutationOpsLabel();
        this.showGenerationStats(data);
        void this.populateRunSelector();
        break;
      case "TRAINER_RESET_COMPLETED":
        l.statGen.innerText = "1";
        l.statBestFit.innerText = "0.00";
        l.statAvgFit.innerText = "0.00";
        l.statTimer.innerText = "5.0s";
        l.txtDna.value = "";
        this.resetGenerationStats();
        this.setRunning(false);
        this.applyState(data);
        // The grid follows the backend state: pushing would send the reset-time values back and overwrite a slider
        // change made while this event was in flight (TCK-157)
        this.rebuildGrid(false);
        void this.populateRunSelector();
        break;
      case "DATABASE_CHANGED":
        void this.populateRunSelector();
        break;
    }
  }

  // --- backend state ---

  private applyState(state: TrainerState) {
    const l = this.layout;
    const s = this.settings;
    if (state.runId !== s.runId) this.progress.clear();
    const gridChanged = s.apply(state);
    this.syncPopulationControls();
    l.sliderGridSize.value = String(s.N);
    l.lblGridSize.innerText = String(s.N);
    l.sliderSpeedup.value = String(warpToSlider(s.warpSpeed));
    l.lblSpeedup.innerText = warpLabel(warpToSlider(s.warpSpeed));
    l.sliderEliteRatio.value = String(Math.round(s.eliteRatio * 100));
    l.lblEliteRatio.innerText = `${Math.round(s.eliteRatio * 100)}%`;
    l.sliderMutation.min = String(s.mutationOpsMin);
    l.sliderMutation.max = String(s.mutationOpsLimit);
    l.sliderMutation.value = String(s.mutationOpsMax);
    l.lblMutation.innerText = s.mutationOpsLabel();
    l.sliderInflow.value = String(Math.round(s.inflowRate * 100));
    l.lblInflow.innerText = `${Math.round(s.inflowRate * 100)}%`;
    l.sliderHof.value = String(Math.round(s.hofRate * 100));
    l.lblHof.innerText = `${Math.round(s.hofRate * 100)}%`;
    l.chkMultiTrial.checked = s.multiTrial;
    l.chkLamarck.checked = s.lamarckian;
    // Training mode, difficulty level and a pending room switch (TCK-176); switches apply at the next generation
    const choice = modeChoice(state.trainingMode, state.levelAuto);
    l.selectMode.value = choice;
    l.lblMode.innerText = TRAINING_MODE_LABELS[choice];
    l.statLevel.innerText = formatLevel(state.level, state.trainingMode, state.levelAuto);
    l.statLevel.title = String(state.level);
    if (document.activeElement !== l.sliderLevel) {
      l.sliderLevel.value = String(Math.round(state.level * 100));
      l.lblLevel.innerText = `${l.sliderLevel.value}%`;
    }
    l.lblPendingSwitch.innerText = pendingSwitchLabel(state.pendingMode);
    if (gridChanged) this.rebuildGrid(false);
  }

  private push(include: { mutation?: boolean; population?: boolean } = {}) {
    const action = this.settings.hyperparams(include);
    if (action) void this.ctx.ipc.send(action);
  }

  private setRunning(running: boolean) {
    this.isRunning = running;
    const b = this.layout.btnStart;
    b.innerText = running ? "Pause Training" : "Start Training";
    b.classList.toggle("btn-danger", running);
    b.classList.toggle("btn-primary", !running);
  }

  // Population slider (index into POPULATION_STEPS) and the visible-sandboxes range (at most the population)
  private syncPopulationControls() {
    const l = this.layout;
    const population = this.settings.population;
    l.sliderPopulation.value = String(populationStepIndex(population));
    l.lblPopulation.innerText = String(population);
    const range = visibleSandboxRange(population);
    l.sliderGridSize.min = String(range.min);
    l.sliderGridSize.max = String(range.max);
  }

  // --- generation stats ---

  private redrawProgress() {
    drawProgressChart(this.layout.canvasProgress, this.progress.points);
  }

  private resetGenerationStats() {
    const l = this.layout;
    for (const el of [l.statSuccess, l.statMedianFit, l.statFirstCatch]) {
      el.innerText = "–";
      el.title = "";
    }
    this.progress.clear();
    this.redrawProgress();
  }

  // Report stats of a completed generation (same values as the headless report's `generations[]`); exact values in `title`
  private showGenerationStats(data: TrainerGenerationCompleted) {
    const l = this.layout;
    const s = data.stats;
    l.statSuccess.innerText = formatRate(s.successRate);
    l.statSuccess.title = String(s.successRate);
    l.statMedianFit.innerText = s.medianFitness.toFixed(2);
    l.statMedianFit.title = String(s.medianFitness);
    l.statFirstCatch.innerText = formatFirstCatch(s.medianTimeToFirstCatch);
    l.statFirstCatch.title = s.medianTimeToFirstCatch === null ? "" : `${s.medianTimeToFirstCatch} ticks (mean ${s.meanTimeToFirstCatch})`;
    this.progress.add(s);
    this.redrawProgress();
  }

  // --- run selector ---

  private switchRun(id: string) {
    if (id !== this.settings.runId) this.resetGenerationStats();
    this.settings.runId = id;
    this.rebuildGrid(true);
    void this.populateRunSelector();
  }

  private async populateRunSelector() {
    let runs;
    try {
      runs = await this.ctx.ipc.call("get_trainer_runs");
    } catch {
      return; // reported by the IPC client
    }
    const l = this.layout;
    const runId = this.settings.runId;
    const items: { id: string; label: string }[] = runs.map((run) => ({ id: run.runId, label: `${run.runId} (Gen ${run.maxGeneration}, F: ${run.maxFitness.toFixed(2)})` }));
    if (!runs.some((r) => r.runId === runId) && runId !== "default_run") items.push({ id: runId, label: `${runId} (New)` });
    if (!runs.some((r) => r.runId === "default_run")) items.unshift({ id: "default_run", label: "default_run (New)" });

    const active = items.find((i) => i.id === runId);
    if (active) l.runCurrent.innerText = active.label;

    l.runList.replaceChildren(...items.map((item) => {
      const row = document.createElement("div");
      row.className = `training-item ${item.id === runId ? "active" : ""}`;
      const label = document.createElement("span");
      label.innerText = item.label;
      label.style.cssText = "flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;";
      label.addEventListener("click", () => {
        l.runList.style.display = "none";
        this.switchRun(item.id);
      });
      row.appendChild(label);
      if (item.id !== "default_run") {
        const del = document.createElement("span");
        del.className = "training-delete-btn";
        del.innerText = "✕";
        del.title = "Delete this training session and clear history";
        del.addEventListener("click", async (e) => {
          e.stopPropagation();
          const wasActive = item.id === this.settings.runId;
          if (!confirm(`Are you sure you want to permanently delete run '${item.id}'?`)) return;
          try {
            await this.ctx.ipc.call("clear_trainer_history", { runId: item.id });
          } catch {
            // reported by the IPC client (toast)
          }
          if (wasActive) this.switchRun("default_run");
          else void this.populateRunSelector();
        });
        row.appendChild(del);
      }
      return row;
    }));
  }

  // --- controls ---

  private bindControls() {
    const l = this.layout;
    const s = this.settings;
    l.btnBack.addEventListener("click", () => this.ctx.navigate("main-menu"));
    l.btnStart.addEventListener("click", () => {
      this.setRunning(!this.isRunning);
      void this.ctx.ipc.send(this.isRunning ? { type: "START_TRAINING" } : { type: "PAUSE_TRAINING" });
    });
    l.btnReset.addEventListener("click", () => {
      if (confirm("Are you sure you want to completely wipe the evolutionary trainer database and start from Gen 1?")) {
        this.setRunning(false);
        void this.ctx.ipc.send({ type: "TRAINER_RESET" });
      }
    });
    l.runTrigger.addEventListener("click", (e) => {
      e.stopPropagation();
      l.runList.style.display = l.runList.style.display === "flex" ? "none" : "flex";
    });
    l.btnCreateRun.addEventListener("click", () => {
      const name = l.txtNewRun.value.trim().replace(/[^a-zA-Z0-9_-]/g, "_");
      if (name.length > 0) {
        l.txtNewRun.value = "";
        this.switchRun(name);
      }
    });
    l.btnCopyDna.addEventListener("click", () => {
      if (!l.txtDna.value.trim()) return;
      void navigator.clipboard.writeText(l.txtDna.value);
      l.btnCopyDna.innerText = "📋 DNA Copied!";
      l.btnCopyDna.style.background = "#10b981";
      setTimeout(() => {
        l.btnCopyDna.innerText = "Copy Champion DNA";
        l.btnCopyDna.style.background = "";
      }, 1500);
    });

    l.sliderGridSize.addEventListener("input", () => {
      l.lblGridSize.innerText = l.sliderGridSize.value;
    });
    l.sliderGridSize.addEventListener("change", () => {
      s.N = Math.min(parseInt(l.sliderGridSize.value), visibleSandboxRange(s.population).max);
      this.rebuildGrid(true);
    });
    l.sliderPopulation.addEventListener("input", () => {
      l.lblPopulation.innerText = String(POPULATION_STEPS[parseInt(l.sliderPopulation.value)]);
    });
    l.sliderPopulation.addEventListener("change", () => {
      s.population = POPULATION_STEPS[parseInt(l.sliderPopulation.value)];
      this.syncPopulationControls();
      s.N = Math.min(s.N, visibleSandboxRange(s.population).max);
      l.sliderGridSize.value = String(s.N);
      l.lblGridSize.innerText = String(s.N);
      this.rebuildGrid(true, true);
    });
    l.sliderSpeedup.addEventListener("input", () => {
      l.lblSpeedup.innerText = warpLabel(parseInt(l.sliderSpeedup.value));
    });
    l.sliderSpeedup.addEventListener("change", () => {
      s.warpSpeed = sliderToWarp(parseInt(l.sliderSpeedup.value));
      this.push();
    });
    const percent = (slider: HTMLInputElement, label: HTMLElement, set: (v: number) => void) => {
      slider.addEventListener("input", () => {
        label.innerText = `${slider.value}%`;
      });
      slider.addEventListener("change", () => {
        set(parseFloat(slider.value) / 100);
        this.push();
      });
    };
    percent(l.sliderEliteRatio, l.lblEliteRatio, (v) => { s.eliteRatio = v; });
    percent(l.sliderInflow, l.lblInflow, (v) => { s.inflowRate = v; });
    percent(l.sliderHof, l.lblHof, (v) => { s.hofRate = v; });
    l.sliderMutation.addEventListener("input", () => {
      l.lblMutation.innerText = s.mutationOpsLabel(parseInt(l.sliderMutation.value));
    });
    l.sliderMutation.addEventListener("change", () => {
      s.mutationOpsMax = parseInt(l.sliderMutation.value);
      this.push({ mutation: true });
    });
    l.chkMultiTrial.addEventListener("change", () => {
      s.multiTrial = l.chkMultiTrial.checked;
      this.push();
    });
    l.chkLamarck.addEventListener("change", () => {
      s.lamarckian = l.chkLamarck.checked;
      this.push();
    });
    // The backend answers with TRAINER_STATE_CHANGED (mode, level, pending switch); "manual" sets the slider's level
    l.selectMode.addEventListener("change", () => {
      const choice = l.selectMode.value as TrainingModeChoice;
      l.lblMode.innerText = TRAINING_MODE_LABELS[choice];
      void this.ctx.ipc.send(choice === "manual"
        ? { type: "SET_TRAINER_LEVEL", level: parseInt(l.sliderLevel.value) / 100 }
        : { type: "SET_TRAINER_MODE", mode: choice });
    });
    l.sliderLevel.addEventListener("input", () => {
      l.lblLevel.innerText = `${l.sliderLevel.value}%`;
    });
    l.sliderLevel.addEventListener("change", () => {
      void this.ctx.ipc.send({ type: "SET_TRAINER_LEVEL", level: parseInt(l.sliderLevel.value) / 100 });
      l.sliderLevel.blur();
    });
    l.sliderZoomTiles.addEventListener("input", () => {
      l.lblZoomTiles.innerText = `${l.sliderZoomTiles.value}px`;
      l.grid.style.setProperty("--tile-size", `${l.sliderZoomTiles.value}px`);
    });
  }

  // --- tiles ---

  private selectSandbox(id: number) {
    this.selectedSentId = id;
    void this.ctx.ipc.send({ type: "SELECT_TRAINER_SANDBOX", id });
  }

  /** Rebuilds the N tiles; `push` sends the hyperparameters afterwards (false when the grid follows a backend state). */
  private rebuildGrid(push: boolean, includePopulation = false) {
    const N = this.settings.N;
    this.dirty.clear();
    this.tiles = [];
    if (this.selectedIdx >= N) {
      // The selected sandbox is no longer displayed: select the first tile (the backend sends brain data only for
      // displayed sandboxes)
      this.selectedIdx = 0;
      if (this.settings.synced) this.selectSandbox(1);
    }
    const cards: HTMLElement[] = [];
    for (let idx = 0; idx < N; idx++) {
      const tile = this.createTile(idx);
      this.tiles.push(tile);
      cards.push(tile.card);
      if (this.mountScope) this.dirty.add(tile);
    }
    this.layout.grid.replaceChildren(...cards);
    if (push) this.push({ population: includePopulation });
  }

  private createTile(idx: number): Tile {
    const card = document.createElement("div");
    card.className = `sandbox-card ${idx === this.selectedIdx ? "selected" : ""}`;
    card.id = `sandbox-card-${idx}`;
    const span = (id: string, html: string) => {
      const el = document.createElement("span");
      el.id = id;
      el.innerHTML = html;
      return el;
    };
    // Tiles are keyed by their index; each shows the sandbox of the telemetry entry at that index (TCK-194)
    const idLabel = span(`sb-id-lbl-${idx}`, `#${idx + 1}`);
    const originLabel = span(`sb-origin-lbl-${idx}`, `${IconPlant} Random`);
    const fitLabel = span(`sb-fit-lbl-${idx}`, "F: 0.0");
    const statusLabel = span(`sb-status-lbl-${idx}`, "🏃 T1");
    const top = document.createElement("div");
    top.className = "sandbox-meta";
    top.append(idLabel, " ", originLabel);
    const canvas = document.createElement("canvas");
    canvas.width = TILE_SIZE;
    canvas.height = TILE_SIZE;
    const bottom = document.createElement("div");
    bottom.className = "sandbox-meta";
    bottom.append(fitLabel, " ", statusLabel);
    card.append(top, canvas, bottom);

    const ctx = canvas.getContext("2d");
    const tile: Tile = {
      id: idx + 1, card, ctx, renderer: ctx ? new CreatureRenderer(canvas) : null,
      idLabel, originLabel, fitLabel, statusLabel, telemetry: null,
    };
    canvas.addEventListener("click", () => {
      this.tiles.forEach((t) => t.card.classList.remove("selected"));
      card.classList.add("selected");
      this.selectedIdx = idx;
      this.selectSandbox(tile.id);
      this.syncSelection(tile, null);
    });
    return tile;
  }

  private updateTile(tile: Tile, tele: TrainerTelemetrySandbox) {
    tile.id = tele.id;
    tile.telemetry = tele;
    if (this.mountScope) this.dirty.add(tile);
    tile.idLabel.innerText = `#${tele.id}`;
    const origin = ORIGIN_LABELS[tele.origin_type] ?? ORIGIN_LABELS.random;
    tile.originLabel.innerHTML = `${origin.icon} ${origin.text}`;
    tile.originLabel.style.color = origin.color;
    tile.fitLabel.innerText = `F: ${tele.current_fitness.toFixed(2)}`;
    tile.statusLabel.innerHTML = tele.finished ? `${IconSuccess} SUCCESS` : `${IconActive} Active`;
    tile.statusLabel.style.color = tele.finished ? "#10b981" : "var(--text-muted)";
  }

  /** The selected tile drives the app selection; the backend selection follows it to a new sandbox (TCK-194). */
  private followSelectedTile(brain: { id: number; activations: number[] } | null) {
    const tile = this.tiles[this.selectedIdx];
    if (!tile?.telemetry) return;
    if (tile.id !== this.selectedSentId) this.selectSandbox(tile.id);
    this.syncSelection(tile, brain);
  }

  private syncSelection(tile: Tile, brain: { id: number; activations: number[] } | null) {
    const tele = tile.telemetry;
    if (!tele) return;
    const pheno = peekPhenotype(tele.genome);
    if (!pheno) return; // requested; the next telemetry frame selects it
    const store = this.ctx.selection;
    const cur: Selection | null = store.current;
    const ref = { source: "trainer" as const, sandboxId: tele.id };
    if (cur && sameRef(cur.ref, ref) && cur.genome === tele.genome) {
      store.updateVitals(ref, sandboxVitals(tele, pheno));
    } else {
      const { selection, vitals } = fromSandbox(tele, pheno, this.generation);
      store.select(selection, vitals);
    }
    if (brain && brain.id === tele.id) store.updateActivations(ref, brain.activations);
  }

  private drawTile(tile: Tile) {
    const tele = tile.telemetry;
    const ctx = tile.ctx;
    if (!tele || !ctx || !tile.renderer) return;
    const chamber = tele.chamberSize || 1000.0;
    const scale = TILE_SIZE / chamber;
    ctx.save();
    ctx.clearRect(0, 0, TILE_SIZE, TILE_SIZE);
    ctx.scale(scale, scale);
    drawChamber(ctx, chamber);
    // Large rooms shrink the world to ~0.1 px per unit on a 360 px tile; spores and the creature keep a minimum
    // on-screen size (MIN_PX_PER_UNIT) so they stay visible, the positions stay true to scale
    const markerZoom = Math.max(1, MIN_PX_PER_UNIT / scale);
    drawSpores(ctx, tele.foods, { kind: "halo", radius: 8 * markerZoom, haloRadius: 12 * markerZoom });
    // A missing phenotype is requested once (failures are retried after a delay, not every frame)
    const pheno = peekPhenotype(tele.genome);
    if (pheno) {
      ctx.translate(tele.px, tele.py);
      ctx.scale(markerZoom, markerZoom);
      ctx.translate(-tele.px, -tele.py);
      tile.renderer.render(pheno, Date.now() * 0.05, tele.px, tele.py, tele.heading_angle, tele.omega_rot);
    }
    ctx.restore();
  }

  // --- assign from catalogue ---

  private async openAssignDialog(sel: Selection) {
    const scope = this.mountScope;
    if (!scope || sel.ref.source !== "trainer") return;
    const sandboxId = sel.ref.sandboxId;
    let creatures;
    try {
      creatures = await this.ctx.ipc.call("get_catalogue_creatures");
    } catch {
      return; // reported by the IPC client
    }
    if (creatures.length === 0) {
      alert("Your creature catalogue is empty! Save creatures in the ocean or trainer first.");
      return;
    }
    const modalScope = scope.child();
    const picker = new CataloguePicker(creatures, modalScope);
    const imprint = checkboxRow("Load with learned synapse weights (pre-imprinting)");
    openModal(modalScope, {
      title: `Pre-populate Sandbox #${sandboxId}`,
      icon: IconCatalogue,
      accent: "59, 130, 246",
      text: `Select a saved individual from your catalogue to use it directly as a new founding father in Sandbox #${sandboxId}:`,
      body: [picker.element, imprint.element],
      actions: [
        { label: "Cancel", onClick: () => modalScope.dispose() },
        {
          label: "Assign",
          primary: true,
          onClick: async () => {
            const creature = picker.selected;
            const accepted = await this.ctx.ipc.send({
              type: "ASSIGN_SANDBOX_CREATURE",
              sandbox_id: sandboxId,
              genome: creature.genome,
              methylations: creature.methylations,
              synapse_weights: creature.synapseWeights,
              load_learned_synapses: imprint.input.checked,
            });
            // A rejected action is reported by the IPC client (toast); the dialog stays open
            if (!accepted) return false;
            modalScope.dispose();
            alert(`Species "${creature.name}" successfully assigned to Sandbox #${sandboxId}!`);
            return true;
          },
        },
      ],
    });
  }
}

let controller: TrainerController | null = null;

export const trainerView: ViewModule<"trainer"> = {
  id: "trainer",
  title: "BroodBattle – Training Room",
  backendMode: "trainer",
  create(ctx) {
    controller = new TrainerController(ctx);
    return controller.root;
  },
  mount(ctx) {
    controller!.mount(ctx.scope);
  },
  onEvent(event) {
    controller?.onEvent(event);
  },
  suspend: () => controller?.suspend() ?? Promise.resolve(),
};
