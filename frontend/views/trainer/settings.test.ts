import { describe, expect, it } from "vitest";

import { TrainerSettings } from "./settings";

const state: any = {
  isRunning: false, population: 200, N: 25, warpSpeed: 50, eliteRatio: 0.3, mutationRate: 4, mutationOpsMin: 2,
  mutationOpsLimit: 8, inflowRate: 0.1, hofRate: 0.2, multiTrial: false, isHeadless: false, runId: "run_b",
  lamarckian: true, trainingMode: "standard", level: 0, levelAuto: false, pendingMode: null, generation: 9,
};

describe("trainer settings (TCK-160)", () => {
  it("sends no hyperparameters before the backend state arrived (no client defaults at boot)", () => {
    const s = new TrainerSettings();
    expect(s.hyperparams()).toBeNull();
    expect(s.hyperparams({ population: true, mutation: true })).toBeNull();
  });

  it("after the backend state, pushes its values; population and mutation only on request", () => {
    const s = new TrainerSettings();
    expect(s.apply(state)).toBe(true);
    const a = s.hyperparams()!;
    expect(a).toMatchObject({ type: "UPDATE_TRAINER_HYPERPARAMS", N: 25, warpSpeed: 50, runId: "run_b", lamarckian: true });
    expect(a).not.toHaveProperty("population");
    expect(a).not.toHaveProperty("mutationRate");
    expect(s.hyperparams({ population: true, mutation: true })).toMatchObject({ population: 200, mutationRate: 4 });
    expect(s.mutationOpsLabel()).toBe("2–4");
  });
});
