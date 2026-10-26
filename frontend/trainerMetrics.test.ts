import { describe, expect, it } from "vitest";
import type { TrainerGenerationStats } from "../bindings/TrainerGenerationStats";
import { ProgressHistory, formatFirstCatch, formatLevel, formatRate, modeChoice, pendingSwitchLabel } from "./trainerMetrics";

const stats = (generation: number, successRate = 0.5): TrainerGenerationStats => ({
  generation,
  successRate,
  levelSuccessRate: successRate,
  rehearsalSuccessRate: null,
  rehearsalTrials: 0,
  bestFitness: 3,
  medianFitness: 1,
  meanFitness: 1.2,
  meanTimeToFirstCatch: 120,
  medianTimeToFirstCatch: 90,
  immobileShare: 0,
  starvedShare: 0,
  scenario: "curriculum",
  level: 0.2,
  levelMove: "up",
  mutationOpsMax: 3,
  layoutId: "18446744073709551615",
});

describe("trainer metrics formatting", () => {
  it("formats rates, first catch in ticks and seconds, and levels", () => {
    expect(formatRate(0.4338)).toBe("43.4 %");
    expect(formatFirstCatch(90)).toBe("90 t (1.5 s)");
    expect(formatFirstCatch(null)).toBe("–");
    expect(formatLevel(0.3, "curriculum", true)).toBe("30 % (auto)");
    expect(formatLevel(0.3, "curriculum", false)).toBe("30 % (manual)");
    expect(formatLevel(1, "exploration", true)).toBe("100 % (fixed)");
  });

  it("maps a curriculum with the gate off to the manual choice and labels pending switches", () => {
    expect(modeChoice("curriculum", true)).toBe("curriculum");
    expect(modeChoice("curriculum", false)).toBe("manual");
    expect(modeChoice("standard", false)).toBe("standard");
    expect(pendingSwitchLabel(null)).toBe("");
    expect(pendingSwitchLabel("exploration")).toContain("next generation");
  });
});

describe("progress history", () => {
  it("appends generations and restarts on a generation that is not after the last one", () => {
    const h = new ProgressHistory();
    h.add(stats(1, 0.1));
    h.add(stats(2, 0.2));
    expect(h.points.map((p) => p.generation)).toEqual([1, 2]);
    h.add(stats(1, 0.3));
    expect(h.points).toEqual([{ generation: 1, successRate: 0.3, bestFitness: 3, medianFitness: 1, level: 0.2 }]);
    h.clear();
    expect(h.points).toEqual([]);
  });
});
