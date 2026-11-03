---
sources: [wiki/simulation/trainer.md, tickets/closed/TCK-173-trainer-sim-headless-runner.md, tickets/closed/TCK-152-hebbian-plasticity-model.md, tickets/closed/TCK-150-locomotion-and-ecology-measurement.md, tickets/closed/TCK-180-wall-contact-behaviour.md, tickets/closed/TCK-151-bmr-brain-cost-model.md, tickets/closed/TCK-178-trainer-energy-budget.md, tickets/closed/TCK-174-fair-evaluation-and-selection.md, tickets/closed/TCK-175-trainer-fitness-rebuild.md, tickets/closed/TCK-154-trainer-scenario-parameters.md, tickets/closed/TCK-181-body-brain-mutation-decoupling.md, tickets/closed/TCK-176-automatic-training-curriculum.md]
---

# Trainer measurements P2–P8 (history)

Measurement history of the training epic (TCK-171, phases P2–P8), moved out of [[wiki/simulation/trainer]] in
TCK-201. The current trainer model is described there; protocol and report fields in
[[wiki/simulation/analytics-and-balancing]]. Numbers come from the code of their phase and are not reproducible with
later code.

## Baseline (P2, TCK-173)

Headless runner, release build, `--parallel true`, current settings; mean of the last 5 generations (reports kept locally in `.agents/artifacts/baseline/p2/`).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.004 | 0.040 | 0.125 | ≈ 190 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.023 | 0.083 | ≈ 1210 ticks |
| small, pop 300, 20 gens, 1 seed | 0.004 | 0.165 | 0.179 | ≈ 216 ticks |
| large, pop 300, 10 gens, 1 seed | 0.006 | 0.054 | 0.064 | ≈ 880 ticks |

- Success rate = share of sandbox-trials with at least one catch. Immobile share (< 5 px travelled) in the small room: 0.108 in generation 1, 0.032 at the end.
- Throughput: standard pop 300 ≈ 206k sandbox-ticks/s total, exploration pop 16 ≈ 469k ([[wiki/architecture/performance]]).
- Mechanics limits measured by the TCK-172 probes (heredity, speed, steering, walls) are in [[wiki/biology/genome-compiler]], [[wiki/simulation/physics]], [[wiki/biology/ctrnn-brain]] and [[wiki/biology/sensing]].

## After Brain v2 (P3)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations), reports kept locally in `.agents/artifacts/p3/after-C/` ([[wiki/decisions/adr-002-brain-v2]]).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.008 | 0.122 | 0.292 | ≈ 200 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.033 | 0.125 | ≈ 860 ticks |

- Small room ×3 over the P2 baseline, large room ×1.4; both stay below the TCK-171 targets. Immobile share in the small room: 0.067 in generation 1, 0.009 at the end.
- Throughput at pop 16 roughly doubled with the compact brain (seed b1, same machine: small 158k → 323k, large 425k → 763k sandbox-ticks/s after slice A).

## After Oja learning (P4)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations, Lamarck off), reports kept locally in `.agents/artifacts/p4/` (TCK-152, [[wiki/biology/ctrnn-brain]]).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.008 | 0.207 (P3: 0.122) | 0.458 | ≈ 180 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.046 (P3: 0.033) | 0.125 | ≈ 890 ticks |

- The gain comes with bounded learning and fixed reflexes; the Hebbian probe still shows no measurable benefit of lifetime learning per genome, so the effect is not attributed to better within-life learning.
- Variants measured and rejected: no pruning (last 5 gens small 0.166, large 0.026); pruning only with active presynaptic input (no measurable change). Kept: Oja + pruning.
- Immobile share in the small room: 0.067 in generation 1, 0.008 at the end (last 5 gens).

## After Locomotion v2 (P5)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations, Lamarck off), reports kept locally in `.agents/artifacts/p5/` (TCK-150, TCK-180, [[wiki/simulation/physics]]).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.013 | 0.218 (P4: 0.207) | 0.396 | ≈ 190 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.033 (P4: 0.046) | 0.083 | ≈ 1040 ticks |

- Both changes are within the 5-seed noise (standard error ≈ 0.05 small, ≈ 0.01–0.02 large); on 10 seeds (b1–b10) small 0.150 → 0.193, large 0.027 → 0.060. Best fitness in the large room (mean of the last 5 generations) 934 → 1427.
- Calibration of `locomotionSpeedMin` / `locomotionSpeedMax` / `turnRateAtRest`, time-boxed to two adjustments (5-seed success small / large): 0.8 / 3.0 / 0.013 → 0.095 / 0.023; 1.0 / 4.0 / 0.008 → 0.168 / 0.055; 1.0 / 5.0 / 0.008 → 0.218 / 0.033, chosen for the best 10-seed total.
- Immobile share in the small room: 0.004 in generation 1, 0 at the end (P4: 0.067 / 0.008).
- Throughput standard pop 300, 10 generations, seed b1: 857k → 882k sandbox-ticks/s.

## After Energy v2 (P6)

Same protocol (pop 16, 30 generations, seeds b1–b5, mean of the last 5 generations, Lamarck off), before = P5 code, after = final calibration (typical total cost = old Ocean drain); reports kept locally in `.agents/artifacts/p6/before/` and `.agents/artifacts/p6/after2/` (TCK-151, TCK-178).

| Setup | Success rate gen 1 | Success rate last 5 gens | Best gen | Mean time to first catch (last 5) |
| :--- | :--- | :--- | :--- | :--- |
| small, pop 16, 30 gens, 5 seeds | 0.013 | 0.223 (P5: 0.218) | 0.396 | ≈ 189 ticks |
| large, pop 16, 30 gens, 5 seeds | 0.000 | 0.050 (P5: 0.033) | 0.271 | ≈ 1007 ticks |

| Energy metrics, last 5 gens (P5 → P6) | small | large |
| :--- | :--- | :--- |
| `starved_share` | 0 → 0 | 0 → 0.182 (gen 1: 0.10) |
| `mean_end_energy` | 161.7 → 139.0 | 144.0 → 71.3 |
| `food_organ_share` (eye or nose) | 0.907 → 0.958 | 0.812 → 0.917 |
| `mean_organ_count` | 2.96 → 3.41 | 1.85 → 3.24 |
| best fitness | 1204 → 1318 | 1427 → 1582 |

- The 300-tick small room does not exhaust the budget; in the 1800-tick large room about a fifth of the trials end starved. Sensor equipment did not collapse under the cost, it grew (TCK-178 acceptance).
- Both success changes are within or near the 5-seed noise; the large room per seed 0.021 / 0.138 / 0.054 / 0.021 / 0.017, the gain carried mainly by seed b2.
- Throughput standard pop 300, 10 generations, seed b1: ≈ 1064k → 983k sandbox-ticks/s (−8 %; [[wiki/architecture/performance]]).
- Report fields added: `mean_end_energy`, `starved_share`, `mean_organ_count`, `food_organ_share` ([[wiki/simulation/analytics-and-balancing]]).
- Best-fitness values of P5/P6 are on the old fitness scale (before catch units, P7).

## After evaluation and fitness (P7)

Protocol: pop 16, 30 generations, mean success of the last 5 generations, SD between seeds in parentheses; raw data kept locally in `.agents/artifacts/p7/` (TCK-174, TCK-175, TCK-154, TCK-181).

**Correction (P8):** all success numbers of this section are *training* success, measured on the layouts the population was selected on, under layout policy `hold`. They overstate the real capability: the population memorised held layouts (final population of the chosen P7 setup on 10 unseen layouts, small-room training: small 0.149, large 0.039; large-room training: large 0.051). The settings were revised to 8 trials and layout policy `rotate`; see section Post-fix: layout policy and trials. From P8 on, targets are read on unseen layouts.

**Evaluation and selection** (TCK-174, seeds b1–b5, old fitness):

| Variant | small | large | runtime |
| :--- | :--- | :--- | :--- |
| P6 state | 0.223 (0.104) | 0.050 (0.051) | — |
| 3 trials, mean | 0.343 (0.106) | 0.194 (0.020) | 1× |
| 3 trials, median | 0.386 (0.105) | 0.133 (0.077) | 1× |
| 5 trials, mean / median | 0.383 / 0.449 | 0.116 / 0.137 | ~1.7× |
| 8 trials, mean | 0.473 (0.188) | 0.077 (0.013) | ~2.7× |
| 3 trials, mean, 1/5 rule | 0.443 (0.076) | 0.220 (0.024) | 1× |

- Chosen: 3 trials, mean, 1–3 ops, 1/5 rule on, elite ratio 0.25. On 10 seeds the 1/5 rule gives 0.444 / 0.260 vs. 0.338 / 0.193 without (≈ 2.4 / 2.1 standard errors). More trials help the small room only within noise and lower large-room success. Between-seed CV drops from 0.47 to 0.22 (small) and 1.02 to 0.38 (large).
- Population size (b1–b3, chosen settings): 16 → 0.444 / 0.260, 100 → 0.333 / 0.300, 300 → 0.521 / 0.297 at ~7.5× wall time (8.2 s vs. 1.1 s per 30 gens; 883k sandbox-ticks/s at pop 300). Protocol and UI default stay at 16 (each UI sandbox is a visible tile); pop 300 is the option for headless curriculum training (P8, TCK-176).

**Fitness and minimal criterion** (TCK-175, seeds b1–b20):

| Variant | small | large | time to first catch large / small (ticks) |
| :--- | :--- | :--- | :--- |
| TCK-174 state (old fitness, 1/5 rule) | 0.432 (0.132) | 0.229 (0.107) | 944 / 219 |
| catch-unit fitness | 0.425 (0.163) | 0.264 (0.144) | 662 / 192 |
| catch-unit fitness + minimal criterion (chosen) | 0.434 (0.123) | 0.306 (0.109) | 722 / 178 |

- Large room +0.077 (≈ 2.3 standard errors), time to first catch −222 ticks; small room unchanged within noise. On training success the large-room target (≥ 0.30) looked reached at pop 16; on unseen layouts it was not (P8 correction above). The small-room target (≥ 0.50) was left to the curriculum (TCK-176).

**Mutation channels** (TCK-181, `mixed`, seeds b1–b10; share that beat the parent / reached the elite):

| Room | body only | brain only | both |
| :--- | :--- | :--- | :--- |
| small | 4.9 % / 11.4 % (n 1346) | 11.3 % / 11.5 % (n 451) | 15.5 % / 14.4 % (n 464) |
| large | 3.9 % / 10.1 % (n 1289) | 7.4 % / 9.0 % (n 458) | 17.3 % / 13.2 % (n 515) |

- Body-only mutants beat their parent less often but reach the elite as often as brain-only mutants; the most frequent last mutation of elites is a body mutation (small 672 of 1200, large 612 of 1200).
- `separate` vs. `mixed` (10 seeds): small 0.415 vs. 0.438 (−0.4 SE), large 0.335 vs. 0.283 (+1.2 SE), no clear benefit.
- **Decision (TCK-181, closed as not needed):** no body/brain decoupling; `mixed` stays the default, the instrumentation and the `separate` option are kept for the active body traits planned after the epic. The body share of `separate` is fixed at 50/50 (`SEPARATE_BODY_SHARE`, not configurable). `mixed` reproduces the pre-instrumentation results exactly.

## Post-fix: layout policy and trials (P8, TCK-174)

The P7 measurement used training success under layout policy `hold`; a layout was replaced only once every elite solved it, which on b1/b2 never happened in 30 generations, so the population specialised on three fixed layouts. Measured again as the final population on 10 unseen layouts (`--eval-final standard,exploration --eval-trials 10`; pop 16, 30 gens, seeds b1–b10; population / elites):

| Training room, policy, trials | training success (last 5) | unseen small | unseen large |
| :--- | :--- | :--- | :--- |
| small, hold, 3 (P7 default) | 0.438 | 0.149 / 0.170 | 0.039 / 0.048 |
| small, rotate, 3 | 0.232 | 0.126 / 0.193 | 0.026 / 0.035 |
| small, hold, 8 | 0.406 | 0.253 / 0.315 | 0.032 / 0.040 |
| small, rotate, 8 | 0.313 | 0.266 / 0.355 | 0.044 / 0.053 |
| large, hold, 3 | 0.283 | 0.115 / 0.158 | 0.051 / 0.065 |
| large, rotate, 8 | 0.045 | 0.076 / 0.092 | 0.028 / 0.035 |

- Corrected decision: **8 trials, layout policy `rotate`** (`trainerTrials` 8, `trainerLayoutPolicy` rotate): best generalisation in the small room, and training success then tracks performance on unseen layouts instead of overstating it. Shared layouts within a generation stay (fair comparison).
- **Evaluation measure from P8 on:** the final population (the last generation's elites plus the offspring bred from them, not yet evaluated) on 10 unseen layouts per fixed room (`TrainerSim::evaluate_population`, stream `EVAL_STREAM`, report `final_eval`; protocol in [[wiki/simulation/analytics-and-balancing]]). Epic targets are read on these numbers, not on training success.

## Curriculum and UI (P8)

Settings for all runs: pop 16, 8 trials, layout policy `rotate`, 60 generations; measure = final population on 10 unseen layouts per room; scripts and raw data local (`.agents/artifacts/baseline/curr.sh`, `evalsum.py`) (TCK-176).

**Calibration** (seeds b1–b5 unless noted):

| Training | unseen small | unseen large | final level |
| :--- | :--- | :--- | :--- |
| direct small room (b1–b10) | 0.383 | 0.044 | — |
| direct large room (b1–b10) | 0.109 | 0.083 | — |
| curriculum, gate 0.3 / 0.1 (b1–b10, chosen) | 0.281 | 0.096 | mean 0.61 (2 of 10 seeds at 1.0) |
| curriculum + rehearsal 0.25 (b1–b10) | 0.270 | 0.071 | mean 0.54 |
| curriculum + rehearsal + ramp start 0.5 | 0.135 | 0.061 | — |
| curriculum, gate 0.2 / 0.05 + rehearsal | 0.163 | 0.065 | — |
| curriculum, gate 0.6 / 0.3 (design start values) | gate never fires (small-room success stays below 0.6) | | 0 |

- Chosen: gate `upper` 0.3, `lower` 0.1, `step` 0.1, `startLevel` 0, `rehearsalShare` 0, `metabolicRampStart` 1.0 (no ramp; the ramp lowered success in both rooms).

**Switch test** (`--schedule standard:30,exploration:30,standard:10`, b1–b10, unseen small / large after each span): 30 gens small → 0.266 / 0.044; then 30 gens large → 0.133 / 0.041; then 10 gens small → 0.329 / 0.050.

- Curriculum reaches level 1 on only 2 of 10 seeds within 60 gens (mean level 0.61). Its large-room success (0.096) equals direct large-room training (0.083, within noise) while keeping much more small-room skill (0.281 vs. 0.109). Partly met.
- Switch test: small-room skill drops while training in the large room (−0.13, outside a −0.05 tolerance) and recovers above its earlier level after 10 gens back (0.329); the large room does not improve over the switch. Partly met.
- The large-room target (≥ 0.30 on unseen layouts) is not reached; next steps are the TCK-175 escalations (novelty blend, multi-objective, connection cost) and longer runs / larger populations, planned after the epic.

## Earlier behaviour (for reading old reports)

- Room switches before P8 (`SET_TRAINER_MODE`) applied at once and kept the epoch counter and trial accumulators;
  since P8 they wait for the generation boundary (TCK-176).
- Rooms before P8 were one plugin struct each; since P8 levels of one room (`LevelScenario`, `LevelRoom` since TCK-201).
- Large-room food distances before P7: 650 px trial 0, 1200 px trials 1+, 550 px respawn (RV-S08, TCK-154).
- Fitness before P7: per-room `calculate_fitness` with `yield · 1000` / `yield · 3000`, path efficiency, speed and
  coverage bonus, wall/looping/standstill multipliers, centre-distance and homing terms and a trial gate (× 0.1 / + 150);
  values ~10²–10³, not comparable with catch units (`MODEL_VERSION` 5).
- Before P7 every sandbox rolled its own layout (stream `derive(seed, [g, t, i])`) and the trainer age stayed 0, so the
  clock input was constant 0.5.
- Hall-of-fame children were unmodified copies before TCK-174; mutation count was `round(len · mutation_rate)` before.
- Fixed while extracting `TrainerSim` (TCK-173): Lamarck source index underflow for fallback mutants; Lamarck mutants
  were built from the unmutated parent genome; catalogue assignment kept the old `antisense` and neuron buffers.
- Before TCK-201 a catalogue assignment kept the old creature's position, heading and trial record (approach episode,
  rotation, coverage), restarted the sandbox's tick counter mid-trial and left the slot's earlier trial fitness and
  lineage in place.

## Key Takeaways

- P2 baseline: success rate 0.040 (small) and 0.023 (large) at population 16 after 30 generations; after Brain v2 0.122
  and 0.033; after Oja learning (P4) 0.207 and 0.046; after Locomotion v2 (P5) 0.218 and 0.033; after Energy v2 (P6)
  0.223 and 0.050; after P7 0.434 and 0.306 (20 seeds). P2–P7 numbers are training success; the P7 values overstate
  capability (on unseen layouts 0.149 small, 0.051 large).
- From P8 on the measure is the final population on 10 unseen layouts: direct small-room training 0.383 / 0.044
  (small / large room), curriculum 0.281 / 0.096 at 60 gens; the large-room target (≥ 0.30) is not reached.
