---
sources: [ideas.md]
---

# Alife Reference Simulators

Summary of the four simulators analysed in `ideas.md` (section I). Full text, formulas and details stay in `ideas.md`; this page only says what is relevant for Pixel DNA Life.

| Simulator | Core idea | Relevance here |
| :--- | :--- | :--- |
| **The Bibites** | Float-encoded traits (size, diet, mutation rate/variance) + growing rt-NEAT brain; BIOME: genes as permanent brain inputs, brain outputs alter epigenetics | Closest model; source of the hormone idea; our genes already feed brain parameters via chromatin hashing ([[biology/genome-compiler]]) |
| **Gene Pool** (Ventrella, "Swimbots") | Segmented bodies, each joint a phase-shifted sine oscillator; non-reciprocal motion (scallop theorem); sexual selection via mating preferences | Inspiration for genetically inherited swimming patterns ([[simulation/physics]]); mate choice not modelled |
| **Framsticks** | Tree-like `f1` genotype for rods + neurons; relative synapse addressing survives crossover; joints are the muscles; gyroscope/touch/smell sensors | Relative wiring could make brain circuits robust to indels ([[biology/ctrnn-brain]]) |
| **Biogenesis** | Colour-coded segments with metabolic function (green photosynthesis, red teeth, cyan flagella, yellow light sensor, blue defence, white mitosis bud) | Inspiration for functional tissue colours; current colours are cosmetic |

## Key Takeaways

- The Bibites is the reference for gene–brain coupling; Gene Pool for locomotion; Framsticks for mutation-robust wiring; Biogenesis for functional colours.
- Details: `ideas.md` section I. Applicability: [[research/porting-concepts]].

Sources: `ideas.md`.
