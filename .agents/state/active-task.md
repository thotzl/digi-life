# Active task: Implement Epic TCK-171 (Training Track)

Last updated: 2026-10-26. Active task (handover note, not an instruction); the user's request defines the session task. If this note contradicts the tickets, the tickets win.

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
- **No phase gates for this epic (user decision):** all phases run through without stops; full phase close each time (tickets, wiki, CHANGELOG, version, all checks incl. own UI smoke, one commit, push, CI); contact the user only on real ambiguities; report after P8 (TCK-171 `## Execution decisions`).
- **No endless tuning:** metabolic constants are computed from share targets (brain + senses 10–20 % of resting cost); thrust, fitness weights, curriculum thresholds and rehearsal share are time-boxed against the targets.
- **Refactoring on the way** (DRY, enums, shared types/helpers, runtime performance) is part of the tickets that touch the code (TCK-173, TCK-177, TCK-185), limited to the trainer path and shared creature code. Ocean-only refactoring stays in Phase 1.

## User decisions (2026-10-21)
- All fixes for creature mechanics, brain, performance, energy cost and training belong to the epic, even if not yet visible in the two rooms.
- Backward compatibility is not required; the DB may be emptied on model changes (TCK-183).
- Pool tickets TCK-144/145/149/150/151/152/153 were pulled in with the session's recommended defaults (recorded in each ticket); the user may still veto. TCK-151 direction confirmed by the user after research (activity-based cost).
- Not added: an explicit non-goals list (user declined).

## Progress
- P1 Config done (v1.12.15): TCK-149 implemented for P1; ticket stays ongoing for its P3/P5/P6 rests (see ticket).
- P2 Sim lab done (v1.12.16): TCK-172/173/185/186 closed, TCK-183/188 ongoing (P3 rests). Runner `target/release/trainer_runner run|probe`; targets in TCK-171; UI smoke tooling `.agents/artifacts/uicheck/run-smoke.sh` (backs up and restores the app DB; local, gitignored).
- P3 Brain v2 done (v1.12.17): TCK-179/182/144/153/184/187/183/188 closed, TCK-149 ongoing (P5/P6 keys). ADR-002 `wiki/decisions/adr-002-brain-v2.md`; golden `0x65f701a6937c1e71`; `MODEL_VERSION` 2; 5-seed success (pop 16, 30 gens, last 5 gens) small 0.122 / large 0.033 (P2 0.040 / 0.023). Open finding in `.agents/state/next-tasks.md` (thalamus sum offset).
- P4 Learning done (v1.12.18): TCK-152 closed (Oja rule, plastic flag, Lamarck as probability; golden `0xe0a2bfd1c7310306`; MODEL_VERSION 3; 5-seed success small 0.207 / large 0.046).

- P5 Locomotion v2 done (v1.12.19): TCK-150/180 closed (speed band 1–5 px/tick, turning at rest, heading reflection at walls/reefs); 5-seed success small 0.218 / large 0.033 (10 seeds 0.193 / 0.060).

- P6 Energy v2 done (v1.12.20): TCK-151/178/145/149 closed. Shared energy function `src/biology/metabolism.rs`, scale matched to typical total cost (resting + movement) = old Ocean drain 0.0243; Ocean 90 s deaths P5 ~149 vs P6 ~147; 5-seed success small 0.223 / large 0.050. Fixed on the way: Vite watcher exclusions; trainer view froze (GPU-bound canvas redraw per telemetry tick, also present in P5 on this machine) -> rAF-coalesced drawing, 360 px canvases. Diagnosis scripts `seqdiag.cjs` (RTT per step) and `tracediag.cjs` (Chromium trace across processes).
- UI check tooling (local, gitignored): `.agents/artifacts/uicheck/run-smoke.sh <outDir> [smoke.cjs|inspect.cjs|resetlat.cjs|oceanlong.cjs]` — backs up and restores the app DB; Ocean start waits for the router's pause before pressing Resume. Never create git worktrees inside the repo (watchers crawl them). Never kill all node.exe processes.

## Next step
P7 Evaluation and fitness (TCK-174, 175, 154; conditional 181), contract `.agents/artifacts/p7-design.md` (includes the trainer `agent.age` fix), then P8 (TCK-176, 177; trainer rAF render loop added to TCK-177). All phases run without stops (TCK-171 `## Execution decisions`). Measure with `trainer_runner` on seeds b1..b5 (pop 16, 30 gens) plus probes; UI smoke `.agents/artifacts/uicheck/run-smoke.sh <out> [smoke.cjs|inspect.cjs|resetlat.cjs]` (backs up and restores the app DB).

## Open points
- Check `git status -sb` for unpushed commits.
