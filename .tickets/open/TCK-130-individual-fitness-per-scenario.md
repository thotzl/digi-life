---
id: TCK-130
title: Individual Fitness Evaluation Per Training Mode
status: open
assigned: Gemini-CLI
created_at: 2026-09-03
---

# TCK-130: Individual Fitness Evaluation Per Training Mode

## Background & Motivation
Currently, the Reinforcement Learning Trainer calculates candidate fitness using a single, unified scoring formula (`calculate_sandbox_fitness` in `trainer_engine.rs`). This formula heavily penalizes long travel distances and kinetic expenditure (`kinetic_waste = distance_traveled * 0.05`). 

While this penalty is critical in small standard chambers (1000x1000) to discourage chaotic, circular swim paths and promote localized grazing efficiency, it is completely counter-productive in large Exploration chambers (3500x3500). In the Exploration arena, pellets spawn far outside of initial sensory range, meaning that successful candidates *must* swim long exploration paths to locate the food. Penalizing travel distance in this scenario results in stagnation, as stationary or minimally moving candidates are favored over active searchers.

To resolve this evolutionary dead-end, this ticket introduces dynamic, mode-specific fitness calculations tailored to the characteristics of the active training chamber size.

## Requirements & Scope
1. **Dynamic Fitness Dispatch (`trainer_engine.rs`):**
   - Refactor `calculate_sandbox_fitness` to accept the active `chamber_size` (or training mode).
   - Alternatively, branch the fitness logic internally based on whether `chamber_size >= 2000.0` (Exploration mode) or `chamber_size < 2000.0` (Standard mode).

2. **Exploration Mode Scoring Rules:**
   - **Remove Distance Penalty:** Set `kinetic_waste = 0.0` (or drastically reduce it) to avoid punishing candidates for active, long-range exploration.
   - **Path Efficiency Scaling:** Adapt path efficiency calculations to account for large, empty blind zones, rewarding candidates that successfully locate the target after a search phase.
   - **Search Reward:** Reward continuous search coverage without crashing into walls, but do not tax pure travel distance.

3. **Standard Mode Scoring Rules:**
   - Keep the existing highly optimized and proven fitness penalties (circular path penalization, kinetic waste, and wall collision caps) exactly as they are to protect baseline learning.

4. **Verification & Tests:**
   - Add a dedicated Rust unit test inside `trainer_engine.rs` to assert that for identical movement telemetry, Exploration mode scores candidates with high travel distances much higher than Standard mode.

## Implementation Steps
1. **Step 1: Backend Refactoring (Rust)**
   - Add the `chamber_size` parameter to `calculate_sandbox_fitness`.
   - Update `trainer_engine.rs` and `engine.rs` to pass this parameter.
   - Separate the scoring paths into Standard (stiff, localized) and Exploration (loose, search-rewarding).
2. **Step 2: Verification**
   - Verify that when training under Exploration mode, average fitness rises continuously as active searching patterns are selected.
