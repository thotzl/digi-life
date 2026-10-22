---
name: simulation-analytics
description: Rules and procedure for measuring, tuning and balancing the trainer and creature mechanics in Pixel DNA Life with the headless runner (trainer_runner run/probe), plus read-only DB analysis of the Ocean.
---
# simulation-analytics: Measurement & Balancing Procedure

## I. Knowledge (read first)
- Runner commands, report fields, probes, comparison method, caveats: `wiki/simulation/analytics-and-balancing.md`
- Trainer model and P2 baseline: `wiki/simulation/trainer.md`
- Target values per phase: `tickets/ongoing/TCK-171-training-track-epic.md` ("Target values")

## II. Rules
- Measure with `trainer_runner` (`cargo run --release --bin trainer_runner -- run|probe ...`, from the repo root), never via the UI or DB queries; the runner stays in memory unless `--persist --db <path>` is given.
- Fixed seeds only: before and after use the same seed set and flags (at least 5 seeds for model changes); compare reports without `perf` and `meta.version`.
- One change at a time; time-box calibration against the TCK-171 targets.
- Verify a config key is read in `src/` before tuning it.
- `perf` numbers only from release builds on the same machine.
- Open the app DB read-only (`sqlite3 -readonly`, `probe --db`); never write to it during analysis.

## III. Procedure
1. Baseline: `run` both rooms (`--scenario standard|exploration`) on the seed set, plus the relevant `probe` (`heredity|speed|equipment|steering|wall|hebbian|all`); keep reports under `.agents/artifacts/`.
2. Apply one change; re-run identical commands.
3. Behaviour-preserving change: reports must be identical (`node tests/reference/compare-trainer-report.mjs a.json b.json`). Model change: aggregate per config (mean of last 5 generations, generation 1 for the slope) and compare against the targets.
4. Record before/after numbers in the ticket; update the wiki article with the resulting state when the ticket closes. An intended model change also updates `tests/reference/trainer-ci.json` in the same commit.
