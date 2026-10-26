---
id: TCK-189
title: Resolve Dependency Security Alerts (npm dev tooling, glib)
status: open
priority: high
decision: pending
assigned: unassigned
created_at: 2026-10-26
---

# TCK-189: Resolve Dependency Security Alerts (npm dev tooling, glib)

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
