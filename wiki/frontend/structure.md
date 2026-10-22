---
sources: [.agents/skills/client-rendering/SKILL.md, README.md, docs/revisit/06-frontend.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-141-frontend-hygiene.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md]
---

# Frontend Structure

`frontend/` — Vite 5 + TypeScript 5 (strict) + `@preact/signals-core`; plain DOM, no UI framework, no virtual DOM. `index.html` is a single mount point `<div id="app">` (SPA since TCK-115, menu and router since TCK-128).

## Boot and navigation

- `main.ts` mounts all views once (`views/MainMenu.ts`, `OceanView.ts`, `TrainerView.ts`, `CatalogueView.ts`), then dynamically imports `ocean.ts` and `trainer.ts`; visibility follows the `currentView` signal.
- `core/router.ts` — `navigateTo(view)` pauses loops and sends `SET_MODE` to the backend; the only place that sets the mode ([[wiki/architecture/invariants]]).
- `api.ts` — `safeInvoke` wrapper around `invoke` (returns `null` outside Tauri and on errors, logging `console.warn`) and a phenotype cache over `get_fossil_phenotype`. `ocean.ts` also calls `invoke` directly. Failed saves/deletes give no UI feedback (RV-F04).

## Loop modules

- `ocean.ts` / `trainer.ts` — side-effect modules with global state: `"simulation-state"` listeners registered on import (never removed, each handles all event types, RV-F05) and render loops (`drawBetaSimulationFrame` via `requestAnimationFrame` in Ocean). Coupled to views via `document.getElementById`.

## Views and components

- `views/UnifiedDiagnosticsPanel.ts` — shared inspector sidebar for Ocean, Trainer, Catalogue: HD preview (own animation loop), vitals/meta, Watson-Crick double helix (chromatin), CTRNN brain graph, raw DNA copy (TCK-128). The catalogue preview loop also exists in `ocean.ts` (RV-F06).
- `views/CatalogueView.ts` — fullscreen catalogue, selection dialog with synapse restore options.
- `core/styled.ts` — small typed CSS-in-JS engine; `applyChildren` treats strings starting with `<` as HTML. `class`/`className` props are added to the generated class via `classList` (before the TCK-186 smoke check they were assigned as a no-op JS property, so e.g. the main-menu card hover styles and catalogue classes never applied). `styles.css` global styles.
- `core/escape.ts` — `escapeHtml` for interpolated DB/user data (TCK-141).
- `components/Icons.ts` — SVG icons.

## Rendering

- `render/creatureRenderer.ts` — procedural Canvas 2D creatures (spine, organelles as smooth receptor domes with `wAngular = 0.18 + bandwidth · 0.22` since FIX-131, sensory cones); no image assets; wiggle coupled to forward speed.
- `render/brainRenderer.ts` — CTRNN graph with hover tooltips for neurons and synapses.
- `core/camera.ts` — `InteractiveCamera`: drag pan, wheel zoom, click selection; no keyboard shortcuts.

## State

- `signals.ts`: `selectedId`, `selectedEnergy`, `selectedGenome`, `selectedPhenotype`, `selectedBrainActivations`, `speciesRosterSignal`, ...; UI reacts via signal `effect`s.
- `signals.ts` also contains legacy positional-genome helpers (`getLocusDescription`, `computeActiveGeneSpans` with `ST`/`GO`) that do not match the promoter-based compiler (RV-F03).
- `shared/types.ts` — handwritten types, not derived from `bindings/` (RV-A03, RV-F01).

## Security

- Untrusted strings (names, genomes, logs) go through `textContent` or `escapeHtml`, never raw `innerHTML` (RV-F02, TCK-141).
- CSP is `null` in `tauri.conf.json`.

## Hardcoded values

- World size 19200/10800 (5×), population display `/ 25`, fossil fallback `selectedAge = 2700` (RV-F08).
- `ActiveView` contains `"creator"` without a view; `concurrently`, `tsx` and Stryker are configured but unused (RV-F07).

## Key Takeaways

- Four views mounted once, router owns the backend mode, renderers are pure Canvas 2D.
- Output escaping is in place; there is no CSP and no per-view listener lifecycle.

Sources: `.agents/skills/client-rendering/SKILL.md`, `docs/revisit/06-frontend.md`, TCK-115, TCK-128, TCK-141, FIX-131, TCK-186.
