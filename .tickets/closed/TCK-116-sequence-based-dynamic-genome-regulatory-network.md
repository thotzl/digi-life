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

### 5. Parameterized Universal Gene Parser Helper (Neutral Raw Signal Extraction)
*   To prevent code duplication and enforce complete decoupling, establish exactly **one** unified helper method inside the Rust biology engine.
*   **The Helper Design:** The helper method does not need to know any phenotypic or neural domain logic. It simply scans the continuous genome for promoter (`start_motif`) and terminator (`stop_motif`) markers, extracts any active gene segments, hashes them, and returns an array of neutral, fully normalized float signals bounded strictly between `0.0` and `1.0`:
    ```rust
    // Der Helper extrahiert nur rohe, neutrale Gendaten als normalisierte Fließkommazahlen [0.0..1.0]
    pub fn extract_raw_gene_signals(
        genome: &str,
        start_motif: &str,
        stop_motif: &str
    ) -> Vec<f32>; // Liefert ein neutrales Array von normalisierten Werten (z. B. [0.12, 0.85, 0.44])
    ```
*   **Separation of Concerns:** How these raw signals are interpreted, scaled, or chunked lies completely within the calling domain modules (the color module, the stiffness module, the sensory organelle module, or the brain compiler).
*   **Examples of Modular Interpretation:**

    #### 1. Das Steifheits-Modul (Nutzung als einzelner Skalar):
    ```rust
    // Das Modul will nur einen Wert und skaliert ihn selbst auf seinen Bereich [0.15..1.0]
    let raw_signals = extract_raw_gene_signals(genome, "STF", "EN");
    let stiffness = if !raw_signals.is_empty() {
        0.15 + raw_signals[0] * (1.0 - 0.15)
    } else {
        0.50 // Fallback
    };
    ```

    #### 2. Das Farbpigment-Modul (Nutzung als Farb-Koordinaten):
    ```rust
    // Das Modul fordert rohe Signale an und mappt sie autonom auf H, S und L
    let raw_signals = extract_raw_gene_signals(genome, "COL", "STP");
    let (h, s, l) = if raw_signals.len() >= 3 {
        (
            raw_signals[0] * 360.0, // Farbton
            55.0 + raw_signals[1] * 45.0, // Sättigung
            35.0 + raw_signals[2] * 45.0, // Helligkeit
        )
    } else {
        (130.0, 75.0, 45.0) // Fallback-Grün
    };
    ```

    #### 3. Das Augen-Modul (Nutzung eines variablen Signal-Arrays):
    ```rust
    // Ein komplexes Gen liefert ein langes Array. Das Modul liest die Werte paarweise aus:
    let raw_signals = extract_raw_gene_signals(genome, "EYE", "EN");
    let mut eyes = Vec::new();
    for chunk in raw_signals.chunks_exact(2) {
        eyes.push(SensoryPatch {
            angle: chunk[0] * std::f32::consts::TAU, // Winkel
            visus_range: 40.0 + chunk[1] * 140.0,    // Sichtweite
        });
    }
    ```

*   **Universelle Reichweite (Wichtigste Mandate):** Die obigen drei Code-Beispiele sind **rein illustrative Szenarien**. Es ist im Sinne eines kompromisslosen Single Point of Truth (SPOT) festgelegt, dass **jede einzelne von der DNA beeinflusste Eigenschaft (Form, Sinne, Stoffwechsel, Pigmentierung)** und insbesondere das **gesamte CTRNN-Gehirn (alle neuronalen Synapsen, Kopplungsgewichte, Decay-Zeitkonstanten $\tau$, Schwellenwerte/Biases $\theta$ und heterogenen Aktivierungsfunktionen)** vollständig so umstrukturiert werden, dass sie ihre Roh-Werte nativ und einheitlich über diesen entkoppelten `extract_raw_gene_signals`-Helper beziehen! Es wird keinerlei Ausnahmen, hartcodierte Slices oder separate Parser mehr geben.

---

## 🛠️ Planned Implementation Steps

1.  **Deconstruct Positional Slices:** Remove all hardcoded index lookups (`get_methylated_val(X)`) inside `parse_genome` in `src-tauri/src/biology/dna.rs`.
2.  **Scaffold Pattern-Scanners:** Write a lightweight sequence scanner in Rust that slides across the strand to detect promoter, terminator, and operator motifs.
3.  **Implement Sequence-Hashing & Codon Frequencies:** Map physical traits to mathematical properties of the sequence segments rather than absolute coordinate letters.
4.  **Align TypeScript Types:** Generate updated phænotype and epigenetic interfaces via `ts-rs` to keep the front-end DNA Helix view perfectly synchronized with the new dynamic active spans.
5.  **Re-balance Trainer Sandbox Scenarios:** Verify that sandbox candidates in the Reinforcement Learning Trainer can evolve coordinated swimming trajectories under the new sequence-based genetics.
