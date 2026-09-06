//! What /api/metrics reports.

use serde::Serialize;

#[derive(Serialize)]
pub struct MetricsResponse {
    pub total_collections: usize,
    pub total_vectors: usize,
    pub collections: Vec<CollectionMetrics>,
    pub app_config: piramid_core::config::Config,
    pub wal_stats: Vec<WalStats>,
    pub embedding: EmbeddingMetricsResponse,
}

#[derive(Serialize)]
pub struct CollectionMetrics {
    pub name: String,
    pub vector_count: usize,
    pub index_type: String,
    pub memory_usage_bytes: usize,
    pub insert_latency_ms: Option<f32>,
    pub search_latency_ms: Option<f32>,
    pub lock_read_ms: Option<f32>,
    pub lock_write_ms: Option<f32>,
    pub filter_overfetch: Option<usize>,
    pub hnsw_ef_search: Option<usize>,
    pub ivf_nprobe: Option<usize>,
}

#[derive(Serialize)]
pub struct WalStats {
    pub collection: String,
    pub last_checkpoint: Option<u64>,
    pub checkpoint_age_secs: Option<u64>,
    pub wal_size_bytes: Option<u64>,
}

#[derive(Serialize)]
pub struct EmbeddingMetricsResponse {
    pub requests: u64,
    pub texts: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms: Option<f32>,
}
