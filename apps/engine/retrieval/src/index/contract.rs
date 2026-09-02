//! What an index must do, and what it reads to do it.

use std::collections::HashMap;

use piramid_core::config::SearchConfig;
use piramid_core::error::Result;
use piramid_core::metadata::{Filter, Metadata};
use uuid::Uuid;

use super::serialize::SerializableIndex;
use super::stats::{IndexStats, IndexType};

pub use piramid_database::storage::vectors::{HashMapVectorReader, VectorReader};

/// Read-only access to per-document metadata during a search.
pub trait MetadataReader: Sync {
    /// Metadata for `id`, if present.
    fn get(&self, id: &Uuid) -> Option<&Metadata>;
}

impl MetadataReader for HashMap<Uuid, Metadata> {
    fn get(&self, id: &Uuid) -> Option<&Metadata> {
        HashMap::get(self, id)
    }
}

/// Everything an index needs to answer one query.
pub struct IndexSearchRequest<'a> {
    /// Query vector.
    pub query: &'a [f32],
    /// Number of neighbors to return.
    pub k: usize,
    /// Access to the collection's vectors.
    pub vectors: &'a dyn VectorReader,
    /// Recall/speed knobs for this query.
    pub config: SearchConfig,
    /// Optional metadata predicate, applied during traversal where the index supports it.
    pub filter: Option<&'a Filter>,
    /// Access to per-document metadata, for evaluating `filter`.
    pub metadata: &'a dyn MetadataReader,
}

impl<'a> IndexSearchRequest<'a> {
    /// Build an unfiltered request.
    pub fn new(
        query: &'a [f32],
        k: usize,
        vectors: &'a dyn VectorReader,
        config: SearchConfig,
        metadata: &'a dyn MetadataReader,
    ) -> Self {
        Self {
            query,
            k,
            vectors,
            config,
            filter: None,
            metadata,
        }
    }

    /// Attach a metadata filter.
    pub fn with_filter(mut self, filter: Option<&'a Filter>) -> Self {
        self.filter = filter;
        self
    }
}

/// An approximate-nearest-neighbor index over a collection's vectors.
pub trait VectorIndex: Send + Sync {
    /// Add `vector` under `id`.
    fn insert(&mut self, id: Uuid, vector: &[f32], vectors: &dyn VectorReader) -> Result<()>;

    /// Return up to `request.k` neighbor ids, nearest first.
    fn search(&self, request: IndexSearchRequest<'_>) -> Result<Vec<Uuid>>;

    /// Remove `id` from the index.
    fn remove(&mut self, id: &Uuid);

    /// Current index statistics.
    fn stats(&self) -> IndexStats;

    /// Which family this index belongs to.
    fn index_type(&self) -> IndexType;

    /// Convert into the persistable form.
    fn to_serializable(&self) -> SerializableIndex;
}
