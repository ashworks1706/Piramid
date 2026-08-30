//! Conversions between the HTTP request/response shapes and domain types.
//!
//! Not a service. `services/vector/search.rs` holds the search endpoints; this holds the parsing,
//! override resolution and DTO mapping they share with the embedding and collection services.

use crate::services::types::HitResponse;
use piramid_compute::Metric;
use piramid_core::config::SearchConfig;
use piramid_core::error::{Result, ServerError};
use piramid_core::metadata::{Metadata, MetadataValue};
use piramid_search::Hit;
use std::collections::HashMap;

pub fn parse_metric(metric: Option<String>) -> Result<Metric> {
    match metric.as_deref() {
        None | Some("cosine") => Ok(Metric::Cosine),
        Some("euclidean") => Ok(Metric::Euclidean),
        Some("dot") | Some("dot_product") => Ok(Metric::DotProduct),
        Some(other) => Err(ServerError::InvalidRequest(format!(
            "Unknown metric '{other}'. Expected cosine, euclidean, dot, or dot_product"
        ))
        .into()),
    }
}

pub fn apply_search_overrides(
    base: SearchConfig,
    req_ef: Option<usize>,
    req_nprobe: Option<usize>,
    req_overfetch: Option<usize>,
    preset: Option<String>,
) -> Result<SearchConfig> {
    let mut cfg = base;
    if let Some(preset) = preset {
        match preset.to_lowercase().as_str() {
            "fast" => {
                cfg.ef = Some(50);
                cfg.nprobe = Some(1);
            }
            "high" => {
                cfg.ef = Some(400);
                cfg.nprobe = Some(20);
            }
            other => {
                return Err(ServerError::InvalidRequest(format!(
                    "Unknown search preset '{other}'. Expected fast or high"
                ))
                .into())
            }
        }
    }
    if let Some(ef) = req_ef {
        cfg.ef = Some(ef);
    }
    if let Some(nprobe) = req_nprobe {
        cfg.nprobe = Some(nprobe);
    }
    if let Some(overfetch) = req_overfetch {
        cfg.filter_overfetch = overfetch.max(1);
    }
    Ok(cfg)
}

pub fn hit_to_response(hit: Hit) -> HitResponse {
    HitResponse {
        id: hit.id.to_string(),
        score: hit.score,
        text: hit.text,
        metadata: metadata_to_json(&hit.metadata),
    }
}

pub fn json_to_metadata(json: HashMap<String, serde_json::Value>) -> Metadata {
    let mut metadata = Metadata::new();

    for (k, v) in json {
        let value = match v {
            serde_json::Value::String(s) => MetadataValue::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    MetadataValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    MetadataValue::Float(f)
                } else {
                    continue;
                }
            }
            serde_json::Value::Bool(b) => MetadataValue::Boolean(b),
            serde_json::Value::Null => MetadataValue::Null,
            _ => continue,
        };
        metadata.insert(k, value);
    }

    metadata
}

pub fn metadata_to_json(metadata: &Metadata) -> HashMap<String, serde_json::Value> {
    metadata
        .iter()
        .map(|(k, v)| {
            let json_val = match v {
                MetadataValue::String(s) => serde_json::Value::String(s.clone()),
                MetadataValue::Integer(i) => serde_json::json!(*i),
                MetadataValue::Float(f) => serde_json::json!(*f),
                MetadataValue::Boolean(b) => serde_json::Value::Bool(*b),
                MetadataValue::Null => serde_json::Value::Null,
                MetadataValue::Array(arr) => serde_json::Value::Array(
                    arr.iter()
                        .map(|item| match item {
                            MetadataValue::String(s) => serde_json::Value::String(s.clone()),
                            MetadataValue::Integer(i) => serde_json::json!(*i),
                            MetadataValue::Float(f) => serde_json::json!(*f),
                            MetadataValue::Boolean(b) => serde_json::Value::Bool(*b),
                            _ => serde_json::Value::Null,
                        })
                        .collect(),
                ),
            };
            (k.clone(), json_val)
        })
        .collect()
}
