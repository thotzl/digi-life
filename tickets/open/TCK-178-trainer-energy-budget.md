---
id: TCK-178
title: Energy Budget in the Trainer
status: open
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173, TCK-151
---

# TCK-178: Energy Budget in the Trainer

Part of epic TCK-171.

## Description
Trained creatures never learn to manage energy: in the trainer `energy` starts at 100 (`src/biology/trainer_engine.rs:101`) and is never reduced, while the Ocean drains basal metabolism every tick (`src/server/engine.rs:1217`, incl. a `* 0.45` compensation literal) plus thermal stress and bite damage. Champions selected without an energy budget can be fast, sensor-rich and still starve in the Ocean, which defeats the goal of the training track (creatures that survive in the Ocean later).

## Scope
- Apply the same energy model as the Ocean in the trainer step (shared function, no second formula): basal metabolism, thrust cost if the Ocean has one, refill on eating.
- A creature with empty energy stops (counts as failure for the rest of the trial).
- Energy-related metrics in the TCK-173 report: energy at trial end, share of starved creatures.
- Fitness (TCK-175) may use energy as a term; decide there.

## Dependency on TCK-151
The current BMR model multiplies by synapse and input count (RV-S03); one sense organ raises BMR ×3–×8. Enabling it in the trainer without TCK-151 would select against sense organs. Order: TCK-151 decision and implementation first, or this ticket with a temporary config scale and TCK-151 directly after.

## Acceptance
- Trainer and Ocean use the same energy function (test: identical drain for the same agent and tick).
- Before/after report: success rate and energy metrics; no collapse of sensor equipment (share of genomes with eyes/noses stays comparable).
