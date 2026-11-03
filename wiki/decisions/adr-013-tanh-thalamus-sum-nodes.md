---
sources: [tickets/closed/TCK-190-thalamus-sum-silent-offset.md, tickets/closed/TCK-171-training-track-epic.md, wiki/biology/ctrnn-brain.md]
---

# ADR-013: tanh Activation for Thalamus Sum Nodes

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-26
- Tickets: TCK-190; epic TCK-171 wrap-up (v1.12.24)

## Context

Thalamus sum nodes used `sigmoid` with bias 0. Sensor inputs are non-negative, so a silent channel output 0.5 and fed a constant drive into hidden neurons and motors — an implicit bias per live channel (finding from P3). The ticket required adopting tanh only with a measurable benefit. The follow-up was chosen by the user after the epic report (TCK-171 wrap-up).

## Decision

Sum nodes use **tanh** (`thalamusSumActivation` "tanh", also the enum default): a silent channel outputs 0, active values lie in [0, 1). The sigmoid option stays configurable.

## Alternatives considered

Measured with the same runner settings (pop 16, 8 trials, rotate, 1/5 rule, minimal criterion, 10 unseen layouts per room, seeds b1..b10), TCK-190:

| Activation | small direct, 30 gens: unseen small | curriculum 60 gens: small | curriculum: large | final level |
| :--- | :--- | :--- | :--- | :--- |
| sigmoid (before) | 0.266 | 0.281 | 0.096 | mean 0.61 |
| tanh (chosen) | 0.559 | 0.394 | 0.184 | mean 0.85 |

Small room +0.29 (≈ 2.5 SE), large room with curriculum +0.09 (≈ 1.7 SE). Steering probe (food at ±45°): turn toward food letter soup 6.6 % → 10.6 %, structured 17.6 % → 19.6 %.

## Consequences

- Model change: `MODEL_VERSION` 6, golden master `0x9d206dfd8124f808` → `0x0ab34073297ace74`, CI reference regenerated.
- Median trial activity of structured brains dropped 9.606 → 4.989, so the energy calibration of ADR-011 no longer held; the metabolism constants were re-derived (typical total cost median 0.0243, brain + senses share 15.5 %).
- Current brain: [[wiki/biology/ctrnn-brain]].
