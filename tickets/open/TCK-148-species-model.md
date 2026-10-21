---
id: TCK-148
title: Species Model: Distance-Based Speciation
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B10
---

# TCK-148: Species Model: Distance-Based Speciation

> Pool ticket: requires a user decision before implementation. Findings (RV-B10) below.

## Description
Species identity equals the full genome string; latin names are UNIQUE and collide silently.

## Proposed Scope
Design distance-based speciation, numeric species ids, non-unique names.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B10 · P2 · static** — `species_id: genome.clone()` everywhere in `engine.rs`; `latin_name ... UNIQUE` in `database.rs`. Every point mutation creates a new species. The Latin name has only ~6,760 strain combinations per prefix/suffix; collisions make `INSERT OR IGNORE` silently drop the species. The predation rule "do not bite the same species" only applies to exact clones.
- Target: speciation by genetic distance to the founder, species ID as UUID/counter, Latin name not UNIQUE. Design decision (user), Phase 1.
