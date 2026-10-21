---
id: TCK-155
title: Typed Enums for Model Strings
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-S10
---

# TCK-155: Typed Enums for Model Strings

> Pool ticket: requires a user decision before implementation. Findings (RV-S10) below.

## Description
Activation type, organ type, symmetry, scenario, origin and diet class are free-form strings.

## Proposed Scope
Replace with enums exported via ts-rs.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-S10 · P3** — model semantics in free-text strings: activation (`Option<String>` "relu"/"sigmoid"/...), organ type (emoji label), symmetry (`"vertical"/"quad"`), scenario (`"standard"/"exploration"`), origin (`"elite"/"hof"/...`), diet class. A typo silently falls back to the default. Target: enums with ts-rs export.
