---
id: FIX-131
title: Hox-Gene driven Bilateral Symmetry, Smooth Organelle Visuals, and Physical/Haptic Reef Collision
status: closed
assigned: Gemini-CLI
created_at: 2026-09-03
closed_at: 2026-09-03
---

# FIX-131: Hox-Gene driven Bilateral Symmetry, Smooth Organelle Visuals, and Physical/Haptic Reef Collision

## Description
This ticket addresses the prerequisite anatomical, physical, and sensory defects identified during codebase analysis of the "Reality Simulation Gap" between the small training chambers and the open Ocean:
1. **Symmetry Defect (Error 1):** Organelles grew asymmetrically on only one body flank, forcing creatures to circle blind spots and causing unbalanced swimming force.
2. **"Pin/Pimmel" Visuals (Error 2):** Wimpern and sensory organs were rendered with an extremely narrow angular attachment, causing them to protrude as rigid pins.
3. **Ghost Reefs (Error 3):** Circular obstacles (reefs) lacked physical collision or tactile sensory feedback in the native engine, allowing creatures to swim through them blindly.

## Resolution & Implementation

### 1. HOX Genetics & Bilateral Symmetry (dna.rs)
- Implemented active `"HOX"` gene scanning using Golden-Middle biochem group matching.
- **Asymmetric Mode (HOX missing):** Organs are distributed asymmetrically across a wider circular range of $10^\circ$ to $350^\circ$, simulating primitive species.
- **Bilateral Symmetrie Mode (HOX active):** 
  - **Coalescence on Midline:** If an organ falls near the midline (within $\pm 5^\circ$ of $90^\circ$ or $270^\circ$), it snaps to exactly $90^\circ$/$270^\circ$ and is rendered as a single un-mirrored central organ (avoiding double-eyes on the spine).
  - **Bilateral Mirroring:** If the angle is lateral, the parser automatically spawns a mirrored partner organ at $360^\circ - \theta$.
- **Stereo Connection:** Due to full-graph synapse exuberance at birth, both mirrored organs connect to the CTRNN network, generating balanced propulsion and stereoskopische sensory processing (Stereo Vision/Olfaction).
- **Graceful Fallback:** Verified that if `"EN"` is missing (open-end fallback), the parser falls back to the 15-char default and still applies symmetry.
- **Unit Tests:** Programmed `test_hox_bilateral_symmetry` to mathematically verify asymmetric primitive range, bilateral paired mirroring, midline snapping, and open-end fallback.

### 2. Beautiful Sensory Hillocks (creatureRenderer.ts)
- Widened the angular base (`wAngular` from `0.05` to `0.18 + patch.bandwidth * 0.22`) and adjusted heights of ciliated protrusions.
- This creates smooth, organic receptor domes/bulges integrating elegantly into the envelope instead of sharp pins.

### 3. Solid Physical & Haptic Reefs (physics.rs / brain.rs)
- **Physical Collision:** Added circle-to-circle collision checks, boundary pushing, and elastic normal vector deflection in `apply_creature_physics` and `step_food_spore_physics` for creatures and food pellets against `ProceduralObstacle` reefs.
- **Haptic Sensing:** Extended `compute_sensory_inputs` Channel 1 (Mechanical Hardness) so that tactile organs feel reef boundaries with a pressure of `1.0` upon proximity.
- **Caller Updates:** Passed the obstacles from `sb.world.obstacles` in the sandboxes and `ocean_world.obstacles` (now persistently pre-generated in `engine.rs` to optimize performance) to all physics and sensory update callers.

## Verification
- Running `npm run test:rust` passes all 34 backend unit-tests successfully.
- Running `npm run build` compiles all frontend components and types cleanly with zero errors.
