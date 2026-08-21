# Active Task & Safety Ledger (TCK-116)

## 🎯 Current Focus
[DONE] Implement **TCK-116: Sequence-Based Dynamic Genome & Gene Regulatory Network (GRN)** inside the native Rust core, establishing a fully decoupled, pattern-scanning raw-signal genomic compiler.

---

## ⚠️ CRITICAL SAFETY RULES (Never Reintroduce These Solved Bugs!)

During any future refactoring of the biology, physics, or layout engines, **strictly enforce** these 5 rules to prevent regressions of the bugs we solved in Version 1.7.0:

1.  **NO Startup Mode Race Conditions:**
    *   *Rule:* `src/tauri/tauri_trainer.ts` must **never** automatically call `SET_MODE: trainer` on startup or load. 
    *   *Reasoning:* If loaded on startup, it overrides the initial state, leaving the active Ocean simulation frozen. The active mode (`ocean` or `trainer`) must be managed **exclusively** by the user's tab selection in `src/main.ts`.

2.  **NO Warp-Speed Thread Locking:**
    *   *Rule:* The unlimited warp speed inside `engine.rs` must **always** clamp its execution steps per 60Hz tick to a maximum of **35** steps:
        ```rust
        let steps_this_frame = if trainer_warp_speed > 35 { 35 } else { trainer_warp_speed };
        ```
    *   *Reasoning:* Running thousands of steps blockingly within a single frame starves the main thread, freezes the UI, and prevents the "Pause" button from registering. Limiting steps to 35 keeps the channel highly responsive (0ms latency) while still compiling ~7 generations per second.

3.  **NO Phantom Shivering Creatures on Reset:**
    *   *Rule:* Triggering `"RESET_EVOLUTION"` in `engine.rs` must **always** clear the newborn queue:
        ```rust
        newly_spawned_creatures.clear();
        ```
    *   *Reasoning:* Failing to clear this queue causes the client to re-add deleted creatures from the previous simulation run, which stand motionless on the screen and shiver because they are no longer simulated.

4.  **NO Microscopic Physics Thrust Force:**
    *   *Rule:* The raw motor activation `out_thrust` ($\in [0.0..1.0]$) must **never** be passed directly into `apply_creature_physics`. It must **always** be upscaled using the biological phenotype multipliers (muscle stiffness, pulse speed, spinal wave phase, and active swimming limbs):
        ```rust
        let mut thrust_mag = agent.phenotype.stiffness * (pulse * 1000.0 * pulse * 1000.0) * 6.0;
        // ... limbs, parapodia scaling ...
        let net_thrust = out_thrust * thrust_mag;
        ```
    *   *Reasoning:* Unscaled activation is up to 10,000x too weak compared to the creature's mass, leaving them shivering in place instead of swimming.

5.  **NO Missing/Corrupted Bundle Icons:**
    *   *Rule:* The `"icon"` list in the `"bundle"` section of `tauri.conf.json` must **always** explicitly register `"icons/icon.ico"` and `"icons/icon.icns"`, and these binary files must remain valid (compiled via `npx tauri icon`) to prevent Windows Wix and macOS packagers from crashing.

---

## 🔬 TCK-116 Architectural Blueprint

### Unified Decoupled Raw-Signal Parser Helper
The helper method must reside inside the Rust biology engine (`src-tauri/src/biology/dna.rs`) as a completely decoupled, pure data-extraction method:
```rust
pub fn extract_raw_gene_signals(
    genome: &str,
    start_motif: &str, // Promoter (e.g. "EYE", "COL", "STF")
    stop_motif: &str   // Terminator (e.g. "EN", "STP", "SP")
) -> Vec<f32>; // Returns neutral, normalized float values [0.0..1.0]
```

### Module Integration (Schema F)
All phenotypic traits and the entire CTRNN neural brain must read their parameters natively from this helper:
*   **Stiffness Module:** Fetches a single value and scales it to `[0.15..1.0]`.
*   **Color Module:** Fetches 3 signals and maps them to H (`[0..360]`), S (`[55..100]`), and L (`[35..80]`).
*   **Sensory Modules:** Fetches custom signals and chunks them into pairs (Angle, Range) to generate variable amounts of eyes or noses.
*   **CTRNN Brain Module:** Scans for `"NEU"` and `"SYN"` gene blocks. Translates the payload codons (using deterministic Modulo-hashing) into weights, biases, and decay rates ($\tau$).
