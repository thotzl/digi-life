---
id: TCK-133
title: Biophysical Sensing Upgrades, Symmetrical Sensor Fusion, and Standardized Angle Space
status: open
assigned: Gemini-CLI
created_at: 2026-09-04
---

# TCK-133: Biophysical Sensing Upgrades, Symmetrical Sensor Fusion, and Standardized Angle Space (Epic)

## Description
Overhaul the sensory processing and neural wiring pipelines. This Epic establishes biophysically-accurate coordinate spaces, decouples self-state proprioception from peripheral environmental organs, and introduces advanced sensory features (flow rheotaxis, gyroscope stabilization, and motion vision). 

Eliminate the fragile and drift-prone birth-time synapse mirroring by introducing a central nervous system equivalent: **Symmetrical Sensor Fusion via HOX-gated Sum & Difference Interneurons**.

---

## Requirements & Scope

### Phase 1: Standardized Symmetrischer Winkelraum
Refactor the coordinate space for all orientation calculations to use a standard, biophysically-intuitive layout:
- **Straight Ahead (Direction of travel):** **$0^{\circ}$** (0.0 Radians).
- **Left Hemisphere:** **$0^{\circ}$ to $-180^{\circ}$** (negative range: $0$ to $-\pi$).
- **Right Hemisphere:** **$0^{\circ}$ to $+180^{\circ}$** (positive range: $0$ to $+\pi$).
- **Implementation:**
  - Remove all legacy `- 90.0` subtraction offsets (e.g., in `compute_sensory_inputs`).
  - Adapt the angular falloff formulas (`cos(delta_beta)`) to directly ingest relative angles.
  - Update TypeScript bindings (`bindings/`) and frontend renderers (`creatureRenderer.ts`, `brainRenderer.ts`) to match the new coordinate space.

### Phase 2: Decoupled Proprioception & State Inputs (CNS Core)
Ensure creatures always perceive their own body state, regardless of mutating peripheral organs:
- **Systemic Base Inputs (Feste Basis-Inputs):** Append a permanent block of state inputs to the end of the CTRNN input vector (adjacent to the clock/CPG):
  1. `clock_val`: CPG heartbeat.
  2. `linear_speed`: Absolute forward movement speed.
  3. `rotational_speed`: Absolute turning speed.
  4. `internal_energy`: Stored stomach/metabolic energy.
  5. `adrenaline_level`: Current survival/fight-or-flight arousal.
  6. `pain_signal`: Critical collision-induced tissue damage.
- **Peripheral Isolation:** Remove all self-state, pain, and rotational tracking from the variable organelle-specific input channels, making organelles 100% focused on exteroception.

### Phase 3: Symmetrical Sensor Fusion (HOX-gated Interneurons)
Abolish manual birth-time synapse copying and sign inversion for bilateral symmetry:
- **HOX-Gated Synthesis:** If `has_hox` is true, do not duplicate raw synapses. Instead, automatically synthesize two groups of virtual Fusions-Interneurons (Hidden Nodes) for each symmetrical organelle pair:
  - **Sum Interneurons ($S_c = L_c + R_c$):** Aggregates total stimulus intensity. Automatically wired to feed forward into **Motor 0 (Thrust)**.
  - **Difference Interneurons ($D_c = L_c - R_c$):** Captures stimulus deflection. Automatically wired to feed forward into **Motor 1 (Bending)**.
- **Result:** Neural symmetry is guaranteed by the input architecture itself, completely avoiding asymmetrical drift during active lifetime Hebbian learning.

### Phase 4: Direktionale Strömungswahrnehmung (Rheotaxis & Gyro)
Introduce directional flow sensing for haptic organelles (`spectral_affinity < 0.25`):
- **Vectorial Drag:** Project relative water movement (external currents + self velocity) onto organ angles to feed a signed $[-1.0, 1.0]$ flow signal (`local_drag`).
- **Genomic Splay via `expression_style`:**
  - *Taktile Borsten ($E < 0.35$):* High tactile pressure and local pain only. No flow tracking.
  - *Seitenlinien-Poren ($0.35 \le E < 0.70$):* Vectorial flow sensing and local pain.
  - *Thermo-Proprioceptors ($E \ge 0.70$):* Muscle spindle flexion (`bend_angle`) and temperature sensing.

### Phase 5: Phasische Sehkanäle ($\Delta I$)
Enable motion-vision to support target tracking in dynamic training scenarios:
- **Temporal Delta:** Record raw eye inputs from the previous frame to calculate the temporal derivative:
  $$\Delta I = I_{\text{current}} - I_{\text{previous}}$$
- **Brain Integration:** Feed $\Delta I$ into separate input channels to allow the CTRNN to detect movement, facilitating tracking of fast-moving targets.

---

## Consolidation & Crossover
- **TCK-123 Interlock:** This Epic directly supports and simplifies `TCK-123` Scenarios:
  - *Rheotaxis* directly solves `DriftingCurrent` navigation.
  - *Phasic vision* directly solves `FastPrey` tracking.
  - *Bilateral difference gating* optimizes the homing behavior in `DistantTarget` and `ObstacleReef`.
- **Elimination:** Completely replaces legacy manual synapse mirroring code in `dna.rs` (lines ~1148-1165).

---

## Verification & Testing
- Write Rust unit tests for the Sum- and Difference-Interneuron generation logic in `dna.rs`.
- Write unit tests verifying that $0^{\circ}$ straight-ahead coordinate space calculations hold true for left-facing and right-facing sensors.
- Verify full TypeScript compilation and Vite build with zero errors.
