---
name: simulation-analytics
description: Procedure for analyzing evolutionary drift, species lineage, population dynamics and ecosystem balancing in Pixel DNA Life.
---
# simulation-analytics: Analysis & Balancing Procedure

## I. Knowledge (read first)
- Data sources, caveats, heuristics: `wiki/simulation/analytics-and-balancing.md`
- Schema: `wiki/architecture/persistence.md`

## II. Procedure
1. Open the DB read-only (`sqlite3 -readonly`) while the app runs; never write to it during analysis.
2. Check the caveats in the wiki article before drawing conclusions (no Ocean offspring, species = genome, dead config keys).
3. Verify a config key is read in `src/` before tuning it.
4. Change one constant at a time; record before/after series in `wiki/simulation/analytics-and-balancing.md` (or a new article in `wiki/simulation/`).
5. Do not use `src/bin/cli_trainer.rs` for balancing (RV-S11).
