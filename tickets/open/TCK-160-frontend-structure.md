---
id: TCK-160
title: Frontend Structure and SSOT Gene Spans
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-F03, RV-F04, RV-F05, RV-F06, RV-F08
---

# TCK-160: Frontend Structure and SSOT Gene Spans

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R4 (after TCK-157; parallel with TCK-199). RV-F03 resolved (TCK-187). Scope from the 2026-11-02 review:
- `core/view.ts` `ViewModule { id, title, backendMode, create, mount(ctx), unmount, onEvent }` + `views/index.ts` registry; `ViewContext { ipc, selection, scope }` where `scope` disposes listeners, effects and rAF on unmount (today no teardown: modal `<style>` leaks with conflicting `.modal-card`, global keydown `r` in `ocean.ts:765`, document click `trainer.ts:606`, resize `ocean.ts:459`, three hidden panels running preview rAF).
- `ocean.ts` / `trainer.ts` -> `views/ocean/`, `views/trainer/` controllers holding element references from `create()` (~50 shared DOM ids, duplicate `#inspect-panel` in OceanView.ts:412 / TrainerView.ts:783).
- `state/selection.ts`: one selection signal with its source view; fixes the cross-view id collision (sandbox id read as Ocean creature id, `ocean.ts:569-576`; assign may get an Ocean/catalogue id) and the string id in `CatalogueView.ts:466`; replaces six copies of the selection fill.
- Components `Modal`, `CreaturePreview`, `CataloguePicker` (three modal copies: `ocean.ts:171-389`, `UnifiedDiagnosticsPanel.ts:667-875`, `CatalogueView.ts:495-666`); `shared/creatureLabels.ts` (diet from `dietClass`, colour from `primaryColor` instead of `genome.charCodeAt(17)`, warp mapping 2x); `render/worldLayer.ts` (spores 2x).
- `render/organs.ts`: painter per `OrganKind` (`Record<OrganKind, ...>`) instead of threshold re-derivation at three sites in `creatureRenderer.ts`; organ names/channels from the backend registry (TCK-199).
- Types only from `bindings/` (drop hand-written phenotype/agent/world types in `shared/types.ts`); world size from INIT_STATE (RV-F08).
- Bugs: bite flash drawn at wrong coordinates (`ocean.ts:594-596` + 695-707); boot pushes client defaults before TRAINER_STATE (`trainer.ts:863-866`); entering the Ocean always pauses it (`router.ts:152`); catalogue save uses nonexistent `p.fitness` and compiled weights (`UnifiedDiagnosticsPanel.ts:640-642`); hover radius from an Ocean-empty cache (`ocean.ts:733`); failed `getPhenotype` re-requested every frame (`trainer.ts:316`); `"creator"` in `ActiveView` without a view.
- Contract `wiki/contracts/frontend-view.md`, guide `add-view.md`.


> Pool ticket: requires a user decision before implementation. Findings (RV-F03, RV-F04, RV-F05, RV-F06, RV-F08) below.

## Description
Legacy positional locus descriptions; swallowed IPC errors; side-effect modules with global listeners; duplicated preview modal; hard-coded world size; `styled.ts applyChildren` parses strings starting with `<` as HTML.

## Proposed Scope
Gene spans from backend, view mount/unmount lifecycle, shared preview component, safe child rendering.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-F03 · P1** — `frontend/signals.ts`: `getLocusDescription(i)` describes a positional genome (locus 0 = symmetry, 7 = radius, ...), `computeActiveGeneSpans` searches `ST`/`GO`. The compiler is promoter-based since TCK-116 → tooltips and genome visualization show wrong meanings. Target: gene spans incl. promoter name from the backend (`parse_genome` knows them), client only renders.
- **RV-F04 · P2** — `safeInvoke` catches every error, `console.warn`, returns `null`; mixed with raw `invoke` in `ocean.ts`; no UI feedback on failed save/delete.
- **RV-F05 · P2** — `ocean.ts`/`trainer.ts` register global `listen("simulation-state")` handlers on import, never removed, each processing all event types; navigation switches the backend mode via 3–4 fire-and-forget actions (`router.ts`) → race window. Target: per view `mount()/unmount()` with listener lifecycle; mode switch as one command with a return value.
- **RV-F06 · P2** — catalogue preview with its own `requestAnimationFrame` loop exists in `ocean.ts` and `UnifiedDiagnosticsPanel.ts`. One component.
- **RV-F08 · P3** — world size 19200/10800 (5×), population display `/ 25`, fossil fallback `selectedAge = 2700` → from `INIT_STATE`/config.
- From TCK-141: `styled.ts applyChildren` treats strings starting with `<` as HTML.
