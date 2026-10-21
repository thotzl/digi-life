---
id: TCK-151
title: BMR Brain Cost Model
status: open
priority: high
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S03
epic_phase: Training Track
---

# TCK-151: BMR Brain Cost Model

> Pool ticket: requires a user decision before implementation. Findings (RV-S03) below.
>
> 2026-10-21: needed by the training track — TCK-178 (energy budget in the trainer) depends on this model; without it an energy budget selects against sense organs. Decision 2026-10-21 (user): direction accepted — linear cost per active (non-pruned) synapse plus a separate cost term per organ; the `* 0.45` Ocean literal goes into config or is removed; constants after measurement (TCK-172/173 reports).

## Description
BMR multiplies by synapse count and input count; sensory organs become metabolically prohibitive. Ocean compensates with a `* 0.45` literal.

## Proposed Scope
Design a linear cost per active synapse and per organ.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S03 · P1 · static** — `src/biology/dna.rs`: `brain_complexity_multiplier = 1 + synapses · 0.06` times `(1 + k_count · 0.08)`. Synapse count grows with `(7 + 10·organs + H) × (4 + H)` (exuberant fully connected). One organ adds ~10 thalamus sources → ~40–120 synapses → BMR ×3–×8. The Ocean compensates with a `* 0.45` hack (`engine.rs`). Strong selection pressure against sense organs, contrary to the design goal.
- Target: cost per active (non-pruned) synapse, linear; organs with their own cost term; move the 0.45 hack into config or remove it.
