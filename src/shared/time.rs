//! Wall-clock timestamps for persistence and logs; the function name states the unit (RV-A06: values named
//! `now_millis` used to hold seconds). A clock before the Unix epoch gives 0.

use std::time::{SystemTime, UNIX_EPOCH};

fn since_epoch() -> std::time::Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()
}

/// Seconds since the Unix epoch.
pub fn now_secs() -> i64 {
    since_epoch().as_secs() as i64
}

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> i64 {
    since_epoch().as_millis() as i64
}

/// Nanoseconds since the Unix epoch (truncated to 64 bits; a nonce, not a timestamp).
pub fn now_nanos() -> u64 {
    since_epoch().as_nanos() as u64
}
