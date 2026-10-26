---
sources: [ideas.md, docs/revisit/04-simulation-model.md, tickets/closed/TCK-149-dead-config-keys.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md]
---

# Porting Concepts

Assessment of the four porting ideas in `ideas.md` (sections II and III, "Epochs 14–17") against the current code. Formulas and specifications stay in `ideas.md`.

**Note on age:** `ideas.md` predates the promoter-based genome (TCK-116). It refers to positional loci ("breath-rate locus 13", "locus 21", "128–384 loci", `ST ... EN` genes) and a "deep MLP"; the current model is promoter genes, length 128–512, and a CTRNN. Each idea needs re-specification against [[wiki/biology/genome-compiler]] and [[wiki/biology/ctrnn-brain]].

| Idea (ideas.md epoch) | Inspiration | Current state in code | Fit |
| :--- | :--- | :--- | :--- |
| Swimbot oscillators for the notochord (14) | Gene Pool | spine wave via `WAV`/`PUL` genes and `SpinalHarmonics`; thrust comes from a per-genome speed band (1–5 px/tick), `wave_phase` and body shape do not affect top speed (TCK-150) | would need thrust derived from the wave instead of the speed band |
| Functional chromatophores / colour-segment metabolism (15) | Biogenesis | colours cosmetic; photosynthesis is a flat config value, biomes without effect, biome keys removed (TCK-149) | would give biomes/light a function |
| Hormonal endocrinology: adrenaline, dopamine, satiety (16) | The Bibites | no hormone exists: the inert adrenaline input, agent field and config keys were removed (TCK-149) | a hormone would be a new systemic input (`SystemicInput`); dopamine interacts with the Hebbian rule |
| Relative f1 circuits in the DNA (17) | Framsticks | `SY` genes address neuron roles modulo the size of their role class; synapse keys and owner genes are stable (TCK-179) | interacts with plasticity (Oja rule, fixed reflexes, TCK-152) |

## Key Takeaways

- All four ideas are compatible in spirit but specified against an outdated genome model.
- Hormones would be new systemic inputs; oscillators depend on the thrust behaviour.

Sources: `ideas.md`, `docs/revisit/04-simulation-model.md`, TCK-150.
