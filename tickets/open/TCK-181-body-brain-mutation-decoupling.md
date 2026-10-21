---
id: TCK-181
title: Body/Brain Mutation Decoupling
status: open
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
