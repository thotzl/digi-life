---
sources: [tickets/closed/TCK-203-r6-docs-contracts-agent-setup.md]
---

# Guide: add a `<thing>` (template)

A guide is the task list for one extension point. Copy this file to `wiki/guides/add-<thing>.md`. Rules: at most 60
lines; it links only to its contract and to [[wiki/architecture/invariants]]; no background, history or
measurements (those live in the contract or the current-state articles).

Read first: [[wiki/contracts/_template]] (replace with the seam's contract) and [[wiki/architecture/invariants]].

## Steps

1. _First file to touch and what to add there._
2. _Registration point (one entry)._
3. _Config keys, if any._
4. _Tests to add._
5. _Docs: the contract's Purpose list, the row in "Registered plugins" of `wiki/architecture/module-map.md`, and
   any article that lists the plugins._

## Done when

- _The contract's test recipe passes._
- The project gate is green: `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`,
  `cargo test`, `npm test`, `npm run lint`, `npm run build`, `node scripts/check-links.mjs`, and the four CI
  reference runs match their committed reports (commands: `.agents/skills/plugin-authoring/SKILL.md`, Gate).
- _Seam-specific checks (golden master, trainer references) unchanged or intentionally updated._
