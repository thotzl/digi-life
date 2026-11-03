# Architecture

- [[wiki/architecture/system-overview]] — process structure, plugin host and registries, layers
- [[wiki/architecture/module-map]] — one row per module and registered plugin, with its contract
- [[wiki/architecture/simulation-engine]] — engine thread, frame timing, warp cap, events, hot-loop rules
- [[wiki/architecture/ipc-protocol]] — commands, actions, `simulation-state` event, ts-rs bindings
- [[wiki/architecture/persistence]] — SQLite location, schema, model-version reset, trainer retention, open DB findings
- [[wiki/architecture/configuration]] — `config.json` sections, loading, key list and key tests
- [[wiki/architecture/invariants]] — the one copy of the rules that must not regress
- [[wiki/architecture/performance]] — hot-loop findings and status

Finished work is listed in `CHANGELOG.md`; articles cite the ticket that produced a result.
