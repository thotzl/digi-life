import { styled } from "../core/styled";
import { effect } from "@preact/signals-core";
import { BrainRenderer } from "../render/brainRenderer";
import { CreatureRenderer } from "../render/creatureRenderer";
import { safeInvoke } from "../api";
import {
  selectedId, selectedName, selectedTaxa, selectedStatus, selectedEnergy,
  selectedMaxEnergy, selectedAdrenaline, selectedAge, selectedGenome,
  selectedMethylations, selectedPhenotype, selectedBrainActivations, computeActiveGeneSpans, getLocusDescription
} from "../signals";
import { IconPlus } from "../components/Icons";

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

// --- CLASS COMPONENT ---

export class UnifiedDiagnosticsPanel {
  public element: HTMLElement;
  private prefix: string;
  private brainRenderer: BrainRenderer;
  private previewRenderer: CreatureRenderer;
  private animFrameId: number | null = null;
  private animTime: number = 0;

  // Bound Elements
  private fallbackEl: HTMLElement;
  private contentEl: HTMLElement;
  private specimenNameEl: HTMLElement;
  private specimenTaxaEl: HTMLElement;
  private specimenStatusEl: HTMLElement;
  private energyBarEl: HTMLElement;
  private energyTextEl: HTMLElement;
  private adrenalineBarEl: HTMLElement;
  private adrenalineTextEl: HTMLElement;
  private ageBarEl: HTMLElement;
  private ageTextEl: HTMLElement;
  private genomeGridEl: HTMLElement;
  private brainContainerEl: HTMLElement;
  private previewCanvasEl: HTMLCanvasElement;
  private previewCtx: CanvasRenderingContext2D | null;
  private dnaTextEl: HTMLTextAreaElement;
  private btnSaveEl: HTMLElement;
  private btnAssignEl: HTMLElement;

  constructor(prefix: string) {
    this.prefix = prefix;
    this.element = PanelContainer();

    // 1. Fallback State element (visible when no creature is focused)
    this.fallbackEl = FallbackState({
      children: [
        `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="opacity:0.35; margin-bottom:12px;"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>`,
        styled("p")``({ children: "Select a specimen or sandbox chamber to decouple its genetics in real-time." })
      ]
    });

    // 2. Instantiate and mount child nodes
    this.specimenNameEl = styled("h3")``({ children: "Nameless Specimen" });
    this.specimenTaxaEl = styled("p")``({ children: "Strain code / Clonal Lineage" });
    this.specimenStatusEl = BadgeStatus({ children: "Living" });

    this.previewCanvasEl = PreviewCanvas({ width: 100, height: 100 });
    this.previewCtx = this.previewCanvasEl.getContext("2d");
    this.previewRenderer = new CreatureRenderer(this.previewCanvasEl);

    this.energyBarEl = ProgressBarFill({ class: "fill-green" });
    this.energyTextEl = VitalText({ children: "0nJ" });

    this.adrenalineBarEl = ProgressBarFill({ class: "fill-cyan" });
    this.adrenalineTextEl = VitalText({ children: "1.00x" });

    this.ageBarEl = ProgressBarFill({ class: "fill-purple" });
    this.ageTextEl = VitalText({ children: "0s" });

    this.genomeGridEl = GenomeLociGrid();
    this.brainContainerEl = BrainVisualizerContainer();

    this.dnaTextEl = DNAExporterTextArea({ readonly: true, placeholder: "DNA Genome..." });
    
    this.btnSaveEl = SaveButton({
      onClick: () => this.saveToCatalogue(),
      children: [
        IconPlus,
        "Save to Catalogue"
      ]
    });

    this.btnAssignEl = SaveButton({
      style: { borderColor: "rgba(59, 130, 246, 0.3)", color: "#3b82f6", background: "rgba(59, 130, 246, 0.04)", marginTop: "4px" },
      onClick: () => this.assignFromCatalogue(),
      children: [
        IconPlus,
        "Assign From Catalogue"
      ]
    });

    // 3. Assemble active contents panel
    this.contentEl = ActiveContent({
      style: { display: "none" },
      children: [
        SpecimenProfile({
          children: [
            ProfileMeta({
              children: [
                this.specimenNameEl,
                this.specimenTaxaEl
              ]
            }),
            this.specimenStatusEl
          ]
        }),
        PreviewContainer({
          children: [
            this.previewCanvasEl,
            PreviewMetaBox({
              children: [
                PreviewMetaText({
                  children: [
                    styled("b")``({ style: { color: "#fff" }, children: "HD Phenotype Preview" }),
                    `<br/>60Hz muscle & spinal preview compiled direct from active chromatin loci.`
                  ]
                })
              ]
            })
          ]
        }),
        DiagnosticsVitals({
          children: [
            VitalRow({
              children: [
                VitalLabel({ children: "Digestive Energy" }),
                ProgressBarBg({ children: this.energyBarEl }),
                this.energyTextEl
              ]
            }),
            VitalRow({
              children: [
                VitalLabel({ children: "Adrenaline Sprint" }),
                ProgressBarBg({ children: this.adrenalineBarEl }),
                this.adrenalineTextEl
              ]
            }),
            VitalRow({
              children: [
                VitalLabel({ children: "Organism Age" }),
                ProgressBarBg({ children: this.ageBarEl }),
                this.ageTextEl
              ]
            })
          ]
        }),
        SubSectionContainer({
          children: [
            styled("h4")``({ children: "🧬 Expressed Chromatin DNA" }),
            GenomeWrapper({ children: this.genomeGridEl })
          ]
        }),
        SubSectionContainer({
          children: [
            styled("h4")``({ children: "🧠 CTRNN Brain Waves" }),
            this.brainContainerEl
          ]
        }),
        CopyAreaCard({
          children: [
            styled("label")``({ style: { fontSize: "0.62rem", color: "#64748b" }, children: "DNA Genome:" }),
            this.dnaTextEl,
            prefix === "trainer" ? this.btnAssignEl : null,
            prefix !== "catalogue" ? this.btnSaveEl : null
          ]
        })
      ]
    });

    this.element.appendChild(this.fallbackEl);
    this.element.appendChild(this.contentEl);

    // 4. Initialize Brain Graph visualizer
    this.brainRenderer = new BrainRenderer(this.brainContainerEl as HTMLDivElement, prefix);

    // 5. Connect reactive bindings to global Signals
    this.setupBindings();
  }

  /**
   * Registers Preact signals `effect` listeners to reactively update the compiled nodes
   * whenever selection states shift anywhere across the SPA.
   */
  private setupBindings() {
    // A. Panel Visibilities Toggle
    effect(() => {
      const id = selectedId.value;
      if (id === null) {
        this.fallbackEl.style.display = "flex";
        this.contentEl.style.display = "none";
        this.stopPreviewLoop();
      } else {
        this.fallbackEl.style.display = "none";
        this.contentEl.style.display = "flex";
        this.startPreviewLoop();
      }
    });

    // B. Header Profile Meta
    effect(() => {
      this.specimenNameEl.innerText = selectedName.value || "Nameless Specimen";
    });
    effect(() => {
      this.specimenTaxaEl.innerText = selectedTaxa.value || "Strain code";
    });
    effect(() => {
      const status = selectedStatus.value || "Living";
      this.specimenStatusEl.innerText = status;
      const isAlive = status === "Alive" || status === "Living";
      this.specimenStatusEl.style.background = isAlive ? "rgba(16, 185, 129, 0.15)" : "rgba(77, 89, 116, 0.12)";
      this.specimenStatusEl.style.color = isAlive ? "#10b981" : "#64748b";
    });

    // C. Real-Time Vitals Progression
    effect(() => {
      const val = selectedEnergy.value;
      const max = selectedMaxEnergy.value || 100;
      const pct = Math.max(0, Math.min(100, (val / max) * 100));
      this.energyBarEl.style.width = `${pct}%`;
      this.energyTextEl.innerText = `${Math.round(val)} / ${Math.round(max)}nJ`;
    });

    effect(() => {
      const val = selectedAdrenaline.value;
      const pct = Math.max(0, Math.min(100, ((val - 1.0) / 0.8) * 100));
      this.adrenalineBarEl.style.width = `${pct}%`;
      this.adrenalineTextEl.innerText = `${val.toFixed(2)}x`;
    });

    effect(() => {
      const val = selectedAge.value;
      const pct = Math.max(0, Math.min(100, (val / 2700) * 100));
      this.ageBarEl.style.width = `${pct}%`;
      this.ageTextEl.innerText = `${Math.round(val)}s`;
    });

    // D. DNA Helix Loci Grid
    effect(() => {
      const g = selectedGenome.value;
      const m = selectedMethylations.value;
      const p = selectedPhenotype.value;
      if (!g) {
        this.genomeGridEl.innerHTML = "";
        this.dnaTextEl.value = "";
        return;
      }

      this.dnaTextEl.value = g;

      let html = "";
      const cs = p ? (p.chromatinState || p.chromatin_state) : null;
      const activeGeneSpans = (p && cs) ? computeActiveGeneSpans(g, cs) : [];

      for (let i = 0; i < g.length; i++) {
        const char = g[i];
        const isPromoter = i < 16;
        const isMethylated = m && m[i] !== 0;

        let isActiveGene = false;
        if (activeGeneSpans) {
          isActiveGene = activeGeneSpans.some((span: any) => i >= span.start && i <= span.end);
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
      this.genomeGridEl.innerHTML = html;
    });

    // E. Brain Graph compiler trigger (updates nodes topology dynamically)
    effect(() => {
      const p = selectedPhenotype.value;
      if (p) {
        this.brainRenderer.compile(p.brain, p.organelles.length * 5);
      }
    });

    // F. Live Neural Activations & Synaptic Firing Glows
    effect(() => {
      const p = selectedPhenotype.value;
      const acts = selectedBrainActivations.value;
      if (p && acts && acts.length > 0) {
        this.brainRenderer.updateLiveGlows(acts, p.brain);
      }
    });
  }

  /**
   * Spawns a high-fps local rendering loop inside the Preview Canvas.
   */
  private startPreviewLoop() {
    if (this.animFrameId) return;

    const loop = () => {
      this.animTime += 0.045;
      const p = selectedPhenotype.value;

      if (this.previewCtx && p) {
        this.previewCtx.fillStyle = '#020617';
        this.previewCtx.fillRect(0, 0, 100, 100);

        const baseLength = p.spinalHarmonics?.baseLength || 130;
        const dynamicScale = 75.0 / (baseLength * 0.5);

        this.previewCtx.save();
        this.previewCtx.translate(50, 50);
        this.previewCtx.scale(dynamicScale, dynamicScale);

        this.previewRenderer.render(
          p,
          this.animTime,
          0,
          0,
          -Math.PI / 2, // pointing North
          0
        );
        this.previewCtx.restore();
      }
      this.animFrameId = requestAnimationFrame(loop);
    };

    loop();
  }

  /**
   * Suspends the local high-fps rendering loop to conserve CPU/GPU clocks.
   */
  private stopPreviewLoop() {
    if (this.animFrameId !== null) {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
    }
  }

  /**
   * Saves the focused cryo-clone (DNA + Epigenetics + Brain Synapses) persistently into SQLite.
   */
  private async saveToCatalogue() {
    const g = selectedGenome.value;
    const p = selectedPhenotype.value;
    if (!g || !p) return;

    const customName = prompt(`Enter a name for this cryo-clone:`, p.latinName || "Aqueus pulsa");
    if (customName === null) return; // Cancelled

    const name = customName.trim() || p.latinName || "Aqueus pulsa";
    const id = "catalogue_" + Date.now() + "_" + Math.floor(Math.random() * 1000);
    const source = this.prefix === "ocean" ? "Ocean Substrate" : "Evolutionary Trainer";
    const fitness = p.fitness || 0.0;
    const m = p.methylations || [];
    const w = p.brain?.synapses?.map((s: any) => s.weight) || [];

    try {
      const ok = await safeInvoke("save_to_catalogue", {
        id,
        name,
        genome: g,
        source,
        fitness,
        methylations: m,
        synapseWeights: w
      });
      if (ok) {
        alert(`Species "${name}" successfully saved to the persistent catalogue!`);
      }
    } catch (e) {
      console.error("[Catalogue] Failed to save creature:", e);
      alert(`Error saving creature: ${e}`);
    }
  }

  /**
   * Assigns a species from the Catalogue directly into the selected Trainer sandbox in real-time.
   * Leverages a premium scrolling card deck list and live wiggling preview bubble inside the modal!
   */
  private async assignFromCatalogue() {
    const sandboxId = selectedId.value;
    if (sandboxId === null) return;

    try {
      const creatures = await safeInvoke("get_catalogue_creatures");
      if (!creatures || creatures.length === 0) {
        alert("Your creature catalogue is empty! Save creatures in the ocean or trainer first.");
        return;
      }

      let selectedIdx = 0;
      let animTime = 0;
      let animId: number;

      // Build cyber-modal backdrop
      const backdrop = document.createElement("div");
      backdrop.style.cssText = "position:absolute; top:0; left:0; right:0; bottom:0; background:rgba(2,6,23,0.85); z-index:1000; display:flex; align-items:center; justify-content:center; backdrop-filter:blur(8px);";

      const modalBox = document.createElement("div");
      modalBox.style.cssText = "background:rgba(15,23,42,0.9); border:1px solid rgba(0,242,254,0.3); border-radius:8px; width:410px; padding:24px; box-sizing:border-box; display:flex; flex-direction:column; gap:14px; box-shadow:0 0 35px rgba(0,242,254,0.15); animation:slide-fade-in 0.2s ease-out; font-family:monospace; color:#f1f5f9;";

      const title = document.createElement("h3");
      title.style.cssText = "margin:0; font-size:1.05rem; font-weight:bold; color:#fff; text-transform:uppercase; letter-spacing:1px; border-bottom:1px solid rgba(148,163,184,0.1); padding-bottom:8px; display:flex; align-items:center; gap:6px;";
      title.innerHTML = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2" style="margin-top:-1px;"><path d="M12 22C17.5228 22 22 19.7614 22 17C22 14.2386 17.5228 12 12 12C6.47715 12 2 14.2386 2 17C2 19.7614 6.47715 22 12 22Z"/><path d="M22 7C22 9.76142 17.5228 12 12 12C6.47715 12 2 9.76142 2 7C2 4.23858 6.47715 2 12 2C17.5228 2 22 4.23858 22 7Z"/><path d="M2 7V17M22 7V17"/></svg> Pre-populate Sandbox #${sandboxId}`;

      const desc = document.createElement("p");
      desc.style.cssText = "margin:0; font-size:0.68rem; color:#94a3b8; line-height:1.45;";
      desc.innerText = `Select a saved individual from your catalogue to use it directly as a new founding father in Sandbox #${sandboxId}:`;

      // Centered, glowing aggregated local preview circle
      const previewContainer = document.createElement("div");
      previewContainer.style.cssText = "display:flex; justify-content:center; align-items:center; background:rgba(2,6,23,0.5); border:1px solid rgba(148,163,184,0.08); border-radius:6px; padding:12px; gap:16px;";

      const pCanvas = document.createElement("canvas");
      pCanvas.width = 72;
      pCanvas.height = 72;
      pCanvas.style.cssText = "background:#020617; border:1.5px solid rgba(59,130,246,0.3); border-radius:50%; box-shadow:0 0 15px rgba(59,130,246,0.12); flex-shrink:0;";
      const pCtx = pCanvas.getContext("2d")!;
      const pRenderer = new CreatureRenderer(pCanvas);

      const pMeta = document.createElement("div");
      pMeta.style.cssText = "flex:1; display:flex; flex-direction:column; gap:4px; min-width:0;";
      const pName = document.createElement("b");
      pName.style.cssText = "font-size:0.8rem; color:#fff;";
      const pDetails = document.createElement("span");
      pDetails.style.cssText = "font-size:0.58rem; color:#64748b; line-height:1.4;";

      previewContainer.appendChild(pCanvas);
      previewContainer.appendChild(pMeta);
      pMeta.appendChild(pName);
      pMeta.appendChild(pDetails);

      // High-tech scrollable list replacing the ugly dropdown select
      const listDeck = document.createElement("div");
      listDeck.style.cssText = "height:140px; overflow-y:auto; display:flex; flex-direction:column; gap:6px; padding-right:4px; border:1px solid rgba(148,163,184,0.1); border-radius:6px; background:rgba(15,23,42,0.3); padding:6px; box-sizing:border-box;";
      
      const styleTag = document.createElement("style");
      styleTag.innerText = `
        .modal-scroll-deck::-webkit-scrollbar { width: 3px; }
        .modal-scroll-deck::-webkit-scrollbar-thumb { background: rgba(148, 163, 184, 0.15); border-radius: 2px; }
        .modal-card { transition: all 0.15s ease-in-out; border: 1px solid rgba(148,163,184,0.1) !important; border-radius: 4px; padding: 8px 10px; cursor: pointer; display: flex; align-items: center; justify-content: space-between; font-size: 0.65rem; background: rgba(30,41,59,0.15) !important; }
        .modal-card:hover { border-color: #3b82f6 !important; background: rgba(59,130,246,0.02) !important; }
        .modal-card.active { border-color: #3b82f6 !important; background: rgba(59,130,246,0.06) !important; box-shadow: 0 0 10px rgba(59,130,246,0.05); }
      `;
      listDeck.classList.add("modal-scroll-deck");
      document.head.appendChild(styleTag);

      let activePhenotype: any = null;

      // Load detailed phenotype for the active selection
      const loadSelectedPreview = async (creature: any) => {
        pName.innerText = creature.name;
        let dietLabel = "Omnivore";
        if (creature.carnivory >= 0.55) dietLabel = "Predator";
        else if (creature.carnivory < 0.3) dietLabel = "Herbivore";
        pDetails.innerHTML = `Diet Class: <span style="color:#3b82f6;">${dietLabel}</span><br/>Fitness rating: ${creature.fitness.toFixed(0)}<br/>Origin: ${creature.source}`;
        
        try {
          const pheno = await safeInvoke("get_fossil_phenotype", { genome: creature.genome });
          if (pheno) {
            activePhenotype = pheno;
          }
        } catch (err) {
          console.error("Failed to load modal preview:", err);
        }
      };

      // Render list cards deck
      const renderCards = () => {
        listDeck.innerHTML = "";
        creatures.forEach((c: any, index: number) => {
          const charVal = c.genome.charCodeAt(17) || 65;
          const colorH = (charVal - 65) * 13.8;

          const card = document.createElement("div");
          card.className = `modal-card ${index === selectedIdx ? "active" : ""}`;
          
          let dietLabel = "Omnivore";
          if (c.carnivory >= 0.55) dietLabel = "Predator";
          else if (c.carnivory < 0.3) dietLabel = "Herbivore";

          card.innerHTML = `
            <div style="display:flex; align-items:center; gap:8px;">
              <div style="width:6px; height:6px; border-radius:50%; background:hsla(${colorH},85%,55%,1); box-shadow:0 0 6px hsla(${colorH},85%,55%,0.8);"></div>
              <b style="color:#fff;">${c.name}</b>
            </div>
            <span style="color:#64748b; font-size:0.58rem; text-transform:uppercase;">${dietLabel} // F: ${c.fitness.toFixed(0)}</span>
          `;

          card.addEventListener("click", () => {
            selectedIdx = index;
            renderCards();
            loadSelectedPreview(c);
          });

          listDeck.appendChild(card);
        });
      };

      // Initial load first item preview
      loadSelectedPreview(creatures[0]);
      renderCards();

      // High-fps Preview loop inside modal
      const previewLoop = () => {
        animTime += 0.045;
        if (pCtx && activePhenotype) {
          pCtx.fillStyle = '#020617';
          pCtx.fillRect(0, 0, 72, 72);

          const baseLength = activePhenotype.spinalHarmonics?.baseLength || 130;
          const dynamicScale = 52.0 / (baseLength * 0.5);

          pCtx.save();
          pCtx.translate(36, 35);
          pCtx.scale(dynamicScale, dynamicScale);
          pRenderer.render(activePhenotype, animTime, 0, 0, -Math.PI / 2, 0);
          pCtx.restore();
        }
        animId = requestAnimationFrame(previewLoop);
      };
      previewLoop();

      // Checkbox + Actions
      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.checked = false;
      checkbox.id = "trainer-chk-load-syn";
      checkbox.style.cssText = "cursor:pointer; width:15px; height:15px; margin:0;";

      const label = document.createElement("label");
      label.htmlFor = "trainer-chk-load-syn";
      label.style.cssText = "display:flex; align-items:center; gap:8px; font-size:0.65rem; cursor:pointer; user-select:none; color:#94a3b8; padding-top:4px;";
      label.appendChild(checkbox);
      const labelSpan = document.createElement("span");
      labelSpan.innerText = "Load with learned synapse weights (pre-imprinting)";
      label.appendChild(labelSpan);

      const btnCancel = document.createElement("button");
      btnCancel.style.cssText = "background:rgba(148,163,184,0.06); border:1px solid rgba(148,163,184,0.15); color:#94a3b8; border-radius:4px; font-family:monospace; font-size:0.7rem; cursor:pointer; padding:8px 16px; font-weight:bold;";
      btnCancel.innerText = "Cancel";
      btnCancel.addEventListener("click", () => {
        cancelAnimationFrame(animId);
        backdrop.remove();
      });

      const btnConfirm = document.createElement("button");
      btnConfirm.style.cssText = "background:#3b82f6; color:#020617; border:none; border-radius:4px; font-family:monospace; font-weight:bold; font-size:0.7rem; cursor:pointer; padding:8px 16px;";
      btnConfirm.innerText = "Assign";
      btnConfirm.addEventListener("click", async () => {
        const selectedCreature = creatures[selectedIdx];
        const loadSyn = checkbox.checked;
        try {
          await safeInvoke("handle_client_action", {
            action: JSON.stringify({
              type: "ASSIGN_SANDBOX_CREATURE",
              sandbox_id: sandboxId,
              genome: selectedCreature.genome,
              methylations: selectedCreature.methylations,
              synapse_weights: selectedCreature.synapseWeights,
              load_learned_synapses: loadSyn
            })
          });
          cancelAnimationFrame(animId);
          backdrop.remove();
          alert(`Species "${selectedCreature.name}" successfully assigned to Sandbox #${sandboxId}!`);
        } catch (err) {
          alert(`Error during assignment: ${err}`);
        }
      });

      const buttonsRow = document.createElement("div");
      buttonsRow.style.cssText = "display:flex; justify-content:flex-end; gap:8px; border-top:1px solid rgba(148,163,184,0.1); padding-top:12px; margin-top:12px;";
      buttonsRow.appendChild(btnCancel);
      buttonsRow.appendChild(btnConfirm);

      modalBox.appendChild(title);
      modalBox.appendChild(desc);
      modalBox.appendChild(previewContainer);
      modalBox.appendChild(listDeck);
      modalBox.appendChild(label);
      modalBox.appendChild(buttonsRow);
      backdrop.appendChild(modalBox);
      document.getElementById("app")?.appendChild(backdrop);
    } catch (e) {
      console.error("[Trainer] Failed to assign from catalogue:", e);
    }
  }
}
