import { describe, expect, it } from "vitest";

import {
  catalogueEntry, catalogueId, fromCatalogue, fromOceanCreature, fromSandbox, oceanCreatureId, selection,
  selectionStore, trainerSandboxId, vitals,
} from "./selection";

const pheno: any = {
  latinName: "Aqueus pulsa", methylations: [0, 1], stomachCapacity: 80, carnivory: 0.7,
  brain: { synapses: [{ weight: 0.5 }, { weight: -0.25 }] },
};
const agent: any = { id: 7, genome: "ACGT", generation: 2, energy: 40, age: 12, phenotype: pheno, synapseWeights: [0.9, -0.1] };
const tele: any = { id: 7, genome: "ACGT", current_fitness: 3.5, finished: false, energy: 30, age: 5 };

describe("selection (TCK-160)", () => {
  it("ids are read only from their own source (sandbox 7 is not Ocean creature 7)", () => {
    const sb = fromSandbox(tele, pheno, 4).selection;
    expect(oceanCreatureId(sb)).toBeNull();
    expect(trainerSandboxId(sb)).toBe(7);
    const oc = fromOceanCreature(agent).selection;
    expect(trainerSandboxId(oc)).toBeNull();
    expect(oceanCreatureId(oc)).toBe(7);
    const cat = fromCatalogue({ id: "catalogue_1", name: "n", genome: "A", source: "s", fitness: 2, carnivory: 0, methylations: [], synapseWeights: [], createdAt: "" }, null);
    expect(catalogueId(cat)).toBe("catalogue_1");
    expect(oceanCreatureId(cat)).toBeNull();
  });

  it("live updates apply only to the selected source and id", () => {
    const { selection: sel, vitals: v } = fromOceanCreature(agent);
    selectionStore.select(sel, v);
    selectionStore.updateVitals({ source: "trainer", sandboxId: 7 }, { status: "x", energy: 1, maxEnergy: 1, age: 1 });
    expect(vitals.value.energy).toBe(40);
    selectionStore.updateVitals({ source: "ocean", creatureId: 7 }, { status: "Alive", energy: 5, maxEnergy: 80, age: 13 });
    expect(vitals.value.energy).toBe(5);
    expect(selection.value).toBe(sel); // static data untouched by live updates
    selectionStore.clear();
  });

  it("catalogue save stores the selection's fitness and the creature's own weights", () => {
    const fromTrainer = catalogueEntry(fromSandbox(tele, pheno, 4).selection, "n");
    expect(fromTrainer.fitness).toBe(3.5);
    expect(fromTrainer.source).toBe("Evolutionary Trainer");
    expect(fromTrainer.synapseWeights).toEqual([]); // not sent by the trainer telemetry: the backend compiles them
    const fromOcean = catalogueEntry(fromOceanCreature(agent).selection, "n");
    expect(fromOcean.synapseWeights).toEqual([0.9, -0.1]);
    expect(fromOcean.source).toBe("Ocean Substrate");
  });
});
