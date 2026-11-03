---
sources: [wiki/contracts/event-sink.md, src/server/engine/sink.rs, src/server/engine/mod.rs]
---

# Guide: add an event sink

An event sink is a destination for the engine's typed UI events (the window today; a replay recorder or an event log
of a headless engine run). Read first: [[wiki/contracts/event-sink]] and [[wiki/architecture/invariants]].

## Steps
1. Add a struct implementing `EventSink` to `src/server/engine/sink.rs` (or a module next to its user):
   `emit` only buffers (`self.pending.push(event)`), `flush` sends the buffer through `drain_coalesced`, so the order
   is kept and `DATABASE_CHANGED` leaves at most once per flush.
2. A send failure in `flush` is logged with `log::error!` and the next event is still sent; `flush` never panics.
3. Registration: there is no registry. Pass the sink to `Engine::new`, `Engine::handle`, `Engine::frame` and
   `run_loop` where the engine is built (`spawn_simulation_thread` builds the `TauriSink`; a headless tool builds its
   own engine with `open_store` or `MemoryDb`).
4. Config keys: none.
5. Payloads stay typed `UiEvent`s; a sink that writes them to a file serializes the `UiEvent` (invariant 13).
6. Tests in `src/server/engine/tests.rs`: emit a few events (two `DATABASE_CHANGED`), flush, assert order and the
   coalescing, as in `test_sink_coalesces_database_changed_per_flush`.
7. Docs, same commit: name the sink in the contract (Purpose) and the row in "Registered plugins" of `wiki/architecture/module-map.md`.

## Done when
- `cargo test server::engine::` passes, including the new test.
- The project gate is green: `cargo fmt --check`, `cargo clippy --all-targets --features runner -- -D warnings`,
  `cargo test`, `npm test`, `npm run lint`, `npm run build`, `node scripts/check-links.mjs`, and the four CI
  reference runs match their committed reports (commands: `.agents/skills/plugin-authoring/SKILL.md`, Gate).
- Golden master and CI references unchanged (a sink never changes the simulation).
