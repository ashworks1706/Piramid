//! Conversions between the HTTP request/response shapes and domain types.
//!
//! Not a service. `services/vector/search.rs` holds the search endpoints; this holds the parsing,
//! override resolution and DTO mapping they share with the embedding and collection services.

use crate::services::types::{HitResponse, SearchTuning};
use piramid_compute::Metric;
use piramid_core::config::SearchConfig;
use piramid_core::error::{Result, ServerError};
use piramid_core::metadata::{Filter, Metadata, MetadataValue};
use piramid_search::Hit;
use std::collections::HashMap;

pub fn parse_metric(metric: Option<String>) -> Result<Metric> {
    match metric.as_deref() {
        None | Some("cosine") => Ok(Metric::Cosine),
        Some("euclidean") => Ok(Metric::Euclidean),
        Some("dot") => Ok(Metric::DotProduct),
        Some(other) => Err(ServerError::InvalidRequest(format!(
            "Unknown metric '{other}'. Expected cosine, euclidean, or dot"
        ))
        .into()),
    }
}

/// Layer per-request tuning onto a collection's configured defaults.
pub fn apply_search_overrides(base: SearchConfig, tuning: &SearchTuning) -> Result<SearchConfig> {
    let mut cfg = base;
    if let Some(ef) = tuning.ef {
        if ef == 0 {
            return Err(ServerError::InvalidRequest("ef must be >= 1".to_string()).into());
        }
        cfg.ef = Some(ef);
    }
    if let Some(nprobe) = tuning.nprobe {
        if nprobe == 0 {
            return Err(ServerError::InvalidRequest("nprobe must be >= 1".to_string()).into());
        }
        cfg.nprobe = Some(nprobe);
    }
    if let Some(overfetch) = tuning.overfetch {
        if overfetch == 0 {
            return Err(ServerError::InvalidRequest("overfetch must be >= 1".to_string()).into());
        }
        cfg.filter_overfetch = overfetch;
    }
    Ok(cfg)
}

/// Build a [`Filter`] from `{"field": {"op": value}}`.
pub fn parse_filter(
    raw: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
) -> Result<Option<Filter>> {
    let Some(raw) = raw else {
        return Ok(None);
    };

    let mut filter = Filter::new();
    for (field, ops) in raw {
        for (op, value) in ops {
            filter = match op.as_str() {
                "in" => {
                    let serde_json::Value::Array(items) = value else {
                        return Err(ServerError::InvalidRequest(format!(
                            "filter '{field}.in' expects an array"
                        ))
                        .into());
                    };
                    let values = items
                        .into_iter()
                        .map(|item| json_to_metadata_value(&field, item))
                        .collect::<Result<Vec<_>>>()?;
                    filter.is_in(&field, values)
                }
                "eq" | "ne" | "gt" | "gte" | "lt" | "lte" => {
                    let value = json_to_metadata_value(&field, value)?;
                    match op.as_str() {
                        "eq" => filter.eq(&field, value),
                        "ne" => filter.ne(&field, value),
                        "gt" => filter.gt(&field, value),
                        "gte" => filter.gte(&field, value),
                        "lt" => filter.lt(&field, value),
                        _ => filter.lte(&field, value),
                    }
                }
                other => {
                    return Err(ServerError::InvalidRequest(format!(
                        "Unknown filter operator '{other}' on '{field}'. \
                         Expected eq, ne, gt, gte, lt, lte, or in"
                    ))
                    .into())
                }
            };
        }
    }
    Ok(Some(filter))
}

pub fn hit_to_response(hit: Hit) -> HitResponse {
    HitResponse {
        id: hit.id.to_string(),
        score: hit.score,
        text: hit.text,
        metadata: metadata_to_json(&hit.metadata),
    }
}

/// Convert one JSON value to a [`MetadataValue`], rejecting shapes metadata cannot hold.
///
/// Dropping the field instead would store a document that does not match what was sent, and the
/// caller would only find out at query time.
fn json_to_metadata_value(field: &str, value: serde_json::Value) -> Result<MetadataValue> {
    Ok(match value {
        serde_json::Value::String(s) => MetadataValue::String(s),
        serde_json::Value::Bool(b) => MetadataValue::Boolean(b),
        serde_json::Value::Null => MetadataValue::Null,
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                MetadataValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                MetadataValue::Float(f)
            } else {
                return Err(ServerError::InvalidRequest(format!(
                    "metadata field '{field}' holds a number outside the i64 and f64 ranges"
                ))
                .into());
            }
        }
        serde_json::Value::Array(items) => MetadataValue::Array(
            items
                .into_iter()
                .map(|item| json_to_metadata_value(field, item))
                .collect::<Result<Vec<_>>>()?,
        ),
        serde_json::Value::Object(_) => {
            return Err(ServerError::InvalidRequest(format!(
                "metadata field '{field}' is an object; metadata holds scalars and arrays only"
            ))
            .into())
        }
    })
}

pub fn json_to_metadata(json: HashMap<String, serde_json::Value>) -> Result<Metadata> {
    json.into_iter()
        .map(|(k, v)| {
            let value = json_to_metadata_value(&k, v)?;
            Ok((k, value))
        })
        .collect()
}

pub fn metadata_to_json(metadata: &Metadata) -> HashMap<String, serde_json::Value> {
    metadata
        .iter()
        .map(|(k, v)| (k.clone(), metadata_value_to_json(v)))
        .collect()
}

fn metadata_value_to_json(value: &MetadataValue) -> serde_json::Value {
    match value {
        MetadataValue::String(s) => serde_json::Value::String(s.clone()),
        MetadataValue::Integer(i) => serde_json::json!(*i),
        MetadataValue::Float(f) => serde_json::json!(*f),
        MetadataValue::Boolean(b) => serde_json::Value::Bool(*b),
        MetadataValue::Null => serde_json::Value::Null,
        MetadataValue::Array(items) => {
            serde_json::Value::Array(items.iter().map(metadata_value_to_json).collect())
        }
    }
}
