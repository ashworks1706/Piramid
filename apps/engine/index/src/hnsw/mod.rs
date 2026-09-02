mod index;

pub use index::HnswIndex;
pub use index::HnswStats;
pub use piramid_core::config::HnswConfig;

use crate::traits::{
    IndexDetails, IndexSearchRequest, IndexStats, IndexType, VectorIndex, VectorReader,
};
use piramid_core::Result;
use uuid::Uuid;

// `HnswIndex` has its own inherent `search` taking an explicit `ef`. This impl adapts the
// generic trait call to it, resolving `ef` from the per-query config or the index default.
impl VectorIndex for HnswIndex {
    fn insert(&mut self, id: Uuid, vector: &[f32], vectors: &dyn VectorReader) -> Result<()> {
        self.insert(id, vector, vectors)
    }

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
