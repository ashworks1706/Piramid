use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::search::{default_k, SearchTuning};

/// Embed one or more texts and store them.
#[derive(Deserialize)]
pub struct EmbedRequest {
    pub texts: Vec<String>,
    /// One map per text. Empty means no metadata on any of them; otherwise it must be the same
    /// length as `texts`.
    #[serde(default)]
    pub metadata: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize)]
pub struct EmbedResponse {
    pub ids: Vec<String>,
    pub embeddings: Vec<Vec<f32>>,
    pub total_tokens: Option<u32>,
}

/// Embed `query` and search with the result.
#[derive(Deserialize)]
pub struct TextSearchRequest {
    pub query: String,
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
