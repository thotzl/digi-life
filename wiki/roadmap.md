---
sources: [docs/revisit/roadmap.md, docs/revisit/README.md, HANDOFF.md, .agents/state/active-task.md, .tickets/open/, .tickets/ongoing/]
---

# Roadmap

Stabilize → targeted rebuild → feature development. Each phase ends with a hard stop and explicit user approval. Tickets are the source of truth for scope and status; this page orders them.

## Current state (2026-10-20)

- **Phase 0 — Stabilize: implemented** (TCK-134..142, TCK-162; v1.12.3–v1.12.12).
- Next: user decisions on pool tickets TCK-143..161 and TCK-165..168 (`priority: pool`), then `GO PHASE 1`.
- Open from Phase 0: manual UI flow check (TCK-164); RV-B01 offspring (TCK-143) deliberately deferred.

## Phase 0 — Stabilize (done)

| Step | Content | IDs | Ticket |
| :--- | :--- | :--- | :--- |
| 0.1 | Rust toolchain on Windows, test/clippy baseline | T05 | TCK-134 |
| 0.2 | CI workflow, `release.yml` only on tags | T01 | TCK-134 |
| 0.3 | Warning banners in the most wrong skills | X01 | TCK-142 |
| 0.4 | Wall reflection fix + 4-wall test | B02 | TCK-135 |
| 0.5 | DB → `app_data_dir`, config `include_str!` | B03, B04 | TCK-137, TCK-138 |
| 0.6 | Trainer `epoch_ticks` + test | B05 | TCK-136 |
| 0.7 | Mitosis children into the simulation + species in memory | B01, P01 | P01 done (TCK-139); B01 → TCK-143 |
| 0.8 | Grid reuse, no agent clones, events per frame | P02, P03, P05 | TCK-139, TCK-140 |
| 0.9 | Log unknown actions; `SPAWN_FOOD` implement or remove | B09 | TCK-141; rest TCK-147 |
| 0.10 | Manual smoke test | — | TCK-142; UI flows TCK-164 |

DoD: CI green; Ocean shows generation growth through real offspring (open, TCK-143); Windows release starts with DB in AppData; no DB query in the physics step.

## Phase 1 — Targeted rebuild (only what blocks Phase 2)

| Step | Content | Justified by | Ticket |
| :--- | :--- | :--- | :--- |
| 1.1 | Engine into `OceanSim`/`TrainerSim` with `step()` + `EventSink`; `CreatureFactory` | TCK-123, headless runner | TCK-156 |
| 1.2 | Seedable RNG | reproducible tests/balancing | TCK-158 |
| 1.3 | Typed `Action`/`SimEvent`, frontend uses `bindings/` | TCK-111 needs a stable contract | TCK-157 |
| 1.4 | Repository layer, `user_version` migrations, retention | TCK-123 HoF export, data growth | TCK-159 |
| 1.5 | Headless runner `--seed --ticks --report` replaces `cli_trainer` | balancing S02/S03/S07 | TCK-158 |
| 1.6 | Behavioural tests per [[quality/testing]] gaps | Phase 2 regressions | TCK-161 |
| 1.7 | Config clean-up, decision per dead key, `deny_unknown_fields` | config SSOT | TCK-149, TCK-145, TCK-146 |
| 1.8 | Species model: distance-based speciation (design decision) | meaningful roster/analytics | TCK-148 |
| 1.9 | Gene spans from backend; drop legacy locus logic; CSP | TCK-111 editor | TCK-160, TCK-161 |
| 1.10 | Docs/skills/tickets current | agent reliability | done in TCK-142, wiki TCK-163 |

DoD: engine logic testable without Tauri; frontend imports `bindings/`; headless report reproducible with seed; docs describe the current state.

## Phase 2 — Feature development

Order by dependency:
1. **Model balancing with measurement series** — RV-S02 thrust, S03 BMR, S07 ecology, S04 Hebbian, B06 motion vision (TCK-150, 151, 152, 144). Measure first; record results in [[simulation/analytics-and-balancing]].
2. **TCK-123 curriculum / multi-scenario + champion seeding** (ongoing) — needs TCK-156 and RV-S08 (TCK-154).
3. **TCK-111 creator / genome editor** — after TCK-157/160; scope: store in the SQLite catalogue, not IndexedDB; live preview via `get_fossil_phenotype`.
4. **TCK-118 diploid genetics** — deep change in `parse_genome`; needs property tests and headless before/after comparison.
5. **Ideas from research** — oscillators, hormones (replacing dead adrenaline), color-segment metabolism, relative wiring — as pool tickets TCK-165..168 with a measurement criterion, see [[research/porting-concepts]].
6. **TCK-113 mobile** — last; needs stable performance and touch-capable views.

Other pool items without a phase slot: TCK-153 brain evaluation order, TCK-155 typed enums.

## Non-goals (decisions)

- No rewrite into an ECS framework or another frontend framework (React/Svelte) — the problem is structure, not tooling.
- No dependency major updates before a green CI baseline.
- No comment/formatting pass as its own task (RV-X06 only when touching a file).
- No micro-optimization (RV-P08) without profiler evidence.

## Key Takeaways

- Phase 0 is done except the deferred offspring fix and the manual UI check.
- Phase 1 work is fully ticketed as pool tickets; each needs a user decision.
- Features start in Phase 2, balancing first.

Sources: `docs/revisit/roadmap.md`, `HANDOFF.md`, `.agents/state/active-task.md`, `.tickets/`.
