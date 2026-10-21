---
id: TCK-159
title: DB Migration Versioning, Retention and Timestamps
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-D02, RV-D03, RV-D04, RV-D06
---

# TCK-159: DB Migration Versioning, Retention and Timestamps

> Pool ticket: requires a user decision before implementation. Details in `wiki/quality/review-findings.md` (RV-D02, RV-D03, RV-D04, RV-D06).

## Description
Ad-hoc migrations without `user_version`; unbounded growth of trainer_genomes/species_records; mixed timestamp units; dead `progenitor_genome`/`sessions` state.

## Proposed Scope
Versioned migrations, retention policy, unified unix-ms timestamps.
