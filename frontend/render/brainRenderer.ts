import { BrainTopology, CTRNNNeuron, CTRNNSynapse } from "../shared/types";

export class BrainRenderer {
  public static readonly SYSTEMIC_BASE_INPUTS_COUNT = 7; // Synchronisiert mit dem Backend (Inklusive Such-Arousal)
  public static readonly MOTOR_OUTPUT_NODES_COUNT = 4;

  public static readonly LEFT_MARGIN = 25;
  public static readonly RIGHT_MARGIN = 285;
  public static readonly AREA_WIDTH = BrainRenderer.RIGHT_MARGIN - BrainRenderer.LEFT_MARGIN;
  public static readonly CENTER_X = BrainRenderer.LEFT_MARGIN + BrainRenderer.AREA_WIDTH / 2;

  public static readonly TOP_MARGIN = 20;
  public static readonly AREA_HEIGHT = 200;

  private elementCache = new Map<string, SVGElement>();
  private compiledCoords = new Map<number, { x: number, y: number }>();

  constructor(
    private container: HTMLDivElement, 
    private elementIdPrefix: string,
    private onNeuronHover?: (id: number | null) => void,
    private onSynapseHover?: (from: number, to: number, weight: number | null) => void
  ) {}

  public getNeuronX(id: number, _K: number, brain: BrainTopology): number {
    const n = brain.neurons.find(node => node.id === id);
    if (!n) return BrainRenderer.CENTER_X;

    const x = n.x !== undefined && n.x !== null ? n.x : 0.5;
    return BrainRenderer.LEFT_MARGIN + x * BrainRenderer.AREA_WIDTH;
  }

  public getNeuronY(id: number, _K: number, brain: BrainTopology): number {
    const n = brain.neurons.find(node => node.id === id);
    if (!n) return BrainRenderer.TOP_MARGIN;

    const y = n.y !== undefined && n.y !== null ? n.y : 0.5;
    return BrainRenderer.TOP_MARGIN + y * BrainRenderer.AREA_HEIGHT;
  }

  public compile(brain: BrainTopology, K: number): void {
    // Lazy Initialization zur Absicherung gegen Vite Hot-Module-Replacement (HMR) State Preservation Bugs!
    if (!this.elementCache) this.elementCache = new Map<string, SVGElement>();
    if (!this.compiledCoords) this.compiledCoords = new Map<number, { x: number, y: number }>();

    this.elementCache.clear();
    this.compiledCoords.clear();
    if (!brain || brain.neurons.length === 0) {
      this.container.innerHTML = `<div class="fallback-state">No genetically encoded CTRNN brain found.</div>`;
      return;
    }

    // Thalamus-First Parsing: Hide raw exteroceptive inputs (ID < K)
    const visibleNeurons = brain.neurons.filter(n => !(n.type === "input" && n.id < K));
    const visibleSynapses = brain.synapses.filter(syn => syn.fromNode >= K);

    // Group neurons into their respective 3 vertical columns for clean spatial distribution
    const col1Nodes = visibleNeurons.filter(n => n.type === "input" || n.label.includes("Thalamus") || n.label.includes("Σ Sum") || n.label.includes("Δ Diff"));
    const col2Nodes = visibleNeurons.filter(n => n.type === "hidden" && !(n.label.includes("Thalamus") || n.label.includes("Σ Sum") || n.label.includes("Δ Diff")));
    const col3Nodes = visibleNeurons.filter(n => n.type === "output");

    col1Nodes.forEach((n, idx) => {
      const x = 0.0; // Left Column
      const y = (idx + 1) / (col1Nodes.length + 1);
      this.compiledCoords.set(n.id, {
        x: BrainRenderer.LEFT_MARGIN + x * BrainRenderer.AREA_WIDTH,
        y: BrainRenderer.TOP_MARGIN + y * BrainRenderer.AREA_HEIGHT
      });
    });

    col2Nodes.forEach((n, idx) => {
      const rawY = n.y !== undefined && n.y !== null ? n.y : 0.5;
      const x = rawY; // Dynamic depth fanning
      const y = (idx + 1) / (col2Nodes.length + 1);
      this.compiledCoords.set(n.id, {
        x: BrainRenderer.LEFT_MARGIN + x * BrainRenderer.AREA_WIDTH,
        y: BrainRenderer.TOP_MARGIN + y * BrainRenderer.AREA_HEIGHT
      });
    });

    col3Nodes.forEach((n, idx) => {
      const x = 1.0; // Right Column
      const y = (idx + 1) / (col3Nodes.length + 1);
      this.compiledCoords.set(n.id, {
        x: BrainRenderer.LEFT_MARGIN + x * BrainRenderer.AREA_WIDTH,
        y: BrainRenderer.TOP_MARGIN + y * BrainRenderer.AREA_HEIGHT
      });
    });

    let svgContent = `<svg width="100%" height="100%" viewBox="0 0 310 240" style="background:#020617;">`;

    // Draw visible synapses
    visibleSynapses.forEach((syn: CTRNNSynapse) => {
      const from = this.compiledCoords.get(syn.fromNode);
      const to = this.compiledCoords.get(syn.toNode);

      if (from && to) {
        const isExcitatory = syn.weight > 0;
        const strokeColor = isExcitatory ? "rgba(16, 185, 129, 0.85)" : "rgba(239, 68, 68, 0.85)";

        const absWeight = Math.abs(syn.weight);
        const displayStyle = absWeight === 0.0 ? "display: none;" : "";
        const weightFactor = Math.min(0.08 + (absWeight / 2.0) * 0.77, 0.85);
        const strokeWidth = Math.min(0.4 + (absWeight / 2.0) * 1.4, 2.0);

        svgContent += `
          <line id="${this.elementIdPrefix}-syn-${syn.fromNode}-${syn.toNode}" x1="${from.x}" y1="${from.y}" x2="${to.x}" y2="${to.y}"
                stroke="${strokeColor}" stroke-width="${strokeWidth}" style="opacity: ${weightFactor}; ${displayStyle}" />
        `;
      }
    });

    // Draw visible neurons
    visibleNeurons.forEach((n: CTRNNNeuron) => {
      const coord = this.compiledCoords.get(n.id);
      if (coord) {
        const isInput = n.type === "input" || n.label.includes("Thalamus") || n.label.includes("Σ Sum") || n.label.includes("Δ Diff");
        const isOutput = n.type === "output";
        const fill = isInput ? "#0ea5e9" : (isOutput ? "#c084fc" : "#94a3b8");
        const radius = isInput || isOutput ? 4.5 : 3.2;

        svgContent += `
          <circle id="${this.elementIdPrefix}-node-${n.id}" cx="${coord.x}" cy="${coord.y}" r="${radius}" fill="${fill}"
                  style="cursor:pointer; filter:drop-shadow(0 0 2px ${fill}); opacity: 0.25;" />
        `;
      }
    });

    svgContent += `</svg>`;
    this.container.innerHTML = svgContent;

    // Cache elements references for high-performance glows
    visibleNeurons.forEach((n: CTRNNNeuron) => {
      const id = `${this.elementIdPrefix}-node-${n.id}`;
      const el = document.getElementById(id) as SVGElement | null;
      if (el) this.elementCache.set(id, el);
    });

    visibleSynapses.forEach((syn: CTRNNSynapse) => {
      const id = `${this.elementIdPrefix}-syn-${syn.fromNode}-${syn.toNode}`;
      const el = document.getElementById(id) as SVGElement | null;
      if (el) this.elementCache.set(id, el);
    });

    // Add event listeners for hover tooltips on neuron nodes
    if (this.onNeuronHover) {
      visibleNeurons.forEach((n: CTRNNNeuron) => {
        const id = `${this.elementIdPrefix}-node-${n.id}`;
        const el = this.elementCache.get(id);
        if (el) {
          el.addEventListener("mouseenter", () => {
            this.onNeuronHover?.(n.id);
            el.setAttribute("stroke", "#ffffff");
            el.setAttribute("stroke-width", "1.5");
            el.style.opacity = "1.0";
          });
          el.addEventListener("mouseleave", () => {
            this.onNeuronHover?.(null);
            el.removeAttribute("stroke");
            el.removeAttribute("stroke-width");
          });
        }
      });
    }

    // Add event listeners for hover tooltips on synapses (lines)
    if (this.onSynapseHover) {
      visibleSynapses.forEach((syn: CTRNNSynapse) => {
        const id = `${this.elementIdPrefix}-syn-${syn.fromNode}-${syn.toNode}`;
        const el = this.elementCache.get(id);
        if (el) {
          el.addEventListener("mouseenter", () => {
            this.onSynapseHover?.(syn.fromNode, syn.toNode, syn.weight);
            el.setAttribute("stroke-width", "3.0");
            el.style.opacity = "1.0";
          });
          el.addEventListener("mouseleave", () => {
            this.onSynapseHover?.(0, 0, null);
            el.removeAttribute("stroke-width");
          });
        }
      });
    }
  }

  public updateLiveGlows(activations: number[], brain: BrainTopology): void {
    if (!brain || !activations) return;

    // 1. Update Neurons (Instant 100% glow on trigger, else dimmed to 0.22 for subtle background visibility)
    brain.neurons.forEach((n: CTRNNNeuron) => {
      const id = `${this.elementIdPrefix}-node-${n.id}`;
      const el = this.elementCache.get(id);
      if (el) {
        const rawAct = Math.max(0.0, Math.min(1.0, Math.abs(activations[n.id] || 0.0)));
        const isInput = n.type === "input";
        const isOutput = n.type === "output";
        const exponent = (this.elementIdPrefix === "trainer" && isInput) ? 1.5 : 4.0;
        const act = Math.pow(rawAct, exponent);

        const colorGlow = isInput ? "#0ea5e9" : (isOutput ? "#c084fc" : "#e2e8f0");
        const radius = isInput || isOutput ? (act > 0.45 ? 6.5 : 4.5) : (act > 0.45 ? 5.0 : 3.2);

        el.setAttribute("fill", colorGlow); // Always retain beautiful colored beads
        el.setAttribute("r", radius.toString());

        // Instant 100% brightness on trigger, else dimmed to 0.22
        const isTriggered = rawAct > 0.02; // Lowered threshold from 0.05 to 0.02 for high sensitivity!
        const opacity = isTriggered ? 1.0 : 0.22; // Raised quiet opacity from 0.15 to 0.22!
        el.style.opacity = opacity.toString();
        
        if (isTriggered) {
          el.style.filter = `drop-shadow(0 0 5px ${colorGlow})`;
        } else {
          el.style.filter = "none";
        }
      }
    });

    // 2. Update Synapses (Instant 100% bright track on trigger, else faded to paper-thin rest)
    brain.synapses.forEach((syn: CTRNNSynapse) => {
      const id = `${this.elementIdPrefix}-syn-${syn.fromNode}-${syn.toNode}`;
      const el = this.elementCache.get(id);
      if (el) {
        const absWeight = Math.abs(syn.weight);
        if (absWeight === 0.0) {
          el.style.display = "none";
          return;
        }

        const preVal = Math.max(0.0, Math.min(1.0, Math.abs(activations[syn.fromNode] || 0.0)));
        const isTriggered = preVal > 0.02; // Lowered threshold from 0.05 to 0.02 for high sensitivity!

        const isExcitatory = syn.weight > 0;
        const baseColor = isExcitatory ? "16, 185, 129" : "239, 68, 68";

        const weightFactor = Math.min(0.08 + (absWeight / 2.0) * 0.77, 0.85);
        
        // Active synapses light up 100% solid, quiet ones fade out!
        const opacity = isTriggered ? 1.0 : weightFactor * 0.48; // Raised quiet multiplier from 0.35 to 0.48!
        const strokeWidth = Math.min(0.4 + (absWeight / 2.0) * 1.4, 2.0) * (isTriggered ? 1.6 : 1.0);

        el.setAttribute("stroke", `rgba(${baseColor}, ${opacity})`);
        el.setAttribute("stroke-width", strokeWidth.toString());
      }
    });
  }
}
