---
id: TCK-204
title: "R7 Proof: a Fresh Agent Adds a Plugin from Guide and Contract Only"
status: closed
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

## Verification (2026-11-03, v1.12.44)
Fresh agents, each in a throw-away worktree, started only from `AGENTS.md` and the `plugin-authoring` skill and followed the guide, contract and invariants; logs in `.agents/artifacts/tck204/` (not merged, as planned).

| Probe | Files read | Changed (host code outside the plugin) | Tests | Result |
| :--- | :--- | :--- | :--- | :--- |
| Organ kind "dummy sensor" (round 1) | 16 | 11 (registry `organs/mod.rs`, painter row in `render/organs.ts`; bindings regenerated, golden master + 1 reference + `MODEL_VERSION` as the contract requires) | all green | built; 8 doc gaps (biggest: generator constraint on the Eye/Nose regions) |
| Room "reef" (round 1) | 23 (~10 not named by the docs) | 10 (registry entry, `mod` line, config block) | all green, references unchanged | trains around the reef (success 0.00 -> 0.45 in 40 gens); 10 doc gaps (obstacle model, test recipes, example against invariant 11) |
| View "Arena" (round 1) | 13 | 8 (registry entry, launcher in the main menu) | all green | built; 8 doc gaps (`backendMode` for a view without simulation, launcher grid) |
| Room "slalom" (round 2, fixed docs, base 172942c) | 17 (2 not named by the docs) | 10 (registry entry, `mod` line, config block) | all green, four references unchanged | low effort (~30 min), contract snippets compiled as given; 5 small gaps |

- Code target met in every probe: changes only in the plugin module, its registry entry and what the contract names (config section, regenerated bindings, golden master/references for a model-visible organ).
- Doc gaps from round 1 (26 in total) fixed in `172942c` (contracts organ-kind, room, frontend-view; guides organ, room/mode, view and the shared gate line in all guides; skill section III "Gate"; ticket-less work rule). Round 2 gaps fixed on close: obstacle construction example and "obstacles fixed per room" in the room contract, room config-key convention and the two missing doc spots in the room guide.
- A process finding on the way: agent worktrees start from the pushed state, so a probe launched before pushing the doc fixes measured the old docs (it reproduced the same gaps independently); rerun after the push.
- Seams that stay host changes, recorded in `.agents/state/next-tasks.md`: polygon obstacles, room-specific respawn / per-trial obstacles, obstacles in trainer telemetry, a public room test helper, `backendMode: null` for views without a simulation.
