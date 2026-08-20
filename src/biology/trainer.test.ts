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

import { calculateSandboxFitness } from '../shared/physics';

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
    // 1000 + 2000 * (100 / 100) + (300 - 200) * 0.2 = 1000 + 2000 + 20 = 3020.0
    expect(fit).toBe(3020.0);
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
    // 1000 + 2000 * 0.5 + 20 = 2020.0
    expect(fit).toBe(2020.0);
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
    // (1000 + 2000 * 1 + 20) * 0.70 = 3020 * 0.70 = 2114.0
    expect(fit).toBe(2114.0);
  });

  it('should absolute clamp standstill and passive crawling unsuccessful runs (< 180px) to 0.0 points', () => {
    // Passive crawling (traveled only 100px), didn't reach food, and didn't get extremely close (50px distance)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      100,        // distanceTraveled (passive crawling)
      0,          // wallCollisions
      50,         // curDist
      510,        // endX
      500         // endY
    );
    expect(fit).toBe(0.0);
  });

  it('should exempt passive crawling from 0.0 penalty if they got extremely close (< 20px) to the food', () => {
    // Passive crawling (traveled only 50px) but got extremely close to food (15px remaining, within 20px limit)
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
    // baseFit = 1000.0 * (1 - 15/100) = 850.0
    // kineticWaste = 50 * 0.1 = 5.0
    // fit = (850 - 5) = 845.0
    expect(fit).toBeCloseTo(845.0, 5);
  });

  it('should deduct a metabolic kinetic waste tax for unsuccessful aimless wandering', () => {
    // Swam 200px (active, >180px limit), didn't reach food, got 80% close to food (20px remaining)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      200,        // distanceTraveled (active, >180px)
      0,          // wallCollisions
      20,         // curDist (got 80px closer)
      700,        // endX (200px displacement, straight line)
      500         // endY
    );
    // baseFit = 1000.0 * (1 - 20/100) = 800.0
    // kineticWaste = 200 * 0.1 = 20.0
    // finalFit = (800.0 - 20.0) = 780.0
    expect(fit).toBeCloseTo(780.0, 5);
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
    // Swam a massive 9000px, didn't reach food, got 80px close (800 base proximity points)
    const fit = calculateSandboxFitness(
      false,      // finished
      undefined,  // finishTick
      300,        // epochDurationTicks
      100,        // startDistance
      9000,       // distanceTraveled (aimless wandering)
      0,          // wallCollisions
      20,         // curDist
      9500,       // endX
      500         // endY
    );
    // baseFit = 1000 * (1 - 20/100) = 800.0
    // kineticWaste = 9000 * 0.1 = 900.0
    // finalFit = Math.max(0, 800 - 900) = 0.0
    expect(fit).toBe(0.0);
  });
});
