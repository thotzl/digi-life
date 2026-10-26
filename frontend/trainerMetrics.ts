import type { Scenario } from "../bindings/Scenario";
import type { TrainerGenerationStats } from "../bindings/TrainerGenerationStats";

/** Simulation ticks per simulated second (the trainer steps at 60 Hz at warp 1). */
export const TICKS_PER_SECOND = 60;

/** Value of the training mode select: the three modes, plus "manual" for a curriculum level with the gate off. */
export type TrainingModeChoice = Scenario | "manual";

export const TRAINING_MODE_LABELS: Record<TrainingModeChoice, string> = {
  standard: "Standard",
  exploration: "Exploration",
  curriculum: "Curriculum (auto)",
  manual: "Manual Level",
};

export function modeChoice(mode: Scenario, levelAuto: boolean): TrainingModeChoice {
  return mode === "curriculum" && !levelAuto ? "manual" : mode;
}

export function formatRate(rate: number): string {
  return `${(rate * 100).toFixed(1)} %`;
}

/** Time to first catch in ticks with the seconds at 60 ticks/s; "–" without any catch. */
export function formatFirstCatch(ticks: number | null): string {
  if (ticks === null || !Number.isFinite(ticks)) return "–";
  return `${Math.round(ticks)} t (${(ticks / TICKS_PER_SECOND).toFixed(1)} s)`;
}

/** Difficulty level as percent plus how it is set: fixed room, automatic gate or manual. */
export function formatLevel(level: number, mode: Scenario, levelAuto: boolean): string {
  const pct = `${Math.round(level * 100)} %`;
  if (mode !== "curriculum") return `${pct} (fixed)`;
  return `${pct} (${levelAuto ? "auto" : "manual"})`;
}

/** Note for a room switch waiting for the generation boundary; empty without one. */
export function pendingSwitchLabel(pending: Scenario | null): string {
  return pending === null ? "" : `Pending: ${TRAINING_MODE_LABELS[pending]} – takes effect next generation`;
}

export interface ProgressPoint {
  generation: number;
  successRate: number;
  bestFitness: number;
  medianFitness: number;
  level: number;
}

/** Per-generation points of the current run for the progress curve. */
export class ProgressHistory {
  points: ProgressPoint[] = [];

  clear() {
    this.points = [];
  }

  /** Appends a generation; a generation not after the last one (other run, restored run) starts a new curve. */
  add(stats: TrainerGenerationStats) {
    const last = this.points[this.points.length - 1];
    if (last && stats.generation <= last.generation) this.clear();
    this.points.push({
      generation: stats.generation,
      successRate: stats.successRate,
      bestFitness: stats.bestFitness,
      medianFitness: stats.medianFitness,
      level: stats.level,
    });
  }
}
