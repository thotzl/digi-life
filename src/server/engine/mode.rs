//! `Mode` (TCK-156): one simulation the engine can run (`OceanMode`, `TrainerMode`). The engine routes each mode's
//! actions to it in every mode, and steps and streams only the active mode. Contract: `wiki/contracts/mode.md`.

use std::time::{Duration, Instant};

use crate::config::AppConfig;
use crate::server::ipc::UiEvent;

use super::sink::EventSink;
use super::store::EngineStore;

/// Warp cap: at most this many simulation steps per 60 Hz frame for every mode (invariant 2). Unbounded steps block
/// the action channel and the UI. A requested warp is clamped to it and the clamped value is echoed to the UI.
pub const MAX_STEPS_PER_FRAME: usize = 35;

/// Steps per frame for a requested warp.
pub fn clamp_warp(requested: usize) -> usize {
    requested.min(MAX_STEPS_PER_FRAME)
}

/// What a mode may touch during a call: the event sink, the store, the config and the frame time.
pub struct ModeCtx<'a> {
    pub sink: &'a mut dyn EventSink,
    pub store: &'a dyn EngineStore,
    pub config: &'a AppConfig,
    /// Start of the current frame (timers compare against it; tests pass their own)
    pub now: Instant,
}

impl ModeCtx<'_> {
    pub fn emit(&mut self, event: impl Into<UiEvent>) {
        self.sink.emit(event.into());
    }
}

/// A simulation mode. Calls never block on IO beyond the store; events go to `ctx.sink` and are flushed by the loop.
pub trait Mode {
    /// The mode's typed client actions (`OceanAction`, `TrainerAction`)
    type Action;
    /// `SET_MODE` to this mode (also when it is already active: a reloaded UI gets the mode's state again).
    fn on_enter(&mut self, ctx: &mut ModeCtx);
    /// `SET_MODE` to another mode while this one is active.
    fn on_exit(&mut self, ctx: &mut ModeCtx);
    /// One of the mode's actions; called in every mode (actions are not dropped while the mode is inactive).
    fn handle(&mut self, action: Self::Action, ctx: &mut ModeCtx);
    /// One frame of the active mode: up to `MAX_STEPS_PER_FRAME` steps when running, plus periodic persistence.
    fn step(&mut self, ctx: &mut ModeCtx);
    /// The active mode's throttled streams (telemetry, food), after `step`.
    fn emit_telemetry(&mut self, ctx: &mut ModeCtx);
}

/// A throttle: `due` is true at most once per `period`.
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    period: Duration,
    last: Instant,
}

impl Interval {
    /// First due one `period` after `start`.
    pub fn new(period: Duration, start: Instant) -> Self {
        Self { period, last: start }
    }

    /// Whether `period` has passed since the last due time; if so, `now` becomes the last due time.
    pub fn due(&mut self, now: Instant) -> bool {
        if now.saturating_duration_since(self.last) >= self.period {
            self.last = now;
            true
        } else {
            false
        }
    }
}
