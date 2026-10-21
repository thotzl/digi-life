---
sources: [README.md, docs/revisit/README.md, .agents/skills/biology-dna/SKILL.md, .agents/skills/server-simulation/SKILL.md]
---

# Glossary

- **Ocean** — open-world simulation mode with a free population, see [[simulation/ocean]].
- **Trainer** — neuroevolution mode with isolated sandboxes, see [[simulation/trainer]].
- **Sandbox / chamber** — one isolated trainer arena with one candidate and target spores.
- **Epoch** — fixed tick budget of one trainer evaluation; **trial** — one of up to 3 epochs per generation (multi-trial).
- **HoF** — Hall of Fame, best genomes of a trainer run (`trainer_genomes`).
- **Catalogue** — user-saved specimens incl. epigenetics and learned weights (`creature_catalogue`); **cryo-cloning** = saving/restoring that state.
- **Restocking** — Ocean refills its population to `targetPopulation` from catalogue clones or random genomes.
- **Spore / pellet** — food item; plant (algae) or meat (`type_id = 2`).
- **Urzelle** — progenitor cell injected via `INJECT_URZELLE`.
- **Warp** — steps per frame, capped at 35.
- **Promoter / terminator** — gene start motif (`EYE`, `NEU`, ...) and end motif (`EN`/`SP`), see [[biology/genome-compiler]].
- **Degenerate (wobble) matching** — motif match by biochemical character group.
- **Chromatin / active DNA** — positions covered by recognized genes; feed brain hashing.
- **HOX** — gene enabling bilateral symmetry.
- **Organelle** — sensory organ (eye, nose, tactile, photophore), see [[biology/sensing]].
- **Thalamus** — preprocessing layer between sensors and hidden neurons, see [[biology/ctrnn-brain]].
- **Exuberance / pruning** — fully connected brain at birth, weak synapses removed during life.
- **Lamarckian inheritance** — learned weight offsets passed to offspring.
- **BMR** — basal metabolic rate.
- **Golden master** — fingerprint test over 300 genomes guarding the compiler, see [[quality/testing]].
- **RV-ID** — review finding ID (`RV-B01`, ...), see [[quality/review-findings]].
- **Pool ticket** — ticket with `priority: pool`, waits for a user decision (TCK-143..161).
- **Phase 0/1/2** — stabilize / targeted rebuild / feature development, see [[roadmap]].

## Key Takeaways

- Terms map to articles; RV-IDs and pool tickets are the planning vocabulary.

Sources: `README.md`, `docs/revisit/README.md`, agent skills.
