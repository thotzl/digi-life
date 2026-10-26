---
id: TCK-174
title: Fair Trainer Evaluation and Selection
status: closed
priority: high
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173
---

# TCK-174: Fair Trainer Evaluation and Selection

Part of epic TCK-171. Depends on TCK-173 (runner and baseline).

## Description
Selection is dominated by noise: every sandbox rolls its own food layout (`src/biology/scenarios.rs:53-76,197-209`, `src/server/engine.rs:1832-1845`), only ~2 parents survive (16 sandboxes, elite ratio 0.15, `engine.rs:96,157`), and each child receives 15–30 mutation ops (`engine.rs:97`, `src/biology/trainer_engine.rs:78-84`).

## Scope
1. **Shared layouts:** one layout (food positions, start heading) per trial, identical for all candidates. Hold layouts across generations and replace a layout once all elites solve it (DelFly behaviour-tree study).
2. **Trial count:** measure 3 vs. 5 vs. 8 trials and mean vs. median; pick by result variance per runtime, not by default (more evaluations are not always better).
3. **Mutation strength:** 1–3 mutation ops per child instead of `len * rate`; then test a success-based adjustment at population level (1/5-rule family, Doerr et al.).
4. **Parents:** more parents (~20–30 % of the population); tournament selection only as an optional variant (benefit over truncation is disputed: Blickle/Thiele vs. Motoki).
5. **Population size:** measure 16 vs. 100 vs. 300 in the headless runner after 1–4 (larger populations alone do not counter large noise efficiently).

## Acceptance
- Before/after report vs. the TCK-173 baseline: lower fitness variance between seeds and higher success rate (small and large chamber).
- Chosen values documented with the measurement in `wiki/simulation/trainer.md`.

Sources: see epic TCK-171.

## Verification (P7 step 1, 2026-10-26)
Implementation: `src/biology/trainer_sim.rs` (module doc: what is shared; `layout_rng` = `derive(seed, [LAYOUT_STREAM, layout_id, trial])`, `elites_solved_layout` advances `layout_id` once every elite caught food in every trial, `aggregate_trials`, `one_fifth_update`, avg fitness and elite count divided by the actual sandbox count), `src/biology/trainer_engine.rs` (`draw_mutation_ops` uniform in [`mutationOpsMin`, `mutationOpsMax`], `breed_genome` mutates `hof` like `mutant`, respawns from `derive(respawn_seed, [catch index])`, trainer `agent.age` increments per tick and resets per trial), config keys `trainerTrials`, `trainerTrialAggregation`, `mutationOpsMin`, `mutationOpsMax`, `mutationOneFifthRule`, `mutationOpsLimit`; runner flags `--trials`, `--aggregation`, `--one-fifth`, `--elite-ratio`. UI: the mutation slider now sets the maximum ops per child (1–8, default 3). Tests: `cargo test` 126 passed (13 new, incl. layout identical across sandboxes, advance rule, ops bounds, hof mutated, avg divisor, age, 1/5 bounds), parallel == sequential reference run.

Measurement (`trainer_runner run`, pop 16, 30 gens, mean success of the last 5 gens; spread = SD between seeds; raw data `.agents/artifacts/p7/`):

| Variant (seeds b1..b5) | small | large | runtime |
| :--- | :--- | :--- | :--- |
| P6 state (before) | 0.223 (SD 0.104) | 0.050 (SD 0.051) | — |
| 3 trials, mean | 0.343 (0.106) | 0.194 (0.020) | 1× |
| 3 trials, median | 0.386 (0.105) | 0.133 (0.077) | 1× |
| 5 trials, mean | 0.383 (0.150) | 0.116 (0.022) | ~1.7× |
| 5 trials, median | 0.449 (0.184) | 0.137 (0.098) | ~1.7× |
| 8 trials, mean | 0.473 (0.188) | 0.077 (0.013) | ~2.7× |
| 3 trials, mean, 1/5 rule | 0.443 (0.076) | 0.220 (0.024) | 1× |

Confirmation on 10 seeds (b1..b10): 3 trials mean 0.338 / 0.193 vs. with 1/5 rule 0.444 / 0.260 (difference ≈ 2.4 and 2.1 standard errors). Chosen: **3 trials, mean, 1–3 ops, 1/5 rule on, elite ratio 0.25**. More trials help the small room only within noise and cost runtime; in the large room they lower success. Variance between seeds relative to the mean (CV) drops from 0.47 to 0.22 (small) and from 1.02 to 0.38 (large).

Population size (seeds b1..b3, chosen settings): 16 → 0.444 / 0.260, 100 → 0.333 / 0.300, 300 → 0.521 / 0.297 at ~7.5× wall time (8.2 s vs. 1.1 s per 30 gens; 883k ticks/s at pop 300). Protocol and UI default stay at 16 (each UI sandbox is a visible tile); pop 300 is the option for headless/curriculum training (P8, TCK-176).

Known limit: `layout_id` is not persisted (a restored run starts at layout 0). UI: with the 1/5 rule on, the mutation slider is sent only on its own change and shows the adapted value after each generation (`mutationOpsMax` in `TRAINER_GENERATION_COMPLETED`).

## Post-fix (P8, 2026-10-26): memorisation of held layouts
The P7 measurement above used the training success rate, which is measured on the layouts the population is selected on. With layout policy "hold" a layout is only replaced once every elite solves it; on seeds b1/b2 that never happened in 30 gens, so the population specialised on three fixed layouts (b1 standard: 0.52 on its training layout, 0.02–0.25 on unseen layouts). The runner now evaluates the final population on unseen layouts (`--eval-final standard,exploration --eval-trials 10`, stream `EVAL_STREAM`, TCK-176) and the layout policy is configurable (`trainerLayoutPolicy` hold | rotate, runner `--layout-policy`).

Final population on 10 unseen layouts (pop 16, 30 gens, seeds b1..b10; population / elites):

| Training room, policy, trials | training success (last 5) | unseen small | unseen large |
| :--- | :--- | :--- | :--- |
| small, hold, 3 (P7 default) | 0.438 | 0.149 / 0.170 | 0.039 / 0.048 |
| small, rotate, 3 | 0.232 | 0.126 / 0.193 | 0.026 / 0.035 |
| small, hold, 8 | 0.406 | 0.253 / 0.315 | 0.032 / 0.040 |
| small, rotate, 8 | 0.313 | 0.266 / 0.355 | 0.044 / 0.053 |
| large, hold, 3 | 0.283 | 0.115 / 0.158 | 0.051 / 0.065 |
| large, rotate, 8 | 0.045 | 0.076 / 0.092 | 0.028 / 0.035 |

Corrected decision: **8 trials, layout policy rotate** (a new shared layout every generation): best generalisation in the small room, and the training success then equals performance on unseen layouts instead of overstating it. Shared layouts within a generation stay (fair comparison). The P7 success numbers (small 0.434 / large 0.306) overstate the real capability; from P8 on, epic targets are read on unseen layouts.
