---
name: client-rendering
description: Design principles for HTML5 Canvas rendering, interactive camera panning/zooming, Preact Signals telemetry, and HUD styling of Pixel DNA Life.
---
# client-rendering: Interactive Telemetry UI & High-FPS Canvas Rendering

## I. HTML5 Canvas Renderer
- **Implementation:** `src/render/creatureRenderer.ts`.
- **Visual Sourcing:** Since physical assets (images/textures) are absent, all rendering is procedural, drawing skeletal spines, vascular structures, organelles, and sensory cones using native Canvas 2D contexts, Bezier curves, and radial gradients.
- **Visuals Strategy:** Visually rich, "alive", high-tech cybernetic monitor aesthetics. Highlights creature segments, feeding actions, and binary brain operations using neon glowing styles.

## II. Cameras & Controls
- **Interactive Camera:** Mouse drag-panning, mouse-wheel zooming, and tactile keyboard shortcuts (e.g., `R` to reset camera, arrow keys for panning).
- **Transformation Matrix:** Uses coordinate conversion methods (`clientToWorld`, `worldToClient`) to map viewport mouse events to world coordinates.

## III. Preact Signals HUD Architecture
- **State Core:** `@preact/signals-core`.
- **Granular Updates:** The UI dashboard avoids heavy React-like virtual DOM re-renders by binding raw canvas-selected creatures to dedicated Signals (`selectedId`, `selectedEnergy`, `selectedGenome`).
- **Telemetry Display:** High-frequency sidebar updating specimen parameters, live brain neural activation meters, and species extinction statistics.
