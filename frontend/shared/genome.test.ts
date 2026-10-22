import { describe, expect, it } from "vitest";
import type { GeneSpan } from "../../bindings/GeneSpan";
import type { SensoryPatch } from "../../bindings/SensoryPatch";
import type { BrainTopology } from "../../bindings/BrainTopology";
import {
  PROMOTER_MEANINGS, describeGene, describeRole, describeSynapse, geneIndexByPosition, geneName, isSilentCopy,
} from "./genome";

const span = (promoter: string, spelling: string, occurrence: number, start: number, end: number): GeneSpan =>
  ({ gene: { promoter, spelling, occurrence }, start, end });

const genes: GeneSpan[] = [
  span("COL", "COL", 0, 2, 10),
  span("EYE", "EWE", 0, 12, 20),
  span("COL", "CQL", 0, 22, 30),
  span("NEU", "NEU", 0, 32, 40),
];
const eye = { geneStartIndex: 12, kind: "eye", side: "left" } as SensoryPatch;
const ctx = { genes, organelles: [eye, { ...eye, side: "right" } as SensoryPatch] };

describe("gene spans", () => {
  it("covers every promoter the compiler expresses", () => {
    for (const p of ["COL", "SIZ", "STF", "WAV", "PUL", "STM", "TEM", "CAR", "REP", "EVO", "HOX", "EYE", "NOS", "TAC", "LUM", "NEU", "SY", "OUT"]) {
      expect(PROMOTER_MEANINGS[p]).toBeDefined();
    }
  });

  it("maps positions to the backend spans, outside is non-coding", () => {
    const owner = geneIndexByPosition(genes, 42);
    expect(owner[0]).toBe(-1);
    expect(owner[2]).toBe(0);
    expect(owner[9]).toBe(0);
    expect(owner[10]).toBe(-1); // end is exclusive
    expect(owner[12]).toBe(1);
    expect(owner[41]).toBe(-1);
  });

  it("names genes by promoter ordinal and flags silent body-gene copies", () => {
    expect(geneName(genes, 2)).toBe("COL 2");
    expect(isSilentCopy(genes, 0)).toBe(false);
    expect(isSilentCopy(genes, 2)).toBe(true);
    expect(describeGene(genes, 1, ctx.organelles)).toBe("EYE 1: Sensory organ (EYE) - Eye pair");
  });
});

describe("neuron roles", () => {
  it("labels roles without emoji or label matching", () => {
    expect(describeRole({ role: "systemic", input: "searchArousal" }, ctx)).toBe("Search arousal");
    expect(describeRole({ role: "sensor", gene: genes[1].gene, side: "left", channel: 5, kind: "eye" }, ctx)).toBe("Eye L (EYE 1) - ΔI change");
    expect(describeRole({ role: "thalamus", gene: genes[1].gene, channel: 2, node: "diff" }, ctx)).toBe("Thalamus Δ L-R diff Eye (EYE 1) - Green");
    expect(describeRole({ role: "hidden", gene: genes[3].gene }, ctx)).toBe("Hidden neuron (NEU 1)");
    expect(describeRole({ role: "motor", motor: "bend" }, ctx)).toBe("Motor: Bend (left/right)");
  });

  it("names the owner gene of a synapse", () => {
    const brain: BrainTopology = {
      neurons: [
        { id: 0, type: "hidden", label: "", role: { role: "hidden", gene: genes[3].gene }, tau: 1, bias: 0, activationType: null, x: null, y: null },
        { id: 1, type: "output", label: "", role: { role: "motor", motor: "thrust" }, tau: 1, bias: 0, activationType: null, x: null, y: null },
      ],
      synapses: [{ fromNode: 0, toNode: 1, weight: 0.25, key: 7, owner: 3 }],
    };
    expect(describeSynapse(brain.synapses[0], brain, ctx)).toBe(
      "Hidden neuron (NEU 1) → Motor: Thrust (fwd/bwd)\nweight 0.250\nowner NEU 1: Hidden neuron and its synapses (NEU)",
    );
  });
});
