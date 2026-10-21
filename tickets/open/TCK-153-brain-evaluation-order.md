---
id: TCK-153
title: Brain Evaluation Order
status: open
priority: low
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S06
epic_phase: Training Track
depends_on: TCK-172
---

# TCK-153: Brain Evaluation Order

> Decision 2026-10-21 (training track, epic TCK-171; default from the session's recommendation, user may veto): keep the order and document the 2-tick sensor→motor latency in `wiki/biology/ctrnn-brain.md`, unless the TCK-172 steering probe shows the latency hurts; then reorder to Inputs → Thalamus → Hidden → Outputs together with the golden-master change of TCK-179. Findings (RV-S06) below.

## Description
Outputs are integrated before thalamus/hidden nodes (id order), giving a 2-tick sensor-to-motor latency.

## Proposed Scope
Decide whether to keep (document) or reorder evaluation.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S06 · P2 · static** — neuron IDs are `[Inputs][Outputs][Thalamus][Hidden]`; the Euler pass runs ascending, so outputs are computed before thalamus and hidden and read their previous-tick activation: sensor → motor latency 2 ticks. Allowed for a CTRNN but should be a deliberate decision: document it or reorder to `Inputs → Thalamus → Hidden → Outputs` (changes the golden master).
