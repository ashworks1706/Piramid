//! Moving averages of operation latency, per collection.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LatencyTracker {
    // Microseconds, so the atomics stay integral.
    insert_latency_us: Arc<AtomicU64>,
    search_latency_us: Arc<AtomicU64>,
    delete_latency_us: Arc<AtomicU64>,
    update_latency_us: Arc<AtomicU64>,
    lock_read_latency_us: Arc<AtomicU64>,
    lock_write_latency_us: Arc<AtomicU64>,

    insert_count: Arc<AtomicU64>,
    search_count: Arc<AtomicU64>,
    delete_count: Arc<AtomicU64>,
    update_count: Arc<AtomicU64>,
    lock_read_count: Arc<AtomicU64>,
    lock_write_count: Arc<AtomicU64>,
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            insert_latency_us: Arc::new(AtomicU64::new(0)),
            search_latency_us: Arc::new(AtomicU64::new(0)),
            delete_latency_us: Arc::new(AtomicU64::new(0)),
            update_latency_us: Arc::new(AtomicU64::new(0)),
            lock_read_latency_us: Arc::new(AtomicU64::new(0)),
            lock_write_latency_us: Arc::new(AtomicU64::new(0)),
            insert_count: Arc::new(AtomicU64::new(0)),
            search_count: Arc::new(AtomicU64::new(0)),
            delete_count: Arc::new(AtomicU64::new(0)),
            update_count: Arc::new(AtomicU64::new(0)),
            lock_read_count: Arc::new(AtomicU64::new(0)),
            lock_write_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_insert(&self, duration: Duration) {
        self.insert_count.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.update_moving_average(&self.insert_latency_us, us, &self.insert_count);
    }

    pub fn record_search(&self, duration: Duration) {
        self.search_count.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.update_moving_average(&self.search_latency_us, us, &self.search_count);
    }

    pub fn record_delete(&self, duration: Duration) {
        self.delete_count.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.update_moving_average(&self.delete_latency_us, us, &self.delete_count);
    }

    pub fn record_update(&self, duration: Duration) {
        self.update_count.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.update_moving_average(&self.update_latency_us, us, &self.update_count);
    }

    pub fn record_lock_read(&self, duration: Duration) {
        self.lock_read_count.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.update_moving_average(&self.lock_read_latency_us, us, &self.lock_read_count);
    }

    pub fn record_lock_write(&self, duration: Duration) {
        self.lock_write_count.fetch_add(1, Ordering::Relaxed);
        let us = duration.as_micros() as u64;
        self.update_moving_average(&self.lock_write_latency_us, us, &self.lock_write_count);
    }

    pub fn avg_insert_latency_ms(&self) -> Option<f32> {
        let us = self.insert_latency_us.load(Ordering::Relaxed);
        if us > 0 {
            Some(us as f32 / 1000.0)
        } else {
            None
        }
    }

    pub fn avg_search_latency_ms(&self) -> Option<f32> {
        let us = self.search_latency_us.load(Ordering::Relaxed);
        if us > 0 {
            Some(us as f32 / 1000.0)
        } else {
            None
        }
    }

    pub fn avg_delete_latency_ms(&self) -> Option<f32> {
        let us = self.delete_latency_us.load(Ordering::Relaxed);
        if us > 0 {
            Some(us as f32 / 1000.0)
        } else {
            None
        }
    }

    pub fn avg_update_latency_ms(&self) -> Option<f32> {
        let us = self.update_latency_us.load(Ordering::Relaxed);
        if us > 0 {
            Some(us as f32 / 1000.0)
        } else {
            None
        }
    }

    pub fn avg_lock_read_latency_ms(&self) -> Option<f32> {
        let us = self.lock_read_latency_us.load(Ordering::Relaxed);
        if us > 0 {
            Some(us as f32 / 1000.0)
        } else {
            None
        }
    }

    pub fn avg_lock_write_latency_ms(&self) -> Option<f32> {
        let us = self.lock_write_latency_us.load(Ordering::Relaxed);
        if us > 0 {
            Some(us as f32 / 1000.0)
        } else {
            None
        }
    }

    /// Fold a new sample into the running average.
    fn update_moving_average(&self, avg: &AtomicU64, new_value: u64, count: &AtomicU64) {
        let current = avg.load(Ordering::Relaxed);
        let cnt = count.load(Ordering::Relaxed);

        // A plain mean until there are enough samples for the EMA to mean anything.
        if cnt <= 5 {
            let new_avg = ((current * (cnt - 1)) + new_value) / cnt;
            avg.store(new_avg, Ordering::Relaxed);
        } else {
            let new_avg = ((current * 4) + new_value) / 5;
            avg.store(new_avg, Ordering::Relaxed);
        }
    }
}

/// Run an async operation, returning its result alongside how long it took.
pub async fn time_operation<F, T>(operation: F) -> (T, Duration)
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let result = operation.await;
    let duration = start.elapsed();
    (result, duration)
}

/// Synchronous counterpart to [`time_operation`].
pub fn time_operation_sync<F, T>(operation: F) -> (T, Duration)
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let result = operation();
    let duration = start.elapsed();
    (result, duration)
}
