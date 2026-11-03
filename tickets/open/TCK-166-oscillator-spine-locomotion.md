---
id: TCK-166
title: Oscillator Spine Locomotion
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-21
epic_phase: Roadmap TCK-169 Phase 2 (feature epics, to be planned)
---

# TCK-166: Oscillator Spine Locomotion

## Update (2026-11-03, after TCK-196)
- Seams since TCK-196: body traits are decoded from a table (`BODY_GENES`, `src/biology/phenotype/body.rs`), locomotion runs in the shared creature tick (`src/sim/systems/`, `wiki/contracts/creature-system.md`), physics in `src/shared/physics.rs` (only limb organs drive locomotion since TCK-202). Re-scope on these seams before starting; golden master / `MODEL_VERSION` rules apply.

> Pool ticket from `ideas.md` (section II.1, epoch 14). Context: `wiki/research/porting-concepts.md`.

## Description
Gene Pool-style phase-shifted sine oscillators along the spine (amplitude tapering head to tail, genetic frequency and wavelength) to produce inheritable swimming patterns, with thrust tied to wave synchronization.

## Proposed Scope
- Depends on the thrust measurement of TCK-150 (RV-S02): only change the locomotion model once the speed distribution over random genomes is known.
- Map frequency/wavelength to the existing `WAV`/`PUL` genes instead of the positional loci named in ideas.md.

## Measurement Criterion
- Terminal speed distribution over 1,000 random genomes before/after; share of immobile genomes.
