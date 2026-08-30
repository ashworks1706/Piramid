use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Insert one or more documents.
///
/// Always a list, even for one document. A singular shape alongside this one meant two request
/// bodies, two response bodies and two validation paths for the same operation.
#[derive(Deserialize)]
pub struct InsertRequest {
    pub vectors: Vec<Vec<f32>>,
    pub texts: Vec<String>,
    /// One map per vector. Empty means no metadata on any of them; otherwise it must be the same
    /// length as `vectors`.
    #[serde(default)]
    pub metadata: Vec<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub normalize: bool,
}

#[derive(Serialize)]
pub struct InsertResponse {
    pub ids: Vec<String>,
    pub count: usize,
    pub latency_ms: f32,
}

#[derive(Serialize)]
pub struct VectorResponse {
    pub id: String,
    pub vector: Vec<f32>,
    pub text: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ListVectorsQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    100
}

#[derive(Deserialize)]
pub struct DeleteVectorsRequest {
    pub ids: Vec<String>,
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub deleted_count: usize,
    pub latency_ms: f32,
}

#[derive(Deserialize)]
pub struct UpsertRequest {
    pub id: Option<String>,
    pub vector: Vec<f32>,
    pub text: String,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub normalize: bool,
}

#[derive(Serialize)]
pub struct UpsertResponse {
    pub id: String,
    pub created: bool,
    pub latency_ms: f32,
}
