---
name: plugin-authoring
description: Entry point for adding a feature to BroodBattle at one of its extension points - organ kind, creature system/dynamic, room, evaluator, engine mode, event sink, IPC message, config key/section, persistence table/repository, frontend view. Read this first for any such feature.
---
# plugin-authoring: Add a Feature at an Extension Point

## I. What to read
Read exactly three documents: the guide for your extension point, its contract, and `wiki/architecture/invariants.md`
(plus section III of this skill, the gate).
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
- One registration point per plugin, as the contract names it; no edits to the host beyond that entry (and the
  expected host edits a guide names, e.g. the main-menu launcher).
- The guide's "Done when" is the gate; the contract's test recipe is the targeted subset to run while working.
- Same commit: the extension point's row in "Registered plugins" of `wiki/architecture/module-map.md` (the one list
  of registered plugins), the contract's Purpose list, and the docs the guide's last step names.
- If the guide or contract disagrees with the code, the code wins: fix the doc in the same commit.
- A missing seam (the feature needs a host change no contract covers) is a stop: report it instead of wiring it in.
- With a ticket: close it with the checklist in `.agents/skills/project-baseline/SKILL.md`. Without a ticket: skip
  the checklist (no ticket move, version bump or `CHANGELOG.md` entry unless the user asks); state the change, and a
  model change (golden master, references, `MODEL_VERSION`), in the commit message.

## III. Gate
Every guide's "Done when" ends with this list; all of it is green before the commit (CI runs the same steps):
- `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`, `cargo test`
- `npm test`, `npm run lint`, `npm run build` (also for a Rust-only feature), `node scripts/check-links.mjs`
- the four CI reference runs of `.github/workflows/ci.yml`, each followed by
  `node tests/reference/compare-trainer-report.mjs tests/reference/<name>.json target/<name>.json`:

```sh
cargo run --release --features runner --bin trainer_runner -- run --seed ci-reference --generations 3 --scenario standard --population 16 --parallel true --out target/trainer-ci.json
cargo run --release --features runner --bin trainer_runner -- run --seed ci-curriculum --generations 4 --scenario curriculum --population 16 --rehearsal 0 --curriculum-upper 0.05 --curriculum-lower 0.02 --eval-final standard,exploration --eval-trials 3 --parallel true --out target/trainer-ci-curriculum.json
cargo run --release --features runner --bin trainer_runner -- run --seed ci-rehearsal --generations 4 --scenario curriculum --population 16 --rehearsal 0.25 --curriculum-upper 0.05 --curriculum-lower 0.02 --parallel true --out target/trainer-ci-rehearsal.json
cargo run --release --features runner --bin trainer_runner -- ocean --seed ci-ocean-2 --ticks 12000 --sample-every 500 --out target/ocean-ci.json
```

An intended model change (invariants 8, 9) copies each differing `target/<name>.json` over
`tests/reference/<name>.json` (not every reference necessarily changes), sets `GOLDEN_FINGERPRINT` in
`src/biology/phenotype/tests.rs` to the value `cargo test test_parse_genome_golden_master -- --nocapture` prints, and
bumps `MODEL_VERSION` in `src/persistence/schema.rs` whenever the golden master changes. Otherwise all stay unchanged.
