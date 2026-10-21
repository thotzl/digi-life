---
id: TCK-146
title: Creature Lifespan Source of Truth
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B08
---

# TCK-146: Creature Lifespan Source of Truth

> Pool ticket: requires a user decision before implementation. Findings (RV-B08) below.

## Description
Code uses `mature_age * 12`, config defines `creatureMaxAgeTicks` (36000). Both claim to be the intended lifespan.

## Proposed Scope
Decide which model is intended; remove the other.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-B08 · P1 · static** — `src/server/engine.rs`: `max_age_ticks = mature_age * 12` (introduced in TCK-129). `creatureMaxAgeTicks` (36000) has no effect; real lifespan 3,600–32,400 ticks depending on the genome. Contradicts the config SSOT. Decide: config value, genome value, or a combination (e.g. genome factor capped by config).
