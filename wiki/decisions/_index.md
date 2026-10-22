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
