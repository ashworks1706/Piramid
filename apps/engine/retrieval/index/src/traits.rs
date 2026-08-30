//! The contract every ANN index implements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use piramid_core::config::SearchConfig;
use piramid_core::error::Result;
use piramid_core::metadata::Filter;
use piramid_core::metadata::Metadata;

pub use piramid_storage::vectors::{HashMapVectorReader, SlabVectorReader, VectorReader};

/// Read-only access to per-document metadata during a search.
///
/// The counterpart to [`VectorReader`]. Indexes that evaluate filters mid-traversal read through
/// this rather than taking a concrete map, so the backing store can change without touching any
/// index implementation.
///
/// `Sync` for the same reason as [`VectorReader`]: batch search shares one view across threads.
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
///
/// A struct rather than a parameter list so that adding a field later — a device handle, a
/// deadline, a quantization hint — does not change [`VectorIndex::search`]'s signature and break
/// every implementation.
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
///
/// Implementations own traversal structure only. They do **not** own the vectors themselves —
/// those arrive through [`VectorReader`] — so the same index can sit over a cache-backed,
/// slab-backed, or device-resident store without changing.
pub trait VectorIndex: Send + Sync {
    /// Add `vector` under `id`.
    ///
    /// `vectors` gives access to already-indexed vectors, which graph indexes need to compute
    /// distances while linking the new node. Errors if the configured compute backend is not
    /// available on this machine.
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

/// Statistics about an index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Index family.
    pub index_type: IndexType,
    /// Number of indexed vectors.
    pub total_vectors: usize,
    /// Approximate resident size in bytes.
    pub memory_usage_bytes: usize,
    /// Family-specific detail.
    pub details: IndexDetails,
}

/// Family-specific index statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IndexDetails {
    /// Flat indexes have no structure to report.
    Flat,
    /// Graph shape for HNSW.
    Hnsw {
        /// Highest occupied layer.
        max_layer: isize,
        /// Node count per layer.
        layer_sizes: Vec<usize>,
        /// Mean out-degree.
        avg_connections: f32,
    },
    /// Partition shape for IVF.
    Ivf {
        /// Number of partitions.
        num_clusters: usize,
        /// Vectors assigned to each partition.
        vectors_per_cluster: Vec<usize>,
        /// Whether centroids have been trained.
        centroids_computed: bool,
    },
}

/// Supported index families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexType {
    /// Brute-force linear scan. `O(N)`; best under ~10k vectors.
    Flat,
    /// Hierarchical Navigable Small World graph. `O(log N)`; best above ~100k vectors.
    Hnsw,
    /// Inverted file index. `O(sqrt N)`; best between ~10k and ~1M vectors.
    Ivf,
}

impl std::fmt::Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::Flat => write!(f, "Flat"),
            IndexType::Hnsw => write!(f, "HNSW"),
            IndexType::Ivf => write!(f, "IVF"),
        }
    }
}

/// Persistable form of any index.
///
/// # Extension cost
///
/// Unlike [`VectorIndex`], this enum is deliberately closed: it defines the on-disk sidecar
/// format, and an open registry would make stored indexes unreadable by a build that lacks the
/// implementation that wrote them. Adding an index family therefore means adding a variant here,
/// and old binaries will refuse to load the new variant. That is the intended trade — a loud
/// failure rather than a silent one.
#[derive(Serialize, Deserialize)]
pub enum SerializableIndex {
    /// A flat index.
    Flat(crate::flat::FlatIndex),
    /// An HNSW index.
    Hnsw(crate::hnsw::HnswIndex),
    /// An IVF index.
    Ivf(crate::ivf::IvfIndex),
}

impl SerializableIndex {
    /// Restore a boxed trait object from the persisted form.
    pub fn to_trait_object(self) -> Box<dyn VectorIndex> {
        match self {
            SerializableIndex::Flat(idx) => Box::new(idx),
            SerializableIndex::Hnsw(idx) => Box::new(idx),
            SerializableIndex::Ivf(idx) => Box::new(idx),
        }
    }
}
