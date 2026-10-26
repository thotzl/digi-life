---
sources: [README.md, docs/revisit/README.md, .agents/skills/biology-dna/SKILL.md, .agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-152-hebbian-plasticity-model.md]
---

# Glossary

- **Ocean** — open-world simulation mode with a free population, see [[wiki/simulation/ocean]].
- **Trainer** — neuroevolution mode with isolated sandboxes, see [[wiki/simulation/trainer]].
- **Sandbox / chamber** — one isolated trainer arena with one candidate and target spores.
- **Epoch** — fixed tick budget of one trainer evaluation; **trial** — one of up to 3 epochs per generation (multi-trial).
- **HoF** — Hall of Fame, best genomes of a trainer run (`trainer_genomes`).
- **Catalogue** — user-saved specimens incl. epigenetics and learned weights (`creature_catalogue`); **cryo-cloning** = saving/restoring that state.
- **Restocking** — Ocean refills its population to `targetPopulation` from catalogue clones or structured random founders.
- **Spore / pellet** — food item; plant (algae) or meat (`type_id = 2`).
- **Urzelle** — progenitor cell injected via `INJECT_URZELLE`.
- **Warp** — steps per frame, capped at 35.
- **Promoter / terminator** — gene start motif (`EYE`, `NEU`, ...) and end motif (`EN`), see [[wiki/biology/genome-compiler]].
- **Gene identity (`GeneRef`)** — promoter, exact promoter spelling and occurrence; stable under letter mutations and unrelated gene operators.
- **Degenerate (wobble) matching** — motif match by biochemical character group.
- **Chromatin** — positions covered by recognized genes (helix view only; `active_dna` was removed with Brain v2).
- **HOX** — gene enabling bilateral symmetry.
- **Organelle** — sensory organ; `OrganKind` eye, nose, thermal, bristle, lateral line, proprioceptor, see [[wiki/biology/sensing]].
- **ΔI** — signed temporal intensity change, 6th input slot of distance senses (motion vision, klinotaxis).
- **Thalamus** — preprocessing layer between sensors and hidden neurons, see [[wiki/biology/ctrnn-brain]].
- **Pruning** — weak synapses (`|w| < 0.015`) frozen at 0 during life. The former exuberance (fully connected brain at birth) was replaced by targeted wiring (TCK-182).
- **Synapse key / owner / weight tape** — role-based stable synapse identity, the one gene that defines a synapse, and that gene's payload letters as initial weights, see [[wiki/biology/ctrnn-brain]].
- **Structured genome / letter soup** — random genome built from valid genes (`generate_structured_genome`) vs. uniformly random letters.
- **Creep mutation** — a letter moved 1–2 alphabet steps (small value change).
- **Lamarckian inheritance** — learned weight offsets passed to offspring, matched by synapse key, per plastic synapse with probability `lamarckianAssimilationChance` (TCK-152).
- **Energy cost** — per-tick drain of the shared energy function (`metabolism::tick_cost`): body, senses, brain (neurons + synaptic activity), movement, see [[wiki/biology/metabolism-and-reproduction]]. Replaced the multiplicative BMR (basal metabolic rate) in P6 (TCK-151).
- **Starved** — trainer state of a creature at 0 energy: frozen for the rest of the trial (TCK-178), see [[wiki/simulation/trainer]].
- **Golden master** — fingerprint test over 300 genomes guarding the compiler, see [[wiki/quality/testing]].
- **Model version** — `MODEL_VERSION`; a bump resets the model tables of the DB, see [[wiki/architecture/persistence]].
- **RV-ID** — review finding ID from the review of 2026-10-19 (`RV-B01`, ...; B bugs, A architecture, P performance, S simulation, D database, F frontend, T test/CI, X docs). Cited in articles as the source of a known limitation or fix.

## Key Takeaways

- Terms map to articles; RV-IDs mark known limitations and fixes.

Sources: `README.md`, `docs/revisit/README.md`, agent skills.
