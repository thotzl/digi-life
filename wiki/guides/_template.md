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
5. _Docs: the contract table or list that names the plugin._

## Done when

- _The contract's test recipe passes._
- `cargo test`, `npm test`, `npm run build` and `node scripts/check-links.mjs` are green.
- _Seam-specific checks (golden master, trainer references) unchanged or intentionally updated._
