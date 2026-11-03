---
sources: [.agents/skills/client-rendering/SKILL.md, README.md, tickets/closed/TCK-115-master-architectural-refactoring.md, tickets/closed/TCK-128-main-menu-navigation-and-unified-diagnostics-sidebars.md, tickets/closed/TCK-141-frontend-hygiene.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-186-regression-safety-ci-runner-and-smoke-checks.md, tickets/closed/TCK-187-gene-spans-from-backend.md, tickets/closed/TCK-177-trainer-ui-progress-metrics.md, tickets/closed/TCK-160-frontend-structure.md, frontend/core/router.ts, frontend/core/view.ts, frontend/state/selection.ts, frontend/views/index.ts]
---

# Frontend Structure

`frontend/` — Vite + TypeScript (strict) + `@preact/signals-core`; plain DOM, no UI framework. `index.html` has one
mount point `<div id="app">`. Since TCK-160 every screen is a `ViewModule` with a teardown scope
([[wiki/contracts/frontend-view]], guide [[wiki/guides/add-view]]). Fixed behaviour before TCK-160:
[[wiki/history/replaced-models]].

## Module map

| Path | Content |
| :--- | :--- |
| `main.ts` | boot: `startRouter(VIEWS, …)`, `onUiEvent(dispatchUiEvent)`, `CLIENT_READY` |
| `core/view.ts` | `ViewModule`, `ViewContext { ipc, selection, scope, navigate }`, `ViewIpc` |
| `core/scope.ts` | `Scope`: `listen`, `effect`, `frame` (rAF loop), `timeout`, `child`, `dispose` |
| `core/router.ts` | `startRouter`, `navigateTo`, `dispatchUiEvent`, `currentView`; owns the backend mode |
| `core/ipc.ts` | typed IPC client (TCK-157, [[wiki/contracts/ipc-message]]) |
| `core/phenotypes.ts` | phenotype cache over `get_fossil_phenotype`: one request per genome, failures retried after 10 s, `phenotypeVersion` signal, `peekPhenotype` |
| `core/camera.ts` | `InteractiveCamera` (pan, zoom, click); world size via `setWorldSize`, `FALLBACK_WORLD_SIZE` until `INIT_STATE` |
| `core/styled.ts`, `core/escape.ts`, `core/toast.ts` | CSS-in-JS (one class per styled component), `escapeHtml`, toasts |
| `state/selection.ts` | app selection with source tag; `selection` / `vitals` / `activations` signals; builders per source |
| `components/` | `Modal`, `CreaturePreview`, `CataloguePicker`, `DiagnosticsPanel` (inspector), `Icons` |
| `render/` | `creatureRenderer.ts`, `organs.ts` (painter per `OrganKind`), `worldLayer.ts` (biomes, vents, rocks, spores, trainer chamber), `brainRenderer.ts`, `progressChart.ts` |
| `shared/` | `genome.ts` (gene/role display tables), `creatureLabels.ts` (diet class, badge colour), `warp.ts` (speed slider mapping) |
| `views/index.ts` | registry `VIEWS = [main-menu, ocean, trainer, catalogue]`, `ViewId` |
| `views/main-menu/` | launchers, app version (`#app-version`) |
| `views/ocean/` | `layout.ts` (DOM + element references), `model.ts` (creatures, spores, hit tests, bite flashes), `index.ts` (controller) |
| `views/trainer/` | `layout.ts`, `settings.ts` (hyperparameters, synced flag), `metrics.ts` (TCK-177 formatting, progress history), `index.ts` (controller, tiles) |
| `views/catalogue/` | catalogue list, release / seed / rename / wipe dialogs |

## Views and lifecycle

- The router creates all four views once (`create` builds the DOM and keeps element references; no
  `getElementById` lookups) and shows one. Leaving a view disposes its mount scope: window/document listeners (Ocean
  resize and `r` camera reset, trainer dropdown close), signal effects (roster, inspector), animation loops (Ocean
  world, trainer tiles, inspector and picker previews) and open dialogs stop.
- Navigation order: pause the other views' simulations (`suspend`: Ocean `TOGGLE_SIMULATION false` if running,
  trainer `PAUSE_TRAINING`), `setMode(target.backendMode)`, then `resume` the target. The Ocean resumes on entry if
  the user had it running (initial backend state or the Pause/Resume button) ([[wiki/architecture/invariants]] 1).
- Events: every view's `onEvent` gets every `UiEvent`, shown or not (the Ocean receives `INIT_STATE` while the menu
  is shown); drawing happens only in mount-scoped loops.
- Trainer: the tile render loop (TCK-177) draws dirty tiles once per frame while the view is mounted; telemetry only
  stores snapshots. Tile canvases have a 360 × 360 backing store ([[wiki/architecture/performance]]). The trainer
  sends no hyperparameters until the backend state (`TRAINER_STATE_CHANGED`, emitted when the trainer mode is
  entered) has arrived (`TrainerSettings.synced`).
- Trainer stats row, progress card, mode select, pending-switch note and level slider: TCK-176/TCK-177
  ([[wiki/simulation/trainer]]).

## Selection

- One selection for the app, tagged by source (`ocean` creature id, `fossil` species id, `trainer` sandbox id,
  `catalogue` string id). Each inspector shows only its sources (Ocean: ocean + fossil, trainer: trainer, catalogue:
  catalogue); ids are read through `oceanCreatureId` / `trainerSandboxId` / `catalogueId`.
- Static data (`selection`) and live data (`vitals`, `activations`) are separate signals: the genome grid and brain
  layout rebuild only when the creature changes. Fossil and catalogue selections have no age (`null`, shown "–").
- "Save to Catalogue" stores the selection's fitness (trainer: the sandbox's current fitness; Ocean: 0) and the
  creature's own synapse weights (Ocean: the agent's weights from its event; trainer: none, the telemetry does not
  carry them, so the backend compiles them on load).

## Rendering

- `render/creatureRenderer.ts` — procedural Canvas 2D creatures; organ shape (limb / head bud / hillock) from
  `organShape` and sway from `organAngle` (`render/organs.ts`), surface marks from `ORGAN_PAINTERS:
  Record<OrganKind, OrganPainter>` (eye lens, thermal pit, bead colour per kind, limb tip). The kind comes from the
  backend organ registry ([[wiki/contracts/organ-kind]]); the renderer no longer re-derives it from affinity
  thresholds. A fleshy limb is drawn exactly when the backend counts the organ as a limb (`isLimb`: limb-capable kind
  from `ORGAN_KINDS[kind].limb` and `expressionStyle ≥ LIMB_STYLE`); eyes, noses and thermal organs are never limbs.
- `render/worldLayer.ts` — `WorldLayer` draws the biome image (built from the biome cells' `x/y/width/height`),
  vents and obstacles of `INIT_STATE.world`; `drawSpores` (glow for the Ocean, halo for trainer tiles) and
  `drawChamber` are shared.
- Ocean bite flashes are drawn at the bite's world position with a screen-size ring.
- Ocean hover radius comes from the creature's own phenotype.
- `render/brainRenderer.ts` (TCK-187) and `render/progressChart.ts` (TCK-177) unchanged.

## Labels

- Diet class from `shared/creatureLabels.ts` with the backend thresholds (`CARNIVORE_MIN` 0.6, `HERBIVORE_MAX` 0.4 of
  `biology::diet::DietClass`, imported from the generated `bindings/diet.ts`).
- Badge colour from the phenotype's `primaryColor` hue (grey until the phenotype is known).
- Genome view colours per base and the gene tooltips: TCK-187 ([[wiki/biology/genome-compiler]]).

## Types

All model, event, action and command types come from `bindings/` (ts-rs, [[wiki/architecture/invariants]] 14).

## Security

- Untrusted strings are text ([[wiki/architecture/invariants]] 15; TCK-141,
  TCK-197). `styled` compiles each template once per component.
- CSP in `tauri.conf.json` (TCK-197): `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src
  'self' data:; connect-src ipc: http://ipc.localhost`.

## Hardcoded values

- `FALLBACK_WORLD_SIZE` (19200 × 10800) in `core/camera.ts`, used only until `INIT_STATE.world` arrives.
- The Ocean shows the creature count without a cap: the cap (`ocean.maxCreatures`) is not in `INIT_STATE` yet.
- `AGE_BAR_MAX` 2700 s in `components/DiagnosticsPanel.ts` (no lifespan in the model, TCK-146).

## Key Takeaways

- Views are registered modules with create/mount/unmount and a scope that stops everything on leave; the router alone
  switches the backend mode and does not pause the Ocean on entry.
- One tagged selection for the app; shared components for dialogs and previews.
- Organ marks are painted per backend `OrganKind`; world geometry and size come from `INIT_STATE`.

Sources: `.agents/skills/client-rendering/SKILL.md`, TCK-115, TCK-128, TCK-141, FIX-131, TCK-177, TCK-186, TCK-187, TCK-160.
