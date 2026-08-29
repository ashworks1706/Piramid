// Flat (brute force) index implementation
// O(N) search - compares query against all vectors
// Best for: small collections, zero build time, 100% recall

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::config::FlatConfig;
use crate::traits::{IndexDetails, IndexStats, IndexType, VectorIndex, VectorReader};
use piramid_core::error::{IndexError, Result};

// Stores nothing except config, vectors are in main storage
#[derive(Clone, Serialize, Deserialize)]
pub struct FlatIndex {
    config: FlatConfig,
    vector_ids: Vec<Uuid>, // Track which vectors we've seen
}

impl FlatIndex {
    pub fn new(config: FlatConfig) -> Self {
        FlatIndex {
            config,
            vector_ids: Vec::new(),
        }
    }
}
impl VectorIndex for FlatIndex {
    fn insert(&mut self, id: Uuid, _vector: &[f32], _vectors: &dyn VectorReader) {
        // Just track the ID - no indexing structure needed
        if !self.vector_ids.contains(&id) {
            self.vector_ids.push(id);
        }
    }

    // Search for nearest neighbors to the query vector. The filter and metadata parameters are also ignored in this simple implementation, but they could be used in a more advanced version to filter results based on metadata or other criteria.
    fn search(&self, request: crate::IndexSearchRequest<'_>) -> Result<Vec<Uuid>> {
        let crate::IndexSearchRequest {
            query, k, vectors, ..
        } = request;
        let mut distances = Vec::with_capacity(self.vector_ids.len());
        for id in &self.vector_ids {
            let vec = vectors.get(id).ok_or_else(|| {
                IndexError::SearchFailed(format!("Flat index references missing vector {id}"))
            })?;
            let score = self.config.metric.calculate(query, vec, self.config.mode);
            distances.push((*id, score));
        }

        // Sort by score (descending for similarity)
        distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top k IDs
        Ok(distances.iter().take(k).map(|(id, _)| *id).collect())
    }

    fn remove(&mut self, id: &Uuid) {
        self.vector_ids.retain(|vid| vid != id);
    }

    fn stats(&self) -> IndexStats {
        IndexStats {
            index_type: IndexType::Flat,
            total_vectors: self.vector_ids.len(),
            memory_usage_bytes: self.vector_ids.len() * std::mem::size_of::<Uuid>(),
            details: IndexDetails::Flat,
        }
    }

    fn index_type(&self) -> IndexType {
        IndexType::Flat
    }

    fn to_serializable(&self) -> crate::SerializableIndex {
        crate::SerializableIndex::Flat(self.clone())
    }
}
