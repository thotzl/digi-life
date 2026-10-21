---
id: TCK-165
title: Hormonal Endocrinology (Adrenaline, Dopamine, Satiety)
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
---

# TCK-165: Hormonal Endocrinology (Adrenaline, Dopamine, Satiety)

> Pool ticket from `ideas.md` (section II.4, epoch 16). Context: `wiki/research/porting-concepts.md`.

## Description
Adrenaline is a constant today: all adrenaline config keys are dead and the brain input is always 0 (RV-S01). The Bibites-style hormone model would give it a function: adrenaline (predator near → more thrust, higher BMR), dopamine (feeding → temporarily higher Hebbian rate), satiety (near stomach capacity → damped feeding channels).

## Proposed Scope
- Re-specify against the current promoter genome and CTRNN (ideas.md uses positional loci and an MLP).
- Decide per adrenaline key together with TCK-149; align dopamine with the plasticity rule of TCK-152.

## Measurement Criterion
- Headless runs (TCK-158): flee speed of prey near predators and Hebbian weight distribution with vs. without hormones.
