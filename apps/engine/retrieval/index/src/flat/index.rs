//! Brute-force index.
//!
//! Compares the query against every vector: `O(N)`, perfect recall, no build cost. The right
//! choice below roughly ten thousand vectors, where traversal overhead outweighs the scan.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::config::FlatConfig;
use crate::traits::{IndexDetails, IndexStats, IndexType, VectorIndex, VectorReader};
use piramid_compute::backends::for_mode;
use piramid_core::error::{IndexError, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct FlatIndex {
    config: FlatConfig,
    vector_ids: Vec<Uuid>,
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
    fn insert(&mut self, id: Uuid, _vector: &[f32], _vectors: &dyn VectorReader) -> Result<()> {
        // Only the id list; there is no structure to maintain.
        if !self.vector_ids.contains(&id) {
            self.vector_ids.push(id);
        }
        Ok(())
    }

    // Scans every vector. Filters are ignored here and applied by the caller after ranking —
    // with no traversal to prune, evaluating them mid-scan would save nothing.
    fn search(&self, request: crate::IndexSearchRequest<'_>) -> Result<Vec<Uuid>> {
        let crate::IndexSearchRequest {
            query, k, vectors, ..
        } = request;
        let kernels = for_mode(self.config.mode)?;
        let mut distances = Vec::with_capacity(self.vector_ids.len());
        for id in &self.vector_ids {
            let vec = vectors.get(id).ok_or_else(|| {
                IndexError::SearchFailed(format!("Flat index references missing vector {id}"))
            })?;
            let score = self.config.metric.calculate(query, vec, kernels);
            distances.push((*id, score));
        }

        distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
