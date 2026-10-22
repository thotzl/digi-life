---
id: TCK-185
title: CreatureFactory for Trainer and Ocean
status: closed
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

## Verification (P2, v1.12.16)
- `src/biology/creature_factory.rs`: `spawn_creature`, `spawn_from_phenotype`, `start_energy`; all 6 Ocean spawn sites and the trainer sandbox use it; no `CreatureAgent {` literal outside the factory except test fixtures (`brain.rs` test).
- Start energy defined once: stomach x `creatureStartStomachRatio` (0.60, renamed from `restockInitialStomachRatio`). Intended changes: trainer start energy 100 -> stomach x 0.60 (the brain input `internal_energy` follows), SPAWN_CLONE and ASSIGN_SANDBOX_CREATURE 0.8 -> 0.60, trial reset uses `start_energy`; `antisense` and neuron buffers always set (buffers were resized before first use, so behaviour is unchanged).
- Tests: trainer-style vs Ocean-style spawn identical except position/heading/id/generation; synapse-weight length mismatch falls back to genome weights. `cargo test` green.
- Ocean smoke check (TCK-186): Ocean runs, inject works, no NaN (UI smoke 2026-10-22).
