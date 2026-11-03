import { describe, expect, it } from "vitest";

import { ageFlashes, BITE_FLASH_FRAMES, biteFlash, creatureAt, hitRadius, isOverCreature, OceanModel } from "./model";

const creature = (id: number, px: number, py: number, meanRadius = 40): any => ({
  id, px, py, vx: 0, vy: 0, headingAngle: 0, omegaRot: 0, energy: 1, age: 1, generation: 1, hasEaten: false,
  speciesId: "S", genome: "G", phenotype: { spinalHarmonics: { meanRadius } },
});

describe("Ocean model (TCK-160)", () => {
  it("bite flashes stay at the bite's world position (not converted to canvas pixels)", () => {
    expect(biteFlash({ x: 9600, y: 5400 })).toEqual({ x: 9600, y: 5400, age: 0 });
    const flashes = [biteFlash({ x: 1, y: 2 })];
    for (let i = 0; i < BITE_FLASH_FRAMES; i++) ageFlashes(flashes);
    expect(flashes).toHaveLength(1);
    ageFlashes(flashes);
    expect(flashes).toHaveLength(0);
  });

  it("hover radius comes from the creature's own phenotype", () => {
    const c = creature(1, 0, 0, 40);
    expect(hitRadius(c.phenotype)).toBe(40 * 2.6 * 0.5 + 10);
    expect(isOverCreature([c], 61, 0)).toBe(true);
    expect(isOverCreature([c], 63, 0)).toBe(false);
  });

  it("click picks the nearest creature within the click radius", () => {
    const a = creature(1, 0, 0);
    const b = creature(2, 50, 0);
    expect(creatureAt([a, b], 40, 0)?.id).toBe(2);
    expect(creatureAt([a, b], 500, 0)).toBeNull();
  });

  it("telemetry adds newborns, moves survivors and drops the dead", () => {
    const m = new OceanModel();
    m.creatures = [creature(1, 0, 0), creature(2, 0, 0)];
    m.applyTelemetry({
      highestGeneration: 3,
      newCreatures: [creature(3, 5, 5)],
      creatures: [{ ...creature(1, 10, 20) }, { ...creature(3, 6, 6) }],
      selectedBrain: null,
    } as any);
    expect(m.creatures.map((c) => [c.id, c.px])).toEqual([[1, 10], [3, 6]]);
    expect(m.highestGeneration).toBe(3);
  });
});
