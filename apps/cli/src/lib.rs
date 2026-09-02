//! Umbrella crate: re-exports every workspace crate under one namespace. See `docs/ARCHITECTURE.md`
//! for crate boundaries and the dependency rule.

pub use piramid_core::{config, error, metadata, stats, validation};

pub use piramid_collections as collections;
pub use piramid_collections::cache;
pub use piramid_compute as compute;
pub use piramid_compute::quantization;
pub use piramid_embeddings as embeddings;
pub use piramid_gpu as gpu;
pub use piramid_index as index;
pub use piramid_inference as inference;
pub use piramid_observability as observability;
pub use piramid_search as search;
pub use piramid_server::{cluster, disk, http as server, services, state};
pub use piramid_storage as storage;

/// Domain managers: one per crate with state to own, each in its crate's `manager.rs`
/// `AppState` is the composition root that holds them, not one of them.
pub use collections::{CacheManager, CheckpointManager, CollectionManager};
pub use embeddings::EmbeddingsManager;
pub use gpu::GpuManager;
pub use inference::InferenceManager;
pub use state::AppState;
pub use storage::SidecarManager;

pub use collections::Collection;
pub use compute::{
    cosine_similarity, dot_product, euclidean_distance, euclidean_distance_squared, ComputeError,
    DistanceKernels, ExecutionMode, Metric,
};
pub use config::*;
pub use embeddings::{EmbeddingConfig, EmbeddingError, EmbeddingProvider};
pub use error::{ErrorContext, PiramidError, Result};
pub use gpu::{Device, DeviceBuffer, GpuError, Stream};
pub use index::{
    FlatConfig, FlatIndex, HashMapVectorReader, HnswConfig, HnswIndex, IndexConfig,
    IndexSearchRequest, IndexStats, IndexType, IvfConfig, IvfIndex, MetadataReader, VectorIndex,
    VectorReader,
};
pub use metadata::{metadata, Filter, FilterCondition, Metadata, MetadataValue};
pub use quantization::QuantizedVector;
pub use search::{Hit, SearchParams};
pub use storage::{CollectionMetadata, Document};
