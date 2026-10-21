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
> 2026-10-21: needed by the training track — TCK-178 (energy budget in the trainer) depends on this model; without it an energy budget selects against sense organs. Decision 2026-10-21 (user, after research): additive, activity-based model instead of the multiplicative formula (`src/biology/dna.rs:1434-1447`):
>
> `cost per tick = body + Σ organ(size) + a · neurons + b · Σ |weight · source activation|`
>
> - Per-neuron base cost: brain energy scales linearly with neuron number, roughly constant per neuron (Herculano-Houzel 2011).
> - Activity cost instead of a per-synapse price with a pruning threshold: most signalling energy goes into action potentials (47 %) and synaptic transmission (34 %) and scales with activity (Attwell & Laughlin 2001). Near-zero weights cost almost nothing without a threshold edge; the exuberant start brain is not punished; sparse brains are rewarded. Computed in the existing synapse pass of the brain step.
> - Organ cost by size (existing `vestigial_organ_tax` shape `scale² · (1.1 − bandwidth)` as a start).
> - Calibrate by shares, not free constants: brain + senses ≈ 10–20 % of resting cost for a typical creature (vertebrate brain 2–8 %, human 20 %; eyes + optic tectum of young Astyanax 15 %, adult 5 %, Moran et al. 2015). Check with the TCK-172/173 reports.
> - Risk: activity cost rewards silent brains; eating must clearly outweigh an active brain (part of calibration; thrust costs separately).
> - The `* 0.45` Ocean literal is removed (or moved to config during transition).
> - Metabolic pressure is phased in during training (TCK-176); connection cost as a separate selection objective (Clune et al. 2013) is an escalation option in TCK-175.
>
> Sources: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3046985/ ; https://discovery-pp.ucl.ac.uk/1340392/1/356.full.pdf ; https://pmc.ncbi.nlm.nih.gov/articles/PMC4643782 ; https://arxiv.org/pdf/1207.2743.pdf

## Description
BMR multiplies by synapse count and input count; sensory organs become metabolically prohibitive. Ocean compensates with a `* 0.45` literal.

## Proposed Scope
Design a linear cost per active synapse and per organ.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S03 · P1 · static** — `src/biology/dna.rs`: `brain_complexity_multiplier = 1 + synapses · 0.06` times `(1 + k_count · 0.08)`. Synapse count grows with `(7 + 10·organs + H) × (4 + H)` (exuberant fully connected). One organ adds ~10 thalamus sources → ~40–120 synapses → BMR ×3–×8. The Ocean compensates with a `* 0.45` hack (`engine.rs`). Strong selection pressure against sense organs, contrary to the design goal.
- Target: cost per active (non-pruned) synapse, linear; organs with their own cost term; move the 0.45 hack into config or remove it.
