# 🧬 Handoff: Pixel DNA Life - Dynamic Species Catalogue & Unified Diagnostics

---

## 1. Current State (Complete Feature Deliverables)

We have successfully established a highly integrated, visually polished, and fully persistent artificial life environment inside our Tauri desktop shell.

### A. The Persistency Layer & Catalogs (SQLite Integration)
*   **Auto-Migrations:** Verified safe, non-destructive SQLite migrations for `creature_catalogue` and `trainer_genomes` in `src/database.rs`.
*   **Corrected Case-Deserialization:** Restored 100% stable database saving by aligning the JS properties with Tauri's camelCase conversion requirements (changing `synapse_weights` to `synapseWeights` inside `UnifiedDiagnosticsPanel.ts`). Saved specimens now persist across app restarts and appear instantly in the Catalogue list.

### B. The Unified Diagnostics HUD Sidebar
*   The `UnifiedDiagnosticsPanel.ts` is fully modularized and integrated across all three primary SPA screens:
    1.  **Ocean View:** Displays real-time vitals, chromatin grid, and directed CTRNN firing graphs of the selected free-swimming creature.
    2.  **Trainer View:** Displays the focused sandbox chamber details.
    3.  **Catalogue View:** Displays details of cryo-preserved ledger entries.

### C. Seamless "Pick and Add" Sim Integrations
*   **Ocean - "Load from Catalogue":** Built a gorgeous, scrollable card deck modal inside the Ocean view that replaces the basic select dropdown. It runs a local 60Hz requestAnimationFrame rendering loop inside a circular bullseye preview window, displaying the wiggling organism before it is cloned.
*   **Trainer - "Assign from Catalogue":** Clicking on any paused sandbox card (1 to 16) displays a blue `Assign from Catalogue` button in the HUD. It opens an identical circular preview selection modal, mutating that specific sandbox's body and mind in-place in-memory (and persisting the assignment in `trainer_genomes` via `"ASSIGN_SANDBOX_CREATURE"`).

### D. UI Refinements & Starvation Tuning
*   **Pre-training default:** Unchecked by default (`checked = false`) across all 3 views.
*   **Unconditional closing:** All modal selection boxes close instantly and unconditionally upon confirmation.
*   **Ocean Starvation:** Reduced the Ocean basal metabolic rate (`bmr_decay`) by 55% (`* 0.45` coefficient in `src/server/engine.rs`), granting organisms extensive organic lifespans to swim, adapt, and graze.
*   **Ocean Loop-Death Fix:** Integrated a Preact Signal `effect` route observer to auto-start/resume `drawBetaSimulationFrame` on entry into `"ocean"` view.

---

## 2. Diagnosis of the Current Point of Failure (Motionless Ocean)

### The Symptom:
*   The Ocean canvas, biomes, and food spores render perfectly.
*   The route-restarts work and loopRunning operates cleanly.
*   **But:** Spelled specimens and restocked/spawned creatures in the Ocean appear mostly motionless or frozen (visually), even though the exact same specimens are highly agile and wiggle intensely inside the circular Catalogue preview modal.

### The Technical Leads for the Next Session:
*   **The Time Locus discrepancy:** 
    *   In the Catalogue preview, `time` is passed relative to `animTime += 0.045`.
    *   In the main Ocean renderer (`drawBetaSimulationFrame`), `time` is passed as `timestamp * 0.015`.
    *   If `timestamp` passed from `requestAnimationFrame` represents elapsed milliseconds (e.g. `20000.0` for 20 seconds), then `timestamp * 0.015` becomes `300.0`. We must verify if the multiplier is scaling the sway too fast or slow compared to the stable `animTime`!
*   **The Brain activations vector:**
    *   In the Ocean, the brain output drives movement: `let net_thrust = out_thrust * thrust_mag;`
    *   If `out_thrust` (the brain's thrust output) is `0.0`, the creature doesn't move forward, so `v_forward` is `0.0`. Since `v_forward` is zero, the spinal waving/wiggling inside `creatureRenderer.ts` is heavily suppressed or static because wiggling is kinematically coupled to speed!
    *   *Why is the brain output zero?* 
        1.  In `execute_brain_with_learning`, check if `neuron_states` and `neuron_activations` are properly resizing to `total_nodes`.
        2.  Check if the inputs fed into the brain (`inputs`) in the Ocean actually contain the oscillating clock. In this session, we added:
            ```rust
            let clock_val = (agent.age as f32 * 0.05).sin();
            inputs[k] = clock_val;
            ```
            Verify if this clock value is successfully driving the CPG engine.

---

## 3. Structured Checklist (For the Next Session)

1.  [ ] **Open the Tauri Inspector:** Right-click inside the Ocean window ➔ Inspect ➔ Console. Verify if there are any silent exceptions during the render frame loop.
2.  [ ] **Test Wiggle in Catalogue:** Save a highly active specimen from a successful Trainer run. Confirm that it wiggles intensely inside the persistent Catalogue and inside the `Load from Catalogue` preview circle.
3.  [ ] **Verify Ocean Time scale:** In `frontend/ocean.ts`, temporarily replace `timestamp * 0.015` inside `renderer.render` with `Date.now() * 0.001` or a manual accumulator `localAnimTime += 0.045` on each frame. See if the body instantly starts wiggling independently of its speed.
4.  [ ] **Print Brain outputs:** In `src/server/engine.rs` line 935, log the outputs `out_thrust` and `out_left` for a spawned catalogue clone. Confirm if they are non-zero.
