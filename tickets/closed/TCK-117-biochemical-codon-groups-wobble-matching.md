---
id: TCK-117
title: Biochemical Codon Groups, Wobble-Matching, and Dynamic Genome Length
status: closed
assigned: Gemini-CLI
created_at: 2026-08-31
completed_at: 2026-08-31
---

# TCK-117: Biochemical Codon Groups, Wobble-Matching, and Dynamic Genome Length

## Description
Refactor the DNA sequence-matching and mutation engine to resolve the evolutionary stagnation bottleneck caused by the low probability of exact 3-character promoter occurrences (e.g., "EYE" in random 26-character strings). This is achieved by dividing the A-Z alphabet into 5 functional biochemical groups (simulating codon degeneracy/wobble-matching) and introducing dynamic genome length scaling (Insertions & Deletions) driven by phenotypic rates.

## Requirements & Scope

### 1. Biochemical Codon Groups Mapping
- Map the `A-Z` alphabet into 5 distinct biological groups:
  - **Alpha (Polar/Hydrophilic):** `[A, E, I, O, U]`
  - **Beta (Aromatic/Basic):** `[Y, W, F, H, K, R]`
  - **Gamma (Hydrophobic):** `[L, M, V, P, T]`
  - **Delta (Acidic/Amide):** `[D, N, Q, S, C]`
  - **Epsilon (Inert/Rare):** `[B, G, J, X, Z]`

### 2. Degenerate Pattern-Matching
- Implement `matches_degenerate(segment: &str, classic_promoter: &str) -> bool` in `src-tauri/src/biology/dna.rs`.
- A candidate segment matches a classic promoter if and only if each of its characters belongs to the same biochemical group as the corresponding character in the classic promoter (e.g., `"OWO"` matches `"EYE"` because O/E are Alpha, W/Y are Beta, O/E are Alpha).
- Update `extract_raw_gene_payloads` to scan and partition payloads using degenerate matching for active promoters (`"EYE"`, `"NOS"`, `"TAC"`, `"LUM"`) and terminators (`"EN"`, `"SP"`).
- Update the epigenetic chromatin scan inside `parse_genome` to use degenerate matching to ensure active gene highlights correspond exactly to parsed organs.

### 3. Dynamic Genome Length Scaling (Insertion & Deletion)
- Expand `mutate_genome` to support structural mutations in addition to point substitutions:
  - Perform point mutation (substitution) as the default.
  - Integrate a small probability for **Insertion** (insert a random character at an unprotected locus) and **Deletion** (remove a character from an unprotected locus).
  - Use the creature's own calculated `insertion_rate` and `deletion_rate` to scale these probabilities, capped strictly between `128` and `512` characters.
  
### 4. Verification & Unit Testing
- Write robust unit tests verifying:
  - Correct categorization of characters into biochem classes.
  - Successful degenerate scanning of promoters (e.g., `"OWO"` yielding a vision eye patch).
  - Perfect backward compatibility: verifying that the classic progenitor string (`"EYE...EN"`) compiles identically.
  - Structural insertions and deletions respect protected gene motifs.
- Verify that both Rust tests (`npm run test:rust`) and overall workspace builds complete with zero warnings.
