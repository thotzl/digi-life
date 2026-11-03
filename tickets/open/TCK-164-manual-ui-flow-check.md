---
id: TCK-164
title: Manual UI Flow Check (Ocean, Trainer, Catalogue)
status: open
priority: medium
assigned: unassigned
created_at: 2026-10-20
---

# TCK-164: Manual UI Flow Check (Ocean, Trainer, Catalogue)

## Update (2026-11-03, after TCK-196)
- The user tested the installed alpha v1.12.44 extensively on 2026-11-03 (no defects reported). Automated UI checks now cover large parts of the scope (local agent tooling: smoke for Ocean/inject/trainer/pause/reset, tile selection and reset, catalogue view, save to catalogue, Ocean and trainer catalogue dialogs, navigation teardown). Close once the user confirms that the listed flows were covered; record the result in `wiki/quality/runtime-verification.md`.

## Description
The Phase 0 smoke tests (TCK-142, 2026-10-19) confirmed app start, AppData DB with WAL, config load, handshake and state restore, but the UI flows were not clicked through by the user.

## Scope
- Ocean: start/pause, warp slider, selection and diagnostics panel, reset evolution.
- Trainer: start/pause/reset, sandbox selection, hyperparameters, Lamarckian toggle, HoF.
- Catalogue: save from Ocean/Trainer, rename, delete, spawn to Ocean, add to training, preview.

## Acceptance
- Each flow checked by the user; defects filed as tickets. Result recorded in `wiki/quality/runtime-verification.md`.
