// Organ painters (TCK-160): how each `OrganKind` marks the body surface, one entry per kind of the backend organ
// registry (`bindings/OrganKind.ts`, contract wiki/contracts/organ-kind.md). The kind comes from the backend; the
// renderer no longer re-derives it from affinity thresholds. The body shape of an organ (limb, head bud, hillock)
// follows the backend's limb rule (`Organ::is_limb`): a limb-capable kind (`ORGAN_KINDS[kind].limb`) with a style of
// at least `LIMB_STYLE`; distance senses never grow as limbs (TCK-202).
import type { HSLColor } from "../../bindings/HSLColor";
import type { Organ } from "../../bindings/Organ";
import type { OrganKind } from "../../bindings/OrganKind";
import { ORGAN_KINDS } from "../../bindings/organRegistry";

/** Expression style from which an organ of a limb-capable kind is a fleshy limb (`LIMB_STYLE` in `src/biology/organs/mod.rs`). */
export const LIMB_STYLE = 0.72;
/** Below this style an organ is a flat dome on the skin (lens, pit). */
export const DOME_STYLE = 0.3;
/** Spinal position below which a non-limb organ grows as a head bud. */
export const HEAD_REGION = 0.25;
/** Mid-style organs narrower than this bandwidth show a glowing bead. */
export const BEAD_BANDWIDTH = 0.35;

/** Membrane shape of an organ: fleshy limb, cephalic bud or ciliated hillock. */
export type OrganShape = "limb" | "bud" | "hillock";

/** The backend's `Organ::is_limb`. */
export function isLimb(organ: Pick<Organ, "kind" | "expressionStyle">): boolean {
  return ORGAN_KINDS[organ.kind].limb && organ.expressionStyle >= LIMB_STYLE;
}

export function organShape(organ: Organ): OrganShape {
  if (isLimb(organ)) return "limb";
  return organ.spinalPos < HEAD_REGION ? "bud" : "hillock";
}

/** Angle (radians, local body frame) the organ points to at `time`: its genome angle plus the sway. */
export function organAngle(organ: Organ, wiggleAmplitude: number, time: number, swayMultiplier: number): number {
  const target = ((90.0 - organ.angle) * Math.PI) / 180;
  const phase = organ.spinalPos * 4.0 + target * 0.03;
  const freq = organ.spectralAffinity > 0.7 ? 0.005 : 0.0035;
  return target + wiggleAmplitude * swayMultiplier * Math.sin(time * freq + phase);
}

export interface OrganPainter {
  /** Surface mark of a flat organ (`expressionStyle < DOME_STYLE`), drawn at the organ's surface point with the
   * outward normal pointing to -y; `null` = no mark. */
  dome: ((ctx: CanvasRenderingContext2D, organ: Organ, hue: HSLColor) => void) | null;
  /** Colour of the bead shown by a mid-style organ with a narrow band */
  bead: string;
}

function eyeLens(ctx: CanvasRenderingContext2D, organ: Organ, hue: HSLColor) {
  const baseWidth = 10 * organ.scale;
  const lens = ctx.createRadialGradient(-baseWidth * 0.2, -baseWidth * 0.3, 1, 0, 0, baseWidth);
  lens.addColorStop(0, "#ffffff");
  lens.addColorStop(0.3, `hsl(${hue.h}, 100%, 75%)`);
  lens.addColorStop(1, "rgba(255,255,255,0)");
  ctx.fillStyle = lens;
  ctx.fill();

  ctx.beginPath();
  ctx.arc(-baseWidth * 0.4, -baseWidth * 0.4, 2.2 * organ.scale, 0, Math.PI * 2);
  ctx.fillStyle = "#ffffff";
  ctx.fill();
}

function thermalPit(ctx: CanvasRenderingContext2D, organ: Organ) {
  const pit = ctx.createRadialGradient(0, 0, 1, 0, 0, 10 * organ.scale);
  pit.addColorStop(0, "#000000");
  pit.addColorStop(0.5, "hsl(350, 95%, 45%)");
  pit.addColorStop(1, "rgba(255,0,0,0)");
  ctx.fillStyle = pit;
  ctx.fill();
}

const TACTILE_BEAD = "hsl(40, 100%, 70%)";

/** One painter per organ kind; a new `OrganKind` fails to compile until it has an entry. */
export const ORGAN_PAINTERS: Record<OrganKind, OrganPainter> = {
  eye: { dome: eyeLens, bead: "#ffffff" },
  thermal: { dome: thermalPit, bead: "hsl(340, 100%, 70%)" },
  nose: { dome: null, bead: "hsl(120, 100%, 70%)" },
  bristle: { dome: null, bead: TACTILE_BEAD },
  lateralLine: { dome: null, bead: TACTILE_BEAD },
  proprioceptor: { dome: null, bead: TACTILE_BEAD },
};

/**
 * Draws the surface mark of `organ`; the context is translated to the organ's surface point and rotated so the
 * outward normal is -y. Flat organs get their kind's dome, other non-limb organs with a narrow band their kind's bead,
 * limbs a tip in the organ's hue.
 */
export function paintOrganMark(ctx: CanvasRenderingContext2D, organ: Organ, bodyColor: HSLColor) {
  const hue: HSLColor = { h: (bodyColor.h + organ.hueShift + 360) % 360, s: bodyColor.s, l: bodyColor.l };
  const painter = ORGAN_PAINTERS[organ.kind];
  const style = organ.expressionStyle;
  if (style < DOME_STYLE) {
    if (!painter.dome) return;
    ctx.beginPath();
    ctx.arc(0, 0, 10 * organ.scale - 1, Math.PI, 0);
    ctx.closePath();
    painter.dome(ctx, organ, hue);
  } else if (!isLimb(organ)) {
    if (organ.bandwidth >= BEAD_BANDWIDTH) return;
    const r = 3.5 * organ.scale;
    ctx.beginPath();
    ctx.arc(0, -r, r, 0, Math.PI * 2);
    ctx.fillStyle = painter.bead;
    ctx.shadowColor = painter.bead;
    ctx.shadowBlur = 10;
    ctx.fill();
  } else {
    const tip = `hsl(${hue.h}, 95%, 65%)`;
    ctx.beginPath();
    ctx.arc(0, -6, 5 * organ.scale, 0, Math.PI * 2);
    ctx.fillStyle = tip;
    ctx.shadowColor = tip;
    ctx.shadowBlur = 8;
    ctx.fill();
  }
}
