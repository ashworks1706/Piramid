//! Key-value data stored alongside a vector, and the values it can hold.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A value in a document's metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<MetadataValue>),
    Null,
}

impl MetadataValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            MetadataValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            MetadataValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            MetadataValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            MetadataValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

// These exist so callers can write `metadata([("k", "v".into())])` without naming the variant.
impl From<String> for MetadataValue {
    fn from(s: String) -> Self {
        MetadataValue::String(s)
    }
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        MetadataValue::String(s.to_string())
    }
}

impl From<i64> for MetadataValue {
    fn from(i: i64) -> Self {
        MetadataValue::Integer(i)
    }
}

impl From<i32> for MetadataValue {
    fn from(i: i32) -> Self {
        MetadataValue::Integer(i64::from(i))
    }
}

impl From<f64> for MetadataValue {
    fn from(f: f64) -> Self {
        MetadataValue::Float(f)
    }
}

impl From<f32> for MetadataValue {
    fn from(f: f32) -> Self {
        MetadataValue::Float(f64::from(f))
    }
}

impl From<bool> for MetadataValue {
    fn from(b: bool) -> Self {
        MetadataValue::Boolean(b)
    }
}

pub type Metadata = HashMap<String, MetadataValue>;

/// Build a [`Metadata`] map from an array of pairs.
pub fn metadata<const N: usize>(pairs: [(&str, MetadataValue); N]) -> Metadata {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}
