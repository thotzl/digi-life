---
sources: [docs/revisit/roadmap.md]
---

# ADR-001: Non-goals of the Stabilization and Rebuild

- Status: accepted
- Date: 2026-10-19
- Origin: review of 2026-10-19

## Context

The review of 2026-10-19 found structural problems (engine god closure, stringly-typed IPC, dead config) in a codebase of ~6,800 lines of Rust and ~5,300 lines of TypeScript with 25–45 simulated agents. A full rewrite was considered and rejected; the problems are structure, not tooling.

## Decision

- No rewrite into an ECS framework (bevy_ecs etc.) or another frontend framework (React/Svelte). `Vec<Creature>` is sufficient at this scale.
- No dependency major updates before a green CI baseline.
- No comment or formatting pass as its own task; mixed-language and history-narrative comments are cleaned only when the file is touched anyway (RV-X06).
- No performance micro-optimization without profiler evidence (RV-P08).

## Consequences

- Refactoring is targeted: each structural change must be justified by a defect, a risk or a blocked feature.
- Any engine decomposition uses plain structs instead of an ECS.

## Key Takeaways

- Fix structure in place; no framework switches, no cosmetic passes, no unmeasured optimization.

Sources: `docs/revisit/roadmap.md` (deleted).
