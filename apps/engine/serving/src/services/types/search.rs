use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn default_k() -> usize {
    10
}

/// Recall/speed knobs a request may override, shared by every search shape.
#[derive(Deserialize, Clone, Default)]
pub struct SearchTuning {
    /// HNSW candidate-list width.
    #[serde(default)]
    pub ef: Option<usize>,
    /// IVF partitions to scan.
    #[serde(default)]
    pub nprobe: Option<usize>,
    /// Multiplier applied to `k` when a filter is present.
    #[serde(default)]
    pub overfetch: Option<usize>,
}

/// Search a collection with one or more query vectors.
#[derive(Deserialize)]
pub struct SearchRequest {
    pub vectors: Vec<Vec<f32>>,
    #[serde(default = "default_k")]
    pub k: usize,
    #[serde(default)]
    pub metric: Option<String>,
    /// Metadata predicate, as `{"field": {"op": value}}`.
    #[serde(default)]
    pub filter: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    #[serde(flatten)]
    pub tuning: SearchTuning,
}

#[derive(Serialize)]
pub struct HitResponse {
    pub id: String,
    pub score: f32,
    pub text: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// One result list per query vector, in request order.
#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<Vec<HitResponse>>,
    pub latency_ms: f32,
}
