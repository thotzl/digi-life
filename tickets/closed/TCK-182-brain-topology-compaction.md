---
id: TCK-182
title: Brain Topology Compaction
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-179, TCK-173
---

# TCK-182: Brain Topology Compaction

Part of epic TCK-171. Intended golden-master change, in the same window as TCK-179 and TCK-144.

## Description
The brain is wired exuberantly: every source (7 base inputs, 10 thalamus nodes per organ block, H hidden) connects to every destination (4 motors, H hidden) (`src/biology/dna.rs:1231-1382`), ≈ `(7 + 10·blocks + H) · (4 + H)` synapses: ~120 (1 block, H 2), ~250 (2 / 4), ~1,000 (4 / 12), ~1,700 (8 / 12); organ count is not capped, H ≤ 12 (`dna.rs:1181`). Every tick runs one pass over all synapses for signals and one for Hebbian learning (`src/shared/brain.rs:80-133`). A large share can never carry a signal (code-verified 2026-10-21):
1. Motors 3 (biolum) and 4 (reserved) have no effect (TCK-149) but receive all sources: 12–33 % of all synapses.
2. Silent difference nodes: for unpaired organs 5 of 10 thalamus nodes stay 0 (`dna.rs:1365-1374`) but are fully wired.
3. Channels without sensitivity: an organ responds to 1–3 of 5 channels depending on bandwidth (`brain.rs:175-180`); the other thalamus nodes stay constant but are fully wired.

Fewer, meaningful synapses help three ways: compute per tick, energy (activity-based cost of TCK-151), and heredity (fewer parameters with stable identity, TCK-179).

## Scope
- Do not wire dead structure: no synapses to no-effect motors (or remove those motors, decision in TCK-149), none from silent difference nodes, none from channels without sensitivity.
- Weight sharing for organ pairs: left/right use mirrored weights (bilateral symmetry; halves the parameters).
- Targeted instead of full connectivity: thalamus → hidden → motor plus few direct reflex paths.
- Alternative to evaluate: minimal start and growth (NEAT-style complexification: start with few connections, genes add new ones). Decide between targeted connectivity and minimal start by measurement with the TCK-173 runner.
- Out of scope: indirect encodings (HyperNEAT), too large a rebuild for this project.

## Acceptance
- Synapse count per creature, ticks per second and training success (TCK-173 report) before/after; the chosen topology variant documented with numbers.
- New golden-master fingerprint documented (together with TCK-179 / TCK-144).
- `wiki/biology/ctrnn-brain.md` describes the new topology.

Source: NEAT (minimal start, complexification): https://axon.cs.byu.edu/~dan/778/papers/NeuroEvolution/stanley2*.pdf

## Verification (P3, v1.12.17)
- Targeted connectivity: thalamus -> hidden/motor, systemic -> hidden/motor (systemic -> motor only with an `OUT` gene), hidden -> motor; no default hidden -> hidden; sensor -> thalamus reflexes fixed.
- No dead structure: motors 3/4 removed (TCK-149), thalamus nodes only for live channels, difference nodes only for HOX pairs; pairs share one weight set through sum/difference nodes.
- Synapses per random letter-soup genome p10/p50/p90 52/142/249 -> 14/41/103 (mean 150 -> 52); wildtype 249 -> 118; structured genomes p50 78 (`probe topology`).
- Throughput (seed b1, pop 16): standard 158k -> 323k ticks/s, exploration 425k -> 763k.
- Chosen variant: targeted connectivity. NEAT-style minimal start was not implemented: there was no add-connection operator before TCK-184, and the targeted wiring already removes ~70 % of synapses while keeping 97 % of genomes moving; gene duplication (TCK-184) now adds structure coherently.
- Golden master `0xa7b82dc4fe552ac8` -> `0x65f701a6937c1e71` (one intended Brain v2 change for TCK-179/182/144/153/184/149; ADR-002). CI reference `tests/reference/trainer-ci.json` regenerated. `cargo test` 94 passed (1 ignored measurement), 0 warnings; `npm run build` and `npm test` green.
