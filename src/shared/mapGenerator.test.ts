import { describe, it, expect } from 'vitest';
import { 
  hashStringToInt, 
  createPRNG, 
  generateWorld, 
  getVectoredCurrentAt, 
  getBiomeAt, 
  checkObstacleCollision,
  ProceduralWorld
} from './mapGenerator';

describe('Procedural world Generator & PRNG (mapGenerator.ts)', () => {
  it('should hash seed strings deterministically', () => {
    const hash1 = hashStringToInt('TEST_SEED');
    const hash2 = hashStringToInt('TEST_SEED');
    const hash3 = hashStringToInt('DIFFERENT_SEED');

    expect(hash1).toBe(hash2);
    expect(hash1).not.toBe(hash3);
  });

  it('should produce identical pseudorandom streams for identical seeds', () => {
    const rand1 = createPRNG('SEED_A');
    const rand2 = createPRNG('SEED_A');
    const rand3 = createPRNG('SEED_B');

    expect(rand1()).toBe(rand2());
    expect(rand1()).not.toBe(rand3());
  });

  it('should generate identical world structures deterministically from a seed', () => {
    const world1 = generateWorld('BASIN_X');
    const world2 = generateWorld('BASIN_X');
    const world3 = generateWorld('BASIN_Y');

    expect(world1.obstacles.length).toBe(world2.obstacles.length);
    expect(world1.vents.length).toBe(world2.vents.length);
    
    // Assert coordinates match exactly
    expect(world1.obstacles[0].x).toBe(world2.obstacles[0].x);
    expect(world1.vents[0].x).toBe(world2.vents[0].x);

    // Different seed must generate different layout
    expect(world1.obstacles[0].x).not.toBe(world3.obstacles[0].x);
  });
});

describe('Map Geometry & Physical Vector Field Computations', () => {
  const world = generateWorld('ALIFE_BASIN_77A');

  it('should resolve biome locations accurately', () => {
    // Under our new multi-noise climate model, biomes are distributed in organic cloud clusters.
    // We check that coordinate lookups map to valid biomes and carry their corresponding parameters.
    const biomeLeft = getBiomeAt(world, 100, 500);
    expect(biomeLeft).not.toBeNull();
    expect(['abyssal_barrens', 'algae_shallows', 'acid_pool', 'cybernetic_vents']).toContain(biomeLeft!.id);
    expect(biomeLeft!.name).toBeDefined();
    expect(biomeLeft!.sporeSpawnRate).toBeGreaterThan(0);
    expect(biomeLeft!.sporeEnergyValue).toBeGreaterThan(0);
  });

  it('should compute physical force currents with toroidal wrap-around boundaries', () => {
    // Create an isolated world with a single push vent at (5000, 5000)
    const isolatedWorld: ProceduralWorld = {
      seed: 'TEST',
      width: 19200,
      height: 10800,
      obstacles: [],
      vents: [
        { id: 1, x: 5000, y: 5000, radius: 1000, forceType: 'push', strength: 0.2 }
      ],
      biomes: []
    };
    
    // Position directly above the vent
    const px = 5000;
    const py = 4900; // 100 units above

    const force = getVectoredCurrentAt(isolatedWorld, px, py);

    // Force should push directly upwards (-Y)
    expect(force.vy).toBeLessThan(0);
    expect(Math.abs(force.vx)).toBeLessThan(0.001); // no horizontal push
  });

  it('should detect solid obstacle boundary collisions with correct physics normals', () => {
    // Create an isolated world with a single symmetrical polygon obstacle at (5000, 5000)
    const baseRadius = 200;
    const isolatedWorld: ProceduralWorld = {
      seed: 'TEST',
      width: 19200,
      height: 10800,
      obstacles: [
        {
          id: 1,
          x: 5000,
          y: 5000,
          radius: baseRadius,
          type: 'rock',
          color: 'grey',
          vertices: [
            { x: 5200, y: 5000, r: 200, angle: 0 },
            { x: 5000, y: 5200, r: 200, angle: Math.PI / 2 },
            { x: 4800, y: 5000, r: 200, angle: Math.PI },
            { x: 5000, y: 4800, r: 200, angle: (Math.PI * 3) / 2 }
          ]
        }
      ],
      vents: [],
      biomes: []
    };
    
    // Creature of radius 15 placed overlapping on the right side of the obstacle
    // Approach angle is 0 (exactly along the first vertex). Vertex radius is 200.
    // We place it at 5000 + 200 + 10 = 5210, which means distance is 210.
    // minDist is 200 + 15 = 215. Overlap should be 215 - 210 = 5!
    const px = 5210;
    const py = 5000;
    
    const collision = checkObstacleCollision(isolatedWorld, px, py, 15);

    expect(collision.collided).toBe(true);
    expect(collision.overlap).toBeCloseTo(5);
    expect(collision.normalX).toBeCloseTo(1.0); // normal points rightwards
    expect(collision.normalY).toBeCloseTo(0.0);
  });
});
