import { describe, expect, it } from "vitest";

import { TrainerSettings } from "./settings";

const state: any = {
  isRunning: false, population: 200, N: 25, warpSpeed: 50, eliteRatio: 0.3, mutationRate: 4, mutationOpsMin: 2,
  mutationOpsLimit: 8, inflowRate: 0.1, hofRate: 0.2, multiTrial: false, isHeadless: false, runId: "run_b",
  lamarckian: true, trainingMode: "standard", level: 0, levelAuto: false, pendingMode: null, generation: 9,
};

describe("trainer settings (TCK-160)", () => {
  it("builds partial updates: only the given fields are sent (TCK-161), also before the backend state", () => {
    const s = new TrainerSettings();
    expect(s.update({ warpSpeed: 35 })).toEqual({ type: "UPDATE_TRAINER_HYPERPARAMS", warpSpeed: 35 });
    expect(s.apply(state)).toBe(true);
    expect(s.update({ N: 25, population: 200 })).toEqual({ type: "UPDATE_TRAINER_HYPERPARAMS", N: 25, population: 200 });
    expect(s.mutationOpsLabel()).toBe("2–4");
  });
});
