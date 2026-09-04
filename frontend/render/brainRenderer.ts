import { BrainTopology, CTRNNNeuron, CTRNNSynapse } from "../shared/types";

export class BrainRenderer {
  public static readonly SYSTEMIC_BASE_INPUTS_COUNT = 6;
  public static readonly MOTOR_OUTPUT_NODES_COUNT = 4;

  public static readonly LEFT_MARGIN = 25;
  public static readonly RIGHT_MARGIN = 285;
  public static readonly AREA_WIDTH = BrainRenderer.RIGHT_MARGIN - BrainRenderer.LEFT_MARGIN;
  public static readonly CENTER_X = BrainRenderer.LEFT_MARGIN + BrainRenderer.AREA_WIDTH / 2;

  public static readonly TOP_MARGIN = 20;
  public static readonly AREA_HEIGHT = 200;

  private elementCache = new Map<string, SVGElement>();

  constructor(
    private container: HTMLDivElement, 
    private elementIdPrefix: string,
    private onNeuronHover?: (id: number | null) => void,
    private onSynapseHover?: (from: number, to: number, weight: number | null) => void
  ) {}

  public getNeuronX(id: number, _K: number, brain: BrainTopology): number {
    const n = brain.neurons.find(node => node.id === id);
    if (!n) return BrainRenderer.CENTER_X;

    if (n.type === "input") {
      return BrainRenderer.LEFT_MARGIN; // 25px (Ganz links)
    }
    if (n.type === "output") {
      return BrainRenderer.RIGHT_MARGIN; // 285px (Ganz rechts)
    }
    
    // Now we are in Hidden Neurons (Interneurons)
    const label = n.label || "";
    if (label.includes("Σ Sum") || label.includes("Δ Diff") || label.includes("Thalamus")) {
      return BrainRenderer.LEFT_MARGIN + 55; // Symmetrieknoten parallel auf Säule 2 (80px)
    }
    
    // Standard DNA Hiddens: scaled dynamically (staggered depth) as before!
    if (n.y !== undefined && n.y !== null) {
      return BrainRenderer.LEFT_MARGIN + n.y * BrainRenderer.AREA_WIDTH;
    }
    return BrainRenderer.CENTER_X;
  }

  public getNeuronY(id: number, K: number, brain: BrainTopology): number {
    const n = brain.neurons.find(node => node.id === id);
    if (!n) return BrainRenderer.TOP_MARGIN;

    if (n.type === "input") {
      const step = BrainRenderer.AREA_HEIGHT / (K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT + 1);
      return BrainRenderer.TOP_MARGIN + (n.id + 1) * step;
    }
    if (n.type === "output") {
      const motorIdx = n.id - (K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT);
      const step = BrainRenderer.AREA_HEIGHT / (BrainRenderer.MOTOR_OUTPUT_NODES_COUNT + 1);
      return BrainRenderer.TOP_MARGIN + (motorIdx + 1) * step;
    }
    
    // Now we are in Hidden Neurons
    const label = n.label || "";
    if (label.includes("Σ Sum") || label.includes("Δ Diff") || label.includes("Thalamus")) {
      // Space HOX fusions beautifully and parallelly in their own vertical column!
      const fusions = brain.neurons.filter(node => node.type === "hidden" && (node.label.includes("Σ Sum") || node.label.includes("Δ Diff") || node.label.includes("Thalamus")));
      const idx = fusions.findIndex(node => node.id === n.id);
      const step = BrainRenderer.AREA_HEIGHT / (fusions.length + 1);
      return BrainRenderer.TOP_MARGIN + (idx + 1) * step;
    } else {
      // Space standard DNA hidden neurons beautifully in their own vertical column!
      const dnaHiddens = brain.neurons.filter(node => node.type === "hidden" && !(node.label.includes("Σ Sum") || node.label.includes("Δ Diff") || node.label.includes("Thalamus")));
      const idx = dnaHiddens.findIndex(node => node.id === n.id);
      const step = BrainRenderer.AREA_HEIGHT / (dnaHiddens.length + 1);
      return BrainRenderer.TOP_MARGIN + (idx + 1) * step;
    }
  }

  public compile(brain: BrainTopology, K: number): void {
    this.elementCache.clear();
    if (!brain || brain.neurons.length === 0) {
      this.container.innerHTML = `<div class="fallback-state">No genetically encoded CTRNN brain found.</div>`;
      return;
    }

    let svgContent = `<svg width="100%" height="100%" viewBox="0 0 310 240" style="background:#020617;">`;

    // Draw synapses (always 3-column layout, no arrow markers!)
    brain.synapses.forEach((syn: CTRNNSynapse) => {
      const fromId = syn.fromNode;
      const toId = syn.toNode;

      const fromX = this.getNeuronX(fromId, K, brain);
      const fromY = this.getNeuronY(fromId, K, brain);
      const toX = this.getNeuronX(toId, K, brain);
      const toY = this.getNeuronY(toId, K, brain);

      const isExcitatory = syn.weight > 0;
      const strokeColor = isExcitatory ? "rgba(16, 185, 129, 0.85)" : "rgba(239, 68, 68, 0.85)";

      const absWeight = Math.abs(syn.weight);
      const displayStyle = absWeight === 0.0 ? "display: none;" : "";
      const weightFactor = Math.min(0.08 + (absWeight / 2.0) * 0.77, 0.85);
      const strokeWidth = Math.min(0.4 + (absWeight / 2.0) * 1.4, 2.0);

      svgContent += `
        <line id="${this.elementIdPrefix}-syn-${fromId}-${toId}" x1="${fromX}" y1="${fromY}" x2="${toX}" y2="${toY}"
              stroke="${strokeColor}" stroke-width="${strokeWidth}" style="opacity: ${weightFactor}; ${displayStyle}" />
      `;
    });

    // Draw neurons (always 3-column layout!)
    brain.neurons.forEach((n: CTRNNNeuron) => {
      const nx = this.getNeuronX(n.id, K, brain);
      const ny = this.getNeuronY(n.id, K, brain);

      const isInput = n.type === "input";
      const isOutput = n.type === "output";
      const fill = isInput ? "#0ea5e9" : (isOutput ? "#c084fc" : "#94a3b8");
      const radius = isInput || isOutput ? 4.5 : 3.2;

      svgContent += `
        <circle id="${this.elementIdPrefix}-node-${n.id}" cx="${nx}" cy="${ny}" r="${radius}" fill="${fill}"
                style="cursor:pointer; filter:drop-shadow(0 0 2px ${fill}); opacity: 0.25;" />
      `;
    });

    svgContent += `</svg>`;
    this.container.innerHTML = svgContent;

    // Cache elements references
    brain.neurons.forEach((n: CTRNNNeuron) => {
      const id = `${this.elementIdPrefix}-node-${n.id}`;
      const el = document.getElementById(id) as SVGElement | null;
      if (el) this.elementCache.set(id, el);
    });

    brain.synapses.forEach((syn: CTRNNSynapse) => {
      const id = `${this.elementIdPrefix}-syn-${syn.fromNode}-${syn.toNode}`;
      const el = document.getElementById(id) as SVGElement | null;
      if (el) this.elementCache.set(id, el);
    });

    // Add event listeners for hover tooltips on neuron nodes
    if (this.onNeuronHover) {
      brain.neurons.forEach((n: CTRNNNeuron) => {
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
      brain.synapses.forEach((syn: CTRNNSynapse) => {
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
