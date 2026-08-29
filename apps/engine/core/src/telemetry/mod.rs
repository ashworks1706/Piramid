//! Observability: latency, lock contention, and embedding throughput.
//!
//! This module is named for what it does. It previously sat at `metrics/` and also owned the
//! distance-`Metric` enum plus re-exports of every `compute/` function, which made "metric" mean
//! two unrelated things and gave each kernel two import paths. Distance math now lives in
//! `piramid-compute`; this module is telemetry only and re-exports nothing from elsewhere.

pub mod embed;
pub mod latency;
pub mod locks;

pub use embed::{EmbedMetrics, EmbedMetricsSnapshot};
pub use latency::{time_operation, time_operation_sync, LatencyTracker};
pub use locks::{record_lock_read, record_lock_write};
