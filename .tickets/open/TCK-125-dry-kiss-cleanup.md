---
id: TCK-125
title: DRY, KISS, and Modularity Pass (Frontend Cleanup)
status: open
assigned: Gemini-CLI
created_at: 2026-08-31
---

# TCK-125: DRY, KISS, and Modularity Pass (Frontend Cleanup)

## Background & Motivation
Following the radical workspace consolidation (TCK-124), the frontend still contained obsolete, duplicated, and overly complex artifacts (such as the 400-line procedural map generator) that violated the DRY and KISS principles, as well as our "Single Point of Truth" (SPOT) mandate (where Rust is the true engine). This ticket tracked the aggressive pruning of this dead code and simplification of rendering paths.

## Scope & Impact
- **Eradicate `mapGenerator.ts`:** The frontend map generator drew procedural obstacles that the Rust backend did not know about. This created a visual disconnect and violated SPOT. We deleted this 400-line file and its test suite entirely.
- **Simplify `ocean.ts` and `trainer.ts`:** Removed the calls to `generateWorld` and `drawWorldTerrain`. The backgrounds are simplified to a clean, performant dark gradient until the Rust backend natively supplies physical obstacles (e.g., for TCK-123 ReefAvoidance).
- **Consolidate Redundancies:** Cleaned up unused properties or leftover imports across the frontend.

## Solution Implemented

### 1. Deleted Procedural Map Generator
- Deleted `frontend/core/mapGenerator.ts`.
- Deleted `frontend/core/mapGenerator.test.ts`.

### 2. Refactored `ocean.ts`
- Removed imports and references to `ProceduralWorld` and `generateWorld`.
- Deleted `createBiomeCache` and the `drawWorldTerrain` function.
- Simplified `drawBetaSimulationFrame` to just clear the canvas and render creatures/spores without drawing the complex procedural biome and vent graphics.

### 3. Refactored `trainer.ts`
- Removed imports and references to `ProceduralWorld` and `generateWorld`.
- Simplified `drawSandbox` by removing the `world.obstacles` drawing loops.
- Removed the `world` property from the `Sandbox` interface.
