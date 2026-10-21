---
sources: [.agents/skills/client-rendering/SKILL.md, README.md, docs/revisit/06-frontend.md, .tickets/closed/TCK-115-master-architectural-refactoring.md, .tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, .tickets/closed/TCK-141-frontend-hygiene.md]
---

# Frontend Structure

`frontend/` — Vite 5 + TypeScript 5 (strict) + `@preact/signals-core`; plain DOM, no UI framework, no virtual DOM. `index.html` is a single mount point `<div id="app">`.

## Boot and navigation

- `main.ts` mounts all views once (`views/MainMenu.ts`, `OceanView.ts`, `TrainerView.ts`, `CatalogueView.ts`), then dynamically imports `ocean.ts` and `trainer.ts`; visibility follows the `currentView` signal.
- `core/router.ts` — `navigateTo(view)` pauses loops and sends `SET_MODE` to the backend; the only place that sets the mode ([[architecture/invariants]]).
- `api.ts` — `safeInvoke` wrapper around `invoke` (returns `null` outside Tauri) and a phenotype cache over `get_fossil_phenotype`.

## Loop modules

- `ocean.ts` / `trainer.ts` — side-effect modules with global state: `"simulation-state"` listeners and render loops (`drawBetaSimulationFrame` via `requestAnimationFrame` in Ocean). Coupled to views via `document.getElementById`.

## Views and components

- `views/UnifiedDiagnosticsPanel.ts` — shared inspector sidebar for Ocean, Trainer, Catalogue: HD preview (own animation loop), vitals/meta, Watson-Crick double helix (chromatin), CTRNN brain graph, raw DNA copy (TCK-128).
- `views/CatalogueView.ts` — fullscreen catalogue, selection dialog with synapse restore options.
- `core/styled.ts` — small typed CSS-in-JS engine; `styles.css` global styles.
- `core/escape.ts` — `escapeHtml` for interpolated DB/user data (TCK-141).
- `components/Icons.ts` — SVG icons.

## Rendering

- `render/creatureRenderer.ts` — procedural Canvas 2D creatures (spine, organelles as smooth hillocks, sensory cones); no image assets; wiggle coupled to forward speed.
- `render/brainRenderer.ts` — CTRNN graph with hover tooltips for neurons and synapses.
- `core/camera.ts` — `InteractiveCamera`: drag pan, wheel zoom, click selection; no keyboard shortcuts.

## State

- `signals.ts`: `selectedId`, `selectedEnergy`, `selectedGenome`, `selectedPhenotype`, `selectedBrainActivations`, `speciesRosterSignal`, ...; UI reacts via signal `effect`s.
- `shared/types.ts` — handwritten types, not derived from `bindings/` (RV-A03).

## Security rule

- Untrusted strings (names, genomes, logs) go through `textContent` or `escapeHtml`, never raw `innerHTML` (RV-F02). CSP is still `null` in `tauri.conf.json` (TCK-161). `styled.ts applyChildren` treats strings starting with `<` as HTML (TCK-160).

## Open findings (pool)

| ID | Finding | Ticket |
| :--- | :--- | :--- |
| RV-F01 | handwritten types, ~41 `any` at the IPC boundary | TCK-157 |
| RV-F02 | CSP missing (escaping done in TCK-141) | TCK-161 |
| RV-F03 | legacy positional genome model in `signals.ts` (`getLocusDescription`, `computeActiveGeneSpans` with `ST`/`GO`); gene spans should come from the backend | TCK-160 |
| RV-F04 | `safeInvoke` swallows errors (`console.warn` → `null`), mixed with raw `invoke`; no UI feedback on failed save/delete | TCK-160 |
| RV-F05 | global listeners never removed, both modules handle all events; mode switch via fire-and-forget actions (race window) | TCK-160 |
| RV-F06 | catalogue preview loop duplicated in `ocean.ts` and `UnifiedDiagnosticsPanel.ts` | TCK-160 |
| RV-F07 | `concurrently`, `tsx` unused; Stryker without tests; `ActiveView` contains `"creator"` without a view (`zustand` removed in TCK-141) | TCK-161 |
| RV-F08 | hardcoded world size (5×), population display `/ 25`, fossil fallback `selectedAge = 2700` | TCK-160 |

## Key Takeaways

- Four views mounted once, router owns the backend mode, renderers are pure Canvas 2D.
- The frontend is in better shape than the Rust side; issues are contract drift and legacy logic.
- Per-view `mount()/unmount()` with listener lifecycle is the minimal target (TCK-160).

Sources: `.agents/skills/client-rendering/SKILL.md`, `docs/revisit/06-frontend.md`, TCK-115, TCK-128, TCK-141.
