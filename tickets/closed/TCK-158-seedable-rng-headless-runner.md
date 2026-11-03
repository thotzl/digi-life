---
id: TCK-158
title: Seedable RNG and Headless Runner
status: closed
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-T03, RV-T04, RV-S11
---

# TCK-158: Seedable RNG and Headless Runner

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R3 (after TCK-156). RV-S11 resolved (`cli_trainer` gone); RV-T04 resolved for the trainer, Ocean `thread_rng` (11x) is removed by TCK-156. Remaining scope:
- Headless Ocean run in the runner (`trainer_runner ocean --seed --ticks`) on `OceanSim`.
- Ocean reference report in CI (fixed seed and ticks: population, births/deaths, species, energy statistics), compared like the trainer reports.
- Tests for Ocean mitosis and the warp cap (RV-T02, RV-X02) on the headless Ocean.


> Pool ticket: requires a user decision before implementation. Findings (RV-T03, RV-T04, RV-S11) below.

## Description
`thread_rng()` everywhere; `cli_trainer` simulates an outdated 3-input model against the production DB.

## Proposed Scope
Seeded RNG per simulation; headless runner with JSON report replacing `cli_trainer`.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-T03 · P1** — no headless measurement tool for balancing (RV-S02, S03, S07 need reproducible series: N ticks, fixed seed, population, speed distribution, eating rate, species diversity). Target: `cargo run --bin headless -- --seed X --ticks N --report json`; rewrite the `simulation-analytics` skill on top of it. Requires TCK-156.
- **RV-T04 · P1** — `rand::thread_rng()` at ~20 sites; neither Ocean nor Trainer reproducible; `config.seed` unused. Target: one `StdRng`/`ChaCha8Rng` per simulation, seeded from config, passed as a parameter.
- **RV-S11 · P3** — `src/bin/cli_trainer.rs` feeds `vec![1.0, 0.0, 0.5]` (3-sensor model from before TCK-122) and writes into the production DB; `npm run sim:balance` is meaningless. Replace with the headless runner.

## Verification (2026-11-03, v1.12.38)
- Headless Ocean run: `trainer_runner ocean --seed <s> --ticks <n> [--sample-every K] [--out f]` (feature `runner`) on `OceanSim` (no Tauri, no DB; restock with structured random founders, the app's empty-catalogue branch); sequential Ocean step, deterministic per seed. Report (`src/diagnostics/ocean_report.rs`): `meta` (seed, Ocean seed as hex, world seed, ticks, sample interval, population settings, world size), `samples[]` every K ticks (population, births, mutated births, restocked, deaths by cause, bites, spores eaten/plant/meat, species alive/extinct/discovered, extinctions, mean/median energy, mean age, highest generation), `summary`, `perf`. `OceanEvent::Died` carries `DeathCause { Starved, Aged, Hunted }` (engine log unchanged).
- Ocean reference in CI: step "ocean reference run" `--seed ci-ocean-2 --ticks 12000 --sample-every 500` vs `tests/reference/ocean-ci.json` (14.6 KB, 25 samples, about 2.8 s release; 2 mitosis births, 15 bites, deaths 43 starved / 3 aged / 3 hunted). `tests/reference/compare-trainer-report.mjs` compares any runner report (ignores `perf`, `meta.version`); a fresh run matches, another seed fails with 299 differing values.
- Tests: `sim::ocean::tests::test_mitosis_splits_a_ready_parent_and_reports_the_child_without_simulating_it` (RV-T02; child reported in `born`, not simulated, RV-B01 open in TCK-143), `diagnostics::ocean_report::tests::test_ocean_report_is_reproducible_per_seed_and_samples_every_k_ticks`; the warp cap (RV-X02) is covered by `test_warp_is_clamped_and_echoed_for_ocean_and_trainer` (TCK-156). RV-T04 resolved (seeded Ocean, TCK-156), RV-T03 resolved.
- Gate on master (`.agents/artifacts/gate.sh`, now incl. the Ocean reference): `cargo test` 279 passed, 1 ignored; golden master `0x6b78c31a4d19b0e8` unchanged; standard, curriculum, rehearsal and Ocean references match; `npm test` 41 passed; build ok; doc links OK; no bindings drift.
- Risk: the Ocean reference was generated on Windows and CI runs on Ubuntu; the Ocean still uses std `sin`/`cos` (clock input `ClockVariant::Std`, mitosis recoil, world generator). If CI fails only on the Ocean reference step, the `fmath` switch (TCK-202 item 4) or a reference regenerated on Linux resolves it. CI results could not be checked from this environment (`gh` not authenticated).
- First Ocean numbers (5 seeds, 12k-40k ticks): population at the target 25 throughout, 4.1-4.4 deaths per 1,000 ticks (84-92 % starved), 0-5 mitosis births per 40k ticks.
- Docs: analytics-and-balancing ("Ocean run"), ci-and-tooling (command, Ocean and rehearsal CI steps), testing, ocean; skill `simulation-analytics`.
