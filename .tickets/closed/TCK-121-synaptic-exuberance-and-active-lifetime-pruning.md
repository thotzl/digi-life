---
id: TCK-121
title: Synaptic Exuberance and Active Lifetime Pruning
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
completed_at: 2026-08-31
---

# TCK-121: Synaptic Exuberance and Active Lifetime Pruning

## Description
Implement the neurodevelopmental paradigm of Synaptic Exuberance (fully-connected neural network at birth) followed by Active Lifetime Pruning (elimination of unused connections during lifetime learning). This allows candidates to explore all possible wiring pathways dynamically and eliminate unnecessary synapses, conserving metabolic energy and stabilizing network dynamics.

## Requirements & Scope

### 1. Synaptic Exuberance at Birth (dna.rs)
- Update `parse_genome` to build a fully connected CTRNN graph:
  - Generate a synapse for every pair of `sources` (Inputs + Hiddens) and `destinations` (Outputs + Hiddens), avoiding direct self-loops.
  - If a specific `SY` gene matches a connection, initialize it with its strong, specialized genetic weight (`[-2.0 .. 2.0]`).
  - Otherwise, initialize it as a weak exploratory synapse with a hashed weight in `[-0.075 .. 0.075]`.

### 2. Active Lifetime Pruning (brain.rs)
- Update `execute_brain_with_learning` inside `src-tauri/src/shared/brain.rs`:
  - If a synapse weight is exactly `0.0`, skip Hebbian learning updates (permanently pruned for the creature's lifetime).
  - During the Hebbian update, if a synapse's absolute weight falls below `0.015`, prune it by setting its weight to exactly `0.0`.

### 3. Verification & Tests
- Write unit tests verifying:
  - Exuberant fully connected graphs are built upon parse.
  - Exploratory synapses start small, while specialized synapses start strong.
  - Active pruning successfully wipes out weights below the threshold and locks them at 0.0.
- Verify that both `npm run test:rust` and overall workspace builds complete with zero warnings.
