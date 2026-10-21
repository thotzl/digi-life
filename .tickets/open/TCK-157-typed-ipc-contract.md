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

> Pool ticket: requires a user decision before implementation. Details in `wiki/quality/review-findings.md` (RV-A02, RV-A03, RV-F01, RV-A06).

## Description
Actions/events are stringly-typed JSON; ts-rs bindings are generated but unused; mixed casing and timestamp units.

## Proposed Scope
`Action`/`SimEvent` enums exported via ts-rs; frontend imports `bindings/`.
