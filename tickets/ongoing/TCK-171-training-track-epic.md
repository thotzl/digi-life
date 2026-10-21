---
id: TCK-171
title: "Epic: Training Quality and Core Mechanics"
status: ongoing
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
---

# TCK-171: Epic: Training Quality and Core Mechanics

## Context
Before the revisit the focus was on making the training scenarios work (scoring, mechanics, setups). Training only produced results in the small chamber (1000 px), not in the large one (3500 px). Analysis on 2026-10-21 (static code reading plus a speed estimate; nothing measured yet) suggests the cause lies less in the trainer than in the core mechanics the trainer builds on.

Placement: Phase 1a of the roadmap TCK-169, before the rest of Phase 1. TCK-172 needs no refactor; TCK-173 is a trainer-only slice of TCK-156/TCK-158 (Ocean untouched).

## Suspects (ranked)
1. Missing heredity: low-locality genotype-phenotype map plus 15–30 mutations per child → children barely resemble parents (TCK-172, finding 1).
2. Speed: a large part of the population physically cannot solve the large-chamber task; fitness selects fast bodies, not better brains (TCK-172, finding 2).
3. Steering without an organ pair (TCK-172, finding 3).
4. Walls: possible stalling at walls (TCK-172, finding 4).
5. Only then fitness and trainer setup (table below).

A trainer rebuild alone would not fix this foundation.

## Trainer findings (code-verified)
| Finding | Location |
| :--- | :--- |
| Large chamber 3500 px, food ≥ 650 px (trial 1) / ≥ 1200 px (trials 2/3) / 550 px (respawn) from start, epoch 1800 ticks; small chamber 1000 px, 200 px, 300 ticks | `src/biology/scenarios.rs:50,195`, `src/server/engine.rs:1753,1803`, `src/biology/trainer_engine.rs:197` |
| Each sandbox rolls its own food layout → candidates of one generation solve different tasks | `scenarios.rs:53-76,197-209`, `engine.rs:1832-1845` |
| 16 sandboxes, elite ratio 0.15 → ~2 parents per generation; truncation selection | `engine.rs:96,157` |
| Mutation rate 0.06 = `genome.len() * 0.06` mutation ops per child → 15–30 | `engine.rs:97`, `trainer_engine.rs:78-84` |
| Large-chamber failure fitness rewards displacement from the centre (`* 1.2`), not approach to food | `scenarios.rs:344-347` |
| Hard multipliers (×0.05, ×0.1, ×0.25, ×0.3, +150 per catch, quadratic path efficiency, 90 % starvation filter) | `scenarios.rs:142-167,363-367`, `engine.rs:1889-1895` |
| Gen 1 with origin `random` uses `generate_random_genome(256)` → mostly blind starters (RV-S09) | `trainer_engine.rs:85` |
| Sensor range `scale * 550` (≈ 190–990 px) → food usually out of sight at start in the large chamber | `src/shared/brain.rs:168` |
| Rejection-sampling spawn loops without iteration limit (RV-S08); epoch/chamber literals in the engine | `scenarios.rs`, `engine.rs:1753,1802` |
| UI shows only best/avg fitness | `engine.rs` (`TRAINER_GENERATION_COMPLETED`) |

Interpretation: in the small chamber food is almost always in sensor range, so approaching pays off directly. In the large chamber there is no signal until food comes into view by chance (the bootstrap problem of evolutionary robotics), and the failure fitness rewards "swim to the wall". With different tasks per sandbox and ~2 parents, the easiest dice roll wins.

## Child tickets (order)
| # | Ticket | Content | Depends on |
| :--- | :--- | :--- | :--- |
| 1 | TCK-172 | Mechanics diagnosis probes (measurement only) | – |
| 2 | TCK-173 | `TrainerSim`, seeded RNG, headless trainer runner + baseline | – |
| 3 | TCK-174 | Fair evaluation and selection | TCK-173 |
| 4 | TCK-175 | Fitness rebuild (includes TCK-154) | TCK-173, TCK-154 |
| 5 | TCK-176 | Automatic curriculum with one knob | TCK-175 |
| 6 | TCK-177 | Trainer UI: success rate, median, curve | TCK-173 |

Mechanic fixes become tickets only after TCK-172 delivers numbers (candidates listed there). Every child ticket uses a before/after measurement with the TCK-173 runner as acceptance; results go into `wiki/simulation/trainer.md` (and `wiki/simulation/physics.md` / `wiki/biology/*` for mechanics) on closing.

## Research summary (2026-10-21)
Checked via search results and abstracts, not full texts.

| Measure | Literature | Verdict |
| :--- | :--- | :--- |
| Stable synapse identity instead of position hash | Rothlauf: efficient evolutionary search needs high locality; NEAT innovation numbers keep gene identity across generations | well supported |
| Mutation 1–3 ops, success-based adaptation | ~1 mutation per child is the theoretical standard; success-based (1/5-rule family) adaptation is provably optimal (Doerr et al.) | supported; population-level rule simpler than a self-adaptive gene |
| Same task for all candidates | DelFly study: re-randomised initial conditions hide whether behaviour improved; hold conditions until all elites succeed | supported, adapted (hold layouts across generations) |
| More trials, median | standard practice, but more evaluations are not always better | measure the trial count |
| Larger population | robotic-noise study: larger populations / more iterations do not efficiently counter large noise | weaker than assumed; remove noise first |
| Smooth fitness with few terms | bootstrap problem; most probable cause an ill-defined fitness; incremental evolution only if no smooth fitness exists | well supported; fitness before curriculum |
| Novelty search | wins on deceptive mazes, struggles in large behaviour spaces with a fixed goal; blends / minimal criteria do better | only as a blend |
| Pareto / multi-objective | ancillary objectives solve the bootstrap problem | supported, escalation option |
| Curriculum gate | POET minimal criterion "neither too easy nor too hard"; teacher-student picks the steepest learning slope | principle supported; thresholds are calibration values |
| Tournament selection | least diversity loss at equal intensity (Blickle/Thiele), but Motoki shows larger loss than estimated | weak; more parents is the real lever |
| Steering | bilateral sensors → tropotaxis (Braitenberg); single sensor + head swing → klinotaxis (C. elegans) | adapted: temporal-change (ΔI) channel, see TCK-144 |
| Body-brain coupling | morphological innovation protection (Cheney et al. 2018); critical follow-up 2025 | new candidate |

Sources:
- Rothlauf, High Locality Representations: https://gpbib.pmacs.upenn.edu/gp-html/Rothlauf_2011_DMH_cp7.html
- Galván-López et al., locality in GP: https://ncra.ucd.ie/papers/localityingpWCCI2010.pdf
- NEAT innovation numbers: https://neat-python.readthedocs.io/en/latest/innovation_numbers.html
- Stanley & Miikkulainen, NEAT: https://axon.cs.byu.edu/~dan/778/papers/NeuroEvolution/stanley2*.pdf
- Doerr, Doerr, Lengler, self-adjusting mutation rates: https://arxiv.org/pdf/1902.02588
- Behaviour trees for ER (DelFly, held initial conditions): https://arxiv.org/pdf/1411.7267
- Morphological variation in ER: https://arxiv.org/html/2208.02809v2
- Jin & Branke, uncertain environments: https://www.cs.le.ac.uk/people/sy11/ECiDUE/JinBranke-TEVC05.pdf
- Fitness noise in PSO (robotic learning): https://dev.nccr-robotics.ch/?p=2621
- Lehman & Stanley, Abandoning Objectives: https://gpbib.pmacs.upenn.edu/gp-html/Lehman_2011_EC.html
- Combined objective and novelty: https://www.cs.swarthmore.edu/~meeden/cs81/f17/projects/JeffKyle.pdf
- Composite objectives and novelty selection: https://arxiv.org/pdf/1803.03744
- Mouret & Doncieux, bootstrap problem: https://hal.archives-ouvertes.fr/hal-00473147
- Incremental evolution as multi-objective optimisation: https://hal.archives-ouvertes.fr/hal-00687647
- Task-specific multi-objective ER: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4546428/
- Nelson et al., fitness functions in ER: https://gpbib.pmacs.upenn.edu/gp-html/Nelson2009345.html
- POET: https://arxiv.org/pdf/1901.01753 ; Enhanced POET: https://arxiv.org/pdf/2003.08536
- Blickle & Thiele, selection schemes: https://gpbib.pmacs.upenn.edu/gp-html/blickle_1995_css.html
- Motoki, loss of diversity: https://niigata-u.repo.nii.ac.jp/record/1649/files/10_4_397-422.pdf
- Braitenberg vehicle: https://en.wikipedia.org/wiki/Braitenberg_vehicle ; tropotaxis review: https://frontiersin.org/articles/10.3389/fbioe.2020.565963/full
- Satoh et al., klinotaxis by a single neuron: https://pmc.ncbi.nlm.nih.gov/articles/PMC6608435 ; klinotaxis review: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC4132367/
- Cheney et al. 2018: https://meclab.w3.uvm.edu/papers/2018_RoySoc_Cheney.pdf ; critical follow-up 2025: https://arxiv.org/pdf/2508.17464

## Definition of Done
- TCK-172..177 closed or consciously dropped; mechanic fixes from TCK-172 decided and ticketed.
- Headless report shows a measurable success rate in the large chamber, compared against the TCK-173 baseline.
- Wiki articles describe the resulting trainer and mechanics.
