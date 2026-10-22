---
sources: [tickets/closed/TCK-122-multispectral-receptive-fields-and-haptics.md, tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md, tickets/ongoing/TCK-149-dead-config-keys.md, tickets/closed/TCK-172-mechanics-diagnosis-for-training.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md]
---

# Sensing

Computed in `compute_sensory_inputs` (`src/shared/brain.rs`), one implementation for Ocean and Trainer (TCK-129). `compute_sensory_inputs_into` writes into a reused buffer (trainer sandboxes, probes); `compute_sensory_inputs` wraps it with a fresh vector (Ocean) (TCK-173). Angles and cone falloff use `src/shared/fmath.rs` (`libm`) for platform-identical results.

## Organelles

- Eyes, noses, tactile organs, photophores — compiled from `EYE`/`NOS`/`TAC`/`LUM` genes ([[wiki/biology/genome-compiler]]).
- Each organ compiles `spectral_affinity` `[0, 1]` and `bandwidth` `[0.05, 0.90]` freely from its payload (TCK-122).
- Sensor range per organ `scale · 550`; measured over random genomes 256 / 575 / 926 px (p10/p50/p90), 193–990 px overall (TCK-172).
- Each organ provides **5 input channels** (receptive-field neurons): 1 UV/pressure 0.10, 2 blue/ester 0.30, 3 green/sugar 0.50, 4 red/amino acid 0.70, 5 IR/pheromone 0.90. HUD labels like `👁️ Green Vision (θ°)`.

## Emission spectra

- Algae/plants: 1.0 on channel 2, 0.5 on channel 3.
- Meat/prey: 1.0 on channel 5, 0.5 on channel 4.
- Eye/nose (`affinity ≥ 0.25`): Gaussian sensitivity over all 5 channels.

## Tactile organs (`affinity < 0.25`)

- Channels: 1 mechanical hardness (walls 1.0, spores 0.3, reefs 1.0 on proximity — FIX-131), 2 fluid drag, 3 water temperature (center gradient), 4 spine strain/rotation, 5 impact pain.
- Subtype by `expression_style` E (TCK-133): bristles `E < 0.35` (pressure + pain), lateral-line pores `0.35 ≤ E < 0.70` (vectorial flow, signed `[-1, 1]` = rheotaxis, + pain), thermo-proprioceptors `E ≥ 0.70` (spine flexion + temperature).

## Systemic base inputs (TCK-133)

Always present, appended after organ channels (7, `SYSTEMIC_BASE_INPUTS_COUNT`): `clock_val` (CPG), `linear_speed`, `rotational_speed`, `internal_energy` (`energy / 100`), `adrenaline_level` (constant 0, agent `adrenaline` fixed at 1.0; TCK-149), `pain_signal`, `search_arousal` (1.0 when the organ channels sum to < 0.05). Organ channels are exteroception only.

- `internal_energy` at spawn is `stomach_capacity · creatureStartStomachRatio / 100` in Ocean and trainer alike (TCK-185, [[wiki/biology/metabolism-and-reproduction]]).

## Angle space (TCK-133)

- 0° = straight ahead; left hemisphere `0..−180°`, right `0..+180°`; no `−90°` offsets. Shared by Rust, bindings and renderers.

## Motion vision (TCK-133)

- Temporal delta `ΔI = I_now − I_prev` for eyes.
- For eyes/noses the delta overwrites channel 5 (`base_idx + 4`) every tick — the meat channel (IR, weight 1.0). Meat reaches the brain only via channel 4 (0.5), so carnivore sensing is weakened (RV-B06). Eye/nose organs are recognised by their emoji labels once per agent when the `BrainRuntime` is built, not per tick (TCK-173, [[wiki/biology/ctrnn-brain]]).

## Measured equipment (TCK-172)

Probe `trainer_runner probe equipment`, 1,000 random 256-base genomes (trainer generation 1 and immigrants):

| Property | Share |
| :--- | :--- |
| No organ | 21 % |
| ≥ 1 eye / ≥ 1 nose | 29 % / 43 % |
| ≥ 1 food-sensing organ (affinity ≥ 0.25) | 67 % |
| `HOX` organ pair (left/right difference nodes) | 21 % |
| Food in sensor range at spawn: small / large room | 57 % / 14 % |
| Non-zero food stimulus at spawn: small / large room | 31 % / 4.8 % |

- Left/right comparison exists only with a `HOX` pair; single organs deliver sums only ([[wiki/biology/ctrnn-brain]]).
- In the large room almost no genome receives a food signal at the start of a trial.

## Key Takeaways

- 5 spectral channels per organ, tactile subtypes by expression style, 7 fixed body-state inputs.
- Only 4.8 % of random genomes sense food at spawn in the large room (TCK-172).
- 0° = ahead, signed hemispheres.
- Motion vision shares channel 5 with meat (RV-B06).

Sources: TCK-122, TCK-129, TCK-133, TCK-172, TCK-173, FIX-131, `docs/revisit/01-critical-bugs.md`.
