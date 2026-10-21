---
id: TCK-166
title: Oscillator Spine Locomotion
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
---

# TCK-166: Oscillator Spine Locomotion

> Pool ticket from `ideas.md` (section II.1, epoch 14). Context: `wiki/research/porting-concepts.md`.

## Description
Gene Pool-style phase-shifted sine oscillators along the spine (amplitude tapering head to tail, genetic frequency and wavelength) to produce inheritable swimming patterns, with thrust tied to wave synchronization.

## Proposed Scope
- Depends on the thrust measurement of TCK-150 (RV-S02): only change the locomotion model once the speed distribution over random genomes is known.
- Map frequency/wavelength to the existing `WAV`/`PUL` genes instead of the positional loci named in ideas.md.

## Measurement Criterion
- Terminal speed distribution over 1,000 random genomes before/after; share of immobile genomes.
