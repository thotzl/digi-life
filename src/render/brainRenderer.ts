import { BrainTopology, CTRNNNeuron, CTRNNSynapse } from "../shared/types";

export class BrainRenderer {
  private elementCache = new Map<string, SVGElement>();

  constructor(
    private container: HTMLDivElement, 
    private elementIdPrefix: string,
    private onNeuronHover?: (id: number | null) => void,
    private onSynapseHover?: (from: number, to: number, weight: number | null) => void
  ) {}

  public getNeuronX(id: number, K: number): number {
    if (id <= K) return 25; // Left Column: Sensors
    if (id >= K + 1 && id <= K + 4) return 280; // Right Column: Motors
    return 150; // Center Column: Interneurons
  }

  public getNeuronY(id: number, K: number, totalNeurons: number): number {
    if (id <= K) {
      const step = 200 / (K + 1);
      return 20 + (id + 1) * step;
    }
    if (id >= K + 1 && id <= K + 4) {
      const motorIdx = id - (K + 1);
      const step = 200 / 5;
      return 20 + (motorIdx + 1) * step;
    }
    const interIdx = id - K - 5;
    const numInter = totalNeurons - K - 5;
    const step = 200 / (numInter + 1);
    return 20 + (interIdx + 1) * step;
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

      const fromX = this.getNeuronX(fromId, K);
      const fromY = this.getNeuronY(fromId, K, brain.neurons.length);
      const toX = this.getNeuronX(toId, K);
      const toY = this.getNeuronY(toId, K, brain.neurons.length);

      const isExcitatory = syn.weight > 0;
      const strokeColor = isExcitatory ? "rgba(16, 185, 129, 0.28)" : "rgba(239, 68, 68, 0.28)";

      const fromLabel = brain.neurons.find(n => n.id === fromId)?.label || `Node ${fromId}`;
      const toLabel = brain.neurons.find(n => n.id === toId)?.label || `Node ${toId}`;

      svgContent += `
        <line id="${this.elementIdPrefix}-syn-${fromId}-${toId}" x1="${fromX}" y1="${fromY}" x2="${toX}" y2="${toY}"
              stroke="${strokeColor}" stroke-width="1.2" />
      `;
    });

    // Draw neurons (always 3-column layout!)
    brain.neurons.forEach((n: CTRNNNeuron) => {
      const nx = this.getNeuronX(n.id, K);
      const ny = this.getNeuronY(n.id, K, brain.neurons.length);

      const isInput = n.type === "input";
      const isOutput = n.type === "output";
      const fill = isInput ? "#0ea5e9" : (isOutput ? "#c084fc" : "#94a3b8");
      const radius = isInput || isOutput ? 4.5 : 3.2;

      svgContent += `
        <circle id="${this.elementIdPrefix}-node-${n.id}" cx="${nx}" cy="${ny}" r="${radius}" fill="${fill}"
                style="cursor:pointer; filter:drop-shadow(0 0 2px ${fill});" />
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
            el.setAttribute("stroke-opacity", "1.0");
          });
          el.addEventListener("mouseleave", () => {
            this.onSynapseHover?.(0, 0, null);
            el.removeAttribute("stroke-width");
            el.removeAttribute("stroke-opacity");
          });
        }
      });
    }
  }

  public updateLiveGlows(activations: number[], brain: BrainTopology): void {
    if (!brain || !activations) return;

    // 1. Update Neurons
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
        const fill = act > 0.35 ? colorGlow : "#111827";
        const radius = isInput || isOutput ? (act > 0.45 ? 6.5 : 4.5) : (act > 0.45 ? 5.0 : 3.2);

        el.setAttribute("fill", fill);
        el.setAttribute("r", radius.toString());
      }
    });

    // 2. Update Synapses
    brain.synapses.forEach((syn: CTRNNSynapse) => {
      const id = `${this.elementIdPrefix}-syn-${syn.fromNode}-${syn.toNode}`;
      const el = this.elementCache.get(id);
      if (el) {
        const preVal = Math.max(0.0, Math.min(1.0, Math.abs(activations[syn.fromNode] || 0.0)));
        const act = Math.pow(preVal, 4.0);

        const isExcitatory = syn.weight > 0;
        const baseColor = isExcitatory ? "16, 185, 129" : "239, 68, 68";
        const opacity = act > 0.35 ? 0.95 : 0.28;
        const strokeWidth = Math.max(0.5, Math.abs(syn.weight) * 1.5) * (act > 0.45 ? 2.2 : 1.0);

        el.setAttribute("stroke", `rgba(${baseColor}, ${opacity})`);
        el.setAttribute("stroke-width", strokeWidth.toString());
      }
    });
  }
}
