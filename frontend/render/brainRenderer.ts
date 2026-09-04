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

  public getNeuronX(id: number, K: number, depth?: number | null): number {
    if (id < K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT) {
      return BrainRenderer.LEFT_MARGIN; // Left Column: Sensors (K organelles * 5 + 6 base inputs)
    }
    if (
      id >= K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT && 
      id < K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT + BrainRenderer.MOTOR_OUTPUT_NODES_COUNT
    ) {
      return BrainRenderer.RIGHT_MARGIN; // Right Column: Motors (4 Output nodes)
    }
    if (depth !== undefined && depth !== null) {
      return BrainRenderer.LEFT_MARGIN + depth * BrainRenderer.AREA_WIDTH; // Scale horizontal coordinate dynamically based on compiled depth!
    }
    return BrainRenderer.CENTER_X; // Center Column: Interneurons
  }

  public getNeuronY(id: number, K: number, totalNeurons: number): number {
    if (id < K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT) {
      const step = BrainRenderer.AREA_HEIGHT / (K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT);
      return BrainRenderer.TOP_MARGIN + (id + 1) * step;
    }
    if (
      id >= K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT && 
      id < K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT + BrainRenderer.MOTOR_OUTPUT_NODES_COUNT
    ) {
      const motorIdx = id - (K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT);
      const step = BrainRenderer.AREA_HEIGHT / (BrainRenderer.MOTOR_OUTPUT_NODES_COUNT + 1);
      return BrainRenderer.TOP_MARGIN + (motorIdx + 1) * step;
    }
    const totalInputsAndOutputs = K + BrainRenderer.SYSTEMIC_BASE_INPUTS_COUNT + BrainRenderer.MOTOR_OUTPUT_NODES_COUNT;
    const interIdx = id - totalInputsAndOutputs;
    const numInter = totalNeurons - totalInputsAndOutputs;
    const step = BrainRenderer.AREA_HEIGHT / (numInter + 1);
    return BrainRenderer.TOP_MARGIN + (interIdx + 1) * step;
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

      const fromNeuron = brain.neurons.find(n => n.id === fromId);
      const toNeuron = brain.neurons.find(n => n.id === toId);
      const fromDepth = fromNeuron ? fromNeuron.y : null;
      const toDepth = toNeuron ? toNeuron.y : null;

      const fromX = this.getNeuronX(fromId, K, fromDepth);
      const fromY = this.getNeuronY(fromId, K, brain.neurons.length);
      const toX = this.getNeuronX(toId, K, toDepth);
      const toY = this.getNeuronY(toId, K, brain.neurons.length);

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
      const nx = this.getNeuronX(n.id, K, n.y);
      const ny = this.getNeuronY(n.id, K, brain.neurons.length);

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
