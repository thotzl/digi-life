---
sources: [tickets/closed/TCK-171-training-track-epic.md, tickets/closed/TCK-196-plugin-architecture-epic.md, tickets/closed/TCK-188-targets-adr-and-analytics-skill.md, tickets/closed/TCK-183-trainer-persistence-and-db-reset.md]
---

# ADR-016: Epic Execution Workflow — Master, Phase Commits, Measured Decisions by the Agent

- Status: accepted (recorded retroactively on 2026-11-03)
- Date: 2026-10-21
- Tickets: TCK-171 (execution decisions, user, 2026-10-21), TCK-188 (target values), TCK-183; confirmed and extended in TCK-196 (principles, 2026-11-02)

## Context

The training epic TCK-171 ran eight phases (P1–P8) with many calibration and design choices. Before P2 the user set how the epic is executed (TCK-171 "Execution decisions", 2026-10-21). The plugin-architecture epic TCK-196 restated the process with parallel subagents.

## Decision

1. **Work directly on `master`**; in TCK-171 one commit per phase at the end (bump, CHANGELOG, wiki, tickets) when everything is green. In TCK-196 one patch release per closed sub-ticket.
2. **Run all phases without stops:** the agent performs the per-phase UI smoke checks itself, pushes after each green commit, checks CI and continues; the user is consulted only on real ambiguities or decisions that cannot be taken clearly (TCK-171).
3. **Measurement-based decisions are taken by the agent** and documented with numbers in the ticket (e.g. TCK-181 yes/no, TCK-182 targeted wiring vs. minimal start, TCK-175 generation-1 equipment); the user is consulted only on real target conflicts or targets missed after the time box.
4. **Target values with a documented rule** derived by the agent from the P2 baseline (TCK-188), recorded in TCK-171; calibration loops (thrust, fitness weights, curriculum thresholds, rehearsal share) are time-boxed against them.
5. **Defaults with veto option:** proposed defaults are accepted unless the user vetoes (TCK-144, TCK-145, TCK-152, TCK-153).
6. **Backward compatibility is not required during the epic;** the DB may be emptied on model changes (user, 2026-10-21; TCK-183; persistence rules in ADR-005).
7. **UI bugs found in passing** are fixed in the same phase and listed in its CHANGELOG entry.
8. **Parallel slices** (TCK-196): subagents work in git worktrees on disjoint file sets; the orchestrating agent integrates, runs the full gate and commits.

## Consequences

- Each decision taken under points 3–5 carries its numbers in the ticket's `## Verification`; ADR-008 … ADR-013 record the main ones.
- Phase acceptance: targets met or a deliberate exception documented in the ticket (TCK-171 target rule).
