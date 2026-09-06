//! Wall-clock timestamps for records that persist.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch. A clock reading before 1970 yields 0.
pub fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}
