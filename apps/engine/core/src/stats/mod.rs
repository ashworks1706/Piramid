//! What the engine measures about itself: latency, lock contention, embedding throughput.
//!
//! Plain atomics and durations, with no dependency on `tracing`, OpenTelemetry, or anything else
//! that ships data anywhere — which is why this can live in `core` and be recorded into from
//! `collections`, `server`, and anywhere else without dragging exporters along.
//!
//! Where these numbers *go* is `piramid-observability`: the tracing subscriber, the OTLP
//! exporter, and the Prometheus encoder. That crate carries the heavy dependencies, and only the
//! binary and `server` link it.
//!
//! The two were called `telemetry` and `observability`, which most engineers use
//! interchangeably. The split is real — recording is cheap and ubiquitous, exporting is
//! expensive and centralized — so the names now say which is which.

pub mod embed;
pub mod latency;
pub mod locks;

pub use embed::{EmbedMetrics, EmbedMetricsSnapshot};
pub use latency::{time_operation, time_operation_sync, LatencyTracker};
pub use locks::{record_lock_read, record_lock_write};
