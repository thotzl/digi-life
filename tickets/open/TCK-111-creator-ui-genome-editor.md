---
id: TCK-111
title: UI Creator Module: DNA Sequence Editor and Phenotypic Preview
status: open
assigned: unassigned
created_at: 2026-08-21
epic_phase: Roadmap TCK-169 Phase 2 (feature epics, to be planned)
---

# TCK-111: UI Creator Module: DNA Sequence Editor and Phenotypic Preview

## Update (2026-11-03, after TCK-196)
- The re-scope note above still applies: build the Creator as a view (`wiki/guides/add-view.md`) on the promoter/organ registries (`bindings/promoterRegistry.ts`, `bindings/organRegistry.ts`) and the catalogue repository; the phenotype comes from the backend (`get_fossil_phenotype`). Candidate for the Phase 2 feature epics.

## Re-scope (2026-11-02, epic TCK-196)
Feature ticket after TCK-196. Outdated assumptions (Preact components, client-side decompiler mirror, IndexedDB) contradict the backend-as-source-of-truth rule and the SQLite catalogue. Rewrite on the `ViewModule` seam (TCK-160) and the organ/promoter registry (TCK-199) before starting.


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
