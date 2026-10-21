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
