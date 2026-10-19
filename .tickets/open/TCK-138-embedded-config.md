---
id: TCK-138
title: Embed config.json as Single Source of Truth
status: open
assigned: Claude Code
created_at: 2026-10-19
revisit: RV-B04
---

# TCK-138: Embed config.json as Single Source of Truth

## Description
`AppConfig::load` searches `config.json` relative to the CWD; release bundles do not ship it, and the hand-written fallback diverges from `config.json` (food 600 vs 300, thrust 6.8 vs 3.4, max age 5400 vs 36000). Parse errors silently fall back.

## Scope
- Compile `config.json` into the binary via `include_str!`; remove the divergent fallback.
- Optional on-disk override (`config.json` in the CWD) for development; parse errors are logged loudly.
- Test: the embedded config parses.

## Acceptance
- Dev and release use identical values. No behaviour change in dev.
