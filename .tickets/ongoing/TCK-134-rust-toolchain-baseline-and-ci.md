---
id: TCK-134
title: Rust Toolchain Baseline and Verification CI
status: ongoing
assigned: Claude Code
created_at: 2026-10-19
revisit: RV-T05, RV-T01
---

# TCK-134: Rust Toolchain Baseline and Verification CI

## Description
No pipeline verifies Rust or TypeScript before `release.yml` builds draft installers on every push to `master`. The Windows dev machine had no Rust toolchain.

## Scope
- Install rustup (stable-msvc) locally; record `cargo test` / `cargo clippy` baseline.
- Add `.github/workflows/ci.yml`: `cargo test`, `npm ci && npm run build`, bindings drift check.
- Restrict `release.yml` to `v*` tags and manual dispatch.

## Acceptance
- CI workflow file valid; `cargo test` and `npm run build` green locally.
- No behaviour change in the app.
