---
id: TCK-152
title: Hebbian Plasticity Normalisation and Fixed Reflexes
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S04, RV-S05
---

# TCK-152: Hebbian Plasticity Normalisation and Fixed Reflexes

> Pool ticket: requires a user decision before implementation. Details in `wiki/quality/review-findings.md` (RV-S04, RV-S05).

## Description
Unnormalised Hebbian rule drifts to the clamp; thalamus reflex synapses are plastic and can be pruned; Lamarckian chance is used as a scale factor.

## Proposed Scope
Plastic flag per synapse, normalised rule, consistent inheritance semantics.
