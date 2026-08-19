import { describe, it, expect } from "vitest";
import { SpatialGrid } from "./spatialGrid";
import { CreatureAgent, FoodSpore } from "../shared/types";

describe("SpatialGrid 2D Partitioning", () => {
  it("should insert and query nearby creatures and food correctly", () => {
    const grid = new SpatialGrid();

    // Mock creature
    const mockCreature: CreatureAgent = {
      id: 1,
      speciesId: "species_a",
      genome: "",
      antisense: "",
      phenotype: {} as any,
      px: 150, // Col index: 1 (150/80)
      py: 120, // Row index: 1 (120/80)
      vx: 0,
      vy: 0,
      headingAngle: 0,
      omegaRot: 0,
      energy: 100,
      age: 0,
      generation: 1,
      adrenaline: 1.0,
      hasEaten: false,
      neuronStates: [],
      neuronActivations: []
    };

    // Mock food spore
    const mockFood: FoodSpore = {
      x: 170, // Col index: 2 (170/80)
      y: 90,  // Row index: 1 (90/80)
      vx: 0,
      vy: 0
    };

    grid.insertCreature(mockCreature);
    grid.insertFood(mockFood);

    // Query close to creature (range 80)
    const nearbyCreatures = grid.getNearbyCreatures(150, 120, 80);
    expect(nearbyCreatures).toContain(mockCreature);
    expect(nearbyCreatures.length).toBe(1);

    const nearbyFood = grid.getNearbyFood(150, 120, 80);
    expect(nearbyFood).toContain(mockFood);
    expect(nearbyFood.length).toBe(1);

    // Query far away (should be empty)
    const emptyCreatures = grid.getNearbyCreatures(1000, 1000, 50);
    expect(emptyCreatures.length).toBe(0);

    const emptyFood = grid.getNearbyFood(1000, 1000, 50);
    expect(emptyFood.length).toBe(0);

    // Clear grid and query again
    grid.clear();
    expect(grid.getNearbyCreatures(150, 120, 80).length).toBe(0);
    expect(grid.getNearbyFood(150, 120, 80).length).toBe(0);
  });

  it("should clip coordinate lookups within valid boundaries safely", () => {
    const grid = new SpatialGrid();

    const mockCreature: CreatureAgent = {
      id: 2,
      speciesId: "species_b",
      genome: "",
      antisense: "",
      phenotype: {} as any,
      px: -50, // completely out of bounds (left)
      py: 20000, // completely out of bounds (bottom)
      vx: 0,
      vy: 0,
      headingAngle: 0,
      omegaRot: 0,
      energy: 100,
      age: 0,
      generation: 1,
      adrenaline: 1.0,
      hasEaten: false,
      neuronStates: [],
      neuronActivations: []
    };

    // Should insert without throwing (clipped to edge cell)
    expect(() => grid.insertCreature(mockCreature)).not.toThrow();
    
    // Querying the bottom-left corner should find it
    const cornerPeers = grid.getNearbyCreatures(0, 10800, 100);
    expect(cornerPeers).toContain(mockCreature);
  });
});
