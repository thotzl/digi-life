# Next tasks

Last updated: 2026-10-22. Planned tasks and findings not yet ticketed; remove an entry once it is ticketed or becomes the active task.

## Findings (not yet ticketed)
- **Thalamus sum nodes give a constant offset when silent** (found in P3, existed in the old model as well): sum nodes use `sigmoid` with bias 0, so a silent channel outputs 0.5 and feeds a constant drive into hidden neurons and motors (`src/biology/dna.rs`, thalamus `ThalamusKind::Sum` → sigmoid). Acts like an implicit bias per live channel. Check in P5/P7 with the steering probe whether a centred activation (e.g. `tanh`, or sigmoid − 0.5) improves directed steering; it is a model change (golden master, CI reference, `MODEL_VERSION`).
