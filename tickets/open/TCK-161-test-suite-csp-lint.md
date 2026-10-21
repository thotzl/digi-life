---
id: TCK-161
title: Behavioural Test Suite, CSP and Lint Baseline
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-T02, RV-F02 (CSP), RV-T05
---

# TCK-161: Behavioural Test Suite, CSP and Lint Baseline

> Pool ticket: requires a user decision before implementation. Findings (RV-T02, RV-F02 (CSP), RV-T05) below.

## Description
Few behavioural tests; `csp: null`; 69 clippy warnings; outdated rusqlite/rand.

## Proposed Scope
Behavioural tests per RV-T02 list, restrictive CSP (verify inline styles), clippy clean-up, dependency updates after green baseline.

## Findings (review 2026-10-19, base commit `998244e`)
- **RV-T02 · P1** — tests check implementation details: `test_execute_brain_determinism` fed 4 inputs into a ≥ 7-input brain; DB tests test SQLite itself. Behavioural tests to add: (1) Ocean step: after mitosis `creatures.len()` +1 and the child moves next step; (2) brain: two independent runs bit-identical, output range per activation; (3) config: every key deserializes, no unknown keys; (4) `proptest`: `parse_genome` never panics on ASCII 0..1024 chars, `mutate_genome` keeps length in [128, 512]; (5) warp cap 35 as constant + test. Stryker only once frontend tests exist.
- **RV-F02 (CSP) · P1** — `tauri.conf.json` `"csp": null`; escaping done in TCK-141. Target: `default-src 'self'`. Relevant once genomes/catalogue entries are imported or shared.
- **RV-F07 (rest) · P3** — `concurrently`, `tsx` unused; Stryker configured without tests; `ActiveView` contains `"creator"` without a view.
- **RV-T05 (rest) · P2** — `Cargo.toml` package name `src-tauri`; `rusqlite 0.29`, `rand 0.8` outdated (update only after a green baseline); clippy baseline 69 warnings (TCK-134).
