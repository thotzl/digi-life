import { vi, describe, it, expect } from 'vitest';

vi.hoisted(() => {
  (global as any).window = {
    location: { hostname: 'localhost' },
    addEventListener: () => {}
  };
  (global as any).document = {
    addEventListener: () => {},
    getElementById: () => ({
      addEventListener: () => {},
      appendChild: () => {},
      setAttribute: () => {},
      querySelector: () => null,
      getContext: () => ({
        fillRect: () => {},
        translate: () => {},
        scale: () => {},
        save: () => {},
        restore: () => {}
      })
    })
  };
});

import { calculateSandboxFitness } from '../trainer';

describe('Trainer Fitness Evaluation & Penalization', () => {
  it('should reward successful direct navigation with maximal path-efficiency score', () => {
    // Perfectly direct path, 0 wall collisions
    const fit = calculateSandboxFitness(
      true,       // finished
      200,        // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      100,        // distanceTraveled
      0,          // wallCollisions
      0           // curDist
    );
    // 2000 * (100 / 100) + (300 - 200) * 0.2 = 2000 * 1 + 20 = 2020.0
    expect(fit).toBe(2020.0);
  });

  it('should penalize winding zigzag successful paths via path efficiency ratio', () => {
    // Winding path (traveled 200px to cover 100px straight line), 0 wall collisions
    const fit = calculateSandboxFitness(
      true,       // finished
      200,        // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      200,        // distanceTraveled
      0,          // wallCollisions
      0           // curDist
    );
    // pathEfficiency = 100 / 200 = 0.5
    // 2000 * 0.5 + 20 = 1020.0
    expect(fit).toBe(1020.0);
  });

  it('should penalize successful runs with a 15% wall-collision tax per hit', () => {
    // 2 wall collisions (30% penalty, multiplier = 0.70)
    const fit = calculateSandboxFitness(
      true,       // finished
      200,        // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      100,        // distanceTraveled
      2,          // wallCollisions
      0           // curDist
    );
    // (2000 * 1 + 20) * 0.70 = 2020 * 0.70 = 1414.0
    expect(fit).toBe(1414.0);
  });

  it('should absolute clamp standstill (no movement) unsuccessful runs to 0.0 points', () => {
    // Stood still (traveled only 5px), didn't reach food
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      5,          // distanceTraveled (stands still)
      0,          // wallCollisions
      50          // curDist (got 50px closer by spawn luck)
    );
    expect(fit).toBe(0.0);
  });

  it('should deduct a metabolic kinetic waste tax for unsuccessful aimless wandering', () => {
    // Swam 300px, didn't reach food, got 80% close to food
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      300,        // distanceTraveled (swam a lot)
      0,          // wallCollisions
      20          // curDist (got 80px closer, so curDist = 20px)
    );
    // baseFit = 100 * (1 - 20/100) = 80.0
    // kineticWaste = 300 * 0.12 = 36.0
    // finalFit = (80.0 - 36.0) = 44.0
    expect(fit).toBe(44.0);
  });

  it('should reduce unsuccessful fitness to exactly 0.0 if metabolic waste exceeds closeness points', () => {
    // Swam a massive 800px, didn't reach food, got 80px close (80 points)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      800,        // distanceTraveled (aimless wandering)
      0,          // wallCollisions
      20          // curDist
    );
    // baseFit = 80.0
    // kineticWaste = 800 * 0.12 = 96.0
    // finalFit = Math.max(0, 80 - 96) = 0.0
    expect(fit).toBe(0.0);
  });
});
