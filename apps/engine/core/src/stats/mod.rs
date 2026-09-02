//! What the engine measures about itself: latency, lock contention, embedding throughput.

pub mod embed;
pub mod latency;
pub mod locks;

pub use embed::{EmbedMetrics, EmbedMetricsSnapshot};
pub use latency::LatencyTracker;
pub use locks::{record_lock_read, record_lock_write};
