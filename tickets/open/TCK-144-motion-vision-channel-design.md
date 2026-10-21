---
id: TCK-144
title: Motion Vision Channel Design
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B06
---

# TCK-144: Motion Vision Channel Design

> Pool ticket: requires a user decision before implementation. Findings (RV-B06) below.
>
> 2026-10-21: candidate steering fix of the training track (TCK-172). A dedicated ΔI channel is a klinotaxis input: C. elegans steers with a single sensor via head swing and temporal comparison, so single organs could steer without a `HOX` pair. Decide after the TCK-172 steering probe.

## Description
Motion vision overwrites channel 5 of eyes/noses, which carries the main meat (IR) signal. Organ type is detected via emoji labels.

## Proposed Scope
Decide: dedicated 6th channel vs. separate neuron; organ type as enum.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B06 · P1 · static** — `src/shared/brain.rs`: for eyes/noses channel 5 (`base_idx + 4`) is overwritten every tick with the motion delta ΔI. Meat emits exactly on channel 5 (IR, weight 1.0), so meat reaches the brain only via channel 4 (weight 0.5): carnivore sensing is systematically weakened. The logic is coupled to emoji labels (`label.contains("👁️")`).
- Decision needed: motion vision as its own 6th channel or as its own neuron type. Organ type as an enum in the phenotype instead of label string matching (see TCK-155).
