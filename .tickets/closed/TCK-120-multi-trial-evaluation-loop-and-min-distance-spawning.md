---
id: TCK-120
title: Multi-Trial Evaluation Loop and Min Distance Spawning
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
completed_at: 2026-08-31
---

# TCK-120: Multi-Trial Evaluation Loop and Min Distance Spawning

## Description
Eradicate fitness spikes and accidental "lucky" champions in the trainer. This module implements a robust 3-trial evaluation loop in the simulation engine and enforces a minimum spawning distance of 200 pixels between candidates and target food spores. This ensures only truly competent, target-tracking genomes are selected as elites.

## Requirements & Scope

### 1. Enforced Minimum Spawning Distance (Lösung 2)
- Update `init_rust_sandbox` inside `src-tauri/src/biology/trainer_engine.rs` to require food spores to spawn at least `200.0` pixels away from the creature agent's center.
- Ensure any mid-evaluation spore randomizations also respect this minimum limit.

### 2. Multi-Trial Evaluation Loop with Reset Phases (Lösung 1)
- Add `trainer_trial_index` (usize) and `trainer_accumulated_fitness` (Vec<f32>) state variables to the Rust simulation background thread loop inside `src-tauri/src/server/engine.rs`.
- When a 300-tick epoch ends:
  - If `trainer_multi_trial` is `true`:
    - Accumulate each sandbox's current epoch fitness.
    - If `trainer_trial_index < 2`:
      - Increment `trainer_trial_index`.
      - Reset `trainer_epoch_ticks = 0`.
      - For each sandbox, reset the agent's coordinates to `500.0, 500.0`, clear velocities, clear neuron states and activations, and randomize food spores (respecting the `200.0` pixel distance constraint).
    - If `trainer_trial_index == 2`:
      - Divide accumulated fitness values by `3.0` to calculate average candidate scores.
      - Perform sorting, select elites, save champions, increment `trainer_generation`, and breed the next generation of sandboxes.
      - Reset `trainer_trial_index = 0` and clear accumulated fitness.
  - If `trainer_multi_trial` is `false`:
    - Behave normally (breed after 1 trial/epoch).

### 3. Verification & Testing
- Write automated tests verifying:
  - Spores are spawned at a minimum distance of `200.0` pixels from the center.
  - Multi-trial resets clear neuron states and velocities while retaining genomes.
  - Elites selected are stable across multiple evaluations.
- Ensure both Rust and frontend builds compile with zero warnings or errors.
