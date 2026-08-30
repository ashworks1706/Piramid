//! Umbrella crate: re-exports every workspace crate under one namespace. See `docs/ARCHITECTURE.md`
//! for crate boundaries and the dependency rule.

pub use piramid_core::{config, error, metadata, stats, validation};

pub use piramid_collections as collections;
pub use piramid_collections::cache;
pub use piramid_compute as compute;
pub use piramid_embeddings as embeddings;
pub use piramid_gpu as gpu;
pub use piramid_index as index;
pub use piramid_inference as inference;
pub use piramid_observability as observability;
pub use piramid_search as search;
pub use piramid_server::{cluster, http as server, runtime, services};
pub use piramid_storage as storage;
pub use piramid_storage::quantization;

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
    IndexSearchRequest, IndexStats, IndexType, IvfConfig, IvfIndex, MetadataReader,
    SlabVectorReader, VectorIndex, VectorReader,
};
pub use metadata::{metadata, Filter, FilterCondition, Metadata, MetadataValue};
pub use quantization::QuantizedVector;
pub use search::{Hit, SearchParams};
pub use storage::vectors::VectorSlab;
pub use storage::{CollectionMetadata, Document};
