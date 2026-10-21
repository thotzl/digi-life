---
id: TCK-168
title: Relative Synapse Addressing in the Genome
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
---

# TCK-168: Relative Synapse Addressing in the Genome

> Pool ticket from `ideas.md` (section II.3, epoch 17). Context: `wiki/research/porting-concepts.md`.

## Description
Framsticks-style relative addressing for `SY` genes so that functional circuits survive insertions/deletions. Today `SY` genes map `from`/`to` by modulo over absolute neuron indices, which shift when organs or neurons are added.

## Proposed Scope
- Depends on the brain evaluation order (TCK-153) and plasticity flags (TCK-152).
- Golden master changes intentionally; document the new fingerprint.

## Measurement Criterion
- Fraction of synapses preserved (same functional endpoints) across indel mutations before/after.
