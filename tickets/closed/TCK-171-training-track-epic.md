---
id: TCK-171
title: "Epic: Training Quality and Core Mechanics"
status: closed
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

## Goal
User, 2026-10-21: training that works — creatures that really develop and improve across different rooms, so that they can later survive in the Ocean. Two rooms suffice for now (small 1000 px `standard`, large 3500 px `exploration`; an arena exists only as a plan in TCK-123); further rooms are defined once the setup is sound and safe.

Scope decision (user, 2026-10-21): all fixes that fundamentally belong to creature mechanics, brain, performance, energy costs and training are part of this epic, even if not yet visible in the first two rooms. Refactoring (DRY, typed enums, shared types, helpers) happens on the way in the tickets that touch the code, limited to the trainer path and shared creature code; Ocean-only refactoring stays in Phase 1. Backward compatibility is not required; the DB may be emptied on model changes (TCK-183).

## Phases
Phases run in order. Each phase closes with one commit, one version bump, one CHANGELOG entry and one wiki update covering all its tickets, plus the smoke checklist of TCK-186. Acceptance per phase: before/after measurement with the runner (TCK-173) and the probes (TCK-172) against the target values (TCK-188).

| Phase | Tickets | Content |
| :--- | :--- | :--- |
| **P1 Config** | TCK-149 | Per-key decision with the user, implement/remove, `deny_unknown_fields`; `seed`/`maxCreatures` needed later |
| **P2 Sim lab** | TCK-173, TCK-172, TCK-185, TCK-183 (runner/retention), TCK-186 (CI), TCK-188 (targets, analytics skill) | `TrainerSim`, seeded RNG, one runner with probe subcommands, trainer refactoring, runtime performance, `CreatureFactory`, in-memory runner and retention, CI reference run; baseline and target values |
| **P3 Brain v2** | TCK-179, TCK-182, TCK-144, TCK-153, TCK-184, TCK-187, TCK-183 (reset rule), TCK-188 (ADR-002), TCK-149 (biolum/reserved motors) | One rebuild of the genome compiler and brain: stable synapse identity, trait locality, compact topology with pair weight sharing, ΔI channel, evaluation order, structured random genomes, gene-level and creep mutation, gene spans from backend; one golden-master change, DB reset |
| **P4 Learning** | TCK-152 | Oja rule, `plastic` flag, Lamarck as probability on the new synapse structure |
| **P5 Locomotion v2** | TCK-150, TCK-180 | Thrust rescaling, turning at low speed, wall contact |
| **P6 Energy v2** | TCK-151, TCK-178, TCK-145 | One shared energy function: per neuron + activity + organ size, constants computed from share targets; energy budget in the trainer; predation energy |
| **P7 Evaluation and fitness** | TCK-174, TCK-175, TCK-154 | Shared layouts, trials, mutation strength, parents, population; fitness rebuild, spawn distances, chamber size from plugin, cross-room comparability, generation-1 equipment; measured in two steps |
| **P8 Curriculum and UI** | TCK-176, TCK-177 | Curriculum small/large with rehearsal, switch at generation boundary, metabolic ramp; UI metrics and typed trainer events |
| *conditional* | TCK-181 | Body/brain mutation decoupling, only if P7 shows body mutations destroy learned behaviour |

Calibration loops are cut by fixed targets: metabolic constants come from share targets (computable from the phenotype, no training needed); thrust, fitness weights, curriculum thresholds and rehearsal share are time-boxed against the TCK-188 targets.

Related, not part of the epic: TCK-168 (relative synapse addressing; research, only if TCK-179 is insufficient), TCK-161 (rest of the test suite; property tests are in TCK-179/TCK-184), TCK-155 / TCK-157 / TCK-160 (non-trainer parts), TCK-159 (Ocean-side persistence).

Later (Ocean phase): Ocean survival benchmark for champions — headless, trained vs. random creatures, lifespan and eating rate; needs TCK-143 and the Ocean part of TCK-156. This is the actual proof of the goal.

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

## Execution decisions (user, 2026-10-21, before P2)
- Work directly on `master`; one commit per phase at the end (bump, CHANGELOG, wiki, tickets) when everything is green.
- Target values (TCK-188): derived by the agent from the P2 baseline by a documented rule, recorded here, no extra stop.
- Defaults with veto option accepted: TCK-144 (ΔI as 6th channel), TCK-145 (bite energy formula + cooldown), TCK-152 (Oja, `plastic` flag, Lamarck as probability), TCK-153 (keep order unless the probe shows a problem); P3 removes motor 3 (biolum), motor 4 (reserved) and the adrenaline brain input.
- Measurement-based decisions (TCK-181 yes/no, TCK-182 targeted wiring vs. minimal start, TCK-175 generation-1 equipment, TCK-153 reorder, steer/drag keys in P5, time-boxed calibration of thrust/fitness/curriculum) are taken by the agent and documented with numbers in the ticket; the user is consulted only on real target conflicts or targets missed after the time box.
- All phases P2–P8 run without stops (user, 2026-10-21): the agent also performs the per-phase UI smoke checks itself, pushes after each green phase commit, checks CI, and continues. The user is consulted only on real ambiguities or decisions that cannot be taken clearly.
- UI bugs found in passing (e.g. during the per-phase UI smoke checks) are fixed in the same phase and listed in its CHANGELOG entry (user, 2026-10-21).

## Target values (TCK-188, set 2026-10-22 from the P2 baseline)
Measured with `trainer_runner` (release, `--parallel true`) and the TCK-172 probes; reports in `.agents/artifacts/baseline/p2/` and `.agents/artifacts/probes/` (local), numbers in TCK-173/TCK-172.

Rule: the goal is real improvement in both rooms with transfer between them. Success targets require a population majority in the small room, the large room at ≥ 60 % of the small room's rate, and a clear learning slope within a run; mechanics targets come from the probe findings (heredity, speed spread, steering, walls); cost and performance targets come from the epic (share 10–20 %) and from "no regression". A phase is accepted when its targets are met or a deliberate exception is documented in its ticket.

| Target | P2 baseline | Target | Measured by |
| :--- | :--- | :--- | :--- |
| Success rate small room (pop 16, 30 gens, mean of last 5 gens, 5 seeds) | 0.040 | ≥ 0.50 | `run --scenario standard` |
| Success rate large room (same protocol) | 0.023 | ≥ 0.30 (≥ 60 % of small) | `run --scenario exploration` |
| Learning slope (success last 5 gens / gen 1, both rooms) | ×10 small (0.004 → 0.040), large 0 → 0.023 | ≥ ×3 and absolute targets above | `run` |
| Speed spread p90/p10 (random genomes, full thrust) | ×65 | ≤ ×5 | `probe speed` |
| Share of random genomes unable to cover 1200 px in 1800 ticks | 50 % | ≤ 10 % | `probe speed` |
| Heredity: exuberant-weight correlation parent→child after 1 / 5 / 15 ops (random parents, trainer operator) | ≈ 0.93 / – / 0.32 | ≥ 0.95 / 0.85 / 0.60 | `probe heredity` |
| Children with ≥ 2× speed change after one mutation | n/a (17–25 % after 15 ops) | ≤ 2 % after 1 op | `probe heredity` |
| Steering: elite genomes after training turning toward food on both sides (±45°) | 0 % (HoF) | ≥ 50 % | `probe steering --db` / elites of a run |
| Walls: forced full thrust into a wall, share free within 600 ticks | 0 % | ≥ 90 % | `probe wall` |
| Brain + senses share of resting metabolic cost (median phenotype) | n/a (multiplicative model) | 10–20 % | P6 cost function test |
| Ticks per second, standard, pop 300, parallel (total incl. breeding) | 206k | ≥ 206k (no regression) | `run --population 300` |

## Definition of Done
- Phases P1–P8 closed; all their tickets closed or consciously dropped (with measurement).
- Target values of TCK-188 reached, compared against the P2 baseline.
- Wiki articles describe the resulting trainer and mechanics.

## Result (2026-10-26, v1.12.22)
Phases P1–P8 closed (v1.12.15 … v1.12.22), one commit per phase on master; all phase tickets closed (TCK-181 measured as not needed). Post-fix in P8: P7's "hold, 3 trials" evaluation let the population memorise its training layouts, so from P8 on success is measured on **unseen layouts** (final population, `--eval-final`, 10 evaluation trials per room); see TCK-174 post-fix.

| Target | P2 baseline | Result | Status |
| :--- | :--- | :--- | :--- |
| Success small room (unseen layouts) | 0.040 | pop 16, 60 gens, 10 seeds: 0.383 (elites 0.497); pop 300, 60 gens, 3 seeds: 0.699 (elites 0.887) | met at pop 300; pop 16 below 0.50 |
| Success large room (unseen layouts) | 0.023 | pop 16 curriculum, 60 gens, 10 seeds: 0.096 (direct: 0.083); pop 300 curriculum, 3 seeds: 0.336 (elites 0.436, SD 0.32) | met at pop 300 (few seeds, high spread); pop 16 far below |
| Learning slope | ×10 / 0 → 0.023 | gen 1 ≈ 0.01 → 0.38 (small), 0.10 (large, pop 16) | met |
| Speed spread p90/p10 | ×65 | ×1.47 | met |
| Genomes too slow for 1200 px in 1800 ticks | 50 % | 0 % | met |
| Heredity r after 1 / 5 / 15 ops | 0.93 / – / 0.32 | 0.97–0.98 / 0.88–0.92 / 0.75–0.81 | met |
| ≥ 2× speed change after 1 op | n/a | 0 % | met |
| Steering of trained elites (both sides) | 0 % | 76.6 % (64 elites of 3 trained runs) | met |
| Walls freed within 600 ticks | 0 % | 100 % | met |
| Brain + senses share of resting cost | n/a | median 10.1 % | met (lower edge) |
| Ticks/s, pop 300 | 206k | 0.6–2.1 M (one run at 195k under concurrent build load) | met |

Open gap: at the protocol population of 16 the success targets are not met on unseen layouts, the large room in particular (0.096). Levers measured but not chosen: rehearsal, metabolic ramp (both without benefit). Next options (after the epic): larger populations / longer runs in headless training, the escalations of TCK-175 (novelty blend, multi-objective, connection cost), and the after-epic plan in `.agents/state/next-tasks.md`.

## Wrap-up (2026-10-26, v1.12.23 – v1.12.25)
Follow-ups that round off the epic (user decision after the epic report): TCK-191 run state persisted (v1.12.23), TCK-190 tanh thalamus sum nodes with re-derived energy constants, model version 6 (v1.12.24), TCK-192 population independent of the visible tiles, default 100 (v1.12.25); dependency alerts ticketed as TCK-189, naming as TCK-193, training escalations in `.agents/state/next-tasks.md`.

Final measurement with the finished model (unseen layouts, final population, 10 evaluation trials per room; raw data `.agents/artifacts/wrapup/final`):

| Population | small room direct | curriculum 60 gens: small / large | final level |
| :--- | :--- | :--- | :--- |
| 16 (10 seeds) | 0.454 (30 gens; elites 0.598) | 0.320 / 0.227 | 0.91 |
| 100 (5 seeds, app default) | 0.708 (60 gens; elites 0.926) | 0.467 / 0.376 (elites 0.481) | 1.0 |
| 300 (10 seeds) | 0.709 (60 gens; elites 0.920) | 0.462 / 0.395 (SD 0.213; elites 0.525) | 0.99 |

Targets: small room ≥ 0.50 and large room ≥ 0.30 on unseen layouts are **met** at the app default population 100 and at 300 (direct training for the small room, curriculum for the large room); at the protocol population of 16 the small room is close (0.454) and the large room is not (0.227). Brain + senses share of the resting cost after the re-derivation: 15.5 %. Throughput pop 300: median 1.4–1.9 M ticks/s.
