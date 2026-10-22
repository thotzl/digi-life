import type { BrainTopology } from "../../bindings/BrainTopology";
import type { CTRNNNeuron } from "../../bindings/CTRNNNeuron";
import type { CTRNNSynapse } from "../../bindings/CTRNNSynapse";
import type { NeuronRole } from "../../bindings/NeuronRole";
import { escapeHtml } from "../core/escape";
import { describeRole, describeSynapse, RoleContext } from "../shared/genome";

type RoleKind = NeuronRole["role"];

/** Visual style per neuron role: column x (fraction of the drawing area), colours and radii. */
const ROLE_STYLE: Record<RoleKind, { x: number; fill: string; glow: string; radius: number; inputLike: boolean }> = {
  sensor: { x: 0.0, fill: "#38bdf8", glow: "#38bdf8", radius: 2.2, inputLike: true },
  systemic: { x: 0.0, fill: "#0ea5e9", glow: "#0ea5e9", radius: 4.5, inputLike: true },
  thalamus: { x: 0.28, fill: "#0ea5e9", glow: "#0ea5e9", radius: 3.8, inputLike: false },
  hidden: { x: 0.7, fill: "#94a3b8", glow: "#e2e8f0", radius: 3.2, inputLike: false },
  motor: { x: 1.0, fill: "#c084fc", glow: "#c084fc", radius: 4.5, inputLike: false },
};

/** Gap (in node slots) between sensor groups of different organs and before the systemic inputs. */
const GROUP_GAP = 0.6;

function groupKey(role: NeuronRole): string {
  if (role.role === "sensor") return `s:${role.gene.promoter}:${role.gene.spelling}:${role.gene.occurrence}:${role.side}`;
  if (role.role === "thalamus") return `t:${role.gene.promoter}:${role.gene.spelling}:${role.gene.occurrence}:${role.node}`;
  return role.role;
}

/** Vertical positions in [0, 1] for one column, with gaps where the group changes. */
function columnYs(nodes: CTRNNNeuron[]): number[] {
  const slots: number[] = [];
  let pos = 0;
  nodes.forEach((n, i) => {
    if (i > 0 && groupKey(n.role) !== groupKey(nodes[i - 1].role)) pos += GROUP_GAP;
    slots.push(pos);
    pos += 1;
  });
  const total = pos - 1;
  return slots.map(s => (total <= 0 ? 0.5 : 0.04 + (s / total) * 0.92));
}

/**
 * Neural inspector (TCK-187): lays out and styles neurons by their backend role (inputs → thalamus → hidden →
 * motors) and draws the phenotype's synapse list; tooltips name roles and the owner gene of each synapse.
 */
export class BrainRenderer {
  public static readonly LEFT_MARGIN = 25;
  public static readonly RIGHT_MARGIN = 285;
  public static readonly AREA_WIDTH = BrainRenderer.RIGHT_MARGIN - BrainRenderer.LEFT_MARGIN;

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

  private nodeId(n: CTRNNNeuron): string {
    return `${this.elementIdPrefix}-node-${n.id}`;
  }

  private synapseId(index: number): string {
    return `${this.elementIdPrefix}-syn-${index}`;
  }

  public compile(brain: BrainTopology | null | undefined, ctx: RoleContext): void {
    // Lazy Initialization zur Absicherung gegen Vite Hot-Module-Replacement (HMR) State Preservation Bugs!
    if (!this.elementCache) this.elementCache = new Map<string, SVGElement>();
    if (!this.compiledCoords) this.compiledCoords = new Map<number, { x: number, y: number }>();

    this.elementCache.clear();
    this.compiledCoords.clear();
    if (!brain || brain.neurons.length === 0) {
      this.container.innerHTML = `<div class="fallback-state">No genetically encoded CTRNN brain found.</div>`;
      return;
    }

    const place = (n: CTRNNNeuron, x: number, y: number) => this.compiledCoords.set(n.id, {
      x: BrainRenderer.LEFT_MARGIN + x * BrainRenderer.AREA_WIDTH,
      y: BrainRenderer.TOP_MARGIN + y * BrainRenderer.AREA_HEIGHT
    });
    const byRole = (...roles: RoleKind[]) => brain.neurons.filter(n => roles.includes(n.role.role));

    // Inputs: sensors grouped per organ (incl. the ΔI channel), then the systemic inputs
    const inputs = byRole("sensor", "systemic");
    columnYs(inputs).forEach((y, i) => place(inputs[i], ROLE_STYLE[inputs[i].role.role].x, y));
    const thalamus = byRole("thalamus");
    columnYs(thalamus).forEach((y, i) => place(thalamus[i], ROLE_STYLE.thalamus.x, y));
    // Hidden neurons fan out by their depth (the backend sends it as y)
    const hidden = byRole("hidden");
    hidden.forEach((n, i) => place(n, n.y ?? ROLE_STYLE.hidden.x, (i + 1) / (hidden.length + 1)));
    const motors = byRole("motor");
    motors.forEach((n, i) => place(n, ROLE_STYLE.motor.x, (i + 1) / (motors.length + 1)));

    let svgContent = `<svg width="100%" height="100%" viewBox="0 0 310 240" style="background:#020617;">`;

    brain.synapses.forEach((syn: CTRNNSynapse, index: number) => {
      const from = this.compiledCoords.get(syn.fromNode);
      const to = this.compiledCoords.get(syn.toNode);
      if (!from || !to) return;

      const strokeColor = syn.weight > 0 ? "rgba(16, 185, 129, 0.85)" : "rgba(239, 68, 68, 0.85)";
      const absWeight = Math.abs(syn.weight);
      const displayStyle = absWeight === 0.0 ? "display: none;" : "";
      const weightFactor = Math.min(0.08 + (absWeight / 2.0) * 0.77, 0.85);
      const strokeWidth = Math.min(0.4 + (absWeight / 2.0) * 1.4, 2.0);

      svgContent += `
        <line id="${this.synapseId(index)}" x1="${from.x}" y1="${from.y}" x2="${to.x}" y2="${to.y}"
              stroke="${strokeColor}" stroke-width="${strokeWidth}" style="opacity: ${weightFactor}; ${displayStyle}"><title>${escapeHtml(describeSynapse(syn, brain, ctx))}</title></line>
      `;
    });

    brain.neurons.forEach((n: CTRNNNeuron) => {
      const coord = this.compiledCoords.get(n.id);
      if (!coord) return;
      const style = ROLE_STYLE[n.role.role];
      const detail = n.role.role === "sensor" || n.role.role === "systemic"
        ? ""
        : `\ntau ${n.tau.toFixed(2)}, bias ${n.bias.toFixed(2)}${n.activationType ? `, ${n.activationType}` : ""}`;
      svgContent += `
        <circle id="${this.nodeId(n)}" cx="${coord.x}" cy="${coord.y}" r="${style.radius}" fill="${style.fill}"
                style="cursor:pointer; filter:drop-shadow(0 0 2px ${style.fill}); opacity: 0.25;"><title>${escapeHtml(describeRole(n.role, ctx) + detail)}</title></circle>
      `;
    });

    svgContent += `</svg>`;
    this.container.innerHTML = svgContent;

    // Cache elements references for high-performance glows
    brain.neurons.forEach((n: CTRNNNeuron) => {
      const el = document.getElementById(this.nodeId(n)) as SVGElement | null;
      if (el) this.elementCache.set(this.nodeId(n), el);
    });
    brain.synapses.forEach((_syn: CTRNNSynapse, index: number) => {
      const el = document.getElementById(this.synapseId(index)) as SVGElement | null;
      if (el) this.elementCache.set(this.synapseId(index), el);
    });

    if (this.onNeuronHover) {
      brain.neurons.forEach((n: CTRNNNeuron) => {
        const el = this.elementCache.get(this.nodeId(n));
        if (!el) return;
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
      });
    }

    if (this.onSynapseHover) {
      brain.synapses.forEach((syn: CTRNNSynapse, index: number) => {
        const el = this.elementCache.get(this.synapseId(index));
        if (!el) return;
        el.addEventListener("mouseenter", () => {
          this.onSynapseHover?.(syn.fromNode, syn.toNode, syn.weight);
          el.setAttribute("stroke-width", "3.0");
          el.style.opacity = "1.0";
        });
        el.addEventListener("mouseleave", () => {
          this.onSynapseHover?.(0, 0, null);
          el.removeAttribute("stroke-width");
        });
      });
    }
  }

  public updateLiveGlows(activations: number[], brain: BrainTopology): void {
    if (!brain || !activations) return;

    // 1. Update Neurons (Instant 100% glow on trigger, else dimmed to 0.22 for subtle background visibility)
    brain.neurons.forEach((n: CTRNNNeuron) => {
      const el = this.elementCache.get(this.nodeId(n));
      if (!el) return;
      const style = ROLE_STYLE[n.role.role];
      const rawAct = Math.max(0.0, Math.min(1.0, Math.abs(activations[n.id] || 0.0)));
      const exponent = (this.elementIdPrefix === "trainer" && style.inputLike) ? 1.5 : 4.0;
      const act = Math.pow(rawAct, exponent);
      const radius = act > 0.45 ? style.radius + 2.0 : style.radius;

      el.setAttribute("fill", style.glow);
      el.setAttribute("r", radius.toString());

      const isTriggered = rawAct > 0.02;
      el.style.opacity = (isTriggered ? 1.0 : 0.22).toString();
      el.style.filter = isTriggered ? `drop-shadow(0 0 5px ${style.glow})` : "none";
    });

    // 2. Update Synapses (Instant 100% bright track on trigger, else faded to paper-thin rest)
    brain.synapses.forEach((syn: CTRNNSynapse, index: number) => {
      const el = this.elementCache.get(this.synapseId(index));
      if (!el) return;
      const absWeight = Math.abs(syn.weight);
      if (absWeight === 0.0) {
        el.style.display = "none";
        return;
      }

      const preVal = Math.max(0.0, Math.min(1.0, Math.abs(activations[syn.fromNode] || 0.0)));
      const isTriggered = preVal > 0.02;
      const baseColor = syn.weight > 0 ? "16, 185, 129" : "239, 68, 68";
      const weightFactor = Math.min(0.08 + (absWeight / 2.0) * 0.77, 0.85);

      // Active synapses light up 100% solid, quiet ones fade out!
      const opacity = isTriggered ? 1.0 : weightFactor * 0.48;
      const strokeWidth = Math.min(0.4 + (absWeight / 2.0) * 1.4, 2.0) * (isTriggered ? 1.6 : 1.0);

      el.setAttribute("stroke", `rgba(${baseColor}, ${opacity})`);
      el.setAttribute("stroke-width", strokeWidth.toString());
    });
  }
}
