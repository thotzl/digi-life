---
sources: [tickets/closed/TCK-122-multispectral-receptive-fields-and-haptics.md, tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-144-motion-vision-channel-design.md, tickets/closed/TCK-182-brain-topology-compaction.md, tickets/closed/TCK-184-genome-operators.md, tickets/closed/TCK-154-trainer-scenario-parameters.md]
---

# Sensing

Computed in `compute_sensory_inputs` (`src/shared/brain.rs`), one implementation for Ocean and Trainer (TCK-129). `compute_sensory_inputs_into` writes into a reused buffer (trainer sandboxes, probes); `compute_sensory_inputs` wraps it with a fresh vector (Ocean) (TCK-173). Angles and cone falloff use `src/shared/fmath.rs` (`libm`) for platform-identical results.

## Organelles

- Compiled from `EYE`/`NOS`/`TAC`/`LUM` genes, one organ (or a `HOX` pair) per gene ([[wiki/biology/genome-compiler]]). The promoter does not decide the type; affinity and expression style do.
- Each organ compiles `spectral_affinity` `[0, 1]` and `bandwidth` `[0.05, 0.90]` freely from its payload (TCK-122).
- **`OrganKind`** (enum in `SensoryPatch.kind`, decided once at compile time, TCK-144): `Eye` affinity ≥ 0.80, `Thermal` 0.65–0.80, `Nose` 0.25–0.65, below 0.25 by expression style E: `Bristle` E < 0.35, `LateralLine` 0.35 ≤ E < 0.70, `Proprioceptor` E ≥ 0.70. `Eye`, `Nose`, `Thermal` are distance senses. No emoji label matching remains in the model (RV-S10 organ part).
- `SensoryPatch.side`: `single`, or `left`/`right` of a `HOX` pair.
- Sensor range per organ `scale · 550`; measured over letter-soup genomes 256 / 575 / 926 px (p10/p50/p90), 193–990 px overall (TCK-172).
- Each organ provides **6 input slots** (`CHANNELS_PER_ORGANELLE`): 5 spectral receptive-field channels (1 UV/pressure 0.10, 2 blue/ester 0.30, 3 green/sugar 0.50, 4 red/amino acid 0.70, 5 IR/pheromone 0.90) plus slot 6 ΔI. Organ slots are clamped to `[0, 1]` except ΔI.

## Emission spectra

- Algae/plants: 1.0 on channel 2, 0.5 on channel 3. Meat/prey: 1.0 on channel 5, 0.5 on channel 4.
- Distance senses: tuning curve `1 − |f_c − affinity| / (1.8 · bandwidth + 0.12)` per channel; a channel registers food when sensitivity × emission > 0.05. Only these channels (plus ΔI) get thalamus nodes ([[wiki/biology/ctrnn-brain]], TCK-182).

## Tactile organs (affinity < 0.25)

- Channels: 1 mechanical hardness (walls 1.0, spores 0.3, reefs 1.0 on proximity — FIX-131), 2 fluid drag, 3 water temperature (center gradient), 4 spine strain, 5 impact pain.
- Subtypes (TCK-133): `Bristle` hardness + pain (channels 1, 5), `LateralLine` flow + pain (channels 2, 5; flow computed signed but clamped to `[0, 1]` at write), `Proprioceptor` temperature + spine flexion (channels 3, 4). Only these two channels per subtype are wired; tactile organs have no ΔI.

## Motion vision ΔI (TCK-144)

- Slot 6 of every distance sense carries the signed change of the organ's total spectral intensity since the previous tick: `ΔI = clamp(2.5 · (I_now − I_prev), −1, 1)`, computed in the brain step (the ΔI neuron's state holds `I_prev`).
- Meat keeps channel 5; before TCK-144 the unsigned delta overwrote it for eyes/noses (RV-B06 resolved). Thermal organs now also get ΔI.
- Purpose: klinotaxis — a single organ can compare intensity over time while the body swings, without a `HOX` pair.

## Systemic base inputs (TCK-133, TCK-149)

Six inputs after all organ slots (`SystemicInput`, `SYSTEMIC_BASE_INPUTS_COUNT`): `Clock` (CPG), `LinearSpeed` (`speed · 0.4`), `RotationalSpeed` (`|ω| · 0.8`), `Energy` (`energy / 100`), `Pain` (high speed or spin), `SearchArousal` (1.0 when the organ slots sum to < 0.05). The adrenaline input (always 0) was removed with `CreatureAgent.adrenaline` (TCK-149).

- `Energy` at spawn is `stomach_capacity · creatureStartStomachRatio / 100` in Ocean and trainer alike (TCK-185, [[wiki/biology/metabolism-and-reproduction]]).

## Angle space (TCK-133)

- 0° = straight ahead; left hemisphere `0..−180°`, right `0..+180°`; organs are placed in `[-170°, 170°]`. Shared by Rust, bindings and renderers.

## Measured equipment (TCK-172 equipment probe)

1,000 genomes per set; structured = `generate_structured_genome` (trainer generation 1 and immigrants since TCK-184), letter soup = the former random 256-base genomes.

| Property | Letter soup | Structured |
| :--- | :--- | :--- |
| No organ | 21 % | 0 % |
| ≥ 1 food-sensing organ (affinity ≥ 0.25) | 67 % | 99 % |
| `HOX` organ pair | 21 % | 31 % |
| Food in sensor range at spawn: small / large room | 57 % / 14 % | 86 % / 24 % |
| Non-zero food stimulus at spawn: small / large room | 31 % / 4.8 % | 46 % / 7.5 % |

- The large-room rows were measured at the pre-P7 spawn distance of 650 px. With 500 px since P7 (TCK-154 sweep, 2000 structured genomes) food is in sensor range at spawn for 35 % (13 % also in the view cone); small room 200 px: 89 % ([[wiki/simulation/trainer]]).
- In the large room most genomes still receive no food signal at the start of a trial.

## Key Takeaways

- 5 spectral channels plus a signed ΔI slot per organ; organ type is an enum; 6 systemic inputs.
- Only channels an organ can sense are wired.
- Structured generation-1 genomes all have an organ and 99 % a food sensor; the large room still gives few spawn-time food signals.

Sources: TCK-122, TCK-129, TCK-133, TCK-144, TCK-149, TCK-172, TCK-173, TCK-154, TCK-182, TCK-184, FIX-131, `docs/revisit/01-critical-bugs.md`.
