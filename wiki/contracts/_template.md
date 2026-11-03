---
sources: [tickets/closed/TCK-203-r6-docs-contracts-agent-setup.md, tickets/closed/TCK-196-plugin-architecture-epic.md]
---

# Contract: `<seam name>` (template)

A contract describes one extension point (seam) of the code: what a plugin implements, how the host calls it and
what it must not break. Copy this file to `wiki/contracts/<seam>.md`, keep the section order, delete the italic
hints. The ticket that creates the seam writes its contract in the same commit (TCK-196 principle 3). A feature agent
reads the guide for its extension point, this contract and [[wiki/architecture/invariants]], nothing else.

## Purpose

_One paragraph: what the seam is for, which plugins exist today, which are planned._

## Signature

_The types and functions a plugin implements or receives (Rust/TS signatures, file paths). Only the public surface._

## Lifecycle / call order

_Who calls what, when, how often (per tick, per trial, per generation, at startup); what state is valid at each call._

## Invariants

_Rules specific to this seam (determinism, no IO, ordering). Link the global ones in
[[wiki/architecture/invariants]] by number instead of restating them._

## Registration point

_The one place a new plugin is added (registry entry, enum variant, match arm, module declaration)._

## Config keys

_Config section and keys the seam reads (see [[wiki/contracts/config-section]]); "none" if it reads none._

## Events / IPC

_Events, actions or commands the seam emits or consumes; "none" if it has none._

## Test recipe

_A runnable command plus the test(s) a new plugin must add or pass, e.g. `cargo test <module>::tests`._

## Minimal example

_The smallest complete plugin in code, compiled against the current signatures._
