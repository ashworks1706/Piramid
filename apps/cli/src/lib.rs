//! Umbrella crate: re-exports every workspace crate under one namespace. Crate boundaries and the
//! dependency rule are in docs/ARCHITECTURE.md.

pub use piramid_core::{config, document, error, metadata, observability, stats, validation};

pub use piramid_database::{cache, index, search, storage};
pub use piramid_hardware::compute::quantization;
pub use piramid_hardware::{compute, gpu};
pub use piramid_model::{embeddings, fusion, inference};
pub use piramid_serving::{cluster, disk, http as server, services, state};

pub use embeddings::EmbeddingsManager;
pub use gpu::GpuManager;
pub use inference::InferenceManager;
/// Domain managers: one per crate with state to own, each in the manager.rs of that crate.
/// AppState is the composition root that holds them.
pub use piramid_database::{CacheManager, CheckpointManager, CollectionManager};
pub use state::AppState;
pub use storage::SidecarManager;

pub use compute::{
    cosine_similarity, dot_product, euclidean_distance, euclidean_distance_squared, ComputeError,
    DistanceKernels, ExecutionMode, Metric,
};
pub use config::*;
pub use document::{Document, Hit};
pub use embeddings::{EmbeddingConfig, EmbeddingError, EmbeddingProvider};
pub use error::{ErrorContext, PiramidError, Result};
pub use gpu::{Device, DeviceBuffer, GpuError, Stream};
pub use index::{
    FlatConfig, FlatIndex, HashMapVectorReader, HnswConfig, HnswIndex, IndexConfig,
    IndexSearchRequest, IndexStats, IndexType, IvfConfig, IvfIndex, MetadataReader, VectorIndex,
    VectorReader,
};
pub use metadata::{metadata, Filter, FilterCondition, Metadata, MetadataValue};
pub use piramid_database::Collection;
pub use quantization::QuantizedVector;
pub use search::SearchParams;
pub use storage::CollectionMetadata;
