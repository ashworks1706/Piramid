use serde::Deserialize;
use std::collections::HashMap;

use super::search::{default_k, SearchTuning};

/// Search restricted to hits scoring at least `min_score`.
#[derive(Deserialize)]
pub struct RangeSearchRequest {
    pub vectors: Vec<Vec<f32>>,
    pub min_score: f32,
    #[serde(default)]
    pub metric: Option<String>,
    #[serde(default = "default_k")]
    pub k: usize,
    /// Metadata predicate, as `{"field": {"op": value}}`.
    #[serde(default)]
    pub filter: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    #[serde(flatten)]
    pub tuning: SearchTuning,
}
