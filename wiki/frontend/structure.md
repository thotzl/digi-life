---
sources: [.agents/skills/client-rendering/SKILL.md, README.md, docs/revisit/06-frontend.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-141-frontend-hygiene.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-187-gene-spans-from-backend.md]
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

- `views/UnifiedDiagnosticsPanel.ts` — shared inspector sidebar for Ocean, Trainer, Catalogue: HD preview (own animation loop), vitals/meta, Watson-Crick double helix, genome view, CTRNN brain graph, raw DNA copy (TCK-128). The catalogue preview loop also exists in `ocean.ts` (RV-F06). The adrenaline vital row was removed with the agent field (TCK-149).
- **Genome view** (TCK-187): every base is coloured by the backend gene span covering it (`phenotype.genes`, [[wiki/biology/genome-compiler]]): promoter letters, gene body, silent copies (later copies of first-only promoters, dimmed) or non-coding; the tooltip names the gene and its meaning (e.g. `EYE 1: Sensory organ (EYE) - Eye pair`).
- `views/CatalogueView.ts` — fullscreen catalogue, selection dialog with synapse restore options.
- `core/styled.ts` — small typed CSS-in-JS engine; `applyChildren` treats strings starting with `<` as HTML. `class`/`className` props are added to the generated class via `classList` (before the TCK-186 smoke check they were assigned as a no-op JS property, so e.g. the main-menu card hover styles and catalogue classes never applied). `styles.css` global styles.
- `core/escape.ts` — `escapeHtml` for interpolated DB/user data (TCK-141).
- `components/Icons.ts` — SVG icons.

## Rendering

- `render/creatureRenderer.ts` — procedural Canvas 2D creatures (spine, organelles as smooth receptor domes with `wAngular = 0.18 + bandwidth · 0.22` since FIX-131, sensory cones); no image assets; wiggle coupled to forward speed.
- `render/brainRenderer.ts` — neural inspector (TCK-187): columns by backend role (inputs → thalamus → hidden by depth → motors), sensors grouped per organ incl. the ΔI channel, thalamus Σ sum / Δ L−R diff nodes, thrust/bend motors; neuron tooltips from `describeRole`, synapse tooltips with source, target, weight and owner gene (`describeSynapse`). No emoji label parsing; `compile(brain, {genes, organelles})`.
- `core/camera.ts` — `InteractiveCamera`: drag pan, wheel zoom, click selection; no keyboard shortcuts.

## State

- `signals.ts`: `selectedId`, `selectedEnergy`, `selectedGenome`, `selectedPhenotype`, `selectedBrainActivations`, `speciesRosterSignal`, ...; UI reacts via signal `effect`s.
- The legacy positional-genome helpers (`getLocusDescription`, `computeActiveGeneSpans` with `ST`/`GO`) were removed; gene meanings come from backend spans (RV-F03 resolved, TCK-187).
- `shared/genome.ts` — display tables only: promoter → meaning (incl. which promoters are read first-only), systemic input, organ kind, channel and motor names; `describeGene`, `describeRole`, `describeSynapse`, `geneIndexByPosition`. The backend decides which genes and neurons exist. Tests `shared/genome.test.ts` (Vitest).
- `shared/types.ts` — re-exports the brain and gene types from `bindings/`; phenotype, agent and telemetry types are still handwritten (RV-A03 partly, RV-F01).

## Security

- Untrusted strings (names, genomes, logs) go through `textContent` or `escapeHtml`, never raw `innerHTML` (RV-F02, TCK-141).
- CSP is `null` in `tauri.conf.json`.

## Hardcoded values

- World size 19200/10800 (5×), population display `/ 25`, fossil fallback `selectedAge = 2700` (RV-F08).
- `ActiveView` contains `"creator"` without a view; `concurrently`, `tsx` and Stryker are configured but unused (RV-F07).

## Key Takeaways

- Four views mounted once, router owns the backend mode, renderers are pure Canvas 2D.
- Genome view and neural inspector render backend gene spans and neuron roles; the frontend only maps them to text (TCK-187).
- Output escaping is in place; there is no CSP and no per-view listener lifecycle.

Sources: `.agents/skills/client-rendering/SKILL.md`, `docs/revisit/06-frontend.md`, TCK-115, TCK-128, TCK-141, FIX-131, TCK-186, TCK-187.
