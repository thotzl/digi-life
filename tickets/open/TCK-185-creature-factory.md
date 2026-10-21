---
id: TCK-185
title: CreatureFactory for Trainer and Ocean
status: open
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
---

# TCK-185: CreatureFactory for Trainer and Ocean

Part of epic TCK-171, phase P2. Trainer/creature slice of TCK-156 (RV-A01).

## Description
Creatures start differently in trainer and Ocean: the trainer sets `energy: 100.0` and an empty `antisense` (`src/biology/trainer_engine.rs:101`); the Ocean uses 60 % or 80 % of stomach capacity and sets `antisense` sometimes, sometimes not, in six copies (`src/server/engine.rs:306-680`); `neuron_states` empty vs. preallocated. With an energy budget in the trainer (TCK-178) and the later transfer to the Ocean, the start state must be identical.

## Scope
- One `CreatureFactory::spawn(genome, opts)` used by trainer and Ocean; options only for what really differs (position, heading, origin).
- Start energy, `antisense`, neuron state buffers, synapse weights defined once (start energy from config).
- Replace all six Ocean copies and the trainer construction.

## Acceptance
- No `CreatureAgent { ... }` literal outside the factory (tests excepted).
- Test: the same genome spawned for trainer and Ocean yields identical agents except position/heading/id.
- Ocean smoke check (TCK-186) passes; intended start-energy change documented.
