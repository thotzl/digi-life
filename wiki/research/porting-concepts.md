---
sources: [ideas.md, docs/revisit/roadmap.md, docs/revisit/04-simulation-model.md]
---

# Porting Concepts

Assessment of the four porting ideas in `ideas.md` (sections II and III, "Epochs 14–17") against the current code. Formulas and specifications stay in `ideas.md`.

**Note on age:** `ideas.md` predates the promoter-based genome (TCK-116). It refers to positional loci ("breath-rate locus 13", "locus 21", "128–384 loci", `ST ... EN` genes) and a "deep MLP"; the current model is promoter genes, length 128–512, and a CTRNN. Each idea must be re-specified against [[biology/genome-compiler]] and [[biology/ctrnn-brain]] before ticketing.

| Idea (ideas.md epoch) | Inspiration | Current state in code | Relation |
| :--- | :--- | :--- | :--- |
| Swimbot oscillators for the notochord (14) | Gene Pool | spine wave via `WAV`/`PUL` genes and `SpinalHarmonics`; thrust scaling may immobilize genomes (RV-S02) | measure RV-S02 first (TCK-150) |
| Functional chromatophores / colour-segment metabolism (15) | Biogenesis | colours cosmetic; photosynthesis is config-driven, biomes without effect | overlaps with dead biome keys (TCK-149) |
| Hormonal endocrinology: adrenaline, dopamine, satiety (16) | The Bibites | adrenaline input exists but is constant; all adrenaline keys dead (RV-S01) | natural replacement for the dead adrenaline system (TCK-149); dopamine interacts with Hebbian rule (TCK-152) |
| Relative f1 circuits in the DNA (17) | Framsticks | `SY` genes map `from`/`to` by modulo over absolute indices | depends on evaluation order and plasticity decisions (TCK-152, TCK-153) |

## Ticket status

- Per [[roadmap]] Phase 2 step 5, each idea becomes an individual ticket with a measurement criterion. Ticket proposals are pending user approval (TCK-163 report).

## Key Takeaways

- All four ideas are compatible in spirit but specified against an outdated genome model.
- Hormones are the most direct fit (dead adrenaline system); oscillators depend on the thrust measurement.

Sources: `ideas.md`, `docs/revisit/roadmap.md`, `docs/revisit/04-simulation-model.md`.
