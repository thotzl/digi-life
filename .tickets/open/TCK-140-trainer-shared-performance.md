---
id: TCK-140
title: Trainer and Genome Compiler Performance (behaviour-preserving)
status: open
assigned: Claude Code
created_at: 2026-10-19
revisit: RV-P06, RV-P03 (trainer), RV-P05 (trainer)
---

# TCK-140: Trainer and Genome Compiler Performance (behaviour-preserving)

## Description
- `mutate_genome` runs a full `parse_genome` (incl. brain synthesis) only to read the insertion/deletion rates.
- Brain reflex detection is a nested loop per candidate synapse.
- The scenario step clones the full agent every tick for spore push physics.
- Trainer telemetry sends each sandbox's full procedural world at 25 Hz although the frontend may not need it per tick (to verify).

## Scope
- Extract a rate reader; reflex lookup via a set; kinematics-only spore push input; drop the redundant telemetry payload only if the frontend does not use it.
- Equivalence tests (same genome gives an identical phenotype/brain before and after).

## Acceptance
- `cargo test` green, equivalence tests green.
