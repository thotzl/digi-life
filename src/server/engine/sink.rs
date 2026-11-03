//! `EventSink` (TCK-156): where the engine's typed UI events go. Modes only `emit`; the run loop `flush`es once per
//! frame. `TauriSink` sends to the window (event `simulation-state`), `VecSink` records for tests. Both buffer the
//! frame's events and coalesce repeated `DATABASE_CHANGED` on flush (the UI only refetches its lists). Contract:
//! `wiki/contracts/event-sink.md`.

use tauri::Emitter;

use crate::server::ipc::{UiEvent, UI_EVENT_NAME};

pub trait EventSink {
    /// Buffers one event (never sends from inside a simulation step).
    fn emit(&mut self, event: UiEvent);
    /// Sends the buffered events in order, `DATABASE_CHANGED` at most once (the first).
    fn flush(&mut self);
}

/// Drains `pending` in order into `send`, dropping every `DATABASE_CHANGED` after the first.
pub fn drain_coalesced(pending: &mut Vec<UiEvent>, mut send: impl FnMut(UiEvent)) {
    let mut database_changed_sent = false;
    for event in pending.drain(..) {
        if event.is_database_changed() {
            if database_changed_sent {
                continue;
            }
            database_changed_sent = true;
        }
        send(event);
    }
}

/// The desktop app's sink: one Tauri event per `UiEvent`.
pub struct TauriSink {
    window: tauri::WebviewWindow,
    pending: Vec<UiEvent>,
}

impl TauriSink {
    pub fn new(window: tauri::WebviewWindow) -> Self {
        Self { window, pending: Vec::new() }
    }
}

impl EventSink for TauriSink {
    fn emit(&mut self, event: UiEvent) {
        self.pending.push(event);
    }

    fn flush(&mut self) {
        let window = &self.window;
        drain_coalesced(&mut self.pending, |event| {
            if let Err(e) = window.emit(UI_EVENT_NAME, event) {
                log::error!("emitting '{}' failed: {}", UI_EVENT_NAME, e);
            }
        });
    }
}

/// Test sink: flushed events in `events`, as the UI would receive them.
#[derive(Default)]
pub struct VecSink {
    pending: Vec<UiEvent>,
    pub events: Vec<UiEvent>,
    pub flushes: usize,
}

impl VecSink {
    pub fn new() -> Self {
        Self::default()
    }

    /// Flushes and returns everything sent so far.
    pub fn take(&mut self) -> Vec<UiEvent> {
        self.flush();
        std::mem::take(&mut self.events)
    }
}

impl EventSink for VecSink {
    fn emit(&mut self, event: UiEvent) {
        self.pending.push(event);
    }

    fn flush(&mut self) {
        let events = &mut self.events;
        drain_coalesced(&mut self.pending, |event| events.push(event));
        self.flushes += 1;
    }
}
