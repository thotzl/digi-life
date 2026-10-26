---
sources: [README.md, docs/revisit/README.md, .agents/skills/biology-dna/SKILL.md, .agents/skills/server-simulation/SKILL.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md]
---

# Glossary

- **Ocean** — open-world simulation mode with a free population, see [[wiki/simulation/ocean]].
- **Trainer** — neuroevolution mode with isolated sandboxes, see [[wiki/simulation/trainer]].
- **Sandbox / chamber** — one isolated trainer arena with one candidate and target spores.
- **Epoch** — fixed tick budget of one trainer evaluation; **trial** — one of the `trainerTrials` (8 since P8) epochs per generation (multi-trial).
- **Layout** — shared trainer task of one trial (start heading, food positions, respawn seed), the same for every sandbox of a generation (TCK-174), see [[wiki/simulation/trainer]].
- **Layout policy** — when the shared layout changes (`trainerLayoutPolicy`): `rotate` = every generation (default since P8), `hold` = until all elites solve it (P7; memorised layouts) (TCK-174 post-fix).
- **Level / difficulty** — `d` in [0, 1] of the one room plugin `LevelScenario`: chamber, epoch and spawn distance geometric, food per type linear between the small room (0, `standard`) and the large room (1, `exploration`) (TCK-176).
- **Curriculum / curriculum gate** — training mode that moves the level at generation ends: success of the non-rehearsal trials above `upper` (0.3) → +`step` (0.1), below `lower` (0.1) → −`step` (`trainerCurriculum`, `gate_level`, TCK-176); a manual level turns the gate off.
- **Rehearsal** — curriculum trials run at an easier level (`rehearsalLevel`) so earlier skills are kept; counted for fitness, ignored by the gate; off by default (`rehearsalShare` 0) (TCK-176).
- **Unseen-layout evaluation** — the final population run on layouts no training generation used (`EVAL_STREAM`, runner `--eval-final`, report `final_eval`); the capability measure for epic targets from P8 on, because training success overstates it (TCK-174 post-fix, TCK-176).
- **Catch unit** — scale of the trainer fitness since P7: 1.0 = one catch at full diet fit; time and approach bonuses stay below 1 (TCK-175).
- **Minimal criterion** — filter for random trainer genomes (generation 1, immigrants): eye or nose and moves ≥ 5 px within 100 ticks (`trainerMinimalCriterion`, RV-S09, TCK-175).
- **1/5 rule** — population-level step-size rule: more than 1/5 of the mutants beat their parent → one more max mutation op per child, fewer → one less (`mutationOneFifthRule`, TCK-174).
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
- **Mutation channel** — which genes a mutation touched: body (trait, `HOX`, organ genes), brain (`NEU`, `SY`, `OUT`), both or neutral (outside every gene) (`dna::promoter_channel`, `classify_mutation`, TCK-181), see [[wiki/biology/genome-compiler]].
- **Lamarckian inheritance** — learned weight offsets passed to offspring, matched by synapse key, per plastic synapse with probability `lamarckianAssimilationChance` (TCK-152).
- **Energy cost** — per-tick drain of the shared energy function (`metabolism::tick_cost`): body, senses, brain (neurons + synaptic activity), movement, see [[wiki/biology/metabolism-and-reproduction]]. Replaced the multiplicative BMR (basal metabolic rate) in P6 (TCK-151).
- **Starved** — trainer state of a creature at 0 energy: frozen for the rest of the trial (TCK-178), see [[wiki/simulation/trainer]].
- **Golden master** — fingerprint test over 300 genomes guarding the compiler, see [[wiki/quality/testing]].
- **Model version** — `MODEL_VERSION`; a bump resets the model tables of the DB, see [[wiki/architecture/persistence]].
- **RV-ID** — review finding ID from the review of 2026-10-19 (`RV-B01`, ...; B bugs, A architecture, P performance, S simulation, D database, F frontend, T test/CI, X docs). Cited in articles as the source of a known limitation or fix.

## Key Takeaways

- Terms map to articles; RV-IDs mark known limitations and fixes.

Sources: `README.md`, `docs/revisit/README.md`, agent skills.
