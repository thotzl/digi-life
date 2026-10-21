---
sources: [docs/revisit/README.md, docs/revisit/00-executive-summary.md, docs/revisit/01-critical-bugs.md, docs/revisit/02-architecture.md, docs/revisit/03-performance.md, docs/revisit/04-simulation-model.md, docs/revisit/05-persistence.md, docs/revisit/06-frontend.md, docs/revisit/07-testing-ci-tooling.md, docs/revisit/08-docs-and-agent-context.md]
---

# Review Findings Register (`RV-*`)

Static deep review of 2026-10-19 (base commit `998244e`, v1.12.2). Tickets reference these IDs in their `revisit:` frontmatter. Detail lives in the linked articles; this page is the index.

## Legend

- **Priority:** `P0` breaks a core function or release; `P1` distorts simulation results or blocks planned features; `P2` maintainability/performance debt; `P3` hygiene, only when touching the file anyway.
- **Evidence:** `[verified]` by execution, `[static]` by code reading, `[hypothesis]` needs measurement first.
- **ID scheme:** `RV-<area><n>` — B bugs, A architecture, P performance, S simulation, D database, F frontend, T test/CI, X docs/context.
- **Status:** `done <commit> (TCK)` or `pool TCK-…` (decision pending). Done items are kept as audit trail.

## Bugs

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-B01 | P0 | Mitosis children never simulated, no Ocean heredity | pool TCK-143 (deliberately deferred) | [[biology/metabolism-and-reproduction]] |
| RV-B02 | P0 | Wall reflection left/top pushes into the wall | done 0afd463 (TCK-135) | [[simulation/physics]] |
| RV-B03 | P0 | DB path CWD-relative under `target/` | done cad6976 (TCK-137) | [[architecture/persistence]] |
| RV-B04 | P0 | `config.json` not found in release, divergent fallback | done 5afd6d1 (TCK-138) | [[architecture/configuration]] |
| RV-B05 | P1 | Trainer `epoch_ticks` never incremented, speed bonus constant | done 1849bab (TCK-136) | [[simulation/trainer]] |
| RV-B06 | P1 | Motion vision overwrites meat channel 5 | pool TCK-144 | [[biology/sensing]] |
| RV-B07 | P1 | Biting gives the attacker no energy, no cooldown | pool TCK-145 | [[biology/metabolism-and-reproduction]] |
| RV-B08 | P1 | Lifespan ignores `creatureMaxAgeTicks` | pool TCK-146 | [[biology/metabolism-and-reproduction]] |
| RV-B09 | P2 | UI sends actions the engine does not know (`SPAWN_FOOD`, `CLIENT_LOG`) | partly 625695d (logging, TCK-141); rest pool TCK-147 | [[architecture/ipc-protocol]] |
| RV-B10 | P2 | Species = genome string; Latin name UNIQUE collisions | pool TCK-148 (design decision) | [[biology/metabolism-and-reproduction]] |
| RV-B11 | P1 | Latin name not deterministic | done beef2fb (TCK-162) | [[biology/genome-compiler]] |

## Architecture

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-A01 | P1 | Engine is one god closure | pool TCK-156 | [[architecture/simulation-engine]] |
| RV-A02 | P1 | Stringly-typed IPC both ways | pool TCK-157 | [[architecture/ipc-protocol]] |
| RV-A03 | P1 | ts-rs bindings generated but unused | pool TCK-157 | [[architecture/ipc-protocol]] |
| RV-A04 | P2 | No shared DB connection/path management | done cad6976 (TCK-137) | [[architecture/persistence]] |
| RV-A05 | P2 | World size and trainer constants duplicated | pool TCK-156 | [[architecture/simulation-engine]] |
| RV-A06 | P3 | Mixed timestamp units (`now_millis` in seconds) | pool TCK-157 | [[architecture/persistence]] |

## Performance

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-P01 | P1 | Species table scan per step | done ef30b6e (TCK-139) | [[architecture/performance]] |
| RV-P02 | P1 | Spatial grid allocated per step | done ef30b6e (TCK-139) | [[architecture/performance]] |
| RV-P03 | P1 | Deep agent clones in the hot loop | done ef30b6e (TCK-139) | [[architecture/performance]] |
| RV-P04 | P2 | Linear nearest-food search | pool TCK-156 (tie-break risk) | [[architecture/performance]] |
| RV-P05 | P1 | IPC flood from the hot loop | done ef30b6e (TCK-139), 27c95ec (TCK-140) | [[architecture/performance]] |
| RV-P06 | P2 | `mutate_genome` compiles full phenotype | done 27c95ec (TCK-140) | [[architecture/performance]] |
| RV-P07 | P2 | DB queries in restock loop | done ef30b6e (TCK-139) | [[architecture/performance]] |
| RV-P08 | P3 | Allocations per brain tick | open, only with profiler evidence | [[architecture/performance]] |

## Simulation model

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-S01 | P1 | 16 of 44 config rules never read | pool TCK-149 | [[architecture/configuration]] |
| RV-S02 | P1 | Hypothesis: thrust ≈ 0 for many genomes ("motionless Ocean") | pool TCK-150 | [[simulation/ocean]] |
| RV-S03 | P1 | BMR penalizes brain size multiplicatively | pool TCK-151 | [[biology/metabolism-and-reproduction]] |
| RV-S04 | P2 | Hebbian without normalization, also on reflexes | pool TCK-152 | [[biology/ctrnn-brain]] |
| RV-S05 | P2 | Inconsistent Lamarckian semantics in the Ocean | pool TCK-152 | [[biology/metabolism-and-reproduction]] |
| RV-S06 | P2 | Brain layer latency from ID order | pool TCK-153 | [[biology/ctrnn-brain]] |
| RV-S07 | P2 | Hypothesis: Ocean ecology extremely sparse | pool TCK-150 | [[simulation/ocean]] |
| RV-S08 | P2 | Trainer spawn distances inconsistent | pool TCK-154 | [[simulation/trainer]] |
| RV-S09 | P2 | Trainer gen 1 ignores `base_dna` | pool TCK-154 | [[simulation/trainer]] |
| RV-S10 | P3 | Model semantics in free-text strings | pool TCK-155 | [[biology/genome-compiler]] |
| RV-S11 | P3 | `cli_trainer` tests an outdated model | pool TCK-158 | [[simulation/analytics-and-balancing]] |

## Database

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-D01 | P0 | DB location (= RV-B03) | done cad6976 (TCK-137) | [[architecture/persistence]] |
| RV-D02 | P1 | Ad-hoc migrations, no schema version | pool TCK-159 | [[architecture/persistence]] |
| RV-D03 | P1 | Unbounded table growth | pool TCK-159 | [[architecture/persistence]] |
| RV-D04 | P2 | Inconsistent timestamps | pool TCK-159 | [[architecture/persistence]] |
| RV-D05 | P2 | Errors swallowed | partly cad6976 (TCK-137); JSON fallbacks pool TCK-159 | [[architecture/persistence]] |
| RV-D06 | P3 | Dead/misleading state | pool TCK-159 | [[architecture/persistence]] |

## Frontend

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-F01 | P1 | Handwritten types instead of ts-rs | pool TCK-157 | [[frontend/structure]] |
| RV-F02 | P1 | `innerHTML` with data, CSP disabled | partly 625695d (escaping, TCK-141); CSP pool TCK-161 | [[frontend/structure]] |
| RV-F03 | P1 | Legacy genome model in the UI | pool TCK-160 | [[frontend/structure]] |
| RV-F04 | P2 | Errors swallowed at the IPC boundary | pool TCK-160 | [[frontend/structure]] |
| RV-F05 | P2 | Side-effect modules, duplicate listeners | pool TCK-160 | [[frontend/structure]] |
| RV-F06 | P2 | Duplicate preview modal code | pool TCK-160 | [[frontend/structure]] |
| RV-F07 | P3 | Unused dependencies | partly 625695d (TCK-141); rest pool TCK-161 | [[frontend/structure]] |
| RV-F08 | P3 | Hardcoded values | pool TCK-160 | [[frontend/structure]] |

## Tests, CI, tooling

| ID | Prio | Finding | Status | Article |
| :--- | :--- | :--- | :--- | :--- |
| RV-T01 | P0 | No verification pipeline | done f04a035 (TCK-134) | [[quality/ci-and-tooling]] |
| RV-T02 | P1 | Tests check implementation details, not behaviour | partly (repro/golden master tests TCK-135..162); rest pool TCK-161 | [[quality/testing]] |
| RV-T03 | P1 | No headless measurement tool | pool TCK-158 | [[simulation/analytics-and-balancing]] |
| RV-T04 | P1 | Non-deterministic RNG | pool TCK-158 | [[architecture/simulation-engine]] |
| RV-T05 | P2 | Toolchain/platform | partly f04a035 (TCK-134); updates pool TCK-161 | [[quality/ci-and-tooling]] |

## Docs and agent context

| ID | Prio | Finding | Status |
| :--- | :--- | :--- | :--- |
| RV-X01 | P0 | Skills described a non-existent architecture (WebSocket, JSON DB files) | done TCK-142 |
| RV-X02 | P1 | Stale safety rules in `active-task.md`; rules 2 and 3 should become tests | done TCK-142 (now [[architecture/invariants]]) |
| RV-X03 | P1 | README / WHITEPAPER / HANDOFF outdated | done TCK-142 |
| RV-X04 | P2 | Ticket status inconsistent | done TCK-142 |
| RV-X05 | P2 | Duplicate truths in repo root (`.gitignore`, `.aiignore`, `ideas.md` placement) | done TCK-142 |
| RV-X06 | P3 | Mixed-language inline comments with history narrative | open, only when touching the file |

## Health scorecard (state of the review, 2026-10-19)

| Dimension | Rating then | Core finding | Since then |
| :--- | :--- | :--- | :--- |
| Ocean correctness | critical | no heredity (B01), wall bug (B02) | B02 fixed; B01 deferred |
| Trainer correctness | medium | dead speed bonus (B05), spawn distances (S08) | B05 fixed |
| Release capability | critical | DB under `target/`, config not bundled (B03/B04) | fixed |
| Architecture | weak | god function + stringly IPC (A01/A02) | open |
| Performance | weak | DB scan per step, clones (P01–P04) | fixed except P04 |
| Testability | weak | engine untestable, 0 FE tests | golden master + repro tests added |
| Docs/agent context | misleading | skills described WebSocket/JSON architecture | fixed (TCK-142), moved into this wiki (TCK-163) |
| Biology model | solid base | creative and deterministic; balancing issues (S01–S07) | open |

Kept as strengths: deterministic promoter-based `parse_genome`; CTRNN with thalamus bottleneck and sorted synapses for an O(S) pass; the scenario plugin trait; ts-rs annotations; ticket/changelog discipline.

## Key Takeaways

- Phase 0 closed all P0s except RV-B01, which is a deliberate pool decision.
- Every remaining finding maps to a pool ticket TCK-143..161.
- When a finding is resolved, update its row here and in the linked article.

Sources: `docs/revisit/` (all files).
