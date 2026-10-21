---
id: TCK-110
title: Development of Headless Rust Simulation Core with SQLite Integration
status: closed
assigned: Gemini-CLI
created_at: 2026-08-21
---

# TCK-110: Development of Headless Rust Simulation Core with SQLite Integration

## Description
Develop a native headless simulation process in Rust. This process will execute the continuous mathematical computations (CTRNN integration, spatial grid mechanics, biological mutations) and save operational run histories directly to a local SQLite database.

## Requirements & Scope
- **Physical Engine Core (Rust):**
  - Implement a 2D physical environment utilizing a Spatial Grid partition for fast collision checking of creatures, boundaries, and static resources.
- **Neural & Biological Core (Rust):**
  - Implement a Continuous-Time Recurrent Neural Network (CTRNN) solver utilizing numerical Euler integration:
    $$\tau_i \frac{dy_i}{dt} = -y_i + \sum w_{ji} \sigma(y_j + \theta_j) + I_i$$
  - Implement DNA de-compilation and double-stranded mutation algorithms (Sense/Antisense base matching).
- **Asynchronous Telemetry Stream:**
  - Build an IPC stream throttling output to 25 Hz when visual rendering is active. The stream must transmit compressed coordinate matrices and active neuron firing states.
- **SQLite Database Persistence:**
  - Integrate an embedded SQLite driver in the Rust service.
  - Store physical run metrics, fitness logs, and speciation history mapped under the corresponding `session_id` provided by the UI.
- **Multithreading:**
  - Support execution of $N$ isolated training sandboxes concurrently across multiple CPU threads.
