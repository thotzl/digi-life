---
id: TCK-122
title: Multispectral Receptive Fields & Haptics
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
completed_at: 2026-08-31
---

# TCK-122: Multispectral Receptive Fields & Haptics

## Description
Eradicate sensory input channel bottlenecks and sensory cross-wiring by introducing a fully uncoupled, multispectral 5-channel receptive field model for all eyes/noses, and a rich haptic & thermal feedback layout for tactiles. This allows complete separation of environmental signals (plants, prey, physical objects) and enables precise neural co-evolution and behavioral differentiation.

## Requirements & Scope

### 1. Decoupled Receptor Frequencies (dna.rs)
- Remove any rigid coupling of spectral boundaries from promoter types.
- Ensure Eyes, Noses, and Tactiles freely compile `spectral_affinity` in `[0.0 .. 1.0]` and `bandwidth` in `[0.05 .. 0.90]` from genome payloads.

### 2. 5-Channel Multispectral Inputs (dna.rs)
- For each physical organelle compiled:
  - Generate exactly 5 separate input receptive-field neurons (cones) mapped dynamically to:
    - **Channel 1 (UV/Druck 0.10)**
    - **Channel 2 (Blue/Ester 0.30)**
    - **Channel 3 (Green/Zucker 0.50)**
    - **Channel 4 (Red/Aminosäure 0.70)**
    - **Channel 5 (IR/Pheromone 0.90)**
  - Assign beautiful, self-describing HUD/Renderer labels (e.g. `"👁️ Green Vision ({}°)"` or `"👃 Protein Smell ({}°)"`).

### 3. Multidimensional Biophysical Physics Loop (trainer_engine.rs)
- Configure environmental objects to emit distinct spectra:
  - Algae/Pflanzen: emits 1.0 on Channel 2 (Blue) and 0.5 on Channel 3 (Green).
  - Meat/Prey: emits 1.0 on Channel 5 (IR) and 0.5 on Channel 4 (Red).
- For Eye/Nose organelles (`aff >= 0.25`), calculate Gaussian sensitivities on all 5 channels stochastically, allowing candidates to see/smell both resources with various degrees of focus and accuracy depending on their evolved genes.
- For Tactile organelles (`aff < 0.25`), map the 5 channels to physical contact and surroundings:
  - **Channel 1:** Mechanical hardness (Wands 1.0 vs Spores 0.3).
  - **Channel 2:** Fluid drag (movement speed).
  - **Channel 3:** Microclimate water temperature (spatial center gradient).
  - **Channel 4:** Proprioceptive spine mechanical strain and angular rotation.
  - **Channel 5:** Zerstörerischer physical impact pain and high-speed wall damage.

### 4. Verification & Testing
- Validate that all Rust tests compile and pass.
- Verify that both visual graphs and HUD hover labels align correctly.
