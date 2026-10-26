---
id: TCK-190
title: Thalamus Sum Nodes Give a Constant Offset When Silent
status: closed
priority: medium
assigned: agent
created_at: 2026-10-26
epic_phase: Training Track (wrap-up)
depends_on: TCK-171
---

# TCK-190: Thalamus Sum Nodes Give a Constant Offset When Silent

Wrap-up of epic TCK-171 (finding from P3, planned for P5/P7, not checked there; `.agents/state/next-tasks.md`).

## Description
Thalamus sum nodes use `sigmoid` with bias 0 (`src/biology/dna.rs`, `ThalamusKind::Sum`). Sensor inputs are non-negative, so a silent channel outputs 0.5 and feeds a constant drive into hidden neurons and motors — an implicit bias per live channel.

## Scope
- Make the activation configurable (`thalamusSumActivation`: "sigmoid" | "tanh"; tanh keeps a silent channel at 0, active values in [0, 1)).
- Measure tanh vs. sigmoid with the steering probe and training on unseen layouts (small room direct, curriculum); adopt tanh only with a measurable benefit (model change: golden master, CI reference, `MODEL_VERSION`).

## Acceptance
- Decision with numbers; if adopted: golden master, CI reference and `MODEL_VERSION` updated, wiki (ctrnn-brain) updated.

## Verification (2026-10-26)
Implementation: config `thalamusSumActivation` ("sigmoid" | "tanh", `ThalamusSumActivation` in `src/shared/types.rs`), used by the compiler for the sum nodes (`src/biology/dna.rs`, `ThalamusKind::Sum`). Measured with a config copy set to tanh against the existing sigmoid baselines (same runner settings: pop 16, 8 trials, layout policy rotate, 1/5 rule, minimal criterion; final population on 10 unseen layouts per room; seeds b1..b10; raw data `.agents/artifacts/tck190/`):

| Activation | small room direct, 30 gens: unseen small | curriculum 60 gens: unseen small | curriculum: unseen large | curriculum final level |
| :--- | :--- | :--- | :--- | :--- |
| sigmoid (before) | 0.266 (SD 0.261; elites 0.355) | 0.281 | 0.096 (SD 0.069) | mean 0.61, 2 of 10 at 1.0 |
| tanh | 0.559 (SD 0.255; elites 0.675) | 0.394 | 0.184 (SD 0.151) | mean 0.85, 4 of 10 at 1.0 |

Small room +0.29 (≈ 2.5 standard errors), large room with curriculum +0.09 (≈ 1.7 SE). Steering probe (random genomes, food at ±45°): directional turn toward food letter soup 6.6 % → 10.6 %, structured 17.6 % → 19.6 %. Decision: **tanh adopted** (model change).

## Adoption (2026-10-26)
- Config default `thalamusSumActivation` "tanh" (enum default too); `MODEL_VERSION` 6 (`src/database.rs`), golden master `0x9d206dfd8124f808` → `0x0ab34073297ace74`, CI reference regenerated (parallel == sequential).
- Consequence: the median trial activity of structured brains drops from 9.606 to 4.989 (silent channels add no signal), so the P6 energy calibration (`test_typical_total_cost_matches_the_old_ocean_drain`) no longer held (typical total cost 0.0231 instead of 0.0243). The metabolism constants were re-derived with the documented procedure (`src/biology/metabolism.rs` module docs, probe `metabolism --seed tck151`): step-1 constants from the share targets with the new activity, scale factor 0.0243 / 0.038598 = 0.6296 → `bodyCostPerMass` 2.241e-5, `organCostPerLoad` 6.575e-4, `neuronCost` 1.639e-5, `activityCost` 1.84e-4, `movementCostPerMass` 2.636e-5. Probe after the change: typical total cost median 0.0243, resting 0.0157, brain + senses share 15.5 % (band 10–20 %).
- The tanh/sigmoid comparison above ran with the P6 constants for both variants; the final numbers with the re-derived constants are measured in the epic wrap-up (TCK-171 `## Result`).
- Tests: `cargo test` 175 passed (golden master, metabolism calibration with the new typical activity/|thrust|).
