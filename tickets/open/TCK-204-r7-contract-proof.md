---
id: TCK-204
title: "R7 Proof: a Fresh Agent Adds a Plugin from Guide and Contract Only"
status: open
priority: medium
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R7
depends_on: TCK-202, TCK-203
---

# TCK-204: Contract Proof

Acceptance test of the epic's goal "a feature needs only its interfaces".

## Scope
1. Three small, throw-away-safe probes, each by a fresh subagent that may read only `AGENTS.md`, the `plugin-authoring` skill, the matching guide, its contract and `invariants.md` (plus the files the guide names):
   - an organ kind (e.g. a test-only "dummy sensor" with one channel),
   - a room (e.g. a level with one reef obstacle),
   - a view (e.g. an empty "Arena" placeholder in the menu).
2. Measure per probe: files read, files changed, lines changed, tests the agent ran, questions it had to ask. Target: changes only in the plugin module + its registry entry (+ config section / binding regeneration where the contract says so).
3. Fix docs or seams where a probe needed more context; repeat until the targets hold.
4. Probe code is not merged unless the user wants a probe as a real feature.

## Acceptance
- Report table in the verification; all three probes within the target.
- Findings turned into doc/seam fixes or tickets.
