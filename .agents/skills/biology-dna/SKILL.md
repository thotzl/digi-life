---
name: biology-dna
description: Rules for changing the genome compiler, CTRNN brain, sensing, metabolism, reproduction or species logic of Pixel DNA Life (src/biology/{genome,phenotype,organs,brain}/, src/shared/brain.rs).
---
# biology-dna: Genome, Brain & Evolution Rules

## I. Knowledge (read first)
- Genome compiler: `wiki/biology/genome-compiler.md`
- CTRNN brain: `wiki/biology/ctrnn-brain.md`
- Sensing: `wiki/biology/sensing.md`
- Metabolism, reproduction, species: `wiki/biology/metabolism-and-reproduction.md`

## II. Rules
- `parse_genome` must stay pure and deterministic; no `HashMap` iteration order in results (see RV-B11).
- Any change that alters phenotypes or brains changes the golden master (`test_parse_genome_golden_master`). Update `GOLDEN_FINGERPRINT` only for intended model changes and state it in the ticket.
- New promoters must be added to the protected motifs in `mutate_genome`.
- Do not draw evolutionary conclusions from Ocean generation counters while RV-B01 (TCK-143) is open.
- Plasticity (TCK-152), evaluation order (TCK-153) and motion vision (TCK-144) are decided and closed ([[wiki/biology/ctrnn-brain]], [[wiki/biology/sensing]]). The species model (TCK-148) is still a pool ticket awaiting the user; do not implement it without a decision.
