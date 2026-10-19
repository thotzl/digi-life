---
id: TCK-138
title: Embed config.json as Single Source of Truth
status: closed
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
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

## Verification
- Test `shared::types::tests::test_embedded_config_matches_config_json` (`src/shared/types.rs`) did not compile before the change (no embedded config existed); now asserts that the embedded config equals `config.json` (food spores, population, max age, thrust multiplier).
- `src/shared/types.rs`: `config.json` compiled in via `include_str!`; a `config.json` in the working directory still overrides it (dev behaviour unchanged, `tauri dev` runs from the repo root); an invalid override is logged as an error and the embedded config is used. The hand-written fallback (food 600, thrust 6.8, max age 5400) is removed.
- `cargo test`: 37 passed, 0 failed.
- Effect: release builds now run with the same values as development.
