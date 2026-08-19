import { CreaturePhenotype } from "../biology/dna";

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
  x: number;
  y: number;
  vx: number;
  vy: number;
  type?: 'plant' | 'meat';
}
