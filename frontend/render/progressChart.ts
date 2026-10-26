import type { ProgressPoint } from "../trainerMetrics";

const COLORS = {
  grid: "rgba(148, 163, 184, 0.12)",
  text: "#64748b",
  success: "#10b981",
  best: "#f59e0b",
  median: "#00f2fe",
  level: "rgba(192, 132, 252, 0.55)",
};

/**
 * Progress curve of the current run (TCK-177): success rate and difficulty level on a fixed 0–100 % scale, best and
 * median fitness scaled to the run's fitness range. Drawn only when a generation completes, so it stays cheap.
 */
export function drawProgressChart(canvas: HTMLCanvasElement, points: ProgressPoint[]) {
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  const w = canvas.width;
  const h = canvas.height;
  const pad = { left: 6, right: 6, top: 8, bottom: 16 };
  const plotW = w - pad.left - pad.right;
  const plotH = h - pad.top - pad.bottom;

  ctx.clearRect(0, 0, w, h);
  ctx.font = "16px monospace";
  ctx.textBaseline = "alphabetic";

  ctx.strokeStyle = COLORS.grid;
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (const f of [0, 0.5, 1]) {
    const y = pad.top + plotH * (1 - f);
    ctx.moveTo(pad.left, y);
    ctx.lineTo(pad.left + plotW, y);
  }
  ctx.stroke();

  if (points.length === 0) {
    ctx.fillStyle = COLORS.text;
    ctx.textAlign = "center";
    ctx.fillText("No generation completed yet", w / 2, pad.top + plotH / 2 + 5);
    return;
  }

  const g0 = points[0].generation;
  const g1 = points[points.length - 1].generation;
  const x = (g: number) => pad.left + (g1 > g0 ? ((g - g0) / (g1 - g0)) * plotW : plotW / 2);
  const yUnit = (v: number) => pad.top + plotH * (1 - Math.min(1, Math.max(0, v)));
  const fits = points.flatMap((p) => [p.bestFitness, p.medianFitness]).filter(Number.isFinite);
  const lo = Math.min(0, ...fits);
  const hi = Math.max(lo + 1e-6, ...fits);
  const yFit = (v: number) => yUnit((v - lo) / (hi - lo));

  const line = (color: string, y: (p: ProgressPoint) => number, width = 2) => {
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.beginPath();
    points.forEach((p, i) => (i === 0 ? ctx.moveTo(x(p.generation), y(p)) : ctx.lineTo(x(p.generation), y(p))));
    if (points.length === 1) ctx.lineTo(x(points[0].generation) + 1, y(points[0]));
    ctx.stroke();
  };
  line(COLORS.level, (p) => yUnit(p.level), 1.5);
  line(COLORS.median, (p) => yFit(p.medianFitness));
  line(COLORS.best, (p) => yFit(p.bestFitness));
  line(COLORS.success, (p) => yUnit(p.successRate), 2.5);

  ctx.fillStyle = COLORS.text;
  ctx.textAlign = "left";
  ctx.fillText(`Gen ${g0}`, pad.left, h - 2);
  ctx.textAlign = "right";
  ctx.fillText(`Gen ${g1} · fit max ${hi.toFixed(2)}`, w - pad.right, h - 2);
}
