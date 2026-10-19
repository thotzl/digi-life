---
id: TCK-156
title: Engine Decomposition (OceanSim / TrainerSim)
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-A01, RV-A05
---

# TCK-156: Engine Decomposition (OceanSim / TrainerSim)

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-A01, RV-A05).

## Description
`spawn_simulation_thread` is a ~2000-line closure; creature construction duplicated 6x; world size literals in Rust and TS.

## Proposed Scope
Structs with `step()` + event sink, `CreatureFactory`, constants from one source.
