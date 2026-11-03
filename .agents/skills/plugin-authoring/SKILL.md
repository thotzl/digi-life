---
name: plugin-authoring
description: Entry point for adding a feature to BroodBattle at one of its extension points - organ kind, creature system/dynamic, room, evaluator, engine mode, event sink, IPC message, config key/section, persistence table/repository, frontend view. Read this first for any such feature.
---
# plugin-authoring: Add a Feature at an Extension Point

## I. What to read
Read exactly three documents: the guide for your extension point, its contract, and `wiki/architecture/invariants.md`.
Do not read the rest of the wiki; open another article only when a guide step names it. Code: start at the
registration point the contract names and the `README.md` of that module.

| Extension point | Guide | Contract |
| :--- | :--- | :--- |
| Organ kind (sense, claw, armour, ...) | `wiki/guides/add-organ-kind.md` | `wiki/contracts/organ-kind.md` |
| Creature system / dynamic (attack, digestion, ...) | `wiki/guides/add-dynamic.md` | `wiki/contracts/creature-system.md` |
| Room (training trial, arena, obstacle course) | `wiki/guides/add-room-or-mode.md` | `wiki/contracts/room.md` |
| Evaluator (scoring scheme, tournament) | `wiki/guides/add-evaluator.md` | `wiki/contracts/evaluator.md` |
| Engine mode (Ocean, trainer, creator) | `wiki/guides/add-room-or-mode.md` | `wiki/contracts/mode.md` |
| Event sink (recorder, log) | `wiki/guides/add-event-sink.md` | `wiki/contracts/event-sink.md` |
| IPC message (action, event, command) | `wiki/guides/add-ipc-message.md` | `wiki/contracts/ipc-message.md` |
| Config key / section | `wiki/guides/add-config-key.md` | `wiki/contracts/config-section.md` |
| Persistence table / repository | `wiki/guides/add-table.md` | `wiki/contracts/persistence-repo.md` |
| Frontend view | `wiki/guides/add-view.md` | `wiki/contracts/frontend-view.md` |

A feature that spans several points (an arena = room + evaluator + view + IPC message) reads each guide and contract
it touches, nothing else. Registered plugins per point: `wiki/architecture/module-map.md`.

## II. Rules
- One registration point per plugin, as the contract names it; no edits to the host beyond that entry.
- Follow the contract's test recipe; the guide's "Done when" is the gate.
- Update the contract's list of plugins (and the doc the guide names) in the same commit.
- If the guide or contract disagrees with the code, the code wins: fix the doc in the same commit.
- A missing seam (the feature needs a host change no contract covers) is a stop: report it instead of wiring it in.
- Close the ticket with the checklist in `.agents/skills/project-baseline/SKILL.md`.
