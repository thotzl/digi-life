---
id: TCK-197
title: "R0 Safety Net: References, Golden Master on Model Fields, XSS/CSP, Doc Checks"
status: closed
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

## Verification (2026-11-02, v1.12.29)
- Reproduction first (XSS): `npx vitest run frontend/core/styled.test.ts` before the fix failed 3/3: `AssertionError: expected [ '<img src=x onerror=alert(1)>' ] to deeply equal []` (string child parsed as HTML), `expected [ '  <svg onload=alert(1)>' ] to deeply equal []`, `expected 'undefined' to be 'function'` (no `html()` marker). After the fix 3/3 pass. `frontend/core/styled.ts`: string/number children are always text, markup only via `html()` (`TrustedHtml`); icons in `Icons.ts`, legend spans in `TrainerView.ts`, fallback SVG in `UnifiedDiagnosticsPanel.ts` converted.
- CSP in `tauri.conf.json` as specified, plus `dangerousDisableAssetCspModification: ["style-src"]` (Tauri's style nonces would make browsers ignore `'unsafe-inline'`, which the inline style attributes and Vite dev styles need). UI smoke in the dev app OK (Ocean, inject, trainer generation, pause, reset; 0 console errors); release exe (`target/release/BroodBattle.exe`, v1.12.28 build) via `installed.cjs`: menu, Ocean 25/25, trainer to generation 3, 0 console errors.
- Golden master (intended, no model change, no `MODEL_VERSION` bump): `0x0ab34073297ace74` -> `0x6b78c31a4d19b0e8`. Hash over model fields only (body traits incl. methylations; organs kind/side/params; neurons role/tau/bias/activation; synapses key/weight/owner gene/plastic) instead of the full serde output; motifs `OUT`, `STM`, `TEM` added to `golden_genomes`.
- References: standard `tests/reference/trainer-ci.json` unchanged and matching; new `tests/reference/trainer-ci-curriculum.json` (`--seed ci-curriculum --generations 4 --scenario curriculum --population 16 --rehearsal 0 --curriculum-upper 0.05 --curriculum-lower 0.02 --eval-final standard,exploration --eval-trials 3`; gate hold, hold, up, up; final_eval standard 0.292, exploration 0.000); new `tests/reference/trainer-ci-rehearsal.json` (`--seed ci-rehearsal ... --rehearsal 0.25`, 2 rehearsal trials per generation, gate hold x3, up). All three reproduce exactly; CI runs all three.
- `cargo test` 182 passed, 1 ignored; `npm test` 13 passed (3 files); `npm run build` ok.
- `scripts/check-links.mjs` in CI: wikilinks, `sources:` and doc paths in wiki, skills and open/ongoing tickets; 55 breaks found and fixed (dead `docs/revisit/*`, `HANDOFF.md`, `.agents/state/active-task.md` sources in 24 articles, TCK-171 paths); now OK.
- Close checklist in `project-baseline`; `client-rendering` rule for `html()`/CSP; stale texts in `README.md`, `AGENTS.md` (focus TCK-196), `biology-dna`, `simulation-analytics` fixed. `.claude/` (agent worktrees) added to `.gitignore`.
