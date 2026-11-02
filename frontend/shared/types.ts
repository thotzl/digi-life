import type { BrainTopology } from "../../bindings/BrainTopology";
import type { GeneSpan } from "../../bindings/GeneSpan";

export interface HSLColor {
  h: number;
  s: number;
  l: number;
}

export interface SpinalHarmonics {
  amplitudes: number[];
  phases: number[];
  baseLength: number;
  meanRadius: number;
  spinalCurve: number;
  spinalCurveFreq: number;
  parapodiaAmp: number;
  parapodiaFreq: number;
  flatteningHead: number;
}

export interface OrganelleLocus {
  spectralAffinity: number;
  bandwidth: number;
  expressionStyle: number;
  scale: number;
  spinalPos: number;
  hueShift: number;
  x?: number;
  y?: number;
  angle: number;
}

// Brain and gene types are generated from Rust (ts-rs, RV-A03)
export type { CTRNNNeuron } from "../../bindings/CTRNNNeuron";
export type { CTRNNSynapse } from "../../bindings/CTRNNSynapse";
export type { BrainTopology } from "../../bindings/BrainTopology";
export type { GeneSpan } from "../../bindings/GeneSpan";

export interface CreaturePhenotype {
  symmetry: string;
  primaryColor: HSLColor;
  secondaryColor: HSLColor;
  bodySeed: number;
  spinalHarmonics: SpinalHarmonics;
  emergentChambersCount: number;
  organelles: OrganelleLocus[];
  pulseSpeed: number;
  wavePhase: number;
  wiggleAmplitude: number;
  stiffness: number;
  stomachCapacity: number;
  matureAge: number;
  reproThreshold: number;
  splitLoss: number;
  brain: BrainTopology;
  carnivory: number;
  isPredator: boolean;
  latinName: string;
  /** Recognised genes in genome order (backend gene spans, TCK-187) */
  genes: GeneSpan[];
  methylations: number[];
}

// Species roster entry: the `get_registered_species` result type (ts-rs, TCK-157). The Ocean event payloads are the
// binding types of `UiEvent` (frontend/core/ipc.ts); the model types below are aligned to the bindings in TCK-160.
export type { SpeciesEntry as SpeciesRecord } from "../../bindings/SpeciesEntry";

export interface CreatureAgent {
  id: number;
  speciesId: string;  // Defined by its unique Sense DNA string
  genome: string;
  antisense: string;
  phenotype: CreaturePhenotype;
  px: number;
  py: number;
  vx: number;
  vy: number;
  headingAngle: number;
  omegaRot: number;
  energy: number;     // Current food reserves
  age: number;        // Age in frames
  generation: number; // Gen lineage
  parentSpeciesId?: string; // Parent Species DNA ID (Lineage!)
  hasEaten: boolean;  // Physiological feeding guarantee (must consume at least 1 spore/prey to reproduce!)

  // NATIVE RECURRENT CTRNN BRAIN STATES (Euler integration state holders):
  neuronStates: number[];
  neuronActivations: number[];
  bendAngle?: number;
}


// Food and world types are generated from Rust (ts-rs, TCK-157)
export type { FoodSpore } from "../../bindings/FoodSpore";
export type { Vertex } from "../../bindings/Vertex";
export type { ProceduralObstacle } from "../../bindings/ProceduralObstacle";
export type { CurrentVent } from "../../bindings/CurrentVent";
export type { BiomeArea } from "../../bindings/BiomeArea";
export type { ProceduralWorld } from "../../bindings/ProceduralWorld";
