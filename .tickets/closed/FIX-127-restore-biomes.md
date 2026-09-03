---
id: FIX-127
title: SPOT-Compliant Rust Procedural Map Generation
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
closed_at: 2026-09-03
---

# FIX-127: SPOT-Compliant Rust Procedural Map Generation

## Background & Motivation
The user noted that the beautiful biome generation and obstacles disappeared after TCK-125. This occurred because we deleted the client-side `mapGenerator.ts`, which was violating our Single Point of Truth (SPOT) paradigm (the Rust physics engine didn't know the obstacles existed). 
The user perfectly decreed: **"We should generate this from the Rust backend and only display it in the frontend."**

This ticket tracked the recreation of the procedural map generation entirely within the native Rust backend, transmitting it to the frontend strictly for "dumb" visual rendering.

## Solution Implemented

### 1. Rust Data Structures & Generator
- Created `src/shared/map_generator.rs` with structs `ProceduralWorld`, `BiomeArea`, `ProceduralObstacle`, `CurrentVent`, and `Vertex`.
- Implemented the deterministic `generate_world(seed: &str, width: f32, height: f32)` logic in Rust using a deterministic Mulberry32 PRNG (hashing the seed string into a u32) to ensure perfect cross-language compatibility.
- Derived `Serialize`, `Deserialize`, `Clone`, `Debug`, and `TS` on all structs.

### 2. Payload Integration
- **Ocean (`src/server/engine.rs`):** Generated the `world` on `CLIENT_READY` and `RESET_WORLD` actions and attached it directly to the `INIT_STATE` JSON payload.
- **Trainer (`src/biology/trainer_engine.rs`):** Added `pub world: ProceduralWorld` to `TrainerSandbox` and `TrainerTelemetrySandbox` to ensure the trainer backends store and stream their respective obstacle configurations natively.

### 3. Frontend Dumb Rendering
- Exported the Rust types so they appear in `/bindings/` and `frontend/shared/types.ts`.
- In `frontend/ocean.ts`, restored `createBiomeCache(world)` and `drawWorldTerrain(ctx)` to render the biomes, current vents, and rocks received from the backend `INIT_STATE` handshake.
- In `frontend/trainer.ts`, restored the simple `ctx.arc` loops inside `drawSandbox(sb)` to render the obstacles received inside the trainer's telemetry.
