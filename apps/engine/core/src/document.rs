//! The record a collection stores, and a scored one.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::metadata::Metadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub vector: Vec<f32>,
    pub text: String,
    #[serde(default)]
    pub metadata: Metadata,
}

impl Document {
    pub fn new(vector: Vec<f32>, text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            vector,
            text,
            metadata: Metadata::new(),
        }
    }

    pub fn with_metadata(vector: Vec<f32>, text: String, metadata: Metadata) -> Self {
        Self {
            id: Uuid::new_v4(),
            vector,
            text,
            metadata,
        }
    }

    /// The stored vector.
    pub fn vector(&self) -> &[f32] {
        &self.vector
    }
}

/// A search result: a stored document and how well it matched.
///
/// Holds the [Document] itself rather than restating its fields.
#[derive(Debug, Clone)]
pub struct Hit {
    /// Similarity, normalised so higher is closer.
    pub score: f32,
    /// The document that matched.
    pub document: Document,
}
