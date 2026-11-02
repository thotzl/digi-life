---
id: TCK-197
title: "R0 Safety Net: References, Golden Master on Model Fields, XSS/CSP, Doc Checks"
status: open
priority: high
assigned: agent
created_at: 2026-11-02
epic_phase: TCK-196 R0
depends_on: TCK-196
---

# TCK-197: R0 Safety Net

Everything the later refactor phases need to prove "behaviour unchanged", plus one exploitable bug that must not wait.

## Scope
1. **Second trainer reference report.** CI covers only the standard room (3 gens, pop 16, `tests/reference/trainer-ci.json`). Add `tests/reference/trainer-ci-curriculum.json`: curriculum mode, rehearsal off, `--eval-final standard,exploration --eval-trials 3`, a few generations so the gate moves at least once; compared with `tests/reference/compare-trainer-report.mjs` in `ci.yml`. Exploration, curriculum gate, rehearsal and `final_eval` are then covered.
2. **Golden master on model fields only.** `GOLDEN_FINGERPRINT` (`src/biology/dna.rs` ~3034) hashes the full serde output incl. display strings and dead fields, so a refactor that moves labels or drops dead payload changes it. Hash only model fields (body traits, organs with kind/side/params, synapse key + weight + owner, neuron roles/tau/bias). Add the missing motifs `OUT`, `STM`, `TEM` to the golden-master genome generator so the OUT gene is exercised. This is one intended fingerprint change, stated here; the model itself does not change (no `MODEL_VERSION` bump).
3. **XSS fix.** `frontend/core/styled.ts:154-161` parses any string child starting with `<` as HTML; `CatalogueView.ts` (~392) passes `creature.name` / `creature.source` (user input via `prompt`). With `tauri.conf.json` `"csp": null` a name like `<img src=x onerror=…>` gets full IPC access. Fix: string children are always text; HTML only through an explicit `html()` marker for static icon SVG. Regression test in vitest.
4. **CSP.** Set a restrictive `app.security.csp` (default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline' as long as styled injects styles; img-src 'self' data:; connect-src ipc: http://ipc.localhost) and verify dev + release build start (UI smoke + installed check).
5. **Doc link check in CI.** A small node script checks wikilinks (`[[wiki/...]]`) and `sources:` paths in `wiki/**/*.md`, skills and tickets/open|ongoing; fails on missing targets. Fix the current breaks (23 articles cite deleted `docs/revisit/*`, `invariants.md` cites the absent `.agents/state/active-task.md`, skill `simulation-analytics` links `tickets/ongoing/TCK-171-...` now in `closed/`).
6. **Close checklist.** Add to `.agents/skills/project-baseline/SKILL.md`: on closing a ticket update wiki/contract, skill links, the parent epic's status, CHANGELOG, version; checked by the link check.
7. **Stale texts.** `README.md:17` ("no frontend test files"), `AGENTS.md:23` (current focus TCK-171 → TCK-196), skill `biology-dna` (pool tickets TCK-152/153/144 are closed).

## Acceptance
- CI runs both trainer references and the link check; all green.
- New golden master stated in the verification; `cargo test` green; trainer references unchanged.
- XSS regression test fails before the fix, passes after; app starts with CSP in dev and release; 0 console errors in the UI smoke.
