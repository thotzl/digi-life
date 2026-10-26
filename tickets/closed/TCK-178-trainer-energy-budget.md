---
id: TCK-178
title: Energy Budget in the Trainer
status: closed
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

## Verification (P6, v1.12.20)
- Trainer drains `trainerMetabolicScale` (1.0) x the shared cost every tick (`test_trainer_drain_equals_the_ocean_energy_function`: bit-identical to the Ocean path over 20 ticks); one food-energy rule (`metabolism::food_energy`, `foodSporeEnergy`); a creature at 0 energy is starved for the rest of the trial (no brain, motion, eating).
- Report fields `mean_end_energy`, `starved_share`, `mean_organ_count`, `food_organ_share`.
- 5 seeds (pop 16, 30 gens, last 5 gens): small success 0.218 -> 0.223, starved 0, end energy 161.7 -> 139.0; large success 0.033 -> 0.050 (per seed 0.017-0.138), starved 0 -> 0.182, end energy 144.0 -> 71.3. No collapse, so `trainerMetabolicScale` stays 1.0; the P8 ramp (TCK-176) can lower it at the start of a curriculum.
- Throughput pop 300 standard 1064k -> 983k sandbox-ticks/s (-8 %, target >= 206k kept).
- Golden master `0xe0a2bfd1c7310306` -> `0x9d206dfd8124f808` (phenotype field `basal_metabolic_rate` removed); `MODEL_VERSION` 3 -> 4; CI reference regenerated. `cargo test` 113 passed (1 ignored), 0 warnings; `npm run build`, vitest green.
