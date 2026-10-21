# Handover: Implement Epic TCK-171 (Training Track)

Last updated: 2026-10-21. Handover note, not an instruction; the user's request defines the session task. If this note contradicts the tickets, the tickets win.

## Task
Implement epic **TCK-171** with all its tickets, phase by phase (P1 → P8). The user decided on 2026-10-21 that the whole epic is to be implemented.

## Goal (user)
Training that works: creatures that really develop and improve across rooms, so that they can later survive in the Ocean. Two rooms for now (small 1000 px `standard`, large 3500 px `exploration`); further rooms are decided once the setup is sound and safe (TCK-123).

## Where to find what
| What | Where |
| :--- | :--- |
| Phases, order, goal, research summary, DoD | `tickets/ongoing/TCK-171-training-track-epic.md` |
| Placement in the roadmap (Phase 1a, before the rest of Phase 1) | `tickets/ongoing/TCK-169-roadmap-epic.md` |
| Per-ticket scope, code locations, acceptance | `tickets/open/TCK-1xx-*.md` (all with `epic_phase: Training Track`) |
| Current state of the code and model (no open work) | `wiki/biology/ctrnn-brain.md`, `wiki/biology/genome-compiler.md`, `wiki/simulation/physics.md`, `wiki/simulation/trainer.md`, `wiki/architecture/performance.md`, `wiki/biology/metabolism-and-reproduction.md` |
| Literature behind the decisions | `wiki/research/neuroevolution-and-training-literature.md` |
| Invariants that must not regress | `wiki/architecture/invariants.md` |
| Agent rules | `.agents/skills/` (`project-baseline`, `server-simulation`, `biology-dna`, `testing-strategies`, `simulation-analytics`) |
| Speed estimate script (local only, gitignored) | `.agents/artifacts/vterm.py`; formula and assumptions also in TCK-172 |

## Phases (detail in TCK-171)
| Phase | Tickets |
| :--- | :--- |
| P1 Config | TCK-149 |
| P2 Sim lab | TCK-173, 172, 185, 183 (runner/retention), 186 (CI), 188 (targets, analytics skill) |
| P3 Brain v2 | TCK-179, 182, 144, 153, 184, 187, 183 (reset rule), 188 (ADR-002), 149 (motors 3/4) |
| P4 Learning | TCK-152 |
| P5 Locomotion v2 | TCK-150, 180 |
| P6 Energy v2 | TCK-151, 178, 145 |
| P7 Evaluation and fitness | TCK-174, 175, 154 |
| P8 Curriculum and UI | TCK-176, 177 |
| conditional | TCK-181 (only if P7 shows body mutations destroy learned behaviour) |

## How it is meant to work
- **Measure first, then change.** P2 builds one tool (headless runner with probe subcommands, seeded, reproducible). Its report is the baseline; every later phase is accepted by a before/after comparison against the target values set after P2 (TCK-188).
- **One rebuild per area.** All changes to the genome compiler and brain wiring (heredity, topology, ΔI channel, evaluation order, genome operators) happen together in P3, with one golden-master change and one DB reset, instead of rebuilding the same code (`dna.rs:1231-1382`) five times.
- **Phase closing:** one commit, one version bump, one CHANGELOG entry and one wiki update per phase, closing all its tickets; plus the smoke checklist of TCK-186 (Ocean runs without NaN, trainer UI works, inspector/tooltips correct, `cargo test` + `npm run build` green).
- **Phase gates:** each phase ends with a hard stop and a short report to the user (`Phase Px complete. Waiting for explicit 'GO PHASE Py'.`), unless the user says otherwise.
- **No endless tuning:** metabolic constants are computed from share targets (brain + senses 10–20 % of resting cost); thrust, fitness weights, curriculum thresholds and rehearsal share are time-boxed against the targets.
- **Refactoring on the way** (DRY, enums, shared types/helpers, runtime performance) is part of the tickets that touch the code (TCK-173, TCK-177, TCK-185), limited to the trainer path and shared creature code. Ocean-only refactoring stays in Phase 1.

## User decisions (2026-10-21)
- All fixes for creature mechanics, brain, performance, energy cost and training belong to the epic, even if not yet visible in the two rooms.
- Backward compatibility is not required; the DB may be emptied on model changes (TCK-183).
- Pool tickets TCK-144/145/149/150/151/152/153 were pulled in with the session's recommended defaults (recorded in each ticket); the user may still veto. TCK-151 direction confirmed by the user after research (activity-based cost).
- Not added: an explicit non-goals list (user declined).

## Next step
P1: go through the dead config keys of TCK-149 with the user, key by key (default grouping is in the ticket), then implement. P2 does not depend on the user and can follow directly after the P1 gate.

## Open points
- Local commits of this session are not pushed yet (check `git status -sb`); `gh` is not installed, so CI runs and the GitHub ticket sync must be checked in the browser.
