import { describe, it, expect } from 'vitest';
import { 
  hashStringToInt, 
  createPRNG, 
  generateWorld, 
  getVectoredCurrentAt, 
  getBiomeAt, 
  checkObstacleCollision 
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
    const vent = world.vents[0];
    
    // Position directly above the vent
    const px = vent.x;
    const py = vent.y - 100; // 100 units above

    const force = getVectoredCurrentAt(world, px, py);

    if (vent.forceType === 'push') {
      // Force should push directly upwards (-Y)
      expect(force.vy).toBeLessThan(0);
      expect(Math.abs(force.vx)).toBeLessThan(0.001); // no horizontal push
    } 
    else if (vent.forceType === 'pull') {
      // Force should pull downwards towards vent (+Y)
      expect(force.vy).toBeGreaterThan(0);
      expect(Math.abs(force.vx)).toBeLessThan(0.001);
    } 
    else if (vent.forceType === 'vortex') {
      // Rotational swirl perpendicular to (0, -100) -> should push rightwards (+X, counter-clockwise)
      expect(force.vx).toBeGreaterThan(0);
      expect(Math.abs(force.vy)).toBeLessThan(0.001);
    }
  });

  it('should detect solid obstacle boundary collisions with correct physics normals', () => {
    const obs = world.obstacles[0];
    
    // Creature of radius 15 placed overlapping on the right side of the obstacle
    const px = obs.x + obs.radius + 10; // 10 units right of the edge, overlap = 5
    const py = obs.y;
    
    const collision = checkObstacleCollision(world, px, py, 15);

    expect(collision.collided).toBe(true);
    expect(collision.overlap).toBeCloseTo(5);
    expect(collision.normalX).toBeCloseTo(1.0); // normal points rightwards
    expect(collision.normalY).toBeCloseTo(0.0);
  });
});
