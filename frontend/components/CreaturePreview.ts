// Animated creature preview (TCK-160): one canvas that draws a phenotype facing north, used by the inspector panel
// and the catalogue picker (formerly two copies of the same animation loop). The loop runs only inside the scope
// passed to `start`, so hidden panels and closed dialogs draw nothing.
import type { CreaturePhenotype } from "../../bindings/CreaturePhenotype";
import type { Scope } from "../core/scope";
import { CreatureRenderer } from "../render/creatureRenderer";

export interface CreaturePreviewOptions {
  /** Canvas size in px (square) */
  size: number;
  /** Fraction of the canvas the half body length fills */
  bodyFraction: number;
  /** Style of the canvas element (border, radius, ...) */
  style?: Partial<CSSStyleDeclaration>;
}

/** Animation time step per frame (the renderer's time unit). */
const TIME_STEP = 0.045;
const FALLBACK_BODY_LENGTH = 130;

export class CreaturePreview {
  readonly element: HTMLCanvasElement;
  private readonly ctx: CanvasRenderingContext2D | null;
  private readonly renderer: CreatureRenderer | null;
  private phenotype: CreaturePhenotype | null = null;
  private time = 0;

  constructor(private readonly options: CreaturePreviewOptions) {
    this.element = document.createElement("canvas");
    this.element.width = options.size;
    this.element.height = options.size;
    if (options.style) Object.assign(this.element.style, options.style);
    this.ctx = this.element.getContext("2d");
    this.renderer = this.ctx ? new CreatureRenderer(this.element) : null;
  }

  setPhenotype(phenotype: CreaturePhenotype | null) {
    this.phenotype = phenotype;
  }

  /** Animates until `scope` is disposed. */
  start(scope: Scope) {
    scope.frame(() => this.draw());
  }

  private draw() {
    this.time += TIME_STEP;
    const p = this.phenotype;
    const ctx = this.ctx;
    if (!ctx || !this.renderer || !p) return;
    const size = this.options.size;
    ctx.fillStyle = "#020617";
    ctx.fillRect(0, 0, size, size);
    const baseLength = p.spinalHarmonics.baseLength || FALLBACK_BODY_LENGTH;
    const scale = (size * this.options.bodyFraction) / (baseLength * 0.5);
    ctx.save();
    ctx.translate(size / 2, size / 2);
    ctx.scale(scale, scale);
    this.renderer.render(p, this.time, 0, 0, -Math.PI / 2, 0);
    ctx.restore();
  }
}
