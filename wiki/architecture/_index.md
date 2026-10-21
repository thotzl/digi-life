# Architecture

- [[architecture/system-overview]] — process structure, layers, design principles, target architecture
- [[architecture/simulation-engine]] — engine thread, frame timing, warp cap, events, hot-loop rules
- [[architecture/ipc-protocol]] — commands, actions, `simulation-state` event, ts-rs bindings
- [[architecture/persistence]] — SQLite location, schema, open DB findings
- [[architecture/configuration]] — `config.json` embedding and dead keys
- [[architecture/invariants]] — rules that must not regress
- [[architecture/performance]] — hot-loop findings and status
- [[architecture/history]] — platform evolution from TS/Node.js to Tauri/Rust (closed tickets)
