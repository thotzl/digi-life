---
id: TCK-144
title: Motion Vision Channel Design
status: open
priority: pool
decision: pending
assigned: unassigned
created_at: 2026-10-19
revisit: RV-B06
---

# TCK-144: Motion Vision Channel Design

> Pool ticket: requires a user decision before implementation. Details in `docs/revisit/` (RV-B06).

## Description
Motion vision overwrites channel 5 of eyes/noses, which carries the main meat (IR) signal. Organ type is detected via emoji labels.

## Proposed Scope
Decide: dedicated 6th channel vs. separate neuron; organ type as enum.
