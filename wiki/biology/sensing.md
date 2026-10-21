---
sources: [tickets/closed/TCK-122-multispectral-receptive-fields-and-haptics.md, tickets/closed/TCK-133-biophysical-sensing-upgrades-and-sensor-fusion.md, tickets/closed/FIX-131-hox-symmetry-visuals-reef-collision.md, tickets/closed/TCK-129-architecture-refactoring.md, docs/revisit/01-critical-bugs.md, docs/revisit/04-simulation-model.md]
---

# Sensing

Computed in `compute_sensory_inputs` (`src/shared/brain.rs`), one implementation for Ocean and Trainer (TCK-129).

## Organelles

- Eyes, noses, tactile organs, photophores — compiled from `EYE`/`NOS`/`TAC`/`LUM` genes ([[wiki/biology/genome-compiler]]).
- Each organ compiles `spectral_affinity` `[0, 1]` and `bandwidth` `[0.05, 0.90]` freely from its payload (TCK-122).
- Each organ provides **5 input channels** (receptive-field neurons): 1 UV/pressure 0.10, 2 blue/ester 0.30, 3 green/sugar 0.50, 4 red/amino acid 0.70, 5 IR/pheromone 0.90. HUD labels like `👁️ Green Vision (θ°)`.

## Emission spectra

- Algae/plants: 1.0 on channel 2, 0.5 on channel 3.
- Meat/prey: 1.0 on channel 5, 0.5 on channel 4.
- Eye/nose (`affinity ≥ 0.25`): Gaussian sensitivity over all 5 channels.

## Tactile organs (`affinity < 0.25`)

- Channels: 1 mechanical hardness (walls 1.0, spores 0.3, reefs 1.0 on proximity — FIX-131), 2 fluid drag, 3 water temperature (center gradient), 4 spine strain/rotation, 5 impact pain.
- Subtype by `expression_style` E (TCK-133): bristles `E < 0.35` (pressure + pain), lateral-line pores `0.35 ≤ E < 0.70` (vectorial flow, signed `[-1, 1]` = rheotaxis, + pain), thermo-proprioceptors `E ≥ 0.70` (spine flexion + temperature).

## Systemic base inputs (TCK-133)

Always present, appended after organ channels: `clock_val` (CPG), `linear_speed`, `rotational_speed`, `internal_energy`, `adrenaline_level` (constant 0, RV-S01), `pain_signal`. Organ channels are exteroception only.

## Angle space (TCK-133)

- 0° = straight ahead; left hemisphere `0..−180°`, right `0..+180°`; no `−90°` offsets. Shared by Rust, bindings and renderers.

## Motion vision (TCK-133)

- Temporal delta `ΔI = I_now − I_prev` for eyes.
- For eyes/noses the delta overwrites channel 5 (`base_idx + 4`) every tick — the meat channel (IR, weight 1.0). Meat reaches the brain only via channel 4 (0.5), so carnivore sensing is weakened. The logic keys on emoji labels (`label.contains("👁️")`) (RV-B06).

## Key Takeaways

- 5 spectral channels per organ, tactile subtypes by expression style, 6 fixed body-state inputs.
- 0° = ahead, signed hemispheres.
- Motion vision shares channel 5 with meat (RV-B06).

Sources: TCK-122, TCK-129, TCK-133, FIX-131, `docs/revisit/01-critical-bugs.md`.
