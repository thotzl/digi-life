---
id: TCK-142
title: Sync Docs, Agent Skills and Ticket Status with Actual Architecture
status: open
assigned: Claude Code
created_at: 2026-10-19
closed_at: 2026-10-19
revisit: RV-X01, RV-X02, RV-X03, RV-X04, RV-X05
---

# TCK-142: Sync Docs, Agent Skills and Ticket Status

## Description
Project skills, README, HANDOFF and active-task describe a retired architecture (WebSocket, JSON DB files, `src-tauri/`). Several tickets carry the wrong status.

## Scope
- Rewrite `.agents/skills/*` and the README to the actual Tauri/Rust/SQLite architecture.
- Refresh `.agents/state/active-task.md` and `HANDOFF.md`; mark the WHITEPAPER as vision.
- Fix ticket status (TCK-123/124/125) and stale `.gitignore`/`.aiignore` entries.
- Tick completed items in `docs/revisit/`.

## Acceptance
- No reference to non-existent files or architecture in the agent context.

## Verification
- Skills, README, HANDOFF, WHITEPAPER, active-task rewritten to the actual architecture (subagent, reviewed and updated for TCK-137/139/140/162).
- Tickets TCK-124/125 closed, TCK-123 ongoing; .gitignore/.aiignore cleaned.
- docs/revisit: every RV-* finding carries a status (done commit or pool ticket); new finding RV-B11.
- Smoke test: `npm run tauri:dev` starts, DB in AppData (WAL), config loaded, handshake OK, 0 errors in log.
- Second app start restored 15 creatures from the AppData database; Trainer grid built (16 chambers); log still free of errors.
