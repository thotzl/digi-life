---
id: TCK-112
title: Local Training Chamber Scenarios & Match Arena Setup
status: open
assigned: Gemini-CLI
created_at: 2026-08-21
---

# TCK-112: Local Training Chamber Scenarios & Match Arena Setup

## Description
Develop the client-side configurations and local Match Arena setups. This module lets users set up distinct environmental sandbox chambers for specialized local training, as well as configure competitive match formats to evaluate locally saved lineages.

## Requirements & Scope
- **Environmental Chambers Configurations:**
  - **Training Room Scenarios:** Configurations for predefined task-based sandboxes (e.g., foraging paths, navigation).
  - **Ocean Dynamics Chamber:** Parameter configurations for fluid drag, momentum, and water resistance parameters.
  - **Wilderness Survival Chamber:** Configurations for metabolic decay multipliers and resource scarcity settings.
  - Expose interactive sliders in the UI to let users dynamically adjust physics constants, food spawn rates, and maximum populations.
- **Match Arena Engine Configuration:**
  - Create a setup UI to select multiple saved local species from the catalogue.
  - Implement duel game configurations: 1v1 confrontation, multi-species Free-for-All, and team territorial matches.
- **Frame-Interpolated Canvas Renderer:**
  - Implement linear/cubic spline interpolation in the HTML5 Canvas rendering loop to transform the incoming 25Hz IPC update stream into smooth 60Hz/120Hz/144Hz visuals.
- **Spectator HUD Overlay:**
  - Add camera tracking modes (focus on high-fitness creatures, manual panning/zooming).
  - Add a live neuron synapse firing inspector that displays the active node states of a clicked creature.
