---
name: testing-strategies
description: Testing methodologies, unit testing guidelines, CTRNN mathematical validation, and behavioral mocking for biological models in Pixel DNA Life.
---
# testing-strategies: Mathematical Validation & Bio-Mocking

## I. TDD & Right-Lane Allocation
- **Reproduction First:** Before patching any behavioral bug (e.g., failure in mitosis, wrong brain signal processing, incorrect collision physics), you MUST write a failing test case or reproduction script.
- **Focus:** Keep unit tests fast and focused. Avoid launching the full WebSocket server unless testing replication.

## II. CTRNN Validation
- **Euler Integration Precision:**
  - Validate that the time constant `tau` is bounded ([0.5 <= tau_i <= 5.0]) to prevent numerical divergence (NaN states).
  - Verify that the activation function limits output strictly to [-1.0, 1.0] or [0.0, 1.0].
- **Test Matrix:**
  - Write test cases checking that stable input signals produce expected asymptotic recurrent node activations.

## III. Biological and Physics Mocking
- **Mocking Creatures:**
  - Create static `CreatureAgent` mock states with pre-configured CTRNN topologies to isolate muscle/steering output calculations from environmental noise.
- **Mocking Collisions:**
  - Verify boundary bounce and spore absorption by supplying fixed coordinate trajectories to the physics solver and asserting velocity changes.
