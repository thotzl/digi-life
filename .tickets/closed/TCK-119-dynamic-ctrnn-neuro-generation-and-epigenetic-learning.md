---
id: TCK-119
title: Dynamic CTRNN Neuro-Generation and Transgenerational Epigenetic Hebbian Learning
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
completed_at: 2026-08-31
---

# TCK-119: Dynamic CTRNN Neuro-Generation and Transgenerational Epigenetic Hebbian Learning

## Description
Unlock dynamic topological brain evolution and active lifetime learning. This module refactors the CTRNN compiler to dynamically build hidden layers (determining width, depth, and activation types from "NEU" and "SY" genes) and integrates real-time Hebbian synaptic plasticity. Learned synaptic changes are passed down to offspring using epigenetic transgenerational inheritance (Lamarckism) matching parent methylations.

## Requirements & Scope

### 1. Dynamic Topography ("NEU" & "SY" Genome Compiler)
- Redefine `h_count` dynamically by scanning the cleaned genome for `"NEU"` promoters (full biochemical wobble matching enabled).
- If no `"NEU"` promoters exist, fallback to a baseline of 2 hidden neurons to ensure network integrity.
- For each `"NEU"` block, compile:
  - **Decay ($\tau$), Bias ($\theta$), Activation Type:** via Modulo-Hashing of its payload.
  - **Depth ($y$-coordinate):** Hash the payload to map between `0.15` and `0.85` to support layered architectures.
- Scan for `"SY"` promoters in the cleaned genome (full-wobble matching enabled).
- For each `"SY"` block, map:
  - **`from_node`:** Safely modulo-mapped to valid source nodes (Inputs + Hiddens).
  - **`to_node`:** Safely modulo-mapped to valid destination nodes (Outputs + Hiddens).
  - **`weight`:** Hashed to a range between `-2.0` and `2.0`.
- Maintain fallback synapses if no functional `"SY"` promoters are found.

### 2. Live Hebbian Learning Updates
- Add `pub synapse_weights: Vec<f32>` to the `CreatureAgent` struct in `src-tauri/src/shared/types.rs`.
- Implement `execute_brain_with_learning` inside `src-tauri/src/shared/brain.rs` to compute Hebbian plasticity at each timestep:
  - $$\Delta W = \text{hebbian\_learning\_rate\_base} \times a_{\text{pre}} \times a_{\text{post}} - \text{hebbian\_forgetting\_decay} \times W$$
  - Clamp dynamic weights strictly between `-4.0` and `4.0` to avoid divergent numerical explosions.

### 3. Lamarckian Transgenerational Inheritance
- During mitosis/reproduction in the simulation loops (`server/engine.rs`):
  - Clone active `synapse_weights` to asexual clones directly.
  - If a mutation triggers a re-parse, copy the parent's learned synaptic offsets ($\Delta W = W_{\text{active}} - W_{\text{base}}$) to homologous child synapses (same `from_node` and `to_node`), scaled by `lamarckian_assimilation_chance` (e.g., 25% scale).

### 4. Verification & Testing
- Write automated tests checking:
  - Correct dynamic hidden node width and depth compiles under `"NEU"` and `"SY"` promoters.
  - Real-time Hebbian weight changes during continuous steps of `execute_brain_with_learning`.
  - Epigenetic assimilation correctly maps learned offsets to mutated homologous synapses in child cells.
- Verify both `npm run test:rust` and overall application builds complete with zero compiler warnings or errors.
