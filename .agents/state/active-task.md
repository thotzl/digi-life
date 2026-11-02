# Active task

Last updated: 2026-11-02. Handover note, not an instruction; tickets win.

## Goal
Epic TCK-196 (plugin architecture rebuild), user GO 2026-11-02 "push und go - mach das alles": run all phases R0–R7 autonomously, full close per sub-ticket (version, CHANGELOG, wiki/contract, verification), ask only on ambiguity or if TCK-202 misses the training targets.

## How it runs
- Orchestrator integrates on master; independent slices run as subagents in git worktrees (disjoint files), orchestrator merges, runs the full gate, bumps version, writes CHANGELOG, closes the ticket, commits, pushes.
- Gate per slice (R1–R4): `cargo test` (golden master), CI trainer references (`tests/reference/*.json` via `compare-trainer-report.mjs`), `npm test`, `npm run build`; UI smoke via `.agents/artifacts/uicheck/run-smoke.sh` for UI-relevant slices.
- Order: R0 (TCK-197 ∥ TCK-189) → R1 (TCK-198, then TCK-159 ∥ TCK-157) → R2 (TCK-199 ∥ TCK-160 part 1) → R3 (TCK-200 → TCK-156 ∥ TCK-201 → TCK-158) → R4 (TCK-160 rest, TCK-161) → R5 (TCK-202 ∥ TCK-143) → R6 (TCK-203) → R7 (TCK-204).

## Progress
- [ ] R0 TCK-197, TCK-189
- [ ] R1 TCK-198, TCK-159, TCK-157
- [ ] R2 TCK-199
- [ ] R3 TCK-200, TCK-156, TCK-201, TCK-158
- [ ] R4 TCK-160, TCK-161
- [ ] R5 TCK-202, TCK-143
- [ ] R6 TCK-203
- [ ] R7 TCK-204

## Next step
R0, interrupted 2026-11-02 by the usage limit. Both R0 subagents were stopped before committing; their uncommitted work is in the worktrees:
- TCK-197: `.claude/worktrees/agent-afed439a00582af26` (skills, AGENTS.md, README, ci.yml, styled.ts + XSS fix, Icons.ts, TrainerView.ts, UnifiedDiagnosticsPanel.ts, dna.rs golden master, ...). Gate state unknown; review the diff, finish the scope, run the full gate.
- TCK-189: `.claude/worktrees/agent-a00f3e292d6206ad2` (package.json, package-lock.json edited; it was about to run `npm ci`, audit, test, build, cargo audit). `coverage/` there is a stray output.
Resume: review each diff, finish and gate it (or redo the ticket), then merge to master, version bump, CHANGELOG, close the ticket, push; then R1.
