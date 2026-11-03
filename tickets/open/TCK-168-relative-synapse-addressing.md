---
id: TCK-168
title: Relative Synapse Addressing in the Genome
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
epic_phase: Roadmap TCK-169 Phase 2 (feature epics, to be planned)
---

# TCK-168: Relative Synapse Addressing in the Genome

## Update (2026-11-03, after TCK-196)
- `dna.rs` was split (TCK-199): the SY addressing (role classes, index modulo the class size) is in `src/biology/brain/compile.rs` (step 5 of the compiler). A change of the addressing changes the golden master and needs a `MODEL_VERSION` bump (invariants 8, 9).

## Re-scope (2026-11-02, epic TCK-196)
Feature ticket after TCK-196. Description stale: `SY` now addresses role classes modulo class size (dna.rs:1789-1795), not absolute indices.


> Pool ticket from `ideas.md` (section II.3, epoch 17). Context: `wiki/research/porting-concepts.md`.

## Description
Framsticks-style relative addressing for `SY` genes so that functional circuits survive insertions/deletions. Today `SY` genes map `from`/`to` by modulo over absolute neuron indices, which shift when organs or neurons are added.

## Proposed Scope
- Depends on the brain evaluation order (TCK-153) and plasticity flags (TCK-152).
- Golden master changes intentionally; document the new fingerprint.

## Measurement Criterion
- Fraction of synapses preserved (same functional endpoints) across indel mutations before/after.
