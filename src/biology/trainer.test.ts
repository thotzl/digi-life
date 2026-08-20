import { vi, describe, it, expect } from 'vitest';

vi.hoisted(() => {
  (global as any).window = {
    location: { hostname: 'localhost' },
    addEventListener: () => {}
  };
  (global as any).document = {
    addEventListener: () => {},
    createElement: () => ({
      setAttribute: () => {},
      appendChild: () => {},
      addEventListener: () => {},
      style: {},
      classList: {
        add: () => {},
        remove: () => {}
      },
      getContext: () => ({
        fillRect: () => {},
        translate: () => {},
        scale: () => {},
        save: () => {},
        restore: () => {}
      })
    }),
    getElementById: () => ({
      addEventListener: () => {},
      appendChild: () => {},
      setAttribute: () => {},
      querySelector: () => null,
      querySelectorAll: () => [],
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
      0,          // curDist
      600,        // endX
      500         // endY
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
      0,          // curDist
      600,        // endX
      500         // endY
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
      0,          // curDist
      600,        // endX
      500         // endY
    );
    // (2000 * 1 + 20) * 0.70 = 2020 * 0.70 = 1414.0
    expect(fit).toBe(1414.0);
  });

  it('should absolute clamp standstill and passive crawling unsuccessful runs (< 120px) to 0.0 points', () => {
    // Passive crawling (traveled only 50px), didn't reach food, and didn't get extremely close (50px distance)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      50,         // distanceTraveled (passive crawling)
      0,          // wallCollisions
      50,         // curDist
      510,        // endX
      500         // endY
    );
    expect(fit).toBe(0.0);
  });

  it('should exempt passive crawling from 0.0 penalty if they got extremely close (< 30px) to the food', () => {
    // Passive crawling (traveled only 50px) but got extremely close to food (15px remaining)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      50,         // distanceTraveled (passive crawling but close!)
      0,          // wallCollisions
      15,         // curDist (extremely close!)
      510,        // endX
      500         // endY
    );
    // baseFit = 100 * (1 - 15/100) = 85.0
    // kineticWaste = 50 * 0.28 = 14.0
    // fit = (85 - 14) = 71.0
    expect(fit).toBe(71.0);
  });

  it('should deduct a metabolic kinetic waste tax for unsuccessful aimless wandering', () => {
    // Swam 150px, didn't reach food, got 50% close to food
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      150,        // distanceTraveled (swam a bit)
      0,          // wallCollisions
      50,         // curDist (got 50px closer)
      650,        // endX (150px displacement, straight line)
      500         // endY
    );
    // baseFit = 100 * (1 - 50/100) = 50.0
    // kineticWaste = 150 * 0.28 = 42.0
    // finalFit = (50.0 - 42.0) = 8.0
    expect(fit).toBeCloseTo(8.0, 5);
  });

  it('should absolute clamp unsuccessful runs to 0.0 if they swam in circles around the center', () => {
    // Swam 300px, didn't reach food, but net displacement from center is only 20px (circle!)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      300,        // distanceTraveled (swam a lot)
      0,          // wallCollisions
      20,         // curDist (got closer)
      520,        // endX (only 20px displacement from center!)
      500         // endY
    );
    expect(fit).toBe(0.0);
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
      20,         // curDist
      1000,       // endX
      500         // endY
    );
    // baseFit = 80.0
    // kineticWaste = 800 * 0.28 = 224.0
    // finalFit = Math.max(0, 80 - 224) = 0.0
    expect(fit).toBe(0.0);
  });
});
