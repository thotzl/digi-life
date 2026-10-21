# Next Tasks: Training Quality and Core Mechanics

Last updated: 2026-10-21. Status: notes, not tickets yet; will likely be split into several tickets.

Context: before the revisit, the focus was on making the training scenarios work (scoring, mechanics, setups). Training only produced results in the small chamber (1000 px), not in the large one (3500 px). The user wants to focus on training next. This conflicts with TCK-169 (no feature work before Phase 1); proposal: treat the trainer as the first slice of Phase 1 (TCK-156/158 scoped to the trainer). Decision pending.

All findings below come from reading the code (static); speed figures come from an estimate script (`.agents/artifacts/vterm.py`); nothing is measured yet.

## 1. Findings: core mechanics (likely root causes)

Ranked by suspected impact.

### 1.1 Heredity: genotype-phenotype map is non-local (code-verified)
- Exuberant synapse weights are read from `active_dna` at index `(from + to*13) % len` (`src/biology/dna.rs:1323-1325`, `get_payload_linear_value_offset` `dna.rs:674-686`). `active_dna` = open-chromatin positions only (`dna.rs:743-750`).
  - An insertion/deletion inside an active gene shifts all later positions → every weight reading from there changes.
  - A mutation that creates/destroys a gene changes `len` → the modulo mapping of *all* weights changes.
- Neuron ids depend on the organ count (`dna.rs:12-26`); one organ more/less shifts every id → all weights and the explicit `SY` synapses (mapped by modulo over sources/destinations, `dna.rs:1251-1258`) re-map.
- Body traits without their own gene (`PUL`, `STF`, `SIZ`, ...) fall back to reading from `active_dna` (`dna.rs:800-861`) → a mutation in a brain gene can change speed.
- Trainer mutation rate 0.06 = `genome.len() * 0.06` mutation ops per child → 15–30 per child (`src/server/engine.rs:97`, `src/biology/trainer_engine.rs:78-84`).
- Interpretation: children barely resemble parents; selection cannot accumulate improvements. The trainer behaves like random search plus unmodified elite clones. Enough for the easy small chamber, not for the large one. Strongest suspect.

### 1.2 Locomotion speed (estimate)
Physics from `src/shared/physics.rs:98-195` + `config.json` (`thrustBaseMultiplier` 3.4, `dragForwardCoefficient` 0.015, `receptorBallastScale` 0.18, damping 0.94); 100k uniform random traits, full thrust, 2 organs:

| terminal speed px/tick | p10 | p25 | p50 | p75 | p90 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| full thrust | 0.08 | 0.23 | 0.74 | 2.2 | 5.0 |

- Median genome: 1200 px straight ≈ 1630 ticks (epoch 1800); 200 px ≈ 270 ticks (small-chamber epoch 300). Best case: straight line, perfect thrust, no search.
- 48 % cannot cover 1200 px in one epoch; 22 % are slower than 0.2 px/tick.
- Spread p10→p90 ≈ ×60; thrust scales with `pulse²` (`physics.rs:113`) → selection favours fast bodies over better brains.
- Turning only while moving: `delta_heading = v_forward * bend * 0.015` (`physics.rs:182`); slow creatures can hardly turn. Min turning radius ≈ 55 px.

### 1.3 Sensing and brain layout (code-verified)
- Left/right comparison only for paired organs: thalamus difference nodes exist only with a `HOX` gene (`dna.rs:1021`, `dna.rs:1352-1374`). Single organs give sums only → steering must emerge from scanning.
- Initial brains barely drive: exuberant weights in [-0.5, 0.5] (`dna.rs:1327`), thrust output `tanh` → mean thrust ≈ 0, negative = backwards.
- Sensor range `scale * 550` (`src/shared/brain.rs:168`, ≈ 190–990 px); in the large chamber food usually starts out of sight.
- Minor: motion vision overwrites the meat channel (TCK-144); 2-tick sensor→motor latency (TCK-153).

### 1.4 Walls (hypothesis)
- Wall contact reflects velocity but not heading (`physics.rs:208-228`); next tick velocity is projected back onto the heading (`physics.rs:194-196`) → a creature pushing into a wall stalls at ≈ 0 speed and therefore cannot turn (1.2). Possible wall trapping; large-chamber failure fitness additionally rewards distance from the centre.

## 2. Findings: trainer (scoring, setups)

| Finding | Location |
| :--- | :--- |
| Large chamber 3500 px, food ≥ 650 px (trial 1) / ≥ 1200 px (trials 2/3) / 550 px (respawn) from start, epoch 1800 ticks; small chamber 1000 px, 200 px, 300 ticks | `src/biology/scenarios.rs:50,195`, `engine.rs:1753,1803`, `trainer_engine.rs:197` |
| Each sandbox rolls its own food layout → candidates of one generation solve different tasks (selection noise) | `scenarios.rs:53-76,197-209`, `engine.rs:1832-1845` |
| 16 sandboxes, elite ratio 0.15 → ~2 parents per generation; truncation selection | `engine.rs:96,157` |
| Large-chamber failure fitness rewards displacement from centre (`* 1.2`), not approach to food | `scenarios.rs:344-347` |
| Many hard multipliers (×0.05, ×0.1, ×0.25, ×0.3, +150 per catch, quadratic path efficiency, 90 % starvation filter) → cliffs, reward hacking | `scenarios.rs:142-167,363-367`, `engine.rs:1889-1895` |
| Gen 1 with origin `random` uses `generate_random_genome(256)` → mostly blind starters (RV-S09) | `trainer_engine.rs:85` |
| Rejection-sampling spawn loops without iteration limit (RV-S08) | `scenarios.rs`, `engine.rs`, `trainer_engine.rs` |
| Epoch/chamber literals in the engine (`1800/300`, `>= 2000.0`) | `engine.rs:1753,1802` |
| UI shows only best/avg fitness | `engine.rs` (`TRAINER_GENERATION_COMPLETED`) |

Interpretation (trainer side):
- Small chamber: food is almost always within sensor range; approaching pays off directly → behaviour emerges.
- Large chamber: no signal until food comes into view by chance; the failure fitness then rewards "swim straight to the wall", not "search for food".
- Noise: with different tasks per sandbox and only ~2 parents, the candidate with the easiest dice roll wins, not the best one.
- 15–30 mutations per child probably destroy more than they improve in a promoter-based genome (to be measured).

## 3. Next step: diagnosis (measure, no model change)

`parse_genome`, `mutate_genome`, `step_creature_kinematics`, `compute_sensory_inputs` are pure → probes need no engine refactor (own binary or `#[ignore]` tests writing a report to `.agents/artifacts/`).

| Probe | Question |
| :--- | :--- |
| Heredity | Parent-child correlation of weights and body traits after 1 / 5 / 15 mutations, split by point / insertion / deletion |
| Speed | Real terminal-speed distribution over 1,000 random genomes and over HoF genomes (DB read-only) |
| Equipment | Share of random genomes with eye/nose, with a `HOX` pair, with food in sensor range at spawn |
| Steering | Food fixed at ±45°: does the creature turn towards it? Random vs. HoF genomes |
| Wall | Full thrust at an angle into a wall: does it get free? |

## 4. Candidate fixes (decide after measuring)

### Mechanics
- Heredity: stable synapse keys (organ index, channel, target) instead of the position hash, or relative addressing (TCK-168); traits only from their own genes; mutation 1–3 ops per child or self-adaptive.
- Speed: rescale thrust vs. mass/drag to narrow the spread (e.g. linear in pulse) (TCK-150, TCK-166).
- Steering: pairs without `HOX`, or an angle channel for single organs.
- Walls: reflect heading too, or allow sliding along the wall.

### Trainer ideas (ordered by leverage, numbering as presented in the session)

A. Measure first
1. Headless trainer runner with fixed seed, no UI, no warp cap; runs N generations and writes a JSON report per generation: success rate, mean time to first catch, best, median, fitness distribution, genome length. Only way to compare two settings. Trainer-specific slice of TCK-156/158.
2. Baseline measurements with the runner: how many random genomes move at all and how fast; how many see food; success rate after 100 generations in the small and the large chamber.

B. Fair evaluation
3. Same task for all candidates of a generation (common random numbers): one layout (food positions, start heading) per generation and trial, shared by every sandbox → fitness differences reflect the genotype, not dice luck.
4. More trials (5–8 instead of 3) and the median instead of the mean (robust against outliers).
5. Larger population in the headless runner (100–300); the UI shows only 16. Simulating a sandbox is cheap; today only rendering sets the limit.

C. Rebuild fitness
6. Few interpretable terms instead of many penalty cliffs: primary = eaten amount / yield; secondary = time to first catch; shaping = approach achieved towards the nearest food that was in sight (no distance-from-centre term); no ×0.05 / ×0.1 factors that create cliffs and invite exploits.
7. Novelty search or a novelty blend for the large chamber (classic remedy for deceptive, sparse reward; Lehman/Stanley maze navigation): reward how new the behaviour is, e.g. end position and visited sectors → search strategies emerge without being prescribed.
8. Multi-objective (Pareto) selection as an alternative to 7: eating, path and energy scored separately instead of a weighted sum. More effort; only after 6.

D. Raise difficulty continuously
9. Automatic curriculum with a single knob instead of ten discrete scenarios: success rate above ~60 % → start distance and/or chamber size grow one step; below ~30 % → they shrink. Makes the jump from "food in sight" to "food out of sight" gradual. The core of TCK-123, much smaller.
10. Food density as the knob instead of chamber size: more food, further away, gives an earlier signal; density decreases along the curriculum.
11. Uniform spawn distances in all trials from the scenario plugin (TCK-154) plus an iteration limit for rejection sampling.

E. Evolution
12. Cut mutation strength drastically to 1–3 ops per child, or make it self-adaptive (inherit the mutation rate). Suspected as the most likely single cause of slow progress; not measured yet.
13. Tournament selection instead of hard truncation, with more parents (~20–30 % of the population); keeps diversity better than 2 parents.
14. Starting population with minimum equipment (eyes, one synapse to the motor, minimum thrust), or a filter that discards immobile/blind genomes in generation 1; otherwise compute is wasted on blind starters. Decide RV-S09: should random genomes start blind?
15. Switch Hebbian learning off during training as a test; the rule is unnormalised and drifts (TCK-152); measurement shows whether it helps or hurts.

F. UI
16. Trainer UI additionally shows success rate and median per generation, plus a curve over generations.

## 5. Conclusion and proposed order

Suspects by weight: (1) missing heredity (1.1) plus the high mutation rate; (2) speed (1.2) — a large part of the population physically cannot solve the task; (3) steering without an organ pair (1.3); (4) walls (1.4); (5) only then fitness and trainer setup (section 2). A trainer rebuild alone would not fix this foundation.

1. First ticket proposal: "Mechanics diagnosis for training" — the five probes of section 3 as a report; no model change, measurement only.
2. Decide mechanic fixes (section 4) based on the numbers.
3. Trainer track (each step its own ticket with a before/after measurement as acceptance; results into `wiki/simulation/trainer.md`):
   1. Extract trainer logic from the engine closure into `TrainerSim` with `step()`, seeded RNG, headless runner with report (ideas 1–2); slice of TCK-156/158, Ocean untouched.
   2. Baseline measurement small vs. large chamber.
   3. Same tasks, more trials, larger population, lower mutation (ideas 3, 4, 5, 12). Measure.
   4. Rebuild fitness (6) plus uniform spawn distances (11). Measure.
   5. Curriculum with one knob (9, 10). Measure.
   6. If needed: novelty (7) and generation-1 filter (14).

## Open decisions (from this session)
- Training first vs. TCK-169 Phase order (trainer as first Phase 1 slice?).
- Pool/feature decision list D1–D23 was presented in chat on 2026-10-21 but not answered; regenerate from the tickets when needed.
