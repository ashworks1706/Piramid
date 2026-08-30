//! Wall-clock timestamps for records that persist.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch.
///
/// Saturates at 0 for a clock set before 1970 rather than panicking. It was `unwrap` at twelve
/// call sites across five crates; one of them tripping would have killed a write path over a
/// misconfigured machine clock, which is not a reason to lose data.
pub fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}
