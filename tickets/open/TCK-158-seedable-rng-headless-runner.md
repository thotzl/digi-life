---
id: TCK-158
title: Seedable RNG and Headless Runner
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-T03, RV-T04, RV-S11
---

# TCK-158: Seedable RNG and Headless Runner

> Pool ticket: requires a user decision before implementation. Findings (RV-T03, RV-T04, RV-S11) below.

## Description
`thread_rng()` everywhere; `cli_trainer` simulates an outdated 3-input model against the production DB.

## Proposed Scope
Seeded RNG per simulation; headless runner with JSON report replacing `cli_trainer`.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-T03 · P1** — no headless measurement tool for balancing (RV-S02, S03, S07 need reproducible series: N ticks, fixed seed, population, speed distribution, eating rate, species diversity). Target: `cargo run --bin headless -- --seed X --ticks N --report json`; rewrite the `simulation-analytics` skill on top of it. Requires TCK-156.
- **RV-T04 · P1** — `rand::thread_rng()` at ~20 sites; neither Ocean nor Trainer reproducible; `config.seed` unused. Target: one `StdRng`/`ChaCha8Rng` per simulation, seeded from config, passed as a parameter.
- **RV-S11 · P3** — `src/bin/cli_trainer.rs` feeds `vec![1.0, 0.0, 0.5]` (3-sensor model from before TCK-122) and writes into the production DB; `npm run sim:balance` is meaningless. Replace with the headless runner.
