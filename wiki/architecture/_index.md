# Architecture

- [[wiki/architecture/system-overview]] — process structure, layers, design principles, target architecture
- [[wiki/architecture/simulation-engine]] — engine thread, frame timing, warp cap, events, hot-loop rules
- [[wiki/architecture/ipc-protocol]] — commands, actions, `simulation-state` event, ts-rs bindings
- [[wiki/architecture/persistence]] — SQLite location, schema, model-version reset, trainer retention, open DB findings
- [[wiki/architecture/configuration]] — `config.json` embedding and dead keys
- [[wiki/architecture/invariants]] — rules that must not regress
- [[wiki/architecture/performance]] — hot-loop findings and status

Finished work is listed in `CHANGELOG.md`; articles cite the ticket that produced a result.
