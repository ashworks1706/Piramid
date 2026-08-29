// Supports: HNSW, Flat, IVF

pub mod flat;
pub mod hnsw;
pub mod ivf;
mod persistence;
mod selector;
mod traits;

// Re-export trait and types
pub use persistence::{get_index_file_path, load_vector_index, save_vector_index};
pub use piramid_core::config::{AutoIndexConfig, IndexConfig, IndexKind};
pub use selector::create_index;
pub use traits::{
    HashMapVectorReader, IndexDetails, IndexSearchRequest, IndexStats, IndexType, MetadataReader,
    SerializableIndex, SlabVectorReader, VectorIndex, VectorReader,
};

// Re-export index implementations
pub use flat::{FlatConfig, FlatIndex};
pub use hnsw::{HnswConfig, HnswIndex, HnswStats};
pub use ivf::{IvfConfig, IvfIndex};
