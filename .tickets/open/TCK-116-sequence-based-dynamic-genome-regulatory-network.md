# TCK-116: Sequence-Based Dynamic Genome & Gene Regulatory Network (GRN)

## 🎯 Goal
Refactor the biological DNA parsing engine inside the native Rust core to completely transition from a coordinate-mapped genome (rigid positional indices like Locus 1, Locus 27-38) to a dynamic, pattern-matching **Gene Regulatory Network (GRN)** model. The genome should act like a real biochemical strand, where active traits and properties are discovered dynamically via sliding reading frames, sequence motifs, and regulatory cascade modifiers.

---

## 🔬 Architectural Concept & Heuristics

### 1. Dynamic Gene Discovery (No Hardcoded Loci)
*   **Promoter-Terminator Scans:** Instead of hardcoded slices, genes are scanned dynamically. A gene starts when a specific promoter sequence motif (e.g., `"ST"` or `"GO"`) is encountered and ends at a terminator motif (e.g., `"SP"` or `"EN"`).
*   **Overlapping Reading Frames:** Genes can sit anywhere on the strand, can be of variable length, and can even overlap, allowing for highly complex genetic compaction.

### 2. Genetic Properties as Sequence Features (No Positional Lock-in)
*   **Continuous Sequence Properties:** Traits like color (Hue, Saturation, Lightness), body size, or metabolism should not depend on a single hardcoded character at index 1. Instead, they should be calculated generically using global or gene-local sequence properties, such as:
    *   **Nucleotide Frequency / GC-Content:** The ratio of specific nucleotide groups determines physical properties (e.g., higher concentration of `'A', 'E', 'I', 'O', 'U'` increase wiggling amplitude).
    *   **Strand Hashing:** Running a lightweight, deterministic Fowler-Noll-Vo (FNV) or Murmur hash over specific functional segments to derive high-entropy traits like body seeds and colors.
*   **Dynamic Colors & Modifiers:** Color should be determined by a combination of a base sequence and a dynamically discovered *modifier gene*. If a modifier gene (e.g., a "pigment factor" motif) is active, it shifts the Hue or Saturation multiplier, allowing color patterns to evolve organically.

### 3. Organelle Diversification (Unbalancing Olfaction)
*   **Pattern-Matching Organelle Types:** Organelle sensory patches currently lean heavily toward olfactory receptors because their positional codons are too easily triggered.
*   **Signature Codons:** Re-balance organelle expressions by using unique, high-entropy signature codons. A sensory patch is only expressed if its exact promoter sequence is matched. The specific sequence *within* that frame determines whether it functions as:
    *   `Eye (Visus)`
    *   `Nose (Olfaction)`
    *   `Tentacle (Tactility)`
    *   `Photophore (Biolum)`
*   This ensures natural organelle rarity, making highly sensory predators or tactile foragers an evolved marvel.

### 4. Gene Regulatory Network (GRN) Cascades
*   Implement a 2-stage transcription model:
    1.  **Primary Transcription:** Parse all active genes to produce transcription factors (proteins).
    2.  **Regulatory Binding:** These factors act as promoters or repressors that bind to downstream operator regions on the same or complementary strands, dynamically scaling up/down motor thrust, steering sensitivity, or metabolic rates.
*   *The DNA literally decides what the DNA determines!*

---

## 🛠️ Planned Implementation Steps

1.  **Deconstruct Positional Slices:** Remove all hardcoded index lookups (`get_methylated_val(X)`) inside `parse_genome` in `src-tauri/src/biology/dna.rs`.
2.  **Scaffold Pattern-Scanners:** Write a lightweight sequence scanner in Rust that slides across the strand to detect promoter, terminator, and operator motifs.
3.  **Implement Sequence-Hashing & Codon Frequencies:** Map physical traits to mathematical properties of the sequence segments rather than absolute coordinate letters.
4.  **Align TypeScript Types:** Generate updated phænotype and epigenetic interfaces via `ts-rs` to keep the front-end DNA Helix view perfectly synchronized with the new dynamic active spans.
5.  **Re-balance Trainer Sandbox Scenarios:** Verify that sandbox candidates in the Reinforcement Learning Trainer can evolve coordinated swimming trajectories under the new sequence-based genetics.
