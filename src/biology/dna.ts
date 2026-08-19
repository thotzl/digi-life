export const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

export interface HSLColor {
  h: number; // 0 - 360
  s: number; // 0 - 100
  l: number; // 0 - 100
}

export type OrganelleType = "photoreceptor" | "chemoreceptor" | "mechanoreceptor" | "energy";

export interface SensoryPatch {
  spectralAffinity: number;  
  bandwidth: number;         
  expressionStyle: number;   
  scale: number;             
  spinalPos: number;         
  angle: number;             
  hueShift: number;          
  geneStartIndex: number;
  geneEndIndex: number;
}

export interface SpinalHarmonics {
  baseLength: number;         
  meanRadius: number;         
  amplitudes: number[];       
  phases: number[];           
  spinalCurve: number;        
  spinalCurveFreq: number;    
  parapodiaAmp: number;       
  parapodiaFreq: number;      
  flatteningHead: number;     
}

// Genetically Compiled CTRNN Synapse (Permits recurrent and self-feedback loops!)
export interface CTRNNSynapse {
  fromNode: number;
  toNode: number;
  weight: number;
}

// Genetically Compiled CTRNN Neuron
export interface CTRNNNeuron {
  id: number;
  type: "input" | "hidden" | "output";
  label: string;
  tau: number;   // time constant [0.5 to 5.0] controlling memory decay speed
  bias: number;  // neural bias [-1.0 to 1.0]
}

// Genetically Compiled CTRNN Directed Graph (NEAT-like Topology)
export interface BrainTopology {
  neurons: CTRNNNeuron[];
  synapses: CTRNNSynapse[];
}

export interface CreaturePhenotype {
  symmetry: "vertical" | "quad";
  primaryColor: HSLColor;
  secondaryColor: HSLColor;
  bodySeed: number;
  segments: any[];                  
  spinalHarmonics: SpinalHarmonics; 
  emergentChambersCount: number;   
  organelles: SensoryPatch[];       
  pulseSpeed: number;       
  wavePhase: number;        
  wiggleAmplitude: number;  
  stiffness: number;        
  
  // Gen-coded Reproduction Strategies (r- vs. K-selection):
  matureAge: number;        // age in frames before eligible for mitosis
  reproThreshold: number;   // stomach capacity % threshold for reproduction
  splitLoss: number;        // energy tax ratio lost to friction [0.05 to 0.40]

  // Genetically Evolved Recurrent CTRNN Brain:
  brain: BrainTopology;
  carnivory: number;        // Genetically predispositioned carnivory index [0.0 to 1.0]
  isPredator: boolean;      // True if carnivory >= 0.55
  
  // Advanced Physiology Profile:
  latinName: string;         
  sensoryVisus: number;      
  sensoryOlfaction: number;  
  sensoryTactility: number;  
  sensoryBiolum: number;     
  dietClass: string;         
  preferredHabitat: string;  
  
  // Metabolic Stats:
  basalMetabolicRate: number; 
  stomachCapacity: number;    
  thermalToleranceMin: number; 
  thermalToleranceMax: number; 
  
  // Biomechanical Stats:
  hydraulicPressure: number;  
  rotationalInertia: number;  
  
  // Dynamic Survival Metrics:
  survivalExpectation: number; 
  survivalAnalysis: string;    
  
  // Epigenetic State Returns:
  chromatinState: boolean[];   
  epigeneticLogs: string[];    
  methylations: number[];      
  
  // Watson-Crick Double-Helix Parameters:
  antisenseStrand: string;     
  repairFidelity: number;      
  insertionRate: number;       
  deletionRate: number;        
  
  genomeString: string;
  activeGeneSpans: { start: number; end: number }[];
}

export function charToValue(char: string): number {
  const index = ALPHABET.indexOf(char.toUpperCase());
  return index === -1 ? 0 : index;
}

export function valueToChar(val: number): string {
  const index = Math.max(0, Math.min(25, Math.floor(val)));
  return ALPHABET[index];
}

export function getComplementaryChar(char: string): string {
  const val = charToValue(char);
  return valueToChar(25 - val);
}

export function getComplementaryString(sense: string): string {
  let anti = "";
  for (let i = 0; i < sense.length; i++) {
    anti += getComplementaryChar(sense[i]);
  }
  return anti;
}

export function generateRandomGenome(length = 256): string {
  let dna = "";
  const randomValues = new Uint32Array(length);
  
  if (typeof window !== "undefined" && window.crypto) {
    window.crypto.getRandomValues(randomValues);
  } else if (typeof crypto !== "undefined" && (crypto as any).getRandomValues) {
    (crypto as any).getRandomValues(randomValues);
  } else {
    for (let i = 0; i < length; i++) {
      randomValues[i] = Math.floor(Math.random() * 4294967296);
    }
  }

  for (let i = 0; i < length; i++) {
    const r = randomValues[i] % 26;
    dna += ALPHABET[r];
  }
  return dna;
}

export function mutateGenome(genome: string): { newGenome: string; mutatedIndex: number; oldChar: string; newChar: string } {
  if (!genome) {
    throw new Error("Genome cannot be empty");
  }
  const randomBuf = new Uint32Array(2);
  
  if (typeof window !== "undefined" && window.crypto) {
    window.crypto.getRandomValues(randomBuf);
  } else if (typeof crypto !== "undefined" && (crypto as any).getRandomValues) {
    (crypto as any).getRandomValues(randomBuf);
  } else {
    randomBuf[0] = Math.floor(Math.random() * 4294967296);
    randomBuf[1] = Math.floor(Math.random() * 4294967296);
  }

  const index = randomBuf[0] % genome.length;
  const currentChar = genome[index];
  let newChar = currentChar;

  while (newChar === currentChar) {
    const r = (randomBuf[1]++) % 26;
    newChar = ALPHABET[r];
  }

  const newGenome = genome.substring(0, index) + newChar + genome.substring(index + 1);
  return {
    newGenome,
    mutatedIndex: index,
    oldChar: currentChar,
    newChar
  };
}

export function classifySensoryPatch(patch: SensoryPatch): { name: string; color: string; desc: string } {
  const aff = patch.spectralAffinity;
  const style = patch.expressionStyle;

  if (style >= 0.72) {
    if (aff >= 0.5) {
      return { name: "Locomotor Swimming Foot", color: "#38bdf8", desc: "Fleshy, muscular swimming fin" };
    } else {
      return { name: "Crawling Tentacle / Pseudopod", color: "#a5f3fc", desc: "Articulated, flexible muscular arm" };
    }
  }

  if (aff >= 0.8) {
    if (style < 0.3) {
      return { name: "Pigment Spot Eye", color: "#a21caf", desc: "Flat photoreceptor cell spot" };
    } else {
      return { name: "Photoreceptive Filament", color: "#c084fc", desc: "Fine light-sensitive cilium" };
    }
  } else if (aff < 0.25) {
    if (style < 0.3) {
      return { name: "Pressure-sensitive Membrane", color: "#b45309", desc: "Flat, tactile vibration area" };
    } else {
      return { name: "Acoustic Auditory Hair", color: "#f59e0b", desc: "Mechanoreceptive vibrating hair" };
    }
  } else if (aff >= 0.25 && aff <= 0.65) {
    if (style < 0.3) {
      return { name: "Taste Bud", color: "#15803d", desc: "Integrated molecular receptor bud" };
    } else {
      return { name: "Olfactory Cilium", color: "#22c55e", desc: "Molecule-absorbing ciliated flagellum" };
    }
  } else {
    if (style < 0.3) {
      return { name: "Infrared Heat Pit", color: "#e11d48", desc: "Flat, thermoreceptive infrared pit" };
    } else {
      return { name: "Thermoreceptive Filament", color: "#fda4af", desc: "Sensitive heat-conducting hair" };
    }
  }
}

/**
 * Procedural Nomenclature, Ecological and Survival Fitness Analyzer (Continuous Spectral Edition)
 */
function deriveEcologicalMetrics(
  genome: string,
  symmetry: "vertical" | "quad",
  emergentSegmentsCount: number,
  organelles: SensoryPatch[],
  primaryColor: HSLColor,
  stiffness: number,
  pulseSpeed: number,
  wavePhase: number,
  carnivory: number
): {
  latinName: string;
  visus: number;
  olfaction: number;
  tactility: number;
  biolum: number;
  diet: string;
  habitat: string;
  survivalScore: number;
  survivalAnalysis: string;
} {
  let visusScore = 0;
  let olfactionScore = 0;
  let tactilityScore = 0;
  let biolumScore = 0;

  organelles.forEach(patch => {
    const bandAdjusted = 0.2 + patch.bandwidth * 0.8;
    const aff = patch.spectralAffinity;
    const power = patch.scale * 22; 

    const wMech = Math.max(0, 1.0 - Math.abs(aff - 0.0) / bandAdjusted);    
    const wChem = Math.max(0, 1.0 - Math.abs(aff - 0.45) / bandAdjusted);   
    const wThermal = Math.max(0, 1.0 - Math.abs(aff - 0.75) / bandAdjusted); 
    const wLight = Math.max(0, 1.0 - Math.abs(aff - 1.0) / bandAdjusted);    

    visusScore += wLight * power;
    olfactionScore += wChem * power;
    tactilityScore += wMech * power;
    biolumScore += wThermal * power;
  });

  visusScore = Math.min(100, Math.round(visusScore));
  olfactionScore = Math.min(100, Math.round(olfactionScore));
  tactilityScore = Math.min(100, Math.round(tactilityScore));
  biolumScore = Math.min(100, Math.round(biolumScore));

  // Determine scientific taxonomy and diet based on continuous carnivory scale (D)
  const isPredator = carnivory >= 0.55;
  let diet = "Ancestral Filter Feeder (Detritus)";

  const scores = [
    { type: "light", val: visusScore },
    { type: "chemical", val: olfactionScore },
    { type: "kinetic", val: tactilityScore },
    { type: "thermal", val: biolumScore }
  ];
  scores.sort((a, b) => b.val - a.val);

  if (carnivory >= 0.65) {
    diet = "Sabertooth Hunter (Carnivore)";
  } else if (carnivory >= 0.40) {
    diet = "Omnivore (Omnivorous)";
  } else {
    if (scores[0].val > 15) {
      if (scores[0].type === "light") {
        diet = "Diatom Grazer (Herbivore)";
      } else if (scores[0].type === "chemical") {
        diet = "Chemotactic Filter Feeder";
      } else if (scores[0].type === "kinetic") {
        diet = "Vibrational Plankton Feeder";
      } else if (scores[0].type === "thermal") {
        diet = "Deep Sea Thermotroph";
      }
    } else {
      diet = "Protozoan Filter Feeder (Detritus)";
    }
  }

  let habitat = "Epipelagic (Sunlit Zone)";
  if (biolumScore > 40) {
    habitat = "Bathypelagic (Abyssal Zone)";
  } else if (tactilityScore > 45 || emergentSegmentsCount >= 4) {
    habitat = "Hadopelagic (Benthic Trench)";
  } else if (primaryColor.l < 45) {
    habitat = "Mesopelagic (Twilight Swamps)";
  }

  // Procedural scientific Latin name
  const segPrefixes = ["Monoplasma", "Biplasma", "Triplasma", "Tetraplasma", "Pentaplasma"];
  const prefix = segPrefixes[Math.min(emergentSegmentsCount - 1, 4)];
  const midSym = symmetry === "vertical" ? "bilateralis" : "tetramerum";

  let suffix = "lēvis"; 
  if (isPredator) {
    suffix = "raptor";
  } else if (organelles.length > 0) {
    const dominantType = organelles.reduce((acc, curr) => {
      const aff = curr.spectralAffinity;
      let key = "kinetic";
      if (aff >= 0.8) key = "light";
      else if (aff >= 0.25 && aff <= 0.65) key = "chem";
      else if (aff > 0.65 && aff < 0.8) key = "thermal";
      
      acc[key] = (acc[key] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    const sortedTypes = Object.entries(dominantType).sort((a, b) => b[1] - a[1]);
    const dType = sortedTypes[0][0];

    if (dType === "light") suffix = "ocularium";
    else if (dType === "chem") suffix = "ciliatum";
    else if (dType === "kinetic") suffix = "vibrans";
    else if (dType === "thermal") suffix = "thermum";
  }

  // Derive a 100% unique, highly scientific Genetic Strain Suffix from the genome string to prevent name collisions
  const firstLociVal = ALPHABET.indexOf(genome.substring(0, 1)) || 0;
  const lastLociVal = ALPHABET.indexOf(genome.substring(genome.length - 1)) || 0;
  const midLociVal = ALPHABET.indexOf(genome.substring(Math.floor(genome.length / 2), Math.floor(genome.length / 2) + 1)) || 0;
  const hashChar1 = ALPHABET[(firstLociVal + lastLociVal) % 26];
  const hashChar2 = ALPHABET[(midLociVal + lastLociVal) % 26];
  const hashNum = (firstLociVal * midLociVal + lastLociVal) % 10;
  const strainSuffix = `(Str. ${hashChar1}${hashChar2}-${hashNum})`;

  const latinName = `${prefix} ${midSym} ${suffix} ${strainSuffix}`;

  // Survival Score computation
  let survivalScore = 40; 

  const freqHz = pulseSpeed * 1000;
  const wavesThrustFactor = Math.sin(wavePhase);
  const locomotiveEfficiency = stiffness * (freqHz * 0.15) * (wavesThrustFactor > 0.1 ? wavesThrustFactor : 0);
  const locoBonus = Math.min(35, Math.round(locomotiveEfficiency * 30));
  survivalScore += locoBonus;

  const maxSensoryVal = Math.max(visusScore, olfactionScore, tactilityScore, biolumScore);
  
  if (maxSensoryVal < 10) {
    survivalScore -= 25; 
  } else if (maxSensoryVal > 55) {
    survivalScore += 15; 
  } else {
    survivalScore += 10;
  }

  const organCount = organelles.length;
  const metabolicTax = organCount * 4.5; 
  survivalScore -= metabolicTax;

  if (isPredator) {
    survivalScore += 15; // Predator physical edge bonus
  } else {
    survivalScore += 5;  // Prey evasive camouflage bonus
  }

  survivalScore = Math.max(1, Math.min(100, Math.round(survivalScore)));

  let survivalAnalysis = "";
  if (survivalScore >= 80) {
    survivalAnalysis = isPredator 
      ? "Excellent hunting potential. Highly elastic thrust paired with rigid, destructive impact force."
      : "Outstanding chance of survival. Extremely agile, highly responsive, and energetically highly efficient.";
  } else if (survivalScore >= 60) {
    survivalAnalysis = "Good. Stable life form with pronounced ecological maturity at moderate efficiency.";
  } else if (survivalScore >= 40) {
    survivalAnalysis = "Mediocre. Undefined spectral receptors or unfavorable power-to-mass ratio.";
  } else if (survivalScore >= 20) {
    survivalAnalysis = "Endangered. Poorly coupled flagellar waves, high metabolic burden.";
  } else {
    survivalAnalysis = "Critical. Sensorially isolated, immobile organism; easy prey for predators.";
  }

  return {
    latinName,
    visus: visusScore,
    olfaction: olfactionScore,
    tactility: tactilityScore,
    biolum: biolumScore,
    diet,
    habitat,
    survivalScore,
    survivalAnalysis
  };
}

/**
 * DNA Parser (Genotype to Phenotype Epigenetic Translation with Genetically Evolving CTRNN Topology!)
 */
export function parseGenome(genome: string, antisenseInput?: string, parentMethylations?: number[]): CreaturePhenotype {
  const currentLength = Math.max(128, Math.min(384, genome.length));
  const cleanGenome = genome.toUpperCase().substring(0, currentLength);
  
  const getVal = (idx: number): number => charToValue(cleanGenome[idx]);

  const rawInsertion = getVal(9);
  const insertionRate = (rawInsertion / 25) * 0.12;

  const rawDeletion = getVal(10);
  const deletionRate = (rawDeletion / 25) * 0.12;

  const rawRepair = getVal(11);
  const repairFidelity = 0.15 + (rawRepair / 25) * 0.8;

  const antisenseStrand = antisenseInput 
    ? antisenseInput.toUpperCase().substring(0, currentLength)
    : getComplementaryString(cleanGenome);

  // ==========================================================================
  // 1. EPIGENETIC INITIALIZATION (Wave t = 0)
  // ==========================================================================
  const chromatinState = Array(currentLength).fill(false);
  
  for (let idx = 0; idx < 16; idx++) {
    chromatinState[idx] = true;
  }

  for (let idx = 16; idx < currentLength - 2; idx++) {
    const charA = cleanGenome[idx];
    const charB = cleanGenome[idx + 1];
    if ((charA === "S" && charB === "T") || (charA === "G" && charB === "O")) {
      chromatinState[idx] = true;
      chromatinState[idx + 1] = true;
      chromatinState[idx + 2] = true;
      chromatinState[idx + 3] = true;
    }
  }

  const epigeneticLogs: string[] = [];
  
  const methylations = parentMethylations && parentMethylations.length === currentLength
    ? [...parentMethylations]
    : Array(currentLength).fill(0);

  if (parentMethylations) {
    epigeneticLogs.push("Transgenerational epigenetic inheritance: Learned brain methylation pattern of the parent cell successfully assimilated.");
  }

  const getMethylatedVal = (idx: number): number => {
    const rawVal = charToValue(cleanGenome[idx]);
    return (rawVal + methylations[idx] + 26) % 26;
  };

  // Embryological hox cascade waves
  for (let wave = 1; wave <= 3; wave++) {
    let waveLociModified = 0;
    let waveMethylations = 0;
    
    let idx = 16;
    while (idx < currentLength - 2) {
      if (chromatinState[idx] && chromatinState[idx + 1]) {
        const charA = cleanGenome[idx];
        const charB = cleanGenome[idx + 1];
        const isHox = (charA === "E" && charB === "P") || (charA === "H" && charB === "X");
        
        if (isHox) {
          const hoxStart = idx;
          const payloadStart = idx + 2;
          let payloadEnd = -1;
          
          for (let j = payloadStart; j < currentLength - 1; j++) {
            const cA = cleanGenome[j];
            const cB = cleanGenome[j + 1];
            if ((cA === "S" && cB === "P") || (cA === "E" && cB === "N")) {
              payloadEnd = j;
              break;
            }
          }

          if (payloadEnd === -1) payloadEnd = currentLength;

          const payloadLength = payloadEnd - payloadStart;
          if (payloadLength >= 3) {
            const targetCharIdx = getMethylatedVal(payloadStart);
            const targetLetter = ALPHABET[targetCharIdx];
            const actionVal = getMethylatedVal(payloadStart + 1);
            const powerRadius = 5 + getMethylatedVal(payloadStart + 2) * 1.5;

            for (let targetIdx = 16; targetIdx < currentLength; targetIdx++) {
              if (cleanGenome[targetIdx] === targetLetter) {
                const distance = Math.abs(targetIdx - hoxStart);
                if (distance <= powerRadius) {
                  if (actionVal < 9) {
                    const startSlot = Math.max(16, targetIdx - 3);
                    const endSlot = Math.min(currentLength - 1, targetIdx + 3);
                    for (let s = startSlot; s <= endSlot; s++) {
                      if (!chromatinState[s]) {
                        chromatinState[s] = true;
                        waveLociModified++;
                      }
                    }
                  } else if (actionVal < 18) {
                    const startSlot = Math.max(16, targetIdx - 3);
                    const endSlot = Math.min(currentLength - 1, targetIdx + 3);
                    for (let s = startSlot; s <= endSlot; s++) {
                      if (chromatinState[s]) {
                        chromatinState[s] = false;
                        waveLociModified++;
                      }
                    }
                  } else {
                    const shiftDirection = actionVal % 2 === 0 ? 5 : -5;
                    methylations[targetIdx] += shiftDirection;
                    waveMethylations++;
                  }
                }
              }
            }
          }
          idx = Math.min(currentLength, payloadEnd + 2);
        } else {
          idx++;
        }
      } else {
        idx++;
      }
    }
    if (waveLociModified > 0 || waveMethylations > 0) {
      epigeneticLogs.push(`Wave ${wave}: Hox networks active. ${waveLociModified} chromatin loops formed, ${waveMethylations} methylations completed.`);
    }
  }

  for (let idx = 0; idx < 16; idx++) {
    chromatinState[idx] = true;
    methylations[idx] = 0;
  }

  // ==========================================================================
  // 2. CHASSIS PARAMETERS DE-COMPILATION
  // ==========================================================================
  const symmetry: "vertical" | "quad" = getMethylatedVal(0) < 13 ? "vertical" : "quad";

  const primaryColor: HSLColor = {
    h: Math.round((getMethylatedVal(1) / 25) * 360),
    s: Math.round(55 + (getMethylatedVal(2) / 25) * 45),
    l: Math.round(35 + (getMethylatedVal(3) / 25) * 45)
  };

  const secondaryColor: HSLColor = {
    h: Math.round((getMethylatedVal(4) / 25) * 360),
    s: Math.round(55 + (getMethylatedVal(5) / 25) * 45),
    l: Math.round(35 + (getMethylatedVal(6) / 25) * 45)
  };

  const bodySeed = getMethylatedVal(7) * 4293 + getMethylatedVal(8) * 117;
  const meanRadius = 18 + (getMethylatedVal(7) / 25) * 18;  
  const baseLength = 90 + (getMethylatedVal(8) / 25) * 110; 

  const amplitudes: number[] = [];
  for (let j = 0; j < 4; j++) {
    amplitudes.push((getMethylatedVal(9 + j) / 25) * 0.3 - 0.15);
  }

  const phases: number[] = [];
  for (let j = 0; j < 4; j++) {
    phases.push((getMethylatedVal((3 + j) % 16) / 25) * Math.PI * 2);
  }

  const spinalCurve = (getMethylatedVal(15) / 25) * 44 - 22;
  const spinalCurveFreq = 1 + Math.floor(getMethylatedVal(14) / 12);
  const parapodiaAmp = (getMethylatedVal(13) / 25) * 0.45;
  const parapodiaFreq = 2 + Math.floor((getMethylatedVal(2) / 25) * 12);
  const flatteningHead = (getMethylatedVal(1) / 25) * 1.0 - 0.4;

  const stiffness = 0.15 + (getMethylatedVal(12) / 25) * 0.85;
  const pulseSpeed = 0.0015 + (getMethylatedVal(13) / 25) * 0.0075;
  const wavePhase = (getMethylatedVal(14) / 25) * 1.6;
  const wiggleAmplitude = (getMethylatedVal(15) / 25) * 0.22;

  const stomachCapacity = 50 + getMethylatedVal(13) * 18;
  const hydraulicPressure = 0.2 + (getMethylatedVal(15) / 25) * 0.8;

  const thermalCenter = 10 + (getMethylatedVal(5) / 25) * 60; 
  const thermalWidth = 10 + (getMethylatedVal(6) / 25) * 30;  
  const thermalToleranceMin = Math.max(-5, Math.round(thermalCenter - thermalWidth / 2));
  const thermalToleranceMax = Math.min(105, Math.round(thermalCenter + thermalWidth / 2));

  // Count spinal segments (local maxima)
  let peaksCount = 0;
  let wasRising = false;
  let prevR = 0;

  const getSpinalRadiusAt = (s: number): number => {
    let modulation = 0;
    for (let j = 0; j < 4; j++) {
      const frequencyFactor = (j + 1) * Math.PI;
      modulation += amplitudes[j] * Math.cos(frequencyFactor * s + phases[j]);
    }
    return meanRadius * (1.0 + modulation);
  };

  for (let j = 0; j <= 50; j++) {
    const s = j / 50;
    const rCurrent = getSpinalRadiusAt(s);
    if (j > 0) {
      const isRising = rCurrent > prevR;
      if (wasRising && !isRising) {
        peaksCount++; 
      }
      wasRising = isRising;
    }
    prevR = rCurrent;
  }
  const emergentChambersCount = Math.max(1, peaksCount);

  const spinalHarmonics: SpinalHarmonics = {
    baseLength,
    meanRadius,
    amplitudes,
    phases,
    spinalCurve,
    spinalCurveFreq,
    parapodiaAmp,
    parapodiaFreq,
    flatteningHead
  };

  // ==========================================================================
  // 3. COMPILE ACTIVE ORGANELLES (Must be done BEFORE Brain for Embodied count)
  // ==========================================================================
  const organelles: SensoryPatch[] = [];
  const activeGeneSpans: { start: number; end: number }[] = [];

  let scanIdx = 16;
  while (scanIdx < currentLength - 1) {
    if (chromatinState[scanIdx] && chromatinState[scanIdx + 1]) {
      const charA = cleanGenome[scanIdx];
      const charB = cleanGenome[scanIdx + 1];
      const isStart = (charA === "S" && charB === "T") || (charA === "G" && charB === "O");

      if (isStart) {
        const geneStart = scanIdx;
        const payloadStart = scanIdx + 2;
        let payloadEnd = -1;
        let stopFoundAt = -1;

        for (let j = payloadStart; j < currentLength - 1; j++) {
          const cA = cleanGenome[j];
          const cB = cleanGenome[j + 1];
          const isStop = (cA === "S" && cB === "P") || (cA === "E" && cB === "N");

          if (isStop) {
            payloadEnd = j;
            stopFoundAt = j;
            break;
          }
        }

        if (payloadEnd === -1) {
          payloadEnd = currentLength;
          stopFoundAt = currentLength - 1;
        }

        const payloadLength = payloadEnd - payloadStart;
        if (payloadLength > 0) {
          const getMethylatedPayloadVal = (offset: number, fallback: number): number => {
            const jLocus = payloadStart + offset;
            return jLocus < payloadEnd ? getMethylatedVal(jLocus) : fallback;
          };

          const spectralAffinity = getMethylatedPayloadVal(0, 0) / 25;
          const bandwidth = getMethylatedPayloadVal(1, 12) / 25;
          const expressionStyle = getMethylatedPayloadVal(2, 10) / 25;
          const scale = 0.35 + (getMethylatedPayloadVal(3, 12) / 25) * 1.5;
          const spinalPos = 0.05 + (getMethylatedPayloadVal(4, 0) / 25) * 0.9;
          const angle = 10 + (getMethylatedPayloadVal(5, 12) / 25) * 160;
          const hueShift = Math.round((getMethylatedPayloadVal(6, 5) / 25) * 360 - 180);

          organelles.push({
            spectralAffinity,
            bandwidth,
            expressionStyle,
            scale,
            spinalPos,
            angle,
            hueShift,
            geneStartIndex: geneStart,
            geneEndIndex: Math.min(currentLength - 1, stopFoundAt + 1)
          });

          activeGeneSpans.push({
            start: geneStart,
            end: Math.min(currentLength - 1, stopFoundAt + 1)
          });
        }
        scanIdx = Math.min(currentLength, stopFoundAt + 2);
      } else {
        scanIdx++;
      }
    } else {
      scanIdx++;
    }
  }

  // Derive Sensory scores for carnivory calculation
  let visusScore = 0;
  organelles.forEach(patch => {
    const bandAdjusted = 0.2 + patch.bandwidth * 0.8;
    const aff = patch.spectralAffinity;
    const wLight = Math.max(0, 1.0 - Math.abs(aff - 1.0) / bandAdjusted);    
    visusScore += wLight * patch.scale * 22;
  });
  visusScore = Math.min(100, Math.round(visusScore));

  // ==========================================================================
  // PREDATOR / PREY DNA CLASSIFICATION
  // ==========================================================================
  // Stiff, less-visual, heavy-notochord specimens emerge as aggressive carnivores!
  const carnivory = Math.max(0.0, Math.min(1.0, stiffness * 0.82 + (1.0 - visusScore / 100) * 0.18));
  const isPredator = carnivory >= 0.55;

  // Gen-coded Reproduction Strategies (r- vs. K-selection):
  // Locus 14 codes sexual maturity age in frames [300 to 2700 frames, e.g. 5s to 45s]
  const matureAge = Math.round(300 + (getMethylatedVal(14) / 25) * 2400);

  // Locus 13 codes reproductive stomach capacity threshold [60% to 95%]
  const reproThreshold = 0.60 + (getMethylatedVal(13) / 25) * 0.35;

  // Locus 15 codes energetic split loss tax [5% to 40%]
  const splitLoss = 0.05 + (getMethylatedVal(15) / 25) * 0.35;

  // ==========================================================================
  // 4. NATIVE CTRNN DIRECTED GRAPH COMPILER (NEAT-like topology)
  // ==========================================================================
  const K = organelles.length;

  // Locus 16 decodes how many Hidden CTRNN Nodes are compiled [2 to 8 nodes]
  const H = 2 + (getMethylatedVal(16) % 7); 
  const totalNodes = K + 1 + 4 + H; // Inputs + Outputs + Hidden

  const neurons: CTRNNNeuron[] = [];

  // A. Input Neurons (0 ... K)
  for (let i = 0; i < K; i++) {
    const patch = organelles[i];
    const deg = Math.round(patch.angle);
    let label = `Receptor (${deg}°)`;
    if (patch.spectralAffinity >= 0.8) label = `👁️ Vision (${deg}°)`;
    else if (patch.spectralAffinity >= 0.25 && patch.spectralAffinity <= 0.65) label = `👃 Smell (${deg}°)`;
    else if (patch.spectralAffinity < 0.25) label = `🔊 Tactile (${deg}°)`;

    neurons.push({
      id: i,
      type: "input",
      label,
      tau: 1.0, // direct sensory follow
      bias: 0.0
    });
  }
  // The Clock (last input)
  neurons.push({
    id: K,
    type: "input",
    label: "⌛ Hunger Clock",
    tau: 1.0,
    bias: 0.0
  });

  // B. Output Neurons (K+1 ... K+4) [Thrust, Left, Right, Flash]
  const outLabels = ["Thrust", "Turn Left", "Turn Right", "Biolum Flash"];
  for (let i = 0; i < 4; i++) {
    // Decode output bias from Loci 17+i
    const biasVal = getMethylatedVal((17 + i) % currentLength);
    const bias = (biasVal / 25) * 2.0 - 1.0; // [-1.0, 1.0]

    // Decode output time constant from Loci 18+i [0.2 to 2.0] Snappy & high speed!
    const tauVal = getMethylatedVal((18 + i) % currentLength);
    const tau = 0.2 + (tauVal / 25) * 1.8;

    neurons.push({
      id: K + 1 + i,
      type: "output",
      label: outLabels[i],
      tau,
      bias
    });
  }

  // C. Hidden Neurons (K+5 ... K+4+H)
  for (let i = 0; i < H; i++) {
    // Decode hidden bias and tau
    const biasVal = getMethylatedVal((19 + i) % currentLength);
    const bias = (biasVal / 25) * 2.0 - 1.0;

    const tauVal = getMethylatedVal((20 + i) % currentLength);
    const tau = 0.2 + (tauVal / 25) * 1.8;

    neurons.push({
      id: K + 5 + i,
      type: "hidden",
      label: `Hidden #${i + 1}`,
      tau,
      bias
    });
  }

  // D. Compile Synapses (4-codon packages starting from Locus 21, wrapping around genome!)
  const synapses: CTRNNSynapse[] = [];
  let dnaPointer = 21;

  // Let's decode up to (currentLength - 21)/4 synaptic connections!
  const synapsesCount = Math.floor((currentLength - 21) / 4);

  for (let s = 0; s < synapsesCount; s++) {
    const rawFrom = getMethylatedVal(dnaPointer % currentLength);
    const rawTo = getMethylatedVal((dnaPointer + 1) % currentLength);
    const rawW = getMethylatedVal((dnaPointer + 2) % currentLength);
    const rawUnused = getMethylatedVal((dnaPointer + 3) % currentLength);
    // 4th codon is unused or can code local synaptic plasticity decay
    dnaPointer += 4;

    const combinedFrom = totalNodes > 26 ? (rawFrom + rawUnused * 26) : rawFrom;
    const fromNode = combinedFrom % totalNodes;
    // Synapse CANNOT target input nodes! Target node must be either Hidden or Output (from K+1 onwards)
    const toNode = (rawTo % (4 + H)) + (K + 1);

    const weight = (rawW / 25) * 4.0 - 2.0; // Synaptic weight [-2.0, 2.0]

    // Prevent duplicate connections targeting the same node from same source (simplifies math!)
    if (!synapses.some(syn => syn.fromNode === fromNode && syn.toNode === toNode)) {
      synapses.push({ fromNode, toNode, weight });
    }
  }

  const brain: BrainTopology = {
    neurons,
    synapses
  };

  // ==========================================================================
  // 5. CALCULATE BMR (WITH GENETIC SYNAPSE DENSITY TAX)
  // ==========================================================================
  let integratedAreaSum = 0;
  for (let steps = 0; steps <= 20; steps++) {
    const sCoord = steps / 20;
    const rAtS = getSpinalRadiusAt(sCoord);
    integratedAreaSum += rAtS * rAtS * Math.PI;
  }
  const meanArea = (integratedAreaSum / 21);

  // Sum total synaptic connections to scale metabolic cost
  const totalSynapses = synapses.length;
  const brainComplexityMultiplier = 1.0 + totalSynapses * 0.06;

  // ongoing vestigial organ tax on BMR
  let vestigialOrganTax = 0.0;
  organelles.forEach(patch => {
    // Large, sharp receptors consume more continuous maintenance energy (Pillar 3: Vestigial tax!)
    const tax = patch.scale * patch.scale * (1.1 - patch.bandwidth) * 1.0;
    vestigialOrganTax += tax;
  });

  const basalMetabolicRate = Math.round(
    (meanArea * baseLength * 0.000008 + vestigialOrganTax) * 
    (1.0 + K * 0.08) * 
    (stiffness * 1.4) * 
    brainComplexityMultiplier
  );

  const eco = deriveEcologicalMetrics(cleanGenome, symmetry, emergentChambersCount, organelles, primaryColor, stiffness, pulseSpeed, wavePhase, carnivory);

  const segmentsDummy = Array(emergentChambersCount).fill(null).map(() => ({
    parentIndex: -1,
    localXOffset: 0,
    localYOffset: 25,
    baseRadius: meanRadius,
    elongation: 0,
    tilt: 0,
    amplitudes: amplitudes
  }));

  return {
    symmetry,
    primaryColor,
    secondaryColor,
    bodySeed,
    segments: segmentsDummy, 
    spinalHarmonics,
    emergentChambersCount,
    organelles,
    pulseSpeed,
    wavePhase,
    wiggleAmplitude,
    stiffness,
    matureAge,
    reproThreshold,
    splitLoss,
    brain,
    carnivory,
    isPredator,
    latinName: eco.latinName,
    sensoryVisus: eco.visus,
    sensoryOlfaction: eco.olfaction,
    sensoryTactility: eco.tactility,
    sensoryBiolum: eco.biolum,
    dietClass: eco.diet,
    preferredHabitat: eco.habitat,
    basalMetabolicRate,
    stomachCapacity,
    thermalToleranceMin,
    thermalToleranceMax,
    hydraulicPressure,
    rotationalInertia: 0, 
    survivalExpectation: eco.survivalScore,
    survivalAnalysis: eco.survivalAnalysis,
    chromatinState,
    epigeneticLogs,
    methylations,
    antisenseStrand,
    repairFidelity,
    insertionRate,
    deletionRate,
    genomeString: cleanGenome,
    activeGeneSpans
  };
}

/**
 * Dynamic Deep CTRNN Recurrent Brain signal execution (Euler temporal memory integration)
 */
export function executeBrain(
  brain: any, 
  inputs: number[], 
  neuronStates: number[], 
  neuronActivations: number[]
): { outputs: number[], allLayerActivations: number[][] } {
  const totalNodes = brain.neurons.length;

  // Initialize or resize temporal membrane states in-memory instantly!
  if (neuronStates.length !== totalNodes) {
    neuronStates.length = totalNodes;
    neuronStates.fill(0.0);
  }
  if (neuronActivations.length !== totalNodes) {
    neuronActivations.length = totalNodes;
    neuronActivations.fill(0.0);
  }

  const K = inputs.length - 1; // last input is the clock

  // 1. Direct Sensory Follow: Assign sensory inputs directly to input nodes
  for (let i = 0; i <= K; i++) {
    neuronActivations[i] = inputs[i];
  }

  // 2. Continuous Euler Integration: Update hidden and output neurons
  for (let i = K + 1; i < totalNodes; i++) {
    const neuron = brain.neurons[i];

    // Accumulate inputs from all incoming temporal synapses
    let sum = 0.0;
    brain.synapses.forEach((syn: any) => {
      if (syn.toNode === i) {
        sum += neuronActivations[syn.fromNode] * syn.weight;
      }
    });

    // Euler integration step (dt = 1.0, tau_i is the genetically encoded decay time)
    neuronStates[i] += (1.0 / neuron.tau) * (-neuronStates[i] + sum + neuron.bias);

    // Bounded potential clamping to prevent numeric drift explosions
    neuronStates[i] = Math.max(-4.0, Math.min(4.0, neuronStates[i]));

    // Sigmoidal / Tanh activation function
    neuronActivations[i] = Math.tanh(neuronStates[i]);
  }

  // 3. Map output node activations directly to the 4 motor directions
  // Output nodes are compiled at indices K+1 to K+4
  const outputs = [
    neuronActivations[K + 1],
    neuronActivations[K + 2],
    neuronActivations[K + 3],
    neuronActivations[K + 4]
  ];

  // Return the activations as a flat array for fine-grained direct visualization
  return { outputs, allLayerActivations: [neuronActivations] };
}
