---
id: TCK-139
title: Ocean Hot-Loop Performance (behaviour-preserving)
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
revisit: RV-P01, RV-P02, RV-P03, RV-P04, RV-P05, RV-P07
---

# TCK-139: Ocean Hot-Loop Performance (behaviour-preserving)

## Description
The Ocean step (up to 35 steps per 60 Hz frame) runs a full `species_records` SQL scan per step, allocates a 32k-cell spatial grid per step, deep-clones agents per food spore, linearly scans all food per creature, emits IPC events per step and re-queries the catalogue per restocked creature.

## Scope (no simulation behaviour change; Ocean logic incl. the missing offspring stays as is)
- Species peak/extinction tracking from in-memory counts; DB writes only on change.
- Reuse the spatial grid; remove per-spore agent clones and the full `creatures.clone()`.
- Event buffering: flush once per frame; coalesce `DATABASE_CHANGED`.
- Load the catalogue once per restock pass.

## Acceptance
- `cargo test` green; manual smoke test: Ocean runs, restock/death/eat/bite events visible as before.

## Verification
- RV-P02 `src/server/engine.rs`: spatial grid (240x135 cells) allocated once and `clear()`ed per step instead of re-allocated per step.
- RV-P03 `src/server/engine.rs` / `src/shared/physics.rs`: grid buckets hold vector indices; spore pushes use `SporePusher` kinematic snapshots (sorted indices = original creature order) instead of deep-cloned agents; `alive_clones` is a `PeerSnapshot` (id, species, position, velocity) instead of a full `creatures.clone()`; food and bite lookups index directly instead of `position`/`find` by id. Trainer scenarios (`src/biology/scenarios.rs`) also push spores via `SporePusher` instead of cloning the agent every tick.
- RV-P01: species tracking reads `species_records` only after an insert or reset (dirty flag set at all 7 inserts and the reset delete); peak/status updates are mirrored into the cache; per-species counts via one `HashMap` pass instead of O(species x creatures).
- RV-P05: events are buffered and flushed once per frame in original order; repeated `DATABASE_CHANGED` within one frame is coalesced.
- RV-P07: catalogue loaded once per restock pass instead of once per missing creature.
- RV-P04 (grid-based nearest-food search) deliberately not done: a ring search can change tie-breaking between equidistant spores; cost is small at 45 x 300. Moved to pool ticket TCK-156.
- Equivalence: new test `shared::physics::tests::test_spore_push_from_snapshot_matches_full_agent`; engine changes preserve iteration order and data (engine closure itself is not unit-testable, see RV-A01/TCK-156). Manual smoke test in the final `tauri dev` run.
- `cargo test`: 40 passed, 0 failed; no new warnings.
