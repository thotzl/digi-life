---
id: TCK-169
title: Roadmap Epic: Stabilize, Targeted Rebuild, Feature Development
status: ongoing
priority: high
assigned: unassigned
created_at: 2026-10-21
---

# TCK-169: Roadmap Epic: Stabilize, Targeted Rebuild, Feature Development

Phased plan from the review of 2026-10-19 (base commit `998244e`, v1.12.2). Each phase ends with a hard stop and explicit user approval (`GO PHASE 1`, `GO PHASE 2`). No feature ticket (TCK-111, TCK-113, TCK-118, TCK-123) starts before Phase 0 is complete. Non-goals: `wiki/decisions/adr-001-non-goals.md`.

## Current state (2026-10-21)
- Phase 0 implemented (TCK-134..142, TCK-162; v1.12.3–v1.12.12).
- Open from Phase 0: TCK-164 (manual UI flow check); TCK-143 (offspring) deliberately deferred.
- Next: user decisions on pool tickets TCK-143..161 and TCK-165..168, then `GO PHASE 1`.
- 2026-10-21: user prioritises training. Phase 1a (training track, epic TCK-171) runs before the rest of Phase 1; see below.

## Phase 1a — Training track (epic TCK-171)
Training produced results only in the small chamber. Analysis points to core mechanics (heredity, speed, steering, walls) before trainer scoring. Order: TCK-172 mechanics diagnosis (no refactor) → TCK-173 `TrainerSim` + seeded headless runner (trainer-only slice of TCK-156/TCK-158) → TCK-174 fair evaluation → TCK-175 fitness rebuild (includes TCK-154) → TCK-176 curriculum (core of TCK-123) → TCK-177 UI metrics. Mechanic fixes are ticketed after the TCK-172 numbers. Each ticket is accepted by a before/after headless measurement.

## Phase 0 — Stabilize (correctness + release capability)
Minimal-invasive; every fix covered by a test.

| Step | Content | IDs | Ticket |
| :--- | :--- | :--- | :--- |
| 0.1 | Rust toolchain on Windows; test/clippy baseline | RV-T05 | TCK-134 ✔ |
| 0.2 | CI workflow; `release.yml` only on tags | RV-T01 | TCK-134 ✔ |
| 0.3 | Warning banners in the most wrong skills | RV-X01 | TCK-142 ✔ |
| 0.4 | Wall reflection fix + 4-wall test | RV-B02 | TCK-135 ✔ |
| 0.5 | DB → `app_data_dir`; config via `include_str!` | RV-B03, RV-B04 | TCK-137 ✔, TCK-138 ✔ |
| 0.6 | Trainer `epoch_ticks` + test | RV-B05 | TCK-136 ✔ |
| 0.7 | Mitosis children into the simulation + `maxCreatures` cap; species in memory | RV-B01, RV-P01 | P01 TCK-139 ✔; B01 TCK-143 (deferred) |
| 0.8 | Grid reuse, no agent clones, events per frame | RV-P02, P03, P05 | TCK-139 ✔, TCK-140 ✔ |
| 0.9 | Log unknown actions; `SPAWN_FOOD` implement or remove | RV-B09 | TCK-141 ✔; TCK-147 |
| 0.10 | Manual smoke test | — | TCK-142 ✔; UI flows TCK-164 |

**DoD:** `cargo test` + `npm run build` green in CI; Ocean shows generation growth through real offspring; Windows release starts with DB in AppData; no DB query in the physics step.

## Phase 1 — Targeted rebuild (only what blocks Phase 2)

| Step | Content | Justified by | Ticket |
| :--- | :--- | :--- | :--- |
| 1.1 | Engine into `OceanSim`/`TrainerSim` with `step()` + `EventSink`; `CreatureFactory` | TCK-123, headless runner | TCK-156 |
| 1.2 | Seedable RNG | reproducible tests/balancing | TCK-158 |
| 1.3 | Typed `Action`/`SimEvent`, ts-rs export, frontend uses `bindings/` | TCK-111 needs a stable contract | TCK-157 |
| 1.4 | Repository layer, `user_version` migrations, WAL, retention | TCK-123 HoF export, data growth | TCK-159 |
| 1.5 | Headless runner `--seed --ticks --report` replaces `cli_trainer` | balancing S02/S03/S07 | TCK-158 |
| 1.6 | Behavioural tests | Phase 2 regressions | TCK-161 |
| 1.7 | Config clean-up, decision per dead key, `deny_unknown_fields` | config SSOT | TCK-149, TCK-145, TCK-146 |
| 1.8 | Species model: distance-based speciation (design decision) | meaningful roster/analytics | TCK-148 |
| 1.9 | Gene spans from backend; drop legacy locus logic; CSP | TCK-111 editor shows genes correctly | TCK-160, TCK-161 |
| 1.10 | Docs, skills, tickets current | agent reliability | TCK-142 ✔, TCK-163 ✔ |

**DoD:** engine logic testable without Tauri; frontend imports `bindings/`; headless report reproducible with seed; docs describe the current state.

## Phase 2 — Feature development (order by dependency)
1. Model balancing with measurement series: RV-S02 thrust, S03 BMR, S07 ecology, S04 Hebbian, B06 motion vision (TCK-150, TCK-151, TCK-152, TCK-144). Measure first, then constants; results documented in the wiki.
2. TCK-123 curriculum / multi-scenario + champion seeding — after TCK-156; TCK-154 (spawn distances into the plugin) first.
3. TCK-111 creator / genome editor — after TCK-157/TCK-160; scope: SQLite catalogue instead of IndexedDB, live preview via `get_fossil_phenotype`.
4. TCK-118 diploid genetics — deep change in `parse_genome`; needs property tests (TCK-161) and a headless before/after comparison.
5. Research ideas from `ideas.md` as individual tickets with a measurement criterion: TCK-165, TCK-166, TCK-167, TCK-168.
6. TCK-113 mobile — last; needs stable performance and touch-capable views.

Unslotted pool items: TCK-153 brain evaluation order, TCK-155 typed enums.
