// Ocean Laboratory view (TCK-160; formerly frontend/ocean.ts + views/OceanView.ts): live world canvas, species
// roster, log terminal and inspector. Element references come from `buildOceanLayout`; global listeners, effects and
// the render loop live in the mount scope.
import { signal } from "@preact/signals-core";

import type { SpeciesEntry } from "../../../bindings/SpeciesEntry";
import type { UiEvent } from "../../../bindings/UiEvent";
import { CataloguePicker } from "../../components/CataloguePicker";
import { DiagnosticsPanel } from "../../components/DiagnosticsPanel";
import { IconCatalogue } from "../../components/Icons";
import { checkboxRow, openModal } from "../../components/Modal";
import { InteractiveCamera } from "../../core/camera";
import { escapeHtml } from "../../core/escape";
import { getPhenotype, phenotypeCache, phenotypeVersion } from "../../core/phenotypes";
import type { Scope } from "../../core/scope";
import type { ViewContext, ViewModule } from "../../core/view";
import { CreatureRenderer } from "../../render/creatureRenderer";
import { drawSpores, WorldLayer } from "../../render/worldLayer";
import { badgeColor, dietLabel } from "../../shared/creatureLabels";
import { sliderToWarp, warpLabel } from "../../shared/warp";
import {
  FOSSIL_VITALS, fromFossil, fromOceanCreature, oceanCreatureId, selection, type SelectionStore,
} from "../../state/selection";
import { buildOceanLayout, type OceanLayout } from "./layout";
import { ageFlashes, biteFlash, creatureAt, isOverCreature, OceanModel, type BiteFlash } from "./model";

const SPORE_RADIUS = 16;

class OceanController {
  private readonly layout: OceanLayout;
  private readonly model = new OceanModel();
  private readonly world = new WorldLayer();
  private readonly camera: InteractiveCamera;
  private readonly renderer: CreatureRenderer | null;
  private readonly ctx2d: CanvasRenderingContext2D | null;
  private readonly panel: DiagnosticsPanel;
  private readonly roster = signal<SpeciesEntry[]>([]);
  private readonly aliveOpen = signal(true);
  private readonly extinctOpen = signal(false);
  private flashes: BiteFlash[] = [];
  private running = true;
  /** The user's choice (initial backend state, then the toggle button); the router's pause does not change it */
  private wantsRunning = true;
  private mountScope: Scope | null = null;

  constructor(private readonly ctx: ViewContext) {
    this.layout = buildOceanLayout();
    const l = this.layout;
    this.ctx2d = l.canvas.getContext("2d");
    this.renderer = this.ctx2d ? new CreatureRenderer(l.canvas) : null;
    this.camera = new InteractiveCamera(l.canvas);
    this.panel = new DiagnosticsPanel({ prefix: "ocean", sources: ["ocean", "fossil"], ipc: ctx.ipc, save: true });
    l.inspector.appendChild(this.panel.element);

    // Element-local listeners live as long as the elements (created once)
    l.btnBack.addEventListener("click", () => ctx.navigate("main-menu"));
    l.btnToggleSim.addEventListener("click", () => {
      this.wantsRunning = !this.running;
      void ctx.ipc.send({ type: "TOGGLE_SIMULATION", running: !this.running });
    });
    l.sliderSpeed.addEventListener("input", () => {
      l.lblSpeed.innerText = warpLabel(parseInt(l.sliderSpeed.value));
    });
    l.sliderSpeed.addEventListener("change", () => {
      void ctx.ipc.send({ type: "SET_SIMULATION_SPEED", speed: sliderToWarp(parseInt(l.sliderSpeed.value)) });
    });
    l.btnInject.addEventListener("click", () => void ctx.ipc.send({ type: "INJECT_URZELLE" }));
    l.btnLoadCatalogue.addEventListener("click", () => void this.openCatalogueDialog());
    l.btnReset.addEventListener("click", () => {
      void ctx.ipc.send({ type: "RESET_EVOLUTION" }).then((accepted) => {
        const source = ctx.selection.current?.ref.source;
        if (accepted && (source === "ocean" || source === "fossil")) ctx.selection.clear();
      });
    });
    l.btnClearLogs.addEventListener("click", () => {
      l.logs.innerHTML = "";
    });
    l.roster.addEventListener("click", (e) => this.onRosterClick(e));
    this.camera.setupEventListeners(
      (x, y) => isOverCreature(this.model.creatures, x, y),
      (x, y) => {
        // A click on empty water does nothing (the former SPAWN_FOOD action was never handled, RV-B09)
        const agent = creatureAt(this.model.creatures, x, y);
        if (agent) this.select(agent.id);
      },
    );

    this.log("Connection to Tauri evolution substrate established.", "system");
    this.fetchRoster();
  }

  get root(): HTMLElement {
    return this.layout.root;
  }

  mount(scope: Scope) {
    this.mountScope = scope;
    const canvas = this.layout.canvas;
    if (canvas.width !== window.innerWidth || canvas.height !== window.innerHeight) this.resize();
    scope.listen(window, "resize", () => this.resize());
    scope.listen(window, "keydown", (e) => {
      if (e.key.toLowerCase() === "r" && !(e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement)) {
        this.resetCamera();
        this.log("Camera view centered on full-screen glass tank and reset.", "system");
      }
    });
    scope.effect(() => this.renderRoster());
    scope.frame(() => this.drawFrame());
    this.panel.mount(scope);
    scope.add(() => {
      this.mountScope = null;
      this.flashes = [];
    });
  }

  /** Router: pause the Ocean while another view is shown. */
  async suspend() {
    if (this.running) await this.ctx.ipc.send({ type: "TOGGLE_SIMULATION", running: false });
  }

  /** Router: entering the Ocean resumes it if the user had it running (it is not paused by entering, TCK-160). */
  async resume() {
    if (this.wantsRunning && !this.running) await this.ctx.ipc.send({ type: "TOGGLE_SIMULATION", running: true });
  }

  onEvent(data: UiEvent) {
    const l = this.layout;
    switch (data.type) {
      case "INIT_STATE":
        this.model.applyInit(data);
        this.setRunning(data.running);
        this.wantsRunning = data.running;
        this.world.setWorld(data.world);
        this.camera.setWorldSize(data.world.width, data.world.height);
        this.resetCamera();
        this.fetchRoster();
        if (this.ctx.selection.current === null && this.model.creatures.length > 0) this.select(this.model.creatures[0].id);
        break;
      case "SIM_STATE":
        this.setRunning(data.running);
        break;
      case "TELEMETRY_TICK": {
        this.model.applyTelemetry(data);
        this.updateSelectedVitals(data.selectedBrain);
        l.statPopulation.innerText = `${this.model.creatures.length}`;
        l.statGeneration.innerText = `${this.model.highestGeneration}`;
        l.statSpores.innerText = `${this.model.foodPellets.length} Spores`;
        break;
      }
      case "FOOD_TICK":
        this.model.foodPellets = data.foodPellets;
        l.statSpores.innerText = `${this.model.foodPellets.length} Spores`;
        break;
      case "BITE_EVENT":
        if (this.mountScope) this.flashes.push(biteFlash(data));
        break;
      case "DATABASE_CHANGED":
        this.fetchRoster();
        break;
      case "LOG_EVENT":
        this.log(data.message, data.logType);
        break;
    }
  }

  private updateSelectedVitals(brain: { id: number; activations: number[] } | null) {
    const store = this.ctx.selection;
    const id = oceanCreatureId(store.current);
    if (id === null) return;
    const ref = { source: "ocean" as const, creatureId: id };
    const agent = this.model.find(id);
    if (agent) {
      store.updateVitals(ref, { status: "Alive", energy: agent.energy, maxEnergy: agent.phenotype.stomachCapacity, age: agent.age });
      if (brain && brain.id === id) store.updateActivations(ref, brain.activations);
    } else {
      store.updateVitals(ref, FOSSIL_VITALS);
    }
  }

  private setRunning(running: boolean) {
    this.running = running;
    const b = this.layout.btnToggleSim;
    b.innerText = running ? "Pause Substrate" : "Resume Substrate";
    b.style.background = running ? "rgba(0, 242, 254, 0.05)" : "rgba(16, 185, 129, 0.05)";
    b.style.color = running ? "#00f2fe" : "#10b981";
    b.style.borderColor = running ? "rgba(0, 242, 254, 0.2)" : "rgba(16, 185, 129, 0.2)";
  }

  private select(id: number) {
    const agent = this.model.find(id);
    if (!agent) return;
    const { selection: sel, vitals } = fromOceanCreature(agent);
    this.ctx.selection.select(sel, vitals);
    void this.ctx.ipc.send({ type: "SELECT_AGENT", id: agent.id });
  }

  private selectFossil(record: SpeciesEntry) {
    const store: SelectionStore = this.ctx.selection;
    const sel = fromFossil(record, phenotypeCache.get(record.genome) ?? null);
    store.select(sel, FOSSIL_VITALS);
    if (!sel.phenotype) {
      void getPhenotype(record.genome).then((p) => {
        if (p) store.setPhenotype(sel.ref, p);
      });
    }
  }

  private fetchRoster() {
    // Errors are reported by the IPC client (toast)
    this.ctx.ipc.call("get_registered_species").then((records) => {
      this.roster.value = records;
    }).catch(() => {});
  }

  private resize() {
    const canvas = this.layout.canvas;
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    this.resetCamera();
  }

  private resetCamera() {
    const { visibleWidth, visibleHeight } = this.camera.getViewportBounds();
    this.camera.reset(visibleWidth, visibleHeight);
  }

  private selectedSpeciesId(): string | null {
    const sel = selection.value;
    if (sel?.ref.source === "fossil") return sel.ref.speciesId;
    const id = oceanCreatureId(sel);
    return id === null ? null : this.model.find(id)?.speciesId ?? null;
  }

  private rosterCard(rec: SpeciesEntry, selectedSpecies: string | null): string {
    const alive = rec.status === "alive";
    const live = alive ? this.model.creatures.find((c) => c.speciesId === rec.id) : undefined;
    const pheno = live?.phenotype ?? phenotypeCache.get(rec.genome);
    if (!pheno) void getPhenotype(rec.genome);
    return `
      <div class="roster-card ${alive ? "alive" : "fossil"} ${rec.id === selectedSpecies ? "selected" : ""}"
           style="border-left: 4px solid ${badgeColor(pheno)};"
           data-id="${escapeHtml(rec.id)}"
           id="roster-card-${escapeHtml(rec.id)}">
        <div class="roster-title-row">
          <span class="roster-name" style="color: ${alive ? "var(--secondary-green)" : "var(--text-muted)"}">${escapeHtml(rec.name)}</span>
          <span class="badge-fossil ${alive ? "alive" : "fossil"}">${alive ? "Alive" : "Fossil"}</span>
        </div>
        <div class="roster-meta-row">
          <span>Diet: ${dietLabel(rec.carnivory)}</span>
          <span>Gen: ${rec.generation}</span>
        </div>
      </div>
    `;
  }

  private renderRoster() {
    const list = this.roster.value;
    const selectedSpecies = this.selectedSpeciesId();
    void phenotypeVersion.value; // re-render when a card colour becomes known
    const container = this.layout.roster;
    if (list.length === 0) {
      container.innerHTML = `<div class="loading-state">No species registered.</div>`;
      return;
    }
    const section = (title: string, color: string, open: boolean, id: string, records: SpeciesEntry[], empty: string, extraBorder: string) => `
      <div class="accordion-header ${open ? "expanded" : ""}" id="${id}" style="border-left: 3px solid ${color};${extraBorder} border-bottom: 1px solid rgba(148, 163, 184, 0.08);">
        <span>${title} (${records.length})</span>
        <span class="chevron">▼</span>
      </div>
      ${open ? `<div class="accordion-content">${records.length === 0 ? `<div class="roster-empty-state">${empty}</div>` : records.map((r) => this.rosterCard(r, selectedSpecies)).join("")}</div>` : ""}
    `;
    container.innerHTML =
      section("🟢 Living Species", "var(--secondary-green)", this.aliveOpen.value, "acc-alive-trigger", list.filter((r) => r.status === "alive"), "No living species in substrate. Inject cell or wait...", "") +
      section("💀 Extinct Species", "var(--text-muted)", this.extinctOpen.value, "acc-extinct-trigger", list.filter((r) => r.status === "extinct"), "No extinctions recorded in database history.", " border-top: 1px solid rgba(148, 163, 184, 0.08);");
  }

  private onRosterClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.closest("#acc-alive-trigger")) {
      this.aliveOpen.value = !this.aliveOpen.value;
      return;
    }
    if (target.closest("#acc-extinct-trigger")) {
      this.extinctOpen.value = !this.extinctOpen.value;
      return;
    }
    const recId = (target.closest(".roster-card") as HTMLElement | null)?.getAttribute("data-id");
    if (!recId) return;
    const agent = this.model.creatures.find((c) => c.speciesId === recId);
    if (agent) {
      this.select(agent.id);
      this.camera.camX = agent.px;
      this.camera.camY = agent.py;
      this.log(`Camera focused on specimen #${agent.id}.`, "system");
      return;
    }
    const record = this.roster.value.find((r) => r.id === recId);
    if (record) this.selectFossil(record);
  }

  /** "Load from Catalog": pick a catalogue creature and release clones of it. */
  private async openCatalogueDialog() {
    const scope = this.mountScope;
    if (!scope) return;
    let creatures;
    try {
      creatures = await this.ctx.ipc.call("get_catalogue_creatures");
    } catch {
      return; // reported by the IPC client
    }
    if (creatures.length === 0) {
      alert("Your creature catalogue is empty! Save elite creatures from the ocean or the trainer first.");
      return;
    }
    const modalScope = scope.child();
    const picker = new CataloguePicker(creatures, modalScope);
    const imprint = checkboxRow("Load with learned synaptic weights (imprinting)");
    const qty = document.createElement("input");
    qty.type = "number";
    qty.min = "1";
    qty.max = "50";
    qty.value = "1";
    qty.style.cssText = "width:50px; background:rgba(15,23,42,0.85); color:#00f2fe; font-family:monospace; font-size:0.75rem; font-weight:bold; border:1px solid rgba(0,242,254,0.3); border-radius:4px; padding:4px 6px; outline:none; text-align:center;";
    const qtyLabel = document.createElement("label");
    qtyLabel.style.cssText = "display:flex; align-items:center; gap:6px; font-size:0.65rem; color:#94a3b8; margin:0;";
    qtyLabel.append("Quantity:", qty);
    const config = document.createElement("div");
    config.style.cssText = "display:flex; align-items:center; justify-content:space-between; gap:12px; padding-top:4px;";
    config.append(imprint.element, qtyLabel);

    openModal(modalScope, {
      title: "Inject Species",
      icon: IconCatalogue,
      accent: "245, 158, 11",
      text: "Select a clone from your catalogue to release it live into the ocean:",
      body: [picker.element, config],
      actions: [
        { label: "Cancel", onClick: () => modalScope.dispose() },
        {
          label: "Release",
          primary: true,
          onClick: async () => {
            const count = Math.max(1, Math.min(50, parseInt(qty.value) || 1));
            try {
              for (let i = 0; i < count; i++) {
                await this.ctx.ipc.call("spawn_catalogue_creature_to_ocean", { id: picker.selected.id, loadLearnedSynapses: imprint.input.checked });
              }
            } catch {
              return false; // reported by the IPC client (toast); the dialog stays open
            }
            modalScope.dispose();
            return true;
          },
        },
      ],
    });
  }

  private drawFrame() {
    const ctx = this.ctx2d;
    const renderer = this.renderer;
    if (!ctx || !renderer) return;
    const canvas = this.layout.canvas;
    const camera = this.camera;
    ctx.fillStyle = "#020617";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.save();
    const { viewCenterX, viewCenterY } = camera.getViewportBounds();
    ctx.translate(viewCenterX, viewCenterY);
    ctx.scale(camera.camZoom, camera.camZoom);
    ctx.translate(-camera.camX, -camera.camY);

    this.world.draw(ctx);
    drawSpores(ctx, this.model.foodPellets, { kind: "glow", radius: SPORE_RADIUS });

    const selectedId = oceanCreatureId(selection.peek());
    const time = Date.now() * 0.05; // same animation clock as the trainer tiles
    for (const agent of this.model.creatures) {
      renderer.render(agent.phenotype, time, agent.px, agent.py, agent.headingAngle, agent.omegaRot);
      if (agent.id === selectedId) {
        ctx.beginPath();
        ctx.arc(agent.px, agent.py, agent.phenotype.spinalHarmonics.baseLength * 0.65, 0, Math.PI * 2);
        ctx.strokeStyle = "#00f2fe";
        ctx.lineWidth = 4.0;
        ctx.setLineDash([12, 12]);
        ctx.stroke();
        ctx.setLineDash([]);
      }
    }

    // Bite flashes at their world position, sized in screen pixels
    ageFlashes(this.flashes);
    for (const flash of this.flashes) {
      ctx.beginPath();
      ctx.arc(flash.x, flash.y, (flash.age * 5.0) / camera.camZoom, 0, Math.PI * 2);
      ctx.strokeStyle = `rgba(239, 68, 68, ${1.0 - flash.age / 15.0})`;
      ctx.lineWidth = 3 / camera.camZoom;
      ctx.stroke();
    }
    ctx.restore();

    ctx.fillStyle = "rgba(255, 255, 255, 0.15)";
    ctx.font = "10px monospace";
    ctx.fillText("🖱️ Drag to pan // Scroll to zoom", 20, canvas.height - 20);
  }

  private log(message: string, logType: string = "system") {
    const row = document.createElement("div");
    row.className = "log-row";
    const time = document.createElement("span");
    time.className = "log-time";
    time.innerText = new Date().toLocaleTimeString();
    const msg = document.createElement("span");
    msg.className = "log-msg";
    msg.innerText = message;
    row.append(time, msg);
    row.style.color = logType === "mutation" ? "#00f2fe" : logType === "extinction" ? "#64748b" : "#10b981";
    this.layout.logs.appendChild(row);
    this.layout.logs.scrollTop = this.layout.logs.scrollHeight;
  }
}

let controller: OceanController | null = null;

export const oceanView: ViewModule<"ocean"> = {
  id: "ocean",
  title: "BroodBattle – Ocean Laboratory",
  backendMode: "ocean",
  create(ctx) {
    controller = new OceanController(ctx);
    return controller.root;
  },
  mount(ctx) {
    controller!.mount(ctx.scope);
  },
  onEvent(event) {
    controller?.onEvent(event);
  },
  suspend: () => controller?.suspend() ?? Promise.resolve(),
  resume: () => controller?.resume() ?? Promise.resolve(),
};
