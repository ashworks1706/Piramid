//! Embedding throughput counters.
//!
//! Atomics rather than a lock: written on every embed call, read only when metrics are scraped.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Default)]
pub struct EmbedMetrics {
    requests: AtomicU64,
    texts: AtomicU64,
    total_tokens: AtomicU64,
    total_latency_ns: AtomicU64,
}

#[derive(Debug, Clone, Copy)]
pub struct EmbedMetricsSnapshot {
    pub requests: u64,
    pub texts: u64,
    pub total_tokens: u64,
    pub avg_latency_ms: Option<f32>,
}

impl EmbedMetrics {
    pub fn record(&self, request_count: u64, text_count: u64, token_count: u64, latency: Duration) {
        self.requests.fetch_add(request_count, Ordering::Relaxed);
        self.texts.fetch_add(text_count, Ordering::Relaxed);
        self.total_tokens.fetch_add(token_count, Ordering::Relaxed);
        self.total_latency_ns
            .fetch_add(latency.as_nanos() as u64, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> EmbedMetricsSnapshot {
        let requests = self.requests.load(Ordering::Relaxed);
        let total_latency_ns = self.total_latency_ns.load(Ordering::Relaxed);
        let avg_latency_ms = if requests > 0 {
            Some((total_latency_ns as f64 / requests as f64 / 1_000_000.0) as f32)
        } else {
            None
        };
        EmbedMetricsSnapshot {
            requests,
            texts: self.texts.load(Ordering::Relaxed),
            total_tokens: self.total_tokens.load(Ordering::Relaxed),
            avg_latency_ms,
        }
    }
}
