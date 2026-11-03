---
id: TCK-196
title: "Epic: Plugin Architecture Rebuild (whole codebase, docs, agent setup)"
status: open
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: Roadmap TCK-169 Phase 1 (targeted rebuild), replaces its open slices
depends_on: TCK-195
---

# TCK-196: Epic: Plugin Architecture Rebuild

## Goal (user, 2026-11-02)
A clean structure across the whole codebase where every new feature — a new organ/trait kind (claws, teeth, armour, limbs, spikes), a new dynamic (attack, digestion, reproduction), a new room/mode (arena, creator, more training rooms), a new view — only needs to know **its interfaces**. The agent implementing a feature reads one guide, one contract and the invariants, not the whole wiki or codebase. DRY, known bugs fixed, all docs (wiki, skills, AGENTS.md, README) aligned. Executed with the agentic-engineering skill; independent slices run as parallel subagents.

## Basis
Read-only review of 2026-11-02 (six parallel reviews: engine/server, biology, trainer/scenarios, infra/config/persistence, frontend, docs/process; base f34a301). Findings are distributed into the sub-tickets with file:line references.

Main findings:
- `engine.rs` is one ~1,465-line closure: Ocean step, action dispatch, ~24 SQL sites, `thread_rng`, needs a `WebviewWindow`; untestable.
- The per-creature tick (sense → brain → drain → move → eat) exists four times (Ocean inline, `scenarios::step_physics`, `probes::brain_tick`, metabolism probe) and diverges (eat radius, food-drift order, `sin`).
- Adding an organ kind touches ~10 places in `dna.rs`, `brain.rs`, `physics.rs`, `metabolism.rs`, `engine.rs`, trainer, probes, frontend renderer.
- `AppConfig::global()` is read inside the model (~60 sites, incl. `parse_genome`).
- The room "plugin" is a closed enum plus a concrete struct; one creature per chamber, no obstacles, no own rules or fitness.
- Frontend: no view lifecycle or teardown, ~50 shared DOM ids, untyped Ocean IPC, triplicated modals, an exploitable XSS via `styled` + `csp: null`.
- Docs: rules copied 4–5×, phase logs inside current-state articles, stale epic/pool tickets and skill links.

## Principles
1. **Behaviour-preserving first.** Phases R1–R4 change structure only. Gate on every slice: `GOLDEN_FINGERPRINT` and both trainer reference reports unchanged, all tests green. Exceptions are stated in the ticket (e.g. Ocean RNG seeding).
2. **Intended model changes are bundled** in TCK-202 (one `MODEL_VERSION` bump, new references, re-measured training results).
3. **Interfaces before implementations** (spec-first). Every new seam ships with its contract doc in the same commit (`wiki/contracts/`).
4. **KISS.** Plain functions and enum dispatch before `dyn` traits; a trait only where a second implementation exists or is planned (arena, creator).
5. **Determinism stays.** Seeded `SimRng` streams with named constants; parallel == sequential.
6. **Process.** Per closed sub-ticket: patch version, CHANGELOG, wiki/contract update, ticket verification. Work on master. Parallel subagents work in git worktrees on disjoint file sets; the orchestrating agent integrates, runs the full gate and commits.

## Phases and sub-tickets
| Phase | Ticket | Content | Parallel with |
| :--- | :--- | :--- | :--- |
| R0 Safety net | TCK-197 | second trainer reference (curriculum + eval), golden master on model fields only, XSS + CSP fix, doc link check, close checklist | TCK-189 |
| R0 | TCK-189 | dependency alerts, dead dev dependencies | TCK-197 |
| R1 Foundations | TCK-198 | explicit `SimContext` / config sections, no globals in the model, shared utils (geom, time, json, logging) | — (touches many files; runs alone) |
| R1 | TCK-159 | persistence: `user_version` migrations, repository traits, DTOs, retention | TCK-157 |
| R1 | TCK-157 | typed IPC: `ClientAction`, `UiEvent`, typed command results, frontend IPC client | TCK-159 |
| R2 Creature model | TCK-199 | genome module split, promoter registry, organ spec registry, motor outputs, typed enums (absorbs TCK-155), dead phenotype fields | TCK-160 |
| R3 Simulation kernel | TCK-200 | `World` + ordered creature systems shared by Ocean, trainer, probes | — |
| R3 | TCK-156 | engine decomposition: `OceanSim` (IO-free, seeded), `Mode` trait, `EventSink`, stores, spawner/spore field/species ledger | TCK-201 |
| R3 | TCK-201 | rooms as plugins (`Room` registry), `Evaluator`, `Breeder`, multi-entity chambers, probes on shared code | TCK-156 |
| R3 | TCK-158 | headless Ocean runner + Ocean reference in CI | after TCK-156 |
| R4 Frontend | TCK-160 | `ViewModule` registry with teardown, selection state, shared components, organ painter registry, bindings-only types | TCK-199 |
| R4 | TCK-161 | test layout (happy-dom, controllers), lint/fmt/clippy in CI, Windows CI job | after TCK-160 |
| R5 Model corrections | TCK-202 | all intended behaviour fixes (thermal bug, eat radius, eyes as limbs, thresholds, HoF dedup, ...), absorbs TCK-146/147 | TCK-143 |
| R5 | TCK-143 | Ocean offspring: mitosis children are simulated, no phantom species | TCK-202 |
| R6 Docs and agent setup | TCK-203 | contracts + guides, architecture/module map, history split, skills by extension point, AGENTS.md/README | partly from R1 on |
| R7 Proof | TCK-204 | a fresh agent adds a small plugin using only guide + contract; measure context and files touched | — |

## Progress (2026-11-03)
- [x] R0: TCK-197 (v1.12.29), TCK-189 (v1.12.28)
- [x] R1: TCK-198 (v1.12.30), TCK-159 (v1.12.31), TCK-157 (v1.12.34)
- [x] R2: TCK-199 (v1.12.32, absorbs TCK-155)
- [x] R3: TCK-200 (v1.12.33), TCK-201 (v1.12.36), TCK-156 (v1.12.37), TCK-158 (v1.12.38)
- [x] R4: TCK-160 (v1.12.35), TCK-161 (v1.12.39; follow-up fix v1.12.40)
- [ ] R5: TCK-202 done (v1.12.41, absorbs TCK-146/147); TCK-143 in progress
- [ ] R6: TCK-203 in progress (contracts and guides for all ten extension points, module map, history split, skill `plugin-authoring`)
- [ ] R7: TCK-204 open

## Dependencies
R0 → R1 (TCK-198 first, then TCK-159 ∥ TCK-157) → R2 (TCK-199) ∥ R4 part 1 (TCK-160) → R3 (TCK-200 → TCK-156 ∥ TCK-201 → TCK-158) → R4 part 2 (TCK-161) → R5 (TCK-202 ∥ TCK-143) → R6 final (TCK-203) → R7 (TCK-204). TCK-203 writes the doc skeleton in R1 and each sub-ticket fills its contract.

## Out of scope (stay feature tickets after this epic)
TCK-111 Creator (needs rewrite on the `ViewModule` seam), TCK-123 arena/multi-scenario (re-scope on `Room`/`Evaluator`), TCK-148 species model, TCK-118 diploid genetics, TCK-165–168 research organs/brain, TCK-113 mobile, TCK-193 remaining naming, new organ kinds and dynamics themselves.

## Definition of done
- Engine, Ocean and trainer run headless and are tested without Tauri; no `AppConfig::global()` below the entry points; no SQL outside the repository module; no untyped IPC message.
- Adding an organ kind, a dynamic, a room or a view each touches its own module plus one registry entry (and its contract); verified by TCK-204.
- Golden master, trainer references and the Ocean reference are in CI; all tests, clippy and lint green on Windows and Linux.
- Wiki: contracts and guides per extension point, one copy of the invariants, current-state articles without phase logs; skills point to guides; no broken links (checked in CI).
