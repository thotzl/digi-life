---
id: TCK-189
title: Resolve Dependency Security Alerts (npm dev tooling, glib)
status: closed
priority: high
decision: pending
assigned: unassigned
created_at: 2026-10-26
---

# TCK-189: Resolve Dependency Security Alerts (npm dev tooling, glib)

## Re-scope (2026-11-02, epic TCK-196)
Sub-ticket of TCK-196 phase R0 (parallel with TCK-197). Review 2026-11-02: removing the unused `concurrently` drops the `shell-quote` critical alert, removing `tsx` and Stryker (no config file, `test:mutation` unusable) drops the `@babel` alert; then update `vite` (^5) and `vitest`/coverage (^1.6) majors; `rusqlite` 0.29 and `rand` 0.8 checked for advisories (`cargo audit`).


## Description
GitHub reports 29 open Dependabot alerts on `master` (2026-10-26: 4 critical, 9 high, 11 moderate, 5 low; `gh api repos/thotzl/digi-life/dependabot/alerts`). Almost all are in npm development tooling (`package-lock.json`); one is in the Rust lock file. The shipped app bundles none of the npm dev packages, but they run on the developer machine (`vite` dev server, `vitest`, Stryker).

| Package (ecosystem) | Alerts | Worst | First patched | Pulled in by |
| :--- | :--- | :--- | :--- | :--- |
| vitest (npm) | 1 | critical | 3.2.6 | `vitest` ^1.6.0 (dev) |
| tinypool (npm) | 2 | critical | 2.1.2 | vitest |
| shell-quote (npm) | 1 | critical | 1.11.0 | concurrently |
| fast-uri (npm) | 8 | high | 3.1.9 | ajv (tooling) |
| vite (npm) | 8 | high | 6.4.4 | `vite` ^5.0.0 (dev) |
| tmp (npm) | 2 | high | 0.2.6 | tooling |
| source-map-js (npm) | 1 | high | 1.2.2 | vite / postcss |
| esbuild (npm) | 1 | moderate | 0.25.0 | vite |
| brace-expansion (npm) | 2 | moderate | 2.1.7 / 1.1.21 | minimatch |
| @babel/core (npm) | 1 | low | 7.29.6 | Stryker |
| glib (rust) | 1 | moderate | 0.20.0 | Tauri / wry (Linux GTK stack) |

Open Dependabot PRs cover most of them (#99 fast-uri, #80 vite + vitest, #39 source-map-js, #38 brace-expansion, #37 shell-quote + concurrently, #36 tinypool + vitest, #3 @babel/core + Stryker); they were left untouched on purpose during epic TCK-171.

## Scope
- Upgrade the dev tooling on `master` in one step instead of merging the Dependabot PRs one by one: `vite` 5 → ≥ 6.4.4 (major: check `vite.config.ts`, `server.watch.ignored`, the Tauri dev integration), `vitest` 1 → ≥ 3.2.6 with `@vitest/coverage-v8`, `concurrently`, Stryker; then `npm audit` for the transitive rest (fast-uri, tmp, source-map-js, esbuild, brace-expansion).
- Rust: check whether `glib` ≥ 0.20 is reachable with the current Tauri 2 version (`cargo tree -i glib`); otherwise document the alert as accepted (Linux-only GTK binding, iterator unsoundness, not used directly).
- Close the superseded Dependabot PRs afterwards.

## Acceptance
- `gh api repos/thotzl/digi-life/dependabot/alerts?state=open` shows no critical/high alerts; remaining ones documented with reason.
- `npm run build`, `npm test`, `npm run tauri:dev` (UI smoke `.agents/artifacts/uicheck/run-smoke.sh`), `cargo test`, CI green.
- CHANGELOG entry and version bump.

## Verification (2026-11-02, v1.12.28)
- Removed unused dev dependencies `concurrently`, `tsx`, `@stryker-mutator/core`, `@stryker-mutator/vitest-runner` and the `test:mutation` script (grep: no uses).
- `vite` 5.4.21 -> 6.4.5, `vitest` and `@vitest/coverage-v8` 1.6.1 -> 4.1.12 (Vite 7/8 need Node >= 20.19, local Node is 20.13.1; vitest 3.2 still carried the `tinypool` and `@vitest/mocker` alerts). `vite.config.ts` / `vitest.config.ts` unchanged; dev server stays on 8765 with `strictPort`.
- `npm audit`: before 24 vulnerabilities (5 low, 9 moderate, 5 high, 5 critical), after 0. The lock file was generated with npm 10.9.9 (`npx -y npm@10.9.9 install`) because npm 10.5.2 crashes resolving the vitest 4 tree; `npm ci` works with 10.5.2.
- `cargo audit` 0.22.2: 0 vulnerabilities, 8 warnings (unmaintained `proc-macro-error`, five `unic-*` from Tauri build tooling; unsound `glib` 0.18.5 RUSTSEC-2024-0429 via the Linux GTK stack and `tokio` 1.53.1 RUSTSEC-2026-0386 `AsyncFd`, both not used directly); no crate upgrades.
- Gate on master: `npm ci` ok, `npm test` 10 passed, `npm run build` ok, `cargo test` 182 passed / 1 ignored (subagent run), UI smoke (`tauri dev` on Vite 6) OK: Ocean 25/25, trainer generation completed, RTT 6 ms at max warp, 0 console errors.
- Not done: the Dependabot alert list and closing superseded Dependabot PRs via `gh api` (gh is not authenticated in this environment); checked again when CI runs on push.
- Wiki: `wiki/quality/ci-and-tooling.md` section "Dependencies and audits (TCK-189)"; stale RV-F07 notes in `wiki/frontend/structure.md` and TCK-161 corrected.
