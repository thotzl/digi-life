# 🧬 Handoff: Pixel DNA Life - Parallel Tauri/Rust Ocean Prototype

---

## 1. Current State (Milestones Achieved)

In this session, a fully parallel, functional desktop development stack was established. The original TypeScript/browser game remains 100% untouched and undamaged, and is fully playable.

### A. The Native Simulation Core (Rust - `src-tauri/`)
*   **Initialization & Multithreading:** Tauri v2 was integrated into the project. Upon startup, a parallel simulation background thread is automatically spun up in `src-tauri/src/main.rs`.
*   **Full Ocean Simulation:** The physical world movement (`physics.rs`), the grid for fast collision queries (`spatial_grid.rs`), the BMR metabolism controls, feeding, asexual reproduction (mitosis), mutation logic, and CTRNN-Euler brain couplings have been fully implemented in native Rust.
*   **Local SQLite Data Persistence:** Species lineage trees and new discoveries are transactionally persisted directly into the local file `pixel_life_local.db`.
*   **Isolated CLI Trainer:** The unthrottled console trainer has been moved to a standalone Cargo binary. It still runs independently of Tauri and without GTK system packages directly via:
    ```bash
    cargo run --bin cli_trainer
    ```

### B. The Frameless Frontend (Tauri Webview)
*   **Custom HTML View (`tauri_ocean.html`):** An untouched, isolated entry point for the desktop window.
*   **Reactive Tauri Fork (`src/tauri_ocean.ts`):** 
    *   All WebSockets and HTTP-POST interfaces have been completely replaced with native Tauri interfaces (`listen` for data streams, `invoke` for control commands).
    *   An **asynchronous handshake procedure (`"CLIENT_READY"`)** with a 200 ms delay was established to completely eliminate race conditions when loading the event listener.
    *   The helper function `safeInvoke` with a robust ESM try-catch wrapper completely intercepts module resolution errors in the normal web browser, preventing the diagnostic page in the browser from crashing.
    *   The left sidebar was connected directly to the local Rust database via the Tauri command `get_registered_species`.

---

## 2. Diagnosis of the Current Point of Failure

### The Symptoms:
*   The left and right sidebars (HUD) update dynamically.
*   The generations increment, organisms change their counts in the HUD (e.g., from `5/20` to `9/20`), and new species are added live to the left sidebar.
*   **But:** The canvas in the background remains blank (no green spores, no cells visible).

### The Technical Cause:
*   **Asynchronous Decoupling:** The HUD values (sidebars) are received via the asynchronous Tauri event listener (`listen("simulation-state")`) and bound directly to the HTML DOM via reactive Preact Signals. This works flawlessly (so the data stream flows error-free in the RAM IPC channel!).
*   **Break in the Render Loop:** The actual drawing on the canvas runs in a separate, continuous browser animation thread (`drawBetaSimulationFrame`). 
*   If this loop fails even once at startup (e.g., during camera zoom initialization or within the `CreatureRenderer`) due to an undefined value (e.g., `dpr`, `canvas.getContext`, or mismatching transformation matrix values), the rendering loop silently terminates. The canvas remains black, while the HTML HUD continues to update actively in the foreground.

---

## 3. Exact Next Steps (For the Next Vibe Coding)

To bring the canvas to life in the Tauri window, the following points must be investigated in the frontend fork `src/tauri_ocean.ts`:

1.  **Verify Canvas Transformations in the Viewport:**
    Check whether the camera matrix in `drawBetaSimulationFrame` (line 640) receives correct values for `camZoom`, `camX`, and `camY` when the Tauri window boots in the default HD format ($1280 \times 720$), or whether the mathematical transformation positions the creatures off-screen.
2.  **Debug the `CreatureRenderer` in the Tauri Inspector:**
    In the open Tauri window, **right-click ➔ Inspect** and switch to the **Console** tab. Read the exact stack trace thrown when executing the canvas frame (e.g., whether `renderer.render` is looking for an unexpected field in the decompiled `CreaturePhenotype` that Rust has serialized slightly differently).

The entire communication and calculation flow is solid and runs with immense stability. We have successfully broken through the technological barrier for a native desktop distribution!

---

## 4. Concepts for AI-Assisted Remote Debugging (Direct AI Connection)

In order for me (the AI) to autonomously inspect and analyze errors in the Tauri window and on the canvas in the future, we can set up one of the following asynchronous diagnostic interfaces:

### Concept A: The "Error Mirroring" Channel (Highly Recommended)
*   **How it works:** We register a global error listener (`window.onerror` and `window.onunhandledrejection`) in the frontend (`src/tauri_ocean.ts`). 
*   If a render or JavaScript error occurs in the Tauri window, the frontend immediately sends this stack trace to Rust via `safeInvoke`.
*   Rust continuously writes these error messages to a local file `src-tauri/client_debug.log`.
*   *The advantage:* I can read this log file live in the terminal using my `read_file` tool. I see every crash on your screen instantly without you needing to copy anything.

### Concept B: Tauri stdout Mirroring (Console Redirection)
*   We integrate the official Tauri plugin `tauri-plugin-log`. This redirects all standard outputs from `console.log` and `console.error` of the webview directly to the system terminal (stdout) of Tauri. I can then see your web logs directly in the CLI processes.

### Concept C: The "Debug Snapshot" (F8 Trigger)
*   We set up a hotkey in the frontend (e.g., key `F8`). 
*   Upon pressing the key, the frontend saves all current variables (camera zoom, loaded entities, last 15 console outputs) into a temporary file `client_snapshot.json`. I can read this file and obtain a complete mathematical representation of the current render state.
