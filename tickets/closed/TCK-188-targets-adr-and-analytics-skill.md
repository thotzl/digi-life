---
id: TCK-188
title: Epic Target Values, ADR-002 Brain v2 and Analytics Skill
status: closed
priority: medium
assigned: unassigned
created_at: 2026-10-21
epic_phase: Training Track
depends_on: TCK-173
---

# TCK-188: Epic Target Values, ADR-002 Brain v2 and Analytics Skill

Part of epic TCK-171. Target values and skill at the end of phase P2, ADR-002 in phase P3.

## Scope
- **Target values** (after the P2 baseline), recorded in TCK-171 and used as phase acceptance: success rate small / large room, speed spread (p90/p10), share of brain + senses in resting cost (10–20 %), ticks per second at population 300, parent-child correlation after 1 / 5 / 15 mutations. Every tuning step (thrust, fitness weights, curriculum thresholds, rehearsal share) is time-boxed against these targets.
- **ADR-002 Brain v2** in `wiki/decisions/`: locality and stable synapse identity, topology compaction, ΔI channel, evaluation order, genome operators, activity-based metabolic cost; alternatives considered and research sources (TCK-171).
- **Analytics skill:** rewrite `.agents/skills/simulation-analytics/SKILL.md` and `wiki/simulation/analytics-and-balancing.md` on top of the headless runner instead of DB analysis and `cli_trainer` (RV-T03, RV-S11).

## Acceptance
- Targets in TCK-171; ADR-002 linked from `wiki/_master-index.md`; skill references the runner commands.

## Progress P2 (v1.12.16)
- Target values set from the P2 baseline and recorded in TCK-171 "Target values" (rule documented there).
- Analytics skill and `wiki/simulation/analytics-and-balancing.md` rewritten on top of the runner (RV-T03, RV-S11).
- Remaining for P3: ADR-002 Brain v2.

## Verification (P3, v1.12.17)
- Targets: TCK-171 "Target values" (P2). Analytics skill and `wiki/simulation/analytics-and-balancing.md` on top of the runner (P2).
- ADR-002 Brain v2: `wiki/decisions/adr-002-brain-v2.md`, linked from `wiki/_master-index.md`.
