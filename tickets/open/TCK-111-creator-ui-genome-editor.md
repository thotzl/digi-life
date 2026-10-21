---
id: TCK-111
title: UI Creator Module: DNA Sequence Editor and Phenotypic Preview
status: open
assigned: Gemini-CLI
created_at: 2026-08-21
---

# TCK-111: UI Creator Module: DNA Sequence Editor and Phenotypic Preview

## Description
Develop the client-side DNA sequence workbench in the Preact-based frontend. This module allows users to generate, modify, and validate double-stranded genomic sequences, producing structured profiles that can be saved locally and sent to the simulation core.

## Requirements & Scope
- **Double-Stranded DNA Editor Interface:**
  - Implement a visual editor displaying the parallel **Sense** and **Antisense** strands (A-Z characters).
  - Support manual base-by-base editing with inline validation (ensuring compliance with structural bounds, length restrictions, and character constraints).
- **Real-Time De-compilation Preview:**
  - Build a visual parser that mirrors the Rust de-compiler. It must display real-time physical previews (HSL colors, body segment count, sensor coordinates) and behavioral metrics (Carnivory Index) as the user edits the character sequence.
- **Genomic Manipulation Tools:**
  - Add interactive sliders to configure point mutations, duplications, and sequence deletions.
  - Implement a randomized generator producing viable starter genomes.
- **Local Catalogue Storage:**
  - Implement client-side storage (IndexedDB) to save custom engineered genomes with custom name tags and species classifications.
