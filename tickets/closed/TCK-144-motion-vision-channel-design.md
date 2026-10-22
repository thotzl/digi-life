---
id: TCK-144
title: Motion Vision Channel Design
status: closed
priority: high
decision: accepted
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B06
epic_phase: Training Track
depends_on: TCK-179
---

# TCK-144: Motion Vision Channel Design

> Pool ticket: requires a user decision before implementation. Findings (RV-B06) below.
>
> 2026-10-21: candidate steering fix of the training track (TCK-172). A dedicated ΔI channel is a klinotaxis input: C. elegans steers with a single sensor via head swing and temporal comparison, so single organs could steer without a `HOX` pair.
>
> Decision 2026-10-21 (default from the session's recommendation, user may veto): dedicated 6th channel per eye/nose for ΔI (meat keeps channel 5); organ type as an enum instead of emoji label matching (TCK-155 subset). Adding a channel shifts neuron ids, so implement after TCK-179 (stable synapse identity) and within the same golden-master change. Option, if the TCK-172 steering probe shows single organs still cannot steer: organ pairs without a `HOX` gene.

## Description
Motion vision overwrites channel 5 of eyes/noses, which carries the main meat (IR) signal. Organ type is detected via emoji labels.

## Proposed Scope
Decide: dedicated 6th channel vs. separate neuron; organ type as enum.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B06 · P1 · static** — `src/shared/brain.rs`: for eyes/noses channel 5 (`base_idx + 4`) is overwritten every tick with the motion delta ΔI. Meat emits exactly on channel 5 (IR, weight 1.0), so meat reaches the brain only via channel 4 (weight 0.5): carnivore sensing is systematically weakened. The logic is coupled to emoji labels (`label.contains("👁️")`).
- Decision needed: motion vision as its own 6th channel or as its own neuron type. Organ type as an enum in the phenotype instead of label string matching (see TCK-155).

## Verification (P3, v1.12.17)
- Dedicated 6th channel per organ for the signed temporal change `clamp(2.5 * (I_now - I_prev), -1, 1)` (I = sum of the 5 spectral inputs); channel 5 (meat/IR) is no longer overwritten (RV-B06 resolved). Applies to eyes, noses and thermal organs.
- `OrganKind` enum (Eye, Nose, Thermal, Bristle, LateralLine, Proprioceptor) in the phenotype; no emoji label matching in Rust or in the inspector (labels are display-only).
- Test `test_delta_channel_and_one_tick_latency`.
- Steering probe (food at +-45 deg): still undirected after P3 (structured genomes: toward 0.090, away 0.100); directed steering is a training outcome measured again in P5/P7 against the TCK-171 target.
- Golden master `0xa7b82dc4fe552ac8` -> `0x65f701a6937c1e71` (one intended Brain v2 change for TCK-179/182/144/153/184/149; ADR-002). CI reference `tests/reference/trainer-ci.json` regenerated. `cargo test` 94 passed (1 ignored measurement), 0 warnings; `npm run build` and `npm test` green.
