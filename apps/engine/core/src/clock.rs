//! Wall-clock timestamps for records that persist.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch; saturates at 0 rather than panicking on a pre-1970 clock.
pub fn unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}
