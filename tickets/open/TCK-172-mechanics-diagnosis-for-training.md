---
id: TCK-172
title: Mechanics Diagnosis for Training
status: open
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
---

# TCK-172: Mechanics Diagnosis for Training

Part of epic TCK-171. Measurement only, no model change.

## Description
Static analysis (2026-10-21) points to core mechanics as the main reason why training fails in the large chamber. Measure the suspected effects before deciding on fixes. `parse_genome`, `mutate_genome`, `step_creature_kinematics` and `compute_sensory_inputs` are pure functions, so the probes need no engine refactor.

## Findings to verify

### 1. Heredity: low-locality genotype-phenotype map (code-verified)
- Exuberant synapse weights are read from `active_dna` at index `(from + to*13) % len` (`src/biology/dna.rs:1323-1325`, `get_payload_linear_value_offset` `dna.rs:674-686`); `active_dna` contains open-chromatin positions only (`dna.rs:743-750`).
  - An insertion/deletion inside an active gene shifts all later positions → every weight reading from there changes.
  - A mutation that creates/destroys a gene changes `len` → the modulo mapping of all weights changes.
- Neuron ids depend on the organ count (`dna.rs:12-26`); one organ more/less shifts every id → all weights and the explicit `SY` synapses (mapped by modulo over sources/destinations, `dna.rs:1251-1258`) re-map.
- Body traits without their own gene (`PUL`, `STF`, `SIZ`, ...) fall back to `active_dna` (`dna.rs:800-861`) → a brain-gene mutation can change speed.
- Trainer mutation: 15–30 mutation ops per child (`src/server/engine.rs:97`, `src/biology/trainer_engine.rs:78-84`).
- Expected effect: children barely resemble parents; the trainer acts as random search plus unmodified elite clones.

### 2. Locomotion speed (estimate)
Estimate from `src/shared/physics.rs:98-195` + `config.json` (`thrustBaseMultiplier` 3.4, `dragForwardCoefficient` 0.015, `dragForwardStiffnessDecay` 0.3, `receptorBallastScale` 0.18, velocity damping 0.94). Assumptions: 100k uniformly distributed trait values (`h_*` in [0, 1]), full thrust (`out_thrust` = 1), 2 organs, limb factor ignored. Terminal speed `v = 0.94·F / (0.06·m + 0.94·d)`; script kept locally in `.agents/artifacts/vterm.py` (gitignored).

| terminal speed px/tick | p10 | p25 | p50 | p75 | p90 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| full thrust | 0.08 | 0.23 | 0.74 | 2.2 | 5.0 |

- Median genome: 1200 px straight ≈ 1630 ticks (epoch 1800); 200 px ≈ 270 ticks (small-chamber epoch 300). Best case: straight line, perfect thrust, no search.
- 48 % cannot cover 1200 px in one epoch; 22 % are slower than 0.2 px/tick.
- Spread p10→p90 ≈ ×60; thrust scales with `pulse²` (`physics.rs:113`) → selection favours fast bodies over better brains.
- Turning only while moving: `delta_heading = v_forward * bend * 0.015` (`physics.rs:182`); min turning radius ≈ 55 px; slow creatures can hardly turn.

### 3. Sensing and brain layout (code-verified)
- Left/right comparison only for paired organs: thalamus difference nodes exist only with a `HOX` gene (`dna.rs:1021`, `dna.rs:1352-1374`); single organs give sums only.
- Initial brains barely drive: exuberant weights in [-0.5, 0.5] (`dna.rs:1327`), thrust output `tanh` → mean thrust ≈ 0, negative = backwards.
- Sensor range `scale * 550` (`src/shared/brain.rs:168`).
- Minor: motion vision overwrites the meat channel (TCK-144); 2-tick sensor→motor latency (TCK-153).

### 4. Walls (hypothesis)
- Wall contact reflects velocity but not heading (`physics.rs:208-228`); next tick velocity is projected back onto the heading (`physics.rs:194-196`) → a creature pushing into a wall may stall at ≈ 0 speed and then cannot turn (finding 2).

## Scope: probes
Own binary or `#[ignore]` tests that write a report to `.agents/artifacts/`; DB access read-only.

| Probe | Question |
| :--- | :--- |
| Heredity | Parent-child correlation of exuberant weights, explicit synapses and body traits after 1 / 5 / 15 mutations, split by point / insertion / deletion |
| Speed | Real terminal-speed distribution over 1,000 random genomes and over HoF genomes from the DB |
| Equipment | Share of random genomes with eye/nose, with a `HOX` pair, with food in sensor range at spawn (small and large chamber) |
| Steering | Food fixed at ±45°: does the creature turn towards it? Random vs. HoF genomes |
| Wall | Full thrust at an angle into a wall: does the creature get free within N ticks? |
| Hebbian | Same HoF genomes with and without Hebbian learning in the trainer scenario (TCK-152) |

## Acceptance
- Report with all six probes, committed numbers summarised in this ticket's `## Verification`.
- Each finding 1–4 marked confirmed / refuted with numbers.
- Follow-up candidates below decided by the user and ticketed (or dropped).

## Follow-up candidates (decide after measuring)
- **Heredity:** stable synapse identity (key per organ index, channel, target, NEAT-style innovation ids) instead of the position hash, or relative addressing (TCK-168); body traits only from their own genes; mutation 1–3 ops per child (TCK-174).
- **Body-brain coupling:** separate body and brain mutation, or morphological innovation protection (Cheney et al. 2018: reduced selection pressure for recently body-mutated individuals; read the critical 2025 follow-up first).
- **Speed:** rescale thrust vs. mass/drag to narrow the spread, e.g. linear in pulse (TCK-150, TCK-166).
- **Steering:** klinotaxis input — a dedicated temporal-change (ΔI) channel instead of overwriting the meat channel (TCK-144); alternatively organ pairs without `HOX`. C. elegans steers with a single sensor via head swing and temporal comparison; Braitenberg pairs give tropotaxis.
- **Walls:** reflect the heading too, or allow sliding along the wall.

Sources: see epic TCK-171.
