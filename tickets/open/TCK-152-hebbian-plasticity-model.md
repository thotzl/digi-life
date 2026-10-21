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

> Pool ticket: requires a user decision before implementation. Findings (RV-S04, RV-S05) below.

## Description
Unnormalised Hebbian rule drifts to the clamp; thalamus reflex synapses are plastic and can be pruned; Lamarckian chance is used as a scale factor.

## Proposed Scope
Plastic flag per synapse, normalised rule, consistent inheritance semantics.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S04 · P2 · static** — `src/shared/brain.rs`: plain `Δw = η · pre · post` without Oja/BCM normalization → positive feedback; weights drift to the ±4 clamp over long lives (36k ticks). Hardwired thalamus reflex synapses (±1.0) are learned and can be pruned → sum/difference semantics decay. `_hebbian_decay` (stiffness decay) is passed and ignored. Target: synapse flag `plastic: bool`, reflexes fixed; normalized rule; test measuring the weight distribution after N ticks.
- **RV-S05 · P2** — depends on RV-B01. `lamarckian_assimilation_chance` is used as a scale factor on weight offsets (`engine.rs`), not as a probability; clone births (65 %) copy the agent incl. `neuron_states` and fully learned weights, mutants get 25 % of the offset. Inconsistent semantics.
