mod config;
mod index;

pub use config::{HnswConfig, HnswStats};
pub use index::HnswIndex;

use crate::traits::{
    IndexDetails, IndexSearchRequest, IndexStats, IndexType, VectorIndex, VectorReader,
};
use piramid_core::Result;
use uuid::Uuid;

// we need a wrapper because HNSW has some specific parameters that affect search quality (ef_search) and we want to allow overriding them at search time without changing the index config
impl VectorIndex for HnswIndex {
    fn insert(&mut self, id: Uuid, vector: &[f32], vectors: &dyn VectorReader) {
        self.insert(id, vector, vectors);
    }

    // Search for nearest neighbors to the query vector with filters.
    fn search(&self, request: IndexSearchRequest<'_>) -> Result<Vec<Uuid>> {
        // Use the per-query `ef` override when present, otherwise the configured `ef_search`.
        let ef = request
            .config
            .ef
            .unwrap_or_else(|| self.get_ef_search())
            .max(request.k);
        self.search(
            request.query,
            request.k,
            ef,
            request.vectors,
            request.filter,
            request.metadata,
        )
    }

    fn remove(&mut self, id: &Uuid) {
        self.remove(id);
    }

    // Get statistics about the HNSW index, including total nodes, max layer, layer sizes, average connections, and memory usage.
    fn stats(&self) -> IndexStats {
        let hnsw_stats = self.stats();

        IndexStats {
            index_type: IndexType::Hnsw,
            total_vectors: hnsw_stats.total_nodes,
            memory_usage_bytes: hnsw_stats.memory_usage_bytes,
            details: IndexDetails::Hnsw {
                max_layer: hnsw_stats.max_layer,
                layer_sizes: hnsw_stats.layer_sizes,
                avg_connections: hnsw_stats.avg_connections,
            },
        }
    }

    fn index_type(&self) -> IndexType {
        IndexType::Hnsw
    }

    fn to_serializable(&self) -> crate::SerializableIndex {
        crate::SerializableIndex::Hnsw(self.clone())
    }
}
