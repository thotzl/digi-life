// Creature Catalogue view (TCK-160): saved creatures with release / seed / rename / wipe and the inspector.
import type { CatalogueCreature } from "../../../bindings/CatalogueCreature";
import { DiagnosticsPanel } from "../../components/DiagnosticsPanel";
import { IconBack, IconCatalogue, IconRocket, IconWeights } from "../../components/Icons";
import { checkboxRow, openModal } from "../../components/Modal";
import type { CallError } from "../../core/ipc";
import { escapeHtml } from "../../core/escape";
import { getPhenotype, phenotypeCache, phenotypeVersion } from "../../core/phenotypes";
import type { Scope } from "../../core/scope";
import { styled } from "../../core/styled";
import type { ViewContext, ViewModule } from "../../core/view";
import { badgeColor, dietLabel } from "../../shared/creatureLabels";
import { CATALOGUE_VITALS, catalogueId, fromCatalogue, selection } from "../../state/selection";

// --- STYLED COMPONENTS ---

const CatalogueContainer = styled("div")`
  position: relative;
  width: 100vw;
  height: 100vh;
  background: radial-gradient(circle at center, #0b1528 0%, #020617 100%);
  color: #f1f5f9;
  font-family: monospace;
  box-sizing: border-box;
  overflow: hidden;
`;

const CatalogueGrid = styled("main")`
  display: grid;
  grid-template-columns: 1fr 360px;
  grid-gap: 15px;
  padding: 15px;
  box-sizing: border-box;
  height: 100%;
  width: 100%;
`;

const GlassPanel = styled("section")`
  background: rgba(15, 23, 42, 0.65);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid rgba(148, 163, 184, 0.12);
  border-radius: 8px;
  padding: 15px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
`;

const PanelHeader = styled("div")`
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  padding-bottom: 8px;
  margin-bottom: 12px;
  display: flex;
  justify-content: space-between;
  align-items: center;
`;

const PanelTitle = styled("h2")`
  font-size: 0.85rem;
  font-weight: bold;
  letter-spacing: 1px;
  color: #fff;
  margin: 0;
  text-transform: uppercase;
  display: flex;
  align-items: center;
`;

const PanelSubtitle = styled("span")`
  font-size: 0.6rem;
  color: #64748b;
  text-transform: uppercase;
  letter-spacing: 1.5px;
  margin-top: 2px;
  display: block;
`;

const BackToMenuButton = styled("button")`
  background: rgba(15, 23, 42, 0.7);
  border: 1px solid rgba(148, 163, 184, 0.15);
  border-radius: 4px;
  color: #94a3b8;
  padding: 4px 10px;
  font-family: monospace;
  font-size: 0.62rem;
  cursor: pointer;
  transition: all 0.2s;
  letter-spacing: 1px;
  text-transform: uppercase;
  display: flex;
  align-items: center;

  &:hover {
    color: #fff;
    border-color: #00f2fe;
    box-shadow: 0 0 10px rgba(0, 242, 254, 0.25);
  }
`;

const ScrollableList = styled("div")`
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 4px;

  &::-webkit-scrollbar {
    width: 4px;
  }
  &::-webkit-scrollbar-track {
    background: transparent;
  }
  &::-webkit-scrollbar-thumb {
    background: rgba(148, 163, 184, 0.15);
    border-radius: 2px;
  }
`;

const SpecimenCard = styled("div")`
  background: rgba(30, 41, 59, 0.15);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  padding: 8px 12px;
  cursor: pointer;
  transition: all 0.18s ease-in-out;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  position: relative;
  overflow: hidden;

  &:hover {
    background: rgba(15, 23, 42, 0.35);
    border-color: #f59e0b;
    box-shadow: 0 0 10px rgba(245, 158, 11, 0.04);
  }

  &.active {
    border-color: #f59e0b;
    background: rgba(245, 158, 11, 0.04);
    box-shadow: 0 0 12px rgba(245, 158, 11, 0.06);
  }
`;

const SpecimenGlowDot = styled("div")`
  width: 8px;
  height: 8px;
  border-radius: 50%;
  box-shadow: 0 0 8px currentColor;
  flex-shrink: 0;
`;

const SpecimenCardHeader = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 160px;
  flex-shrink: 0;

  h3 {
    margin: 0;
    font-size: 0.8rem;
    color: #fff;
    font-weight: bold;
  }

  span.source {
    font-size: 0.55rem;
    color: #64748b;
    text-transform: uppercase;
  }
`;

const SpecimenMetricsRow = styled("div")`
  display: flex;
  align-items: center;
  gap: 24px;
  font-size: 0.65rem;
  color: #94a3b8;
  flex: 1;
`;

const CardButtonGroup = styled("div")`
  display: flex;
  gap: 6px;
  flex-shrink: 0;
`;

const SmallCardButton = styled("button")`
  flex: 1;
  background: rgba(148, 163, 184, 0.04);
  border: 1px solid rgba(148, 163, 184, 0.15);
  color: #94a3b8;
  border-radius: 4px;
  padding: 4px;
  font-family: monospace;
  font-size: 0.58rem;
  cursor: pointer;
  transition: all 0.15s;
  font-weight: bold;

  &:hover {
    background: rgba(245, 158, 11, 0.08);
    border-color: #f59e0b;
    color: #fff;
  }

  &.btn-spawn {
    background: rgba(16, 185, 129, 0.05);
    border-color: rgba(16, 185, 129, 0.2);
    color: #10b981;

    &:hover {
      background: rgba(16, 185, 129, 0.15);
      border-color: #10b981;
      color: #fff;
      box-shadow: 0 0 10px rgba(16, 185, 129, 0.2);
    }
  }

  &.btn-delete {
    background: rgba(239, 68, 68, 0.03);
    border-color: rgba(239, 68, 68, 0.15);
    color: #ef4444;

    &:hover {
      background: rgba(239, 68, 68, 0.12);
      border-color: #ef4444;
      color: #fff;
      box-shadow: 0 0 10px rgba(239, 68, 68, 0.2);
    }
  }
`;

const Div = styled("div")``;
const H3 = styled("h3")``;
const Span = styled("span")``;

const RunSelect = styled("select")`
  width: 100%;
  background: rgba(15, 23, 42, 0.85);
  color: #00f2fe;
  font-family: monospace;
  font-size: 0.75rem;
  font-weight: bold;
  border: 1px solid rgba(0, 242, 254, 0.3);
  border-radius: 4px;
  padding: 8px;
  outline: none;
`;

class CatalogueController {
  readonly root: HTMLElement;
  private readonly listEl: HTMLElement;
  private readonly panel: DiagnosticsPanel;
  private creatures: CatalogueCreature[] = [];
  private cards = new Map<string, { card: HTMLElement; dot: HTMLElement; genome: string }>();
  private mountScope: Scope | null = null;

  constructor(private readonly ctx: ViewContext) {
    this.root = CatalogueContainer({ id: "catalogue-view-container" });
    const header = PanelHeader({
      children: [
        Div({ children: [PanelTitle({ children: [IconCatalogue, "Creature Catalogue"] }), PanelSubtitle({ children: "Persistent Cryo-Preservation Ledger" })] }),
        BackToMenuButton({
          onClick: () => {
            if (catalogueId(ctx.selection.current) !== null) ctx.selection.clear();
            ctx.navigate("main-menu");
          },
          children: [IconBack, "Main Menu"],
        }),
      ],
    });
    this.listEl = ScrollableList();
    const rightPanel = GlassPanel({ class: "panel-right" });
    this.panel = new DiagnosticsPanel({ prefix: "catalogue", sources: ["catalogue"], ipc: ctx.ipc });
    rightPanel.appendChild(this.panel.element);
    this.root.appendChild(CatalogueGrid({ children: [GlassPanel({ children: [header, this.listEl] }), rightPanel] }));
  }

  mount(scope: Scope) {
    this.mountScope = scope;
    void this.refreshList();
    // Highlight of the selected card and card colours once phenotypes are known
    scope.effect(() => {
      const selected = catalogueId(selection.value);
      phenotypeVersion.value;
      for (const [id, entry] of this.cards) {
        entry.card.classList.toggle("active", id === selected);
        const color = badgeColor(phenotypeCache.get(entry.genome));
        entry.dot.style.color = color;
      }
    });
    this.panel.mount(scope);
    scope.add(() => {
      this.mountScope = null;
    });
  }

  /** Reads the catalogue and renders the list of saved creatures. */
  private async refreshList() {
    this.listEl.innerHTML = `<div class="fallback-state">Sensing database ledger...</div>`;
    try {
      this.creatures = await this.ctx.ipc.call("get_catalogue_creatures");
    } catch (e) {
      const message = (e as CallError).message ?? String(e);
      this.listEl.innerHTML = `<div class="fallback-state" style="color:#ef4444;">Failed to read SQLite database: ${escapeHtml(message)}</div>`;
      return;
    }
    this.cards.clear();
    if (this.creatures.length === 0) {
      this.listEl.innerHTML = `
        <div class="fallback-state">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity:0.3; margin-bottom:10px;"><circle cx="12" cy="12" r="10"/><path d="M8 14s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg>
          <p>Your cryo-preservation vault is empty.<br/>Save elites from the Training Chamber or specimens from the Ocean.</p>
        </div>
      `;
      return;
    }
    const selected = catalogueId(this.ctx.selection.current);
    this.listEl.replaceChildren(...this.creatures.map((creature) => {
      const pheno = phenotypeCache.get(creature.genome);
      if (!pheno) void getPhenotype(creature.genome);
      const dot = SpecimenGlowDot({ style: { color: badgeColor(pheno) } });
      const button = (label: string, cls: string, style: Partial<CSSStyleDeclaration> | undefined, onClick: () => void) =>
        SmallCardButton({ class: cls, style, onClick: (e: Event) => { e.stopPropagation(); onClick(); }, children: label });
      const card = SpecimenCard({
        class: selected === creature.id ? "active" : "",
        onClick: () => this.selectCreature(creature),
        children: [
          dot,
          SpecimenCardHeader({ children: [H3({ children: creature.name }), Span({ class: "source", children: creature.source })] }),
          SpecimenMetricsRow({ children: [Span({ children: `Diet: ${dietLabel(creature.carnivory)}` }), Span({ children: `Fit: ${creature.fitness.toFixed(2)}` })] }),
          CardButtonGroup({
            children: [
              button("Release Clone", "btn-spawn", undefined, () => this.openSpawnDialog(creature)),
              button("Seed Training", "", { borderColor: "rgba(59, 130, 246, 0.3)", color: "#3b82f6", background: "rgba(59, 130, 246, 0.04)" }, () => void this.openTrainDialog(creature)),
              button("Rename", "", undefined, () => void this.renameCreature(creature)),
              button("Wipe", "btn-delete", undefined, () => void this.deleteCreature(creature)),
            ],
          }),
        ],
      });
      this.cards.set(creature.id, { card, dot, genome: creature.genome });
      return card;
    }));
  }

  private selectCreature(creature: CatalogueCreature) {
    const store = this.ctx.selection;
    const sel = fromCatalogue(creature, phenotypeCache.get(creature.genome) ?? null);
    store.select(sel, CATALOGUE_VITALS);
    if (!sel.phenotype) {
      void getPhenotype(creature.genome).then((p) => {
        if (p) store.setPhenotype(sel.ref, p);
      });
    }
  }

  private openSpawnDialog(creature: CatalogueCreature) {
    if (!this.mountScope) return;
    const imprint = checkboxRow("Load with learned synaptic weights (imprinting)");
    openModal(this.mountScope, {
      title: "Release Clone",
      icon: IconRocket,
      accent: "16, 185, 129",
      width: 360,
      text: `Should "${creature.name}" be released into the active ocean as an exact clone (including learned brain reflexes) or as a new genetic seed?`,
      body: [imprint.element],
      actions: [
        { label: "Cancel" },
        {
          label: "Release Clone",
          primary: true,
          onClick: async () => {
            try {
              await this.ctx.ipc.call("spawn_catalogue_creature_to_ocean", { id: creature.id, loadLearnedSynapses: imprint.input.checked });
              alert(`Clone of "${creature.name}" was successfully released into the ocean!`);
            } catch {
              // reported by the IPC client (toast)
            }
          },
        },
      ],
    });
  }

  private async openTrainDialog(creature: CatalogueCreature) {
    const scope = this.mountScope;
    if (!scope) return;
    let runs;
    try {
      runs = await this.ctx.ipc.call("get_trainer_runs");
    } catch {
      return; // reported by the IPC client (toast)
    }
    if (runs.length === 0) {
      alert("No training runs found in the database. First create a run in the Training Chamber!");
      return;
    }
    const select = RunSelect() as HTMLSelectElement;
    runs.forEach((run, index) => {
      const opt = document.createElement("option");
      opt.value = String(index);
      opt.innerText = `${run.runId} (Generation ${run.maxGeneration}, Best Fit: ${run.maxFitness.toFixed(2)})`;
      select.appendChild(opt);
    });
    openModal(scope, {
      title: "Inject into Training",
      icon: IconWeights,
      accent: "59, 130, 246",
      width: 360,
      text: `Select which of your persistent training runs you want to inject "${creature.name}" into:`,
      body: [select],
      actions: [
        { label: "Cancel" },
        {
          label: "Confirm Injection",
          primary: true,
          onClick: async () => {
            const run = runs[parseInt(select.value)];
            try {
              await this.ctx.ipc.call("add_catalogue_creature_to_training", { creatureId: creature.id, runId: run.runId, generation: run.maxGeneration });
              alert(`Creature "${creature.name}" successfully injected into training run "${run.runId}" (Generation ${run.maxGeneration})!\n\nIt will compete in one of the sandbox chambers starting from the next generation.`);
            } catch {
              // reported by the IPC client (toast)
            }
          },
        },
      ],
    });
  }

  private async renameCreature(creature: CatalogueCreature) {
    const input = prompt(`Enter a new name for "${creature.name}":`, creature.name);
    if (input === null) return;
    const name = input.trim();
    if (!name) return;
    try {
      await this.ctx.ipc.call("rename_catalogue_creature", { id: creature.id, newName: name });
      this.ctx.selection.rename({ source: "catalogue", catalogueId: creature.id }, name);
      void this.refreshList();
    } catch {
      // reported by the IPC client (toast)
    }
  }

  private async deleteCreature(creature: CatalogueCreature) {
    if (!confirm(`Are you sure you want to permanently delete species "${creature.name}" from the catalogue?`)) return;
    try {
      await this.ctx.ipc.call("delete_from_catalogue", { id: creature.id });
      if (catalogueId(this.ctx.selection.current) === creature.id) this.ctx.selection.clear();
      void this.refreshList();
    } catch {
      // reported by the IPC client (toast)
    }
  }
}

let controller: CatalogueController | null = null;

export const catalogueView: ViewModule<"catalogue"> = {
  id: "catalogue",
  title: "BroodBattle – Creature Catalog",
  backendMode: "ocean",
  create(ctx) {
    controller = new CatalogueController(ctx);
    return controller.root;
  },
  mount(ctx) {
    controller!.mount(ctx.scope);
  },
};
