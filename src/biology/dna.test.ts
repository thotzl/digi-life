import { describe, it, expect } from 'vitest';
import { 
  charToValue, 
  valueToChar,
  getComplementaryChar, 
  getComplementaryString, 
  generateRandomGenome, 
  mutateGenome,
  classifySensoryPatch,
  parseGenome,
  executeBrain
} from './dna';

describe('Watson-Crick Double Helix Operations', () => {
  it('should map character positions correctly within 26-letter alphabet', () => {
    expect(charToValue('A')).toBe(0);
    expect(charToValue('Z')).toBe(25);
    expect(charToValue('a')).toBe(0); // Case-insensitive
    expect(charToValue(' ')).toBe(0); // Non-alphabet characters return 0
  });

  it('should convert values back to characters correctly', () => {
    expect(valueToChar(0)).toBe('A');
    expect(valueToChar(25)).toBe('Z');
    expect(valueToChar(26)).toBe('Z');  // Upper bounded
    expect(valueToChar(-5)).toBe('A');  // Lower bounded
  });

  it('should compute base pairing complements correctly', () => {
    expect(getComplementaryChar('A')).toBe('Z');
    expect(getComplementaryChar('Z')).toBe('A');
    expect(getComplementaryChar('B')).toBe('Y');
    expect(getComplementaryString('ABC')).toBe('ZYX');
  });
});

describe('Genome Generation & Mutation Mechanics', () => {
  it('should generate random genomes of desired length within alphabet bounds', () => {
    const dna = generateRandomGenome(150);
    expect(dna.length).toBe(150);
    for (let i = 0; i < dna.length; i++) {
      expect(charToValue(dna[i])).toBeLessThanOrEqual(25);
    }
  });

  it('should mutate genomes by replacing exactly one letter', () => {
    const original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const { newGenome, mutatedIndex, oldChar, newChar } = mutateGenome(original);

    expect(newGenome.length).toBe(original.length);
    expect(newGenome[mutatedIndex]).toBe(newChar);
    expect(original[mutatedIndex]).toBe(oldChar);
    expect(oldChar).not.toBe(newChar);
  });

  it('[BUG-TEST 3] should handle empty genomes without crashing (throws error or handles safely)', () => {
    // EXPECTED SOLL-BEHAVIOR: mutateGenome on an empty string should throw a descriptive error rather than a runtime crash.
    expect(() => mutateGenome("")).toThrow("Genome cannot be empty");
  });
});

describe('Sensory Organelle Classifications', () => {
  it('should classify various sensory patches based on style and affinity', () => {
    // Taste Bud: aff 0.25-0.65, style < 0.3
    const tasteBud = classifySensoryPatch({
      spectralAffinity: 0.4,
      bandwidth: 0.2,
      expressionStyle: 0.1,
      scale: 1.0,
      spinalPos: 0.5,
      angle: 45,
      hueShift: 0,
      geneStartIndex: 0,
      geneEndIndex: 10
    });
    expect(tasteBud.name).toBe('Taste Bud');
    expect(tasteBud.color).toBe('#15803d');

    // Pigment Spot Eye: aff >= 0.8, style < 0.3
    const eyeSpot = classifySensoryPatch({
      spectralAffinity: 0.9,
      bandwidth: 0.1,
      expressionStyle: 0.2,
      scale: 1.0,
      spinalPos: 0.1,
      angle: 90,
      hueShift: 0,
      geneStartIndex: 0,
      geneEndIndex: 10
    });
    expect(eyeSpot.name).toBe('Pigment Spot Eye');
  });
});

describe('Genotype to Phenotype Translation (parseGenome)', () => {
  it('should transcribe valid phänotypes from random genomes', () => {
    const genome = generateRandomGenome(200);
    const phenotype = parseGenome(genome);

    expect(phenotype).toBeDefined();
    expect(phenotype.latinName).toContain('Str.');
    expect(phenotype.primaryColor.h).toBeGreaterThanOrEqual(0);
    expect(phenotype.primaryColor.h).toBeLessThanOrEqual(360);
    expect(phenotype.basalMetabolicRate).toBeGreaterThan(0);
    expect(phenotype.brain.neurons.length).toBeGreaterThanOrEqual(6); // At least clock + 4 outputs + 2 hiddens
  });

  it('[BUG-TEST 1] should allow synapses originating from nodes >= 26 in complex brains', () => {
    // Generate a high-entropy genome engineered to trigger $>26$ total nodes by inserting random fillers between start/stop codons:
    let complexGenome = "";
    for (let i = 0; i < 25; i++) {
      complexGenome += "ST" + generateRandomGenome(8) + "SP"; // ST (start), random payload (8), SP (stop)
    }
    const phenotype = parseGenome(complexGenome);

    expect(phenotype.brain.neurons.length).toBeGreaterThan(26);

    // Verify if there is at least one synapse whose fromNode >= 26 (originates in output or hidden neuron)
    const hasComplexPresynaptic = phenotype.brain.synapses.some(syn => syn.fromNode >= 26);
    expect(hasComplexPresynaptic).toBe(true);
  });
});

describe('CTRNN Brain Simulation Integration (executeBrain)', () => {
  it('should clamp neuron membrane states to [-4.0, 4.0] and activations to [-1.0, 1.0]', () => {
    const genome = generateRandomGenome(128);
    const phenotype = parseGenome(genome);
    const { brain } = phenotype;

    const numInputs = brain.neurons.filter((n: any) => n.type === 'input').length;
    const K = numInputs - 1; // Number of organelles

    // Supply highly saturated sensory values to force clamping
    const inputs = Array(numInputs).fill(100.0);
    const states = Array(brain.neurons.length).fill(0.0);
    const activations = Array(brain.neurons.length).fill(0.0);

    const { outputs, allLayerActivations } = executeBrain(brain, inputs, states, activations);

    // Outputs must represent the 4 motor directions
    expect(outputs.length).toBe(4);
    outputs.forEach(val => {
      expect(val).toBeGreaterThanOrEqual(-1.0);
      expect(val).toBeLessThanOrEqual(1.0);
    });

    // Check states are bounded in [-4.0, 4.0]
    states.forEach(state => {
      expect(state).toBeGreaterThanOrEqual(-4.0);
      expect(state).toBeLessThanOrEqual(4.0);
    });

    // Activations for hidden and output nodes (i > K) must be sigmoidal bounded in [-1.0, 1.0]
    allLayerActivations[0].slice(K + 1).forEach(act => {
      expect(act).toBeGreaterThanOrEqual(-1.0);
      expect(act).toBeLessThanOrEqual(1.0);
    });
  });
});
