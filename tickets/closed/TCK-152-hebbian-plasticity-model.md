---
id: TCK-152
title: Hebbian Plasticity Normalisation and Fixed Reflexes
status: closed
priority: medium
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S04, RV-S05
epic_phase: Training Track
depends_on: TCK-179
---

# TCK-152: Hebbian Plasticity Normalisation and Fixed Reflexes

> Decision 2026-10-21 (training track, epic TCK-171; default from the session's recommendation, user may veto): Oja rule (normalised); `plastic: bool` per synapse with thalamus reflexes fixed; `lamarckian_assimilation_chance` as a real probability. Builds on the stable synapse identity of TCK-179 (the flag lives on the synapse). Check the TCK-172 Hebbian probe first. Findings (RV-S04, RV-S05) below.

## Description
Unnormalised Hebbian rule drifts to the clamp; thalamus reflex synapses are plastic and can be pruned; Lamarckian chance is used as a scale factor.

## Proposed Scope
Plastic flag per synapse, normalised rule, consistent inheritance semantics.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S04 · P2 · static** — `src/shared/brain.rs`: plain `Δw = η · pre · post` without Oja/BCM normalization → positive feedback; weights drift to the ±4 clamp over long lives (36k ticks). Hardwired thalamus reflex synapses (±1.0) are learned and can be pruned → sum/difference semantics decay. `_hebbian_decay` (stiffness decay) is passed and ignored. Target: synapse flag `plastic: bool`, reflexes fixed; normalized rule; test measuring the weight distribution after N ticks.
- **RV-S05 · P2** — depends on RV-B01. `lamarckian_assimilation_chance` is used as a scale factor on weight offsets (`engine.rs`), not as a probability; clone births (65 %) copy the agent incl. `neuron_states` and fully learned weights, mutants get 25 % of the offset. Inconsistent semantics.

## Verification (P4, v1.12.18)
- `CTRNNSynapse.plastic`: sensor -> thalamus reflexes (+-1) fixed (no learning, no pruning), all other synapses plastic (`test_reflexes_are_fixed_and_the_rest_plastic`).
- Oja rule `dw = eta * post * (pre - post * w)` (eta = `hebbianLearningRateBase`); +-4 clamp as safety bound; pruning (`|w| < 0.015`) for plastic synapses only. Removed keys: `hebbianLearningStiffnessDecay` (never read), `hebbianForgettingDecay` (redundant with the Oja decay term). RV-S04 resolved.
- Weight distribution, learning on (`probe hebbian`): 36k-tick life mean |w| 1.318 -> 0.342, at the clamp 10.3 % -> 0.22 %, reflex drift 0.097 -> 0; `test_oja_learning_stays_bounded_and_reflexes_stay_fixed` (24 genomes, 36,000 ticks: 0 % at |w| >= 3.9).
- Lamarck as a probability per synapse key (`inherit_learned_weights`, chance `lamarckianAssimilationChance` 0.25) in trainer (seeded per-slot stream) and Ocean (age floor kept); Ocean clone and mutant births go through the factory (fresh neuron states). RV-S05 resolved. Tests `test_lamarck_chance_is_a_probability_per_synapse`, `test_lamarck_draws_are_seeded`, trainer Lamarck tests.
- Hebbian probe: lifetime learning shows no measurable benefit at the current success level (standard paired median -1.4, exploration -8.4; means dominated by single genomes).
- Variants measured (time-boxed) and rejected: no pruning (5-seed success 0.166 small / 0.026 large), pruning only with active presynaptic input (no measurable change). Kept: Oja + pruning.
- 5-seed runs (pop 16, 30 gens, mean success of the last 5 gens): small 0.122 -> 0.207, large 0.033 -> 0.046.
- Golden master `0x65f701a6937c1e71` -> `0xe0a2bfd1c7310306` (only the `plastic` flag); `MODEL_VERSION` 2 -> 3; CI reference regenerated. `cargo test` 98 passed (1 ignored), 0 warnings; `npm run build`, vitest green.
