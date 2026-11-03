import { describe, expect, it } from "vitest";

import { ORGAN_KINDS } from "../../bindings/organRegistry";
import { ORGAN_PAINTERS, organShape } from "../render/organs";
import { badgeColor, dietLabel } from "./creatureLabels";
import { sliderToWarp, warpLabel, warpToSlider } from "./warp";

describe("shared labels (TCK-160)", () => {
  it("diet classes use the backend thresholds (CARNIVORE_MIN 0.6, HERBIVORE_MAX 0.4)", () => {
    expect([0.39, 0.4, 0.59, 0.6].map(dietLabel)).toEqual(["Herbivore", "Omnivore", "Omnivore", "Predator"]);
  });

  it("badge colour comes from the phenotype's primary colour, neutral without one", () => {
    expect(badgeColor({ primaryColor: { h: 120, s: 50, l: 40 } })).toBe("hsla(120, 85%, 55%, 1)");
    expect(badgeColor(null)).toContain("hsla(215");
  });

  it("warp slider mapping round-trips", () => {
    for (let p = 1; p <= 12; p++) expect(warpToSlider(sliderToWarp(p))).toBe(p);
    expect([warpLabel(3), warpLabel(11)]).toEqual(["3x", "50x"]);
  });

  it("every backend organ kind has a painter", () => {
    expect(Object.keys(ORGAN_PAINTERS).sort()).toEqual(Object.keys(ORGAN_KINDS).sort());
    expect(organShape({ expressionStyle: 0.72, spinalPos: 0.1 } as any)).toBe("limb");
    expect(organShape({ expressionStyle: 0.5, spinalPos: 0.1 } as any)).toBe("bud");
    expect(organShape({ expressionStyle: 0.5, spinalPos: 0.5 } as any)).toBe("hillock");
  });
});
