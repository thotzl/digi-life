---
name: biology-dna
description: Pointers for changing the BroodBattle creature model - genome compiler, organs, CTRNN brain, sensing, metabolism, reproduction, species (src/biology/, src/shared/brain.rs).
---
# biology-dna: Creature Model Pointers

## I. Read first
- New organ kind: `.agents/skills/plugin-authoring/SKILL.md` (guide + contract `wiki/contracts/organ-kind.md`).
- Rules: `wiki/architecture/invariants.md` 8 (golden master), 9 (`MODEL_VERSION`), 16 (pure, deterministic).
- Current model: `wiki/biology/genome-compiler.md`, `wiki/biology/ctrnn-brain.md`, `wiki/biology/sensing.md`,
  `wiki/biology/metabolism-and-reproduction.md`; measured history in `wiki/history/_index.md`.

## II. Task rules
- A phenotype or brain change shows up in `test_parse_genome_golden_master`; state an intended change in the ticket.
- New promoters go into the promoter registry (`src/biology/genome/promoters.rs`); mutation protects every registered
  motif automatically.
- Do not draw evolutionary conclusions from Ocean generation counters while RV-B01 (TCK-143) is open.
- The species model (TCK-148) awaits a user decision; do not implement it without one.
