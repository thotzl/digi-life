import { describe, it, expect, beforeEach } from 'vitest';
import { readDb, writeDb, readState, writeState, clearState } from './db';
import { SpeciesRecord } from '../shared/types';
import { generateRandomGenome, getComplementaryString } from '../biology/dna';

describe('SQLite database Manager (src/server/db.ts)', () => {
  // Let's clear the database table before each test to guarantee isolated state
  beforeEach(() => {
    writeDb([]); // passing empty array wipes table in our manager transaction
    clearState();
  });

  it('should write and read species records accurately in WAL mode', () => {
    const genome = generateRandomGenome(128);
    const anti = getComplementaryString(genome);
    
    const record: SpeciesRecord = {
      id: genome,
      name: "Testus bilateralis",
      genome,
      antisense: anti,
      parentSpeciesId: null,
      status: "alive",
      peakPopulation: 5,
      birthTime: Date.now(),
      generation: 1,
      carnivory: 0.15
    };

    // Insert
    writeDb([record]);

    // Read back
    const list = readDb();
    expect(list.length).toBe(1);
    expect(list[0].id).toBe(genome);
    expect(list[0].name).toBe("Testus bilateralis");
    expect(list[0].peakPopulation).toBe(5);
  });

  it('should update peak population on conflict upsert correctly', () => {
    const genome = generateRandomGenome(128);
    const anti = getComplementaryString(genome);
    
    const record1: SpeciesRecord = {
      id: genome,
      name: "Testus bilateralis",
      genome,
      antisense: anti,
      parentSpeciesId: null,
      status: "alive",
      peakPopulation: 5,
      birthTime: Date.now(),
      generation: 1,
      carnivory: 0.15
    };

    writeDb([record1]);

    const record2: SpeciesRecord = {
      ...record1,
      peakPopulation: 12, // higher population
      status: "extinct"
    };

    writeDb([record2]);

    const list = readDb();
    expect(list.length).toBe(1);
    expect(list[0].status).toBe("extinct");
    expect(list[0].peakPopulation).toBe(12); // updated to 12
  });

  it('should serialize, save, retrieve, and clear simulation snapshots correctly', () => {
    const mockState = {
      creaturesCount: 25,
      foodCount: 300,
      metadata: { seed: 4293 }
    };

    // Verify empty baseline
    expect(readState()).toBeNull();

    // Save state
    writeState(mockState);

    // Retrieve state
    const retrieved = readState();
    expect(retrieved).not.toBeNull();
    expect(retrieved.creaturesCount).toBe(25);
    expect(retrieved.metadata.seed).toBe(4293);

    // Clear state
    clearState();
    expect(readState()).toBeNull();
  });
});
