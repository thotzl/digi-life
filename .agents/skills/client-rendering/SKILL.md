---
name: client-rendering
description: Design principles for HTML5 Canvas rendering, interactive camera panning/zooming, Preact Signals telemetry, and HUD styling of Pixel DNA Life.
---
# client-rendering: Canvas Rendering, Views & Signals HUD

## I. Structure (`frontend/`)
- `main.ts` mounts all views once (`views/MainMenu.ts`, `OceanView.ts`, `TrainerView.ts`, `CatalogueView.ts`), then dynamically imports `ocean.ts` and `trainer.ts`; visibility follows the `currentView` signal.
- `core/router.ts` — `navigateTo(view)` pauses loops and sends `SET_MODE` to the backend. Only the router sets the mode; `trainer.ts` must never send `SET_MODE: trainer` on load.
- `api.ts` — `safeInvoke` wrapper around Tauri `invoke` (returns `null` outside Tauri) and a phenotype cache over `get_fossil_phenotype`.
- `ocean.ts` / `trainer.ts` — event listeners for `"simulation-state"` and the render loops (`drawBetaSimulationFrame` via `requestAnimationFrame` in Ocean).
- `views/UnifiedDiagnosticsPanel.ts` — shared inspector sidebar (vitals, chromatin, brain graph) used by Ocean, Trainer, Catalogue.
- `shared/types.ts` — handwritten types; not yet derived from `bindings/` (RV-A03).

## II. Canvas Renderers
- `render/creatureRenderer.ts` — procedural creature drawing (spine, organelles, sensory cones) with Canvas 2D; no image assets. Body wiggle is coupled to forward speed.
- `render/brainRenderer.ts` — CTRNN graph with hover tooltips for neurons and synapses.

## III. Camera (`core/camera.ts`)
- `InteractiveCamera`: mouse-drag panning, wheel zoom, click selection. No keyboard shortcuts are implemented.

## IV. Signals HUD
- State core: `@preact/signals-core` (`frontend/signals.ts`): `selectedId`, `selectedEnergy`, `selectedGenome`, `selectedPhenotype`, `selectedBrainActivations`, `speciesRosterSignal`, ...
- UI is built with plain DOM (`core/styled.ts`) and signal `effect`s; no virtual DOM framework.
- Untrusted strings (names, genomes) must be written with `textContent`, not `innerHTML` (RV-F02).
