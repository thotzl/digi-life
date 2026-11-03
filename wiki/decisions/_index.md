# Decisions (ADRs)

Architecture decision records: decisions that are taken. Open design questions live in tickets, not here.

## Convention

- File: `decisions/adr-NNN-short-title.md`, numbered consecutively, never renumbered.
- A superseded ADR stays; its status points to the successor.

## Template

```markdown
# ADR-NNN: <title>

- Status: proposed | accepted | superseded by ADR-MMM
- Date: YYYY-MM-DD
- Tickets: TCK-…

## Context
## Decision
## Consequences
```

## Records

- [[wiki/decisions/adr-001-non-goals]] — no ECS/frontend framework rewrite, no premature dependency majors, no cosmetic passes, no unmeasured optimization
- [[wiki/decisions/adr-002-brain-v2]] — Brain v2: role-based neurons, stable synapse keys and gene ownership, compact topology, ΔI channel, one-pass evaluation with exponential integration, structured genomes and gene-level/creep mutation, model-version DB reset (TCK-171 P3)
- [[wiki/decisions/adr-003-plugin-architecture]] — plugin architecture: one seam and registry per extension point, enum dispatch before `dyn`, explicit config, one shared creature tick, contracts + guides as the feature agent's interface, structure before behaviour, proof with fresh agents (TCK-196)
- [[wiki/decisions/adr-004-determinism-and-references]] — seeded IO-free simulations with named RNG streams, `fmath` everywhere, golden master over model fields, four reference runs in CI (TCK-197, TCK-156, TCK-158, TCK-202)
- [[wiki/decisions/adr-005-persistence-migrations]] — SQL only in `src/persistence/`, repositories with SQLite and in-memory stores, `user_version` migrations separate from `MODEL_VERSION`, backup before a model reset, retention (TCK-159, TCK-195)
- [[wiki/decisions/adr-006-typed-ipc]] — typed IPC end to end via ts-rs, one frontend IPC client with visible errors, mode via the router's `set_mode`, partial updates from UI controls, text-only untrusted strings and CSP (TCK-157, TCK-160, TCK-161, TCK-197)
- [[wiki/decisions/adr-007-release-identity]] — product name BroodBattle, alpha-only identifier, separate dev/installed data, per-user NSIS installer + portable ZIP, Windows first, release log file (TCK-195, TCK-193, TCK-194)
- [[wiki/decisions/adr-008-trainer-evaluation-and-selection]] — shared layouts, 8 trials with layout rotation, 1–3 mutation ops with the 1/5 rule, elite ratio 0.25, no body/brain mutation decoupling, population 100 independent of the visible tiles (TCK-174, TCK-181, TCK-192)
- [[wiki/decisions/adr-009-trainer-fitness-and-minimal-criterion]] — one fitness in catch units without cliffs, minimal criterion for generation 1 and immigrants, escalations not adopted (TCK-175)
- [[wiki/decisions/adr-010-training-curriculum]] — one difficulty level between small and large room, gate 0.3 / 0.1, no rehearsal or metabolic ramp by default, switches at generation boundaries (TCK-176)
- [[wiki/decisions/adr-011-energy-model-v2]] — additive activity-based energy cost from share targets, one energy function for Ocean and trainer, predation moves energy with a bite cooldown (TCK-151, TCK-178, TCK-145)
- [[wiki/decisions/adr-012-locomotion-v2]] — speed band instead of `pulse²`, turning at rest, heading reflection at walls and reefs (TCK-150, TCK-180)
- [[wiki/decisions/adr-013-tanh-thalamus-sum-nodes]] — tanh instead of sigmoid for thalamus sum nodes, so silent channels give no offset (TCK-190)
- [[wiki/decisions/adr-014-config-keys-must-have-effect]] — every config key has a read site, unknown keys are rejected, values in `config.json` (TCK-149)
- [[wiki/decisions/adr-015-project-knowledge-organisation]] — wiki for current state and results, tickets for open work, CHANGELOG for finished work, measurements in `wiki/history/`, repo root as Obsidian vault (TCK-163, TCK-170, TCK-203)
- [[wiki/decisions/adr-016-epic-execution-workflow]] — epics on master, phases without stops, measurement-based decisions by the agent against target values, defaults with veto, worktree subagents (TCK-171, TCK-196)
