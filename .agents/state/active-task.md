# Active task

Last updated: 2026-11-02. Handover note, not an instruction; tickets win.

## Goal
Epic TCK-196 (plugin architecture rebuild), user GO 2026-11-02 "push und go - mach das alles": run all phases R0–R7 autonomously, full close per sub-ticket (version, CHANGELOG, wiki/contract, verification), ask only on ambiguity or if TCK-202 misses the training targets.

## How it runs
- Orchestrator integrates on master; independent slices run as subagents in git worktrees (disjoint files), orchestrator merges, runs the full gate, bumps version, writes CHANGELOG, closes the ticket, commits, pushes.
- Gate per slice (R1–R4): `cargo test` (golden master), CI trainer references (`tests/reference/*.json` via `compare-trainer-report.mjs`), `npm test`, `npm run build`; UI smoke via `.agents/artifacts/uicheck/run-smoke.sh` for UI-relevant slices.
- Order: R0 (TCK-197 ∥ TCK-189) → R1 (TCK-198, then TCK-159 ∥ TCK-157) → R2 (TCK-199 ∥ TCK-160 part 1) → R3 (TCK-200 → TCK-156 ∥ TCK-201 → TCK-158) → R4 (TCK-160 rest, TCK-161) → R5 (TCK-202 ∥ TCK-143) → R6 (TCK-203) → R7 (TCK-204).

## Progress
- [x] R0 TCK-197 (v1.12.29), TCK-189 (v1.12.28)
- [ ] R1 TCK-198 (v1.12.30), TCK-159 (v1.12.31) done; TCK-157 next
- [x] R2 TCK-199 (v1.12.32, absorbs TCK-155)
- [ ] R3 TCK-200, TCK-156, TCK-201, TCK-158
- [ ] R4 TCK-160, TCK-161
- [ ] R5 TCK-202, TCK-143
- [ ] R6 TCK-203
- [ ] R7 TCK-204

## Next step
R1: TCK-159 and TCK-157 in parallel worktrees (both touch api.rs/engine.rs: 159 owns SQL/repositories, 157 owns action/event types and api split; merge 159 first). Gate: `bash .agents/artifacts/gate.sh`. Version helper: `python .agents/artifacts/bump.py <version>`.

## Sleep blocker
`.agents/artifacts/keep-awake.ps1` (user request: max 4 h, stop early when done, restart if work continues). Current run started 2026-11-02 20:39, ends 00:39.
