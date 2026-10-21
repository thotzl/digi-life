---
id: TCK-157
title: Typed IPC Contract and ts-rs Bindings in Frontend
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-A02, RV-A03, RV-F01, RV-A06
---

# TCK-157: Typed IPC Contract and ts-rs Bindings in Frontend

> Pool ticket: requires a user decision before implementation. Findings (RV-A02, RV-A03, RV-F01, RV-A06) below.

## Description
Actions/events are stringly-typed JSON; ts-rs bindings are generated but unused; mixed casing and timestamp units.

## Proposed Scope
`Action`/`SimEvent` enums exported via ts-rs; frontend imports `bindings/`.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-A02 · P1** — frontend → Rust: `invoke("handle_client_action", { action: JSON.stringify({type}) })`, double-serialized and unchecked; missing fields silently ignored (`as_u64()` → `None`). Rust → frontend: one event `simulation-state` with `json!` payloads, ~12 types, mixed casing (`load_learned_synapses`, `sandbox_id` vs `warpSpeed`, `runId`). Target: `#[derive(Deserialize, TS)] #[serde(tag = "type")] enum Action`, taken directly by the command; `#[derive(Serialize, TS)] enum SimEvent` on separate events (`ocean://telemetry`, `trainer://telemetry`, `db://changed`, `log`); both exported via ts-rs.
- **RV-A03 / RV-F01 · P1** — `bindings/` (17 files) generated but never imported; handwritten `frontend/shared/types.ts` (210 LOC) already deviates (`OrganelleLocus` `x?/y?` vs `SensoryPatch` `geneStartIndex/geneEndIndex`); ~41 `any`, mostly at the IPC boundary. Target: `types.ts` re-exports from `bindings/`, only client-side types stay local.
- **RV-A06 · P3** — `now_millis` is seconds at 9 sites (`as_secs`) and ms at 2; catalogue `created_at` is SQLite text. Align with RV-D04 (TCK-159).
