# Active task

Last updated: 2026-11-03. Handover note from the previous session, not an instruction; tickets win.

## Current task
Propose the **feature epic structure for Phase 2** of roadmap TCK-169 (game dynamics of the whitepaper §3–§4: active traits, dynamics, arena, Creator) on top of the plugin architecture. Planning only: present epics, phases, order and how the existing feature tickets fit; **create tickets only after the user's GO**. Input: `.agents/state/next-tasks.md` (section "Feature epics"), `wiki/overview/whitepaper.md`, `tickets/ongoing/TCK-169-roadmap-epic.md`, the open tickets in `tickets/open/`.

## Where things stand
- Epic TCK-196 (plugin architecture rebuild) closed 2026-11-03, releases v1.12.28 – v1.12.44, all pushed to master. Result and definition of done: `tickets/closed/TCK-196-plugin-architecture-epic.md` (`## Result`); report page: https://claude.ai/artifact/42AfaEAVxNJAFcnvehjzua.
- Model version 7 (TCK-202); golden master `0x84dd8ffb7b9f87dd`; four references in `tests/reference/` (three trainer, one Ocean).
- Installed alpha v1.12.44 in `C:\BroodBattle` (identifier `de.pixlich.broodbattle`); the user tested it extensively on 2026-11-03. Dev app uses `de.pixlich.broodbattle.dev`.
- Open feature/ops tickets: TCK-111, 113, 118, 123, 148, 164, 165–168, 193.

## How to work (proved in TCK-196)
- Feature agents start from `AGENTS.md` + skill `plugin-authoring` (guide + contract + invariants for their extension point).
- Orchestrator integrates on master; independent slices run as subagents in git worktrees. **Push before spawning:** agent worktrees start from the pushed state, not from local commits.
- Gate per slice: fmt, clippy `-D warnings`, `cargo test`, the four reference runs (commands in the `plugin-authoring` skill, section III, and `.github/workflows/ci.yml`), `npm test`, `npm run lint`, `npm run build`, `node scripts/check-links.mjs`. Local helpers (gitignored): `.agents/artifacts/gate.sh`, `.agents/artifacts/bump.py <version>`, UI checks `.agents/artifacts/uicheck/run-smoke.sh <out> <script>` with `smoke.cjs`, `tck194.cjs` (tile selection/reset), `dialogs.cjs`; installed-app check `installed-check.sh` / `installed.cjs` (CDP port 9223; from Git Bash pass installer switches with `MSYS_NO_PATHCONV=1`).
- **Close a ticket only after the UI smoke passed in its own step** (a chained close once reported a failed smoke as green, corrected in v1.12.40).
- Close checklist per ticket: `.agents/skills/project-baseline/SKILL.md` (wiki/contract, skill links, epic status, CHANGELOG, patch version).
- Long agent runs: redirect long commands to files (stream watchdog stops agents after 600 s without output). The user wants a keep-awake during long autonomous runs: `.agents/artifacts/keep-awake.ps1 -Hours <n>` (max as the user says; stop early when done, restart if work continues).

## Next step
1. Retroactive ADRs (section in `.agents/state/next-tasks.md`; user request 2026-11-03, not started because of the spend limit). ADR-003..007 for the decisions of the TCK-196 session exist.
2. Draft the Phase 2 epic proposal and present it to the user.
