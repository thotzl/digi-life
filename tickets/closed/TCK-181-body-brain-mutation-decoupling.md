---
id: TCK-181
title: Body/Brain Mutation Decoupling
status: closed
priority: medium
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-179, TCK-174
---

# TCK-181: Body/Brain Mutation Decoupling

Part of epic TCK-171.

## Description
Body changes usually break a controller that was adapted to the old body (Cheney et al. 2018). In this project body traits dominate fitness (speed spread ≈ ×60, TCK-150) and a single mutation can change body and brain at once (TCK-179). Without protection, a body change that would pay off after the brain readapts is selected out immediately.

## Scope
- Separate mutation channels: per child either a body mutation or a brain mutation (configurable ratio), so the effect of each is attributable.
- Morphological innovation protection as an option: individuals with a recent body mutation get reduced selection pressure for N generations (e.g. compete only within their age group). Read the critical follow-up (2025, "Evolutionary Brain-Body Co-Optimization Consistently Fails to Select for Morphological Potential") before implementing.
- Report metrics: share of body vs. brain mutations among elites, survival of body-mutated lineages.

## Acceptance
- Headless before/after: success rate and diversity of body traits among elites; decision on keeping the protection documented with numbers.

Sources: Cheney et al. 2018 https://meclab.w3.uvm.edu/papers/2018_RoySoc_Cheney.pdf ; follow-up https://arxiv.org/pdf/2508.17464

## Verification (P7, 2026-10-26) — closed as not needed
Measured whether body mutations destroy learned behaviour; the separation is not implemented as default.
- Instrumentation: `dna::promoter_channel` / `classify_mutation` (compiler gene table; brain = `NEU`, `SY`, `OUT`; body = trait genes `COL SIZ STF WAV PUL STM TEM CAR REP EVO`, `HOX`, organ genes `EYE NOS TAC LUM`; outside genes = neutral; organ genes also feed thalamus→motor weights and are counted separately as `organ_mutants`). Runner report per generation: `mutation_channels` (mutants, `beat_parent_share`, `elite_share` per body/brain/both/neutral) and `elite_last_mutation`. Option `mutationChannels` "mixed" (default) | "separate" (each mutant draws body or brain 50/50 from its breed stream and mutates only that channel), runner flag `--mutation-channels`. "mixed" reproduces the previous results exactly; tests for classification, single-channel mutation and determinism.
- Channel statistics (mixed, pop 16, 30 gens, seeds b1..b10; beat parent / reached elite):

| room | body only | brain only | both |
| :--- | :--- | :--- | :--- |
| small | 4.9 % / 11.4 % (n 1346) | 11.3 % / 11.5 % (n 451) | 15.5 % / 14.4 % (n 464) |
| large | 3.9 % / 10.1 % (n 1289) | 7.4 % / 9.0 % (n 458) | 17.3 % / 13.2 % (n 515) |

Body-only mutants beat their parent less often but reach the elite as often as brain-only mutants; the most frequent last mutation of elites is a body mutation (small 672 of 1200, large 612 of 1200).
- Separate vs. mixed channels (success, last 5 gens, 10 seeds): small 0.415 vs. 0.438 (−0.4 SE), large 0.335 vs. 0.283 (+1.2 SE): no clear benefit.
- Decision: no decoupling; keep the instrumentation and the "separate" option for the active body traits planned after the epic. Not configurable yet: the body/brain ratio of "separate" (fixed 50/50, `SEPARATE_BODY_SHARE`).
