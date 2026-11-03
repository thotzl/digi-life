---
sources: [tickets/open/TCK-156-engine-decomposition.md, src/server/engine/sink.rs, src/server/engine/mod.rs, src/server/ipc.rs]
---

# Contract: event sink

## Purpose

An event sink is where the engine's typed UI events go (`src/server/engine/sink.rs`, TCK-156). Modes and the engine
only `emit`; the run loop `flush`es once per frame. `TauriSink` sends each `UiEvent` as the Tauri event
`simulation-state` to the window; `VecSink` records them for tests, so the engine runs without a window. A planned
sink (headless Ocean runner of TCK-158, a recorder for replays) implements the same two methods.

## Signature

```rust
// src/server/engine/sink.rs
pub trait EventSink {
    fn emit(&mut self, event: UiEvent);   // buffer, in order
    fn flush(&mut self);                  // send the buffer, DATABASE_CHANGED at most once
}
pub fn drain_coalesced(pending: &mut Vec<UiEvent>, send: impl FnMut(UiEvent));
pub struct TauriSink;                     // TauriSink::new(window: tauri::WebviewWindow)
pub struct VecSink { pub events: Vec<UiEvent>, pub flushes: usize }   // VecSink::new(), take() = flush + drain
```

## Lifecycle / call order

1. Any number of `emit` calls during a frame: inputs (`Engine::handle`), the active mode's `step` and
   `emit_telemetry`.
2. One `flush` per frame at the end of `run_loop`'s iteration: events leave in emit order; every `DATABASE_CHANGED`
   after the first of the frame is dropped (the UI only refetches its lists).
3. Events emitted before the UI listens (engine start) are flushed with the first frame; the UI asks for the full state
   with `CLIENT_READY`, so nothing a late UI needs depends on them (the store error is repeated on every handshake).

## Invariants

- `emit` never sends: simulation steps cannot block on the window ([[wiki/architecture/invariants]] 2, 6).
- Order is kept; only `DATABASE_CHANGED` is coalesced, per flush.
- A send failure (`TauriSink`) is logged and the next event is still sent.
- The payload is always a typed `UiEvent` (no `json!`, [[wiki/contracts/ipc-message]]).

## Registration point

`spawn_simulation_thread` builds the `TauriSink` from the window; tests and headless tools pass their own sink to
`Engine::new`, `Engine::handle`, `Engine::frame` and `run_loop`.

## Config keys

None.

## Events / IPC

Carries every `UiEvent` of the engine on the one Tauri event `simulation-state` (`UI_EVENT_NAME`,
[[wiki/architecture/ipc-protocol]]).

## Test recipe

`cargo test server::engine::tests::test_sink_coalesces_database_changed_per_flush` and the engine tests that assert on
`VecSink::take()`.

## Minimal example

```rust
use crate::server::engine::{EventSink, VecSink};
use crate::server::ipc::{LogKind, UiEvent};

let mut sink = VecSink::new();
sink.emit(UiEvent::database_changed());
sink.emit(UiEvent::log("hello", LogKind::System));
sink.emit(UiEvent::database_changed());
assert_eq!(sink.take().len(), 2);   // DATABASE_CHANGED, LOG_EVENT
```
