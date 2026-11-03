// Inspector panel (TCK-128, TCK-160): shows the app selection (`state/selection.ts`) if it comes from one of the
// panel's sources: profile, animated preview, vitals, genome view, brain graph, DNA and save/assign actions. All
// effects and the preview loop live in the scope passed to `mount`, so a hidden panel does no work.
import type { CreaturePhenotype } from "../../bindings/CreaturePhenotype";
import type { GeneSpan } from "../../bindings/GeneSpan";
import { escapeHtml } from "../core/escape";
import type { Scope } from "../core/scope";
import { html, styled } from "../core/styled";
import type { ViewIpc } from "../core/view";
import { BrainRenderer } from "../render/brainRenderer";
import { describeGene, geneIndexByPosition, isSilentCopy } from "../shared/genome";
import {
  activations, catalogueEntry, selection, vitals, type Selection, type SelectionSource,
} from "../state/selection";
import { CreaturePreview } from "./CreaturePreview";
import { IconBrain, IconDna, IconPlus } from "./Icons";

/** Age that fills the age bar, in seconds (no lifespan in the model yet, TCK-146). */
export const AGE_BAR_MAX = 2700;

// --- STYLED COMPONENTS ---

const PanelContainer = styled("div")`
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  box-sizing: border-box;
`;

const FallbackState = styled("div")`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  color: #475569;
  font-size: 0.7rem;
  padding: 20px;
  box-sizing: border-box;
`;

const ActiveContent = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
  padding-right: 4px;

  /* Custom scrollbar style */
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
  &::-webkit-scrollbar-thumb:hover {
    background: rgba(148, 163, 184, 0.3);
  }
`;

const SpecimenProfile = styled("div")`
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(15, 23, 42, 0.45);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  padding: 10px;
`;

const ProfileMeta = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 2px;

  h3 {
    font-size: 0.85rem;
    font-weight: bold;
    margin: 0;
    color: #fff;
    font-family: monospace;
  }

  p {
    font-size: 0.58rem;
    color: #64748b;
    margin: 0;
    font-family: monospace;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
`;

const BadgeStatus = styled("span")`
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  font-size: 0.55rem;
  padding: 2px 6px;
  border-radius: 2px;
  font-weight: bold;
  text-transform: uppercase;
`;

const PreviewContainer = styled("div")`
  display: flex;
  gap: 12px;
  align-items: flex-start;
  background: rgba(15, 23, 42, 0.2);
  border: 1px dashed rgba(148, 163, 184, 0.08);
  border-radius: 6px;
  padding: 10px;
`;

const PreviewCanvas = styled("canvas")`
  background: #020617;
  border: 1px solid rgba(148, 163, 184, 0.15);
  border-radius: 4px;
  width: 100px;
  height: 100px;
  flex-shrink: 0;
`;

const PreviewMetaBox = styled("div")`
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const PreviewMetaText = styled("p")`
  font-size: 0.62rem;
  color: #94a3b8;
  line-height: 1.45;
  margin: 0;
  word-break: break-all;
  font-family: monospace;
`;

const DiagnosticsVitals = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 8px;
`;

const VitalRow = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;
`;

const VitalLabel = styled("span")`
  font-size: 0.58rem;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.5px;
`;

const ProgressBarBg = styled("div")`
  background: rgba(15, 23, 42, 0.8);
  border: 1px solid rgba(148, 163, 184, 0.1);
  height: 6px;
  border-radius: 3px;
  position: relative;
  overflow: hidden;
`;

const ProgressBarFill = styled("div")`
  height: 100%;
  width: 0%;
  border-radius: 3px;
  transition: width 0.15s ease-out;

  &.fill-green { background: #10b981; box-shadow: 0 0 6px #10b981; }
  &.fill-cyan { background: #00f2fe; box-shadow: 0 0 6px #00f2fe; }
  &.fill-purple { background: #c084fc; box-shadow: 0 0 6px #c084fc; }
`;

const VitalText = styled("span")`
  font-size: 0.58rem;
  font-family: monospace;
  color: #fff;
  align-self: flex-end;
  margin-top: -2px;
`;

const SubSectionContainer = styled("div")`
  display: flex;
  flex-direction: column;
  gap: 4px;

  h4 {
    font-size: 0.65rem;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin: 0;
  }
`;

const GenomeWrapper = styled("div")`
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  height: 70px;
  overflow-y: auto;
  padding: 6px;
  box-sizing: border-box;

  /* Custom scrollbar style */
  &::-webkit-scrollbar {
    width: 3px;
  }
  &::-webkit-scrollbar-track {
    background: transparent;
  }
  &::-webkit-scrollbar-thumb {
    background: rgba(148, 163, 184, 0.15);
    border-radius: 2px;
  }
`;

const GenomeLociGrid = styled("div")`
  display: grid;
  grid-template-columns: repeat(16, 1fr);
  grid-gap: 3px;
`;

const BrainVisualizerContainer = styled("div")`
  background: rgba(15, 23, 42, 0.5);
  border: 1px solid rgba(148, 163, 184, 0.08);
  border-radius: 4px;
  height: 150px;
  position: relative;
  overflow: hidden;
`;

const CopyAreaCard = styled("div")`
  border-top: 1px solid rgba(148, 163, 184, 0.1);
  padding-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
`;

const DNAExporterTextArea = styled("textarea")`
  font-family: monospace;
  font-size: 0.55rem;
  background: rgba(15, 23, 42, 0.6);
  border: 1px solid rgba(148, 163, 184, 0.15);
  color: #10b981;
  width: 100%;
  height: 48px;
  padding: 6px;
  box-sizing: border-box;
  resize: none;
  word-break: break-all;
  outline: none;
  border-radius: 4px;
`;

const SaveButton = styled("button")`
  background: rgba(0, 242, 254, 0.08);
  border: 1px solid rgba(0, 242, 254, 0.3);
  color: #00f2fe;
  font-family: monospace;
  font-size: 0.65rem;
  font-weight: bold;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  transition: all 0.2s;

  &:hover {
    background: rgba(0, 242, 254, 0.18);
    border-color: #00f2fe;
    box-shadow: 0 0 10px rgba(0, 242, 254, 0.2);
  }
`;

export interface DiagnosticsPanelOptions {
  /** Id prefix of the brain graph elements */
  prefix: string;
  /** Selection sources this panel shows; other selections show the fallback */
  sources: SelectionSource[];
  ipc: ViewIpc;
  /** "Save to Catalogue" button */
  save?: boolean;
  /** "Assign From Catalogue" button; called with the shown selection */
  onAssign?: (selection: Selection) => void;
}

const H3 = styled("h3")``;
const P = styled("p")``;
const H4 = styled("h4")``;
const B = styled("b")``;
const Label = styled("label")``;

export class DiagnosticsPanel {
  readonly element: HTMLElement;
  private readonly brainRenderer: BrainRenderer;
  private readonly preview: CreaturePreview;

  private readonly fallbackEl: HTMLElement;
  private readonly contentEl: HTMLElement;
  private readonly nameEl: HTMLElement;
  private readonly taxaEl: HTMLElement;
  private readonly statusEl: HTMLElement;
  private readonly energyBarEl: HTMLElement;
  private readonly energyTextEl: HTMLElement;
  private readonly ageBarEl: HTMLElement;
  private readonly ageTextEl: HTMLElement;
  private readonly genomeGridEl: HTMLElement;
  private readonly dnaTextEl: HTMLTextAreaElement;

  constructor(private readonly options: DiagnosticsPanelOptions) {
    this.element = PanelContainer();

    this.fallbackEl = FallbackState({
      children: [
        html(`<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity:0.35; margin-bottom:12px;"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>`),
        P({ children: "Select a specimen or sandbox chamber to decouple its genetics in real-time." }),
      ],
    });

    this.nameEl = H3({ children: "Nameless Specimen" });
    this.taxaEl = P({ children: "Strain code / Clonal Lineage" });
    this.statusEl = BadgeStatus({ children: "Living" });
    this.preview = new CreaturePreview({ size: 100, bodyFraction: 0.75 });
    // Same look as before: the styled preview canvas class on the component's canvas
    this.preview.element.className = PreviewCanvas().className;
    this.energyBarEl = ProgressBarFill({ class: "fill-green" });
    this.energyTextEl = VitalText({ children: "0nJ" });
    this.ageBarEl = ProgressBarFill({ class: "fill-purple" });
    this.ageTextEl = VitalText({ children: "0s" });
    this.genomeGridEl = GenomeLociGrid();
    const brainContainer = BrainVisualizerContainer();
    this.dnaTextEl = DNAExporterTextArea({ readonly: true, placeholder: "DNA Genome..." });

    const buttons: HTMLElement[] = [];
    if (options.onAssign) {
      const onAssign = options.onAssign;
      buttons.push(SaveButton({
        style: { borderColor: "rgba(59, 130, 246, 0.3)", color: "#3b82f6", background: "rgba(59, 130, 246, 0.04)", marginTop: "4px" },
        onClick: () => {
          const sel = this.shown();
          if (sel) onAssign(sel);
        },
        children: [IconPlus, "Assign From Catalogue"],
      }));
    }
    if (options.save) {
      buttons.push(SaveButton({ onClick: () => void this.saveToCatalogue(), children: [IconPlus, "Save to Catalogue"] }));
    }

    this.contentEl = ActiveContent({
      style: { display: "none" },
      children: [
        SpecimenProfile({ children: [ProfileMeta({ children: [this.nameEl, this.taxaEl] }), this.statusEl] }),
        PreviewContainer({
          children: [
            this.preview.element,
            PreviewMetaBox({
              children: [
                PreviewMetaText({
                  children: [
                    B({ style: { color: "#fff" }, children: "HD Phenotype Preview" }),
                    html("<br/>"),
                    "60Hz muscle & spinal preview compiled direct from active chromatin loci.",
                  ],
                }),
              ],
            }),
          ],
        }),
        DiagnosticsVitals({
          children: [
            VitalRow({ children: [VitalLabel({ children: "Digestive Energy" }), ProgressBarBg({ children: this.energyBarEl }), this.energyTextEl] }),
            VitalRow({ children: [VitalLabel({ children: "Organism Age" }), ProgressBarBg({ children: this.ageBarEl }), this.ageTextEl] }),
          ],
        }),
        SubSectionContainer({ children: [H4({ children: [IconDna, " Expressed Chromatin DNA"] }), GenomeWrapper({ children: this.genomeGridEl })] }),
        SubSectionContainer({ children: [H4({ children: [IconBrain, " CTRNN Brain Waves"] }), brainContainer] }),
        CopyAreaCard({
          children: [
            Label({ style: { fontSize: "0.62rem", color: "#64748b" }, children: "DNA Genome:" }),
            this.dnaTextEl,
            ...buttons,
          ],
        }),
      ],
    });

    this.element.appendChild(this.fallbackEl);
    this.element.appendChild(this.contentEl);
    this.brainRenderer = new BrainRenderer(brainContainer as HTMLDivElement, options.prefix);
  }

  /** The app selection if this panel shows its source, else `null`. */
  private shown(): Selection | null {
    const sel = selection.value;
    return sel && this.options.sources.includes(sel.ref.source) ? sel : null;
  }

  /** Binds the panel to the selection while `scope` lives (the view's mount scope). */
  mount(scope: Scope) {
    this.preview.start(scope);

    // A. Visibility, header, preview
    scope.effect(() => {
      const sel = this.shown();
      this.fallbackEl.style.display = sel ? "none" : "flex";
      this.contentEl.style.display = sel ? "flex" : "none";
      this.nameEl.innerText = sel?.name || "Nameless Specimen";
      this.taxaEl.innerText = sel?.taxa || "Strain code";
      this.preview.setPhenotype(sel?.phenotype ?? null);
    });

    // B. Vitals (live)
    scope.effect(() => {
      const v = vitals.value;
      const status = v.status || "Living";
      this.statusEl.innerText = status;
      const isAlive = status === "Alive" || status === "Living";
      this.statusEl.style.background = isAlive ? "rgba(16, 185, 129, 0.15)" : "rgba(77, 89, 116, 0.12)";
      this.statusEl.style.color = isAlive ? "#10b981" : "#64748b";

      const max = v.maxEnergy || 100;
      this.energyBarEl.style.width = `${Math.max(0, Math.min(100, (v.energy / max) * 100))}%`;
      this.energyTextEl.innerText = `${Math.round(v.energy)} / ${Math.round(max)}nJ`;

      const age = v.age;
      this.ageBarEl.style.width = age === null ? "0%" : `${Math.max(0, Math.min(100, (age / AGE_BAR_MAX) * 100))}%`;
      this.ageTextEl.innerText = age === null ? "–" : `${Math.round(age)}s`;
    });

    // C. Genome view and DNA text (only when the selected genome or phenotype changes)
    scope.effect(() => {
      const sel = this.shown();
      this.renderGenome(sel?.genome ?? "", sel?.methylations ?? [], sel?.phenotype ?? null);
    });

    // D. Brain graph layout
    scope.effect(() => {
      const p = this.shown()?.phenotype;
      if (p) this.brainRenderer.compile(p.brain, { genes: p.genes ?? [], organelles: p.organelles ?? [] });
    });

    // E. Live neural activations
    scope.effect(() => {
      const p = this.shown()?.phenotype;
      const acts = activations.value;
      if (p && acts.length > 0) this.brainRenderer.updateLiveGlows(acts, p.brain);
    });
  }

  private renderGenome(g: string, m: number[], p: CreaturePhenotype | null) {
    if (!g) {
      this.genomeGridEl.innerHTML = "";
      this.dnaTextEl.value = "";
      return;
    }
    this.dnaTextEl.value = g;

    // Gene spans come from the backend compiler (TCK-187); positions outside every span are non-coding
    const genes: GeneSpan[] = p?.genes ?? [];
    const organelles = p?.organelles ?? [];
    const owner = geneIndexByPosition(genes, g.length);
    const geneText = genes.map((_, idx) => describeGene(genes, idx, organelles));
    const silent = genes.map((_, idx) => isSilentCopy(genes, idx));

    let markup = "";
    for (let i = 0; i < g.length; i++) {
      const char = g[i];
      const isMethylated = !!m[i];
      const geneIdx = owner[i];
      const span = geneIdx >= 0 ? genes[geneIdx] : null;
      const isPromoter = !!span && i < span.start + span.gene.spelling.length;
      const charVal = char.charCodeAt(0) - 65;

      let bg: string;
      let border = "1px solid rgba(255,255,255,0.03)";
      let lociClass: string;
      let colorStyle = "";
      let activityText: string;

      if (isPromoter) {
        lociClass = "promoter";
        bg = "#ffffff";
        activityText = `${geneText[geneIdx]} [promoter]`;
      } else if (span) {
        const alpha = silent[geneIdx] ? 0.15 : 0.35;
        lociClass = "active";
        bg = `hsla(${charVal * 13.8}, 75%, 45%, ${alpha})`;
        border = `1.2px solid hsla(${charVal * 13.8}, 75%, 45%, ${alpha + 0.45})`;
        activityText = geneText[geneIdx];
      } else {
        lociClass = "inactive";
        bg = `hsla(${charVal * 13.8}, 35%, 15%, 0.03)`;
        border = `1.0px dashed hsla(${charVal * 13.8}, 35%, 15%, 0.15)`;
        colorStyle = "color: rgba(255,255,255,0.15);";
        activityText = "Non-coding";
      }

      const title = `Locus ${i}: ${char} - ${activityText}${isMethylated ? ` (Methylated +${m[i]})` : ""}`;
      markup += `
        <div class="loci-node ${lociClass} ${isMethylated ? "methylated" : ""}"
             style="background: ${bg}; border: ${border}; ${colorStyle}"
             title="${escapeHtml(title)}">
          ${escapeHtml(char)}
        </div>
      `;
    }
    this.genomeGridEl.innerHTML = markup;
  }

  /** Saves the shown creature (genome, methylations, its fitness and own synapse weights) to the catalogue. */
  private async saveToCatalogue() {
    const sel = this.shown();
    if (!sel || !sel.genome) return;
    const fallbackName = sel.phenotype?.latinName || sel.name || "Aqueus pulsa";
    const customName = prompt("Enter a name for this cryo-clone:", fallbackName);
    if (customName === null) return;
    const name = customName.trim() || fallbackName;
    const id = `catalogue_${Date.now()}_${Math.floor(Math.random() * 1000)}`;
    try {
      await this.options.ipc.call("save_to_catalogue", { id, ...catalogueEntry(sel, name) });
      alert(`Species "${name}" successfully saved to the persistent catalogue!`);
    } catch {
      // reported by the IPC client (toast)
    }
  }
}
