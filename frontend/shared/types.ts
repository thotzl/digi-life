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

export interface CTRNNNeuron {
  id: number;
  type: "input" | "hidden" | "output";
  label: string;
  tau: number;
  bias: number;
  activationType?: string;
  x?: number;
  y?: number;
}

export interface CTRNNSynapse {
  fromNode: number;
  toNode: number;
  weight: number;
}

export interface BrainTopology {
  neurons: CTRNNNeuron[];
  synapses: CTRNNSynapse[];
}

export interface GeneSpan {
  start: number;
  end: number;
}

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
  basalMetabolicRate: number;
  matureAge: number;
  reproThreshold: number;
  splitLoss: number;
  brain: BrainTopology;
  carnivory: number;
  isPredator: boolean;
  latinName: string;
  activeGeneSpans: GeneSpan[];
  methylations: number[];
}

export interface SpeciesRecord {
  id: string;             // Sense DNA string serves as the unique ID
  name: string;           // Procedural scientific name (Latin)
  genome: string;         // The Sense DNA
  antisense: string;      // The complementary strand
  parentSpeciesId: string | null; // Parent Species DNA ID (Lineage / Stammbaum!)
  status: "alive" | "extinct";    // Evolved state
  peakPopulation: number; // Highest concurrent specimen count reached
  birthTime: number;      // Creation epoch
  extinctionTime?: number; // Sinking epoch
  generation: number;     // Ancestral depth
  carnivory: number;      // Predation bias
}

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
  adrenaline: number; // Endocrine hormonal adrenaline multiplier [1.0 to 1.8]
  hasEaten: boolean;  // Physiological feeding guarantee (must consume at least 1 spore/prey to reproduce!)

  // NATIVE RECURRENT CTRNN BRAIN STATES (Euler integration state holders):
  neuronStates: number[];
  neuronActivations: number[];
  bendAngle?: number;
}

export interface FoodSpore {
  id: number;
  typeId: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
  amount: number;
}

export interface TelemetryCreatureInput {
  id: number;
  speciesId: string;
  genome: string;
  antisense: string;
  px: number;
  py: number;
  vx: number;
  vy: number;
  headingAngle: number;
  omegaRot: number;
  energy: number;
  adrenaline: number;
  age: number;
  generation: number;
  hasEaten: boolean;
  phenotype?: CreaturePhenotype;
}

export interface TelemetryPayload {
  type: "TELEMETRY_TICK" | "INIT_STATE" | "DATABASE_CHANGED" | "BITE_EVENT" | "LOG_EVENT";
  highestGeneration: number;
  running: boolean;
  timeStr?: string;
  creatures: TelemetryCreatureInput[];
  foodPellets: FoodSpore[];
  selectedBrain?: {
    id: number;
    activations: number[];
  };
}

export interface Vertex {
  x: number;
  y: number;
  r: number;
  angle: number;
}

export interface ProceduralObstacle {
  id: number;
  x: number;
  y: number;
  radius: number;
  type: 'rock' | 'coral';
  color: string;
  vertices: Vertex[];
}

export interface CurrentVent {
  id: number;
  x: number;
  y: number;
  radius: number;
  forceType: 'push' | 'pull' | 'vortex';
  strength: number;
}

export interface BiomeArea {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  sporeSpawnRate: number;
  sporeEnergyValue: number;
  hazardDamage: number;
  color: string;
}

export interface ProceduralWorld {
  seed: string;
  width: number;
  height: number;
  obstacles: ProceduralObstacle[];
  vents: CurrentVent[];
  biomes: BiomeArea[];
}
