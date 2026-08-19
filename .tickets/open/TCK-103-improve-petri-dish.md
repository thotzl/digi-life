---
id: TCK-103
title: Improve Petri Dish simulation and environmental physics
status: open
assigned: Gemini-CLI
created_at: 2026-08-19
---

# TCK-103: Improve Petri Dish simulation and environmental physics

## Context
The Petri Dish environment houses the creatures and food spores. To increase evolutionary complexity, we need to improve the environment's physics, boundary rules, and nutrient distribution. The final details of these improvements will be refined in collaboration with the user.

## Requirements
1. **Physical Boundaries:** Evaluate and refine wall-collision behaviors (e.g., elastic bouncing vs. friction/drag, or toroidal wrap-around options).
2. **Nutrient Dynamics:** Improve food spore spawn algorithms (e.g., localized nutrient clusters, decay cycles, or depth-based light gradients).
3. **Fluid Simulation (Draft):** Add basic friction/viscosity forces to simulate fluid movement resistance.
4. **Visual Elements (Draft):** Enhance procedural rendering of the dish (e.g., custom border shadows, coordinate grids, or toxic zone areas).

## Tasks
- [ ] Discuss and finalize physical boundary mechanics.
- [ ] Refine the food spore distribution/replenishment heuristics.
- [ ] Implement fluid drag coefficients for creature movement.
- [ ] Update rendering to visually reflect improved Petri Dish features.
---
