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

> Pool ticket: requires a user decision before implementation. Details in `wiki/quality/review-findings.md` (RV-F03, RV-F04, RV-F05, RV-F06, RV-F08).

## Description
Legacy positional locus descriptions; swallowed IPC errors; side-effect modules with global listeners; duplicated preview modal; hard-coded world size; `styled.ts applyChildren` parses strings starting with `<` as HTML.

## Proposed Scope
Gene spans from backend, view mount/unmount lifecycle, shared preview component, safe child rendering.
