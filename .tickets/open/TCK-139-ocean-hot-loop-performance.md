---
id: TCK-139
title: Ocean Hot-Loop Performance (behaviour-preserving)
status: open
assigned: Claude Code
created_at: 2026-10-19
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
