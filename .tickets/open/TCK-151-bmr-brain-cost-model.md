---
id: TCK-151
title: BMR Brain Cost Model
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S03
---

# TCK-151: BMR Brain Cost Model

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-S03).

## Description
BMR multiplies by synapse count and input count; sensory organs become metabolically prohibitive. Ocean compensates with a `* 0.45` literal.

## Proposed Scope
Design a linear cost per active synapse and per organ.
