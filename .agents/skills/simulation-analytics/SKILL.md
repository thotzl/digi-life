---
name: simulation-analytics
description: Rules and procedure for measuring, tuning and balancing the trainer, the Ocean and creature mechanics in BroodBattle with the headless runner (trainer_runner run/probe/ocean), plus read-only DB analysis of the Ocean.
---
# simulation-analytics: Measurement & Balancing Procedure

## I. Knowledge (read first)
- Runner commands, report fields (trainer and Ocean), probes, comparison method, caveats: `wiki/simulation/analytics-and-balancing.md`
- Ocean model: `wiki/simulation/ocean.md`
- Trainer model: `wiki/simulation/trainer.md`; measured phase results P2–P8: `wiki/history/trainer-measurements-p2-p8.md`; rooms and evaluator: `wiki/contracts/room.md`, `wiki/contracts/evaluator.md`
- Target values per phase: `tickets/closed/TCK-171-training-track-epic.md` ("Target values")

## II. Rules
- Measure with `trainer_runner` (`cargo run --release --features runner --bin trainer_runner -- run|probe|ocean ...`, from the repo root), never via the UI or DB queries; the runner stays in memory unless `--persist --db <path>` is given (`ocean` never opens a DB).
- Ocean: `trainer_runner -- ocean --seed <text> --ticks <N> [--sample-every <K>] --out .agents/artifacts/<dir>/<name>.json`; time series of population, births, deaths by cause, species, energy and spores. Restock uses structured random founders (no catalogue), the world comes from `core.seed`. Ocean turnover is slow (≈ 4 deaths per 1,000 ticks, mitosis rare): use ≥ 40,000 ticks and ≥ 5 seeds for Ocean balancing decisions.
- Fixed seeds only: before and after use the same seed set and flags (at least 5 seeds for model changes); compare reports without `perf` and `meta.version`.
- One change at a time; time-box calibration against the TCK-171 targets.
- Verify a config key is read in `src/` before tuning it.
- `perf` numbers only from release builds on the same machine.
- Open the app DB read-only (`sqlite3 -readonly`, `probe --db`); never write to it during analysis.

## III. Procedure
1. Baseline: `run` both rooms (`--scenario standard|exploration`) on the seed set, plus the relevant `probe` (`heredity|speed|equipment|steering|wall|hebbian|all`); keep reports under `.agents/artifacts/`.
2. Apply one change; re-run identical commands.
3. Behaviour-preserving change: reports must be identical (`node tests/reference/compare-trainer-report.mjs a.json b.json`, trainer and Ocean reports). Model change: aggregate per config (mean of last 5 generations, generation 1 for the slope) and compare against the targets.
4. Record before/after numbers in the ticket; update the wiki article with the resulting state when the ticket closes. An intended model change also updates the affected CI references (`tests/reference/trainer-ci.json`, `trainer-ci-curriculum.json`, `trainer-ci-rehearsal.json`, `ocean-ci.json`; commands in `.github/workflows/ci.yml`) in the same commit.
