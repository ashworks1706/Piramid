//! What the engine measures about itself: latency, lock contention, embedding throughput.
//!
//! Plain atomics and durations, with no dependency on `tracing` or any exporter, so anything can
//! record into it. Where the numbers go is `piramid-observability`, which carries those deps and
//! is linked only by `server` and the binary. Recording is cheap and everywhere; exporting is not.

pub mod embed;
pub mod latency;
pub mod locks;

pub use embed::{EmbedMetrics, EmbedMetricsSnapshot};
pub use latency::{time_operation, time_operation_sync, LatencyTracker};
pub use locks::{record_lock_read, record_lock_write};
