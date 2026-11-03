---
id: TCK-118
title: Double-Stranded Diploid Genetics (Dominant vs. Recessive Alleles)
status: open
assigned: unassigned
created_at: 2026-08-31
epic_phase: Roadmap TCK-169 Phase 2 (feature epics, to be planned)
---

# TCK-118: Double-Stranded Diploid Genetics (Dominant vs. Recessive Alleles)

## Update (2026-11-03, after TCK-196)
- The genome is now split into modules (`src/biology/genome/`: alphabet, promoter registry, scan, gene table, payload reader, generator, mutation) and the phenotype compiler (`src/biology/phenotype/`). Diploid genetics would touch the scan/table (two strands) and the mutation operators; it changes the golden master and needs a `MODEL_VERSION` bump with a migration decision for stored genomes (invariants 8, 9; backup before a model reset exists since TCK-195). Re-scope before starting.

## Description
Unlock the full potential of the existing double-stranded genome representation (Sense and Antisense strands). This module transitions the genetic compiler from a single-strand interpretation to a true diploid-inspired system, where traits are calculated by comparing alleles on both strands, implementing dominance hierarchies and DNA mismatch repair mechanisms.

## Requirements & Scope

### 1. Dominance & Recessiveness Rules
- Establish a dominance hierarchy among the 5 biochemical groups (e.g., `Beta > Alpha > Gamma > Delta > Epsilon`).
- For global physical traits (Muscle Stiffness, Body Sizes, Color Pigmentation), calculate values for both strands independently. Use the dominance hierarchy of corresponding loci to determine which allele is expressed in the final phenotype, leaving the other as a silent recessive gene.

### 2. Dual-Strand Organ Transcription
- Scan both the Sense and Antisense strands for specialized organ promoters (`"EYE"`, `"NOS"`, etc.).
- **Homozygous Expression:** If the organ is present at matching physical relative positions on both strands, express it at full capacity (e.g., maximum scale and sensitivity).
- **Heterozygous Expression:** If the organ is only present on one strand, express it with a 50% scale/efficiency penalty, representing recessive or incomplete dominance.

### 3. DNA Mismatch Repair Mechanism
- Update the Mitosis phase in the simulation engine to allow single-strand mutations.
- Implement a **Mismatch Repair Step** driven by `repair_fidelity`:
  - With high fidelity, repair enzymes use the undamaged complementary strand to revert the mutated single-strand character back to its original paired base.
  - With low fidelity, the mismatch is assimilated, copying the mutation to the second strand, making it a stable homozygous mutation.

### 4. Verification & Testing
- Write unit tests verifying:
  - Dominance hierarchy logic (dominant alleles correctly overriding recessive ones).
  - Silent recessive traits remain intact in the genome and can re-emerge upon mutation.
  - Mismatch repair successfully heals single-strand mutations based on fidelity.
