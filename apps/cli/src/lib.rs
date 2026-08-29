//! Piramid — an inference-native retrieval engine.
//!
//! This is the umbrella crate: it re-exports every workspace crate under one namespace so a user
//! writes `use piramid::Collection` and never has to know how the workspace is cut. The `piramid`
//! binary lives in the same package so `cargo install piramid` keeps working.
//!
//! # Workspace layout
//!
//! Crates are grouped by ownership. A crate may depend on one listed below it; the reverse is a
//! layering violation, and `scripts/check-deps.sh` fails CI if it appears.
//!
//! | Crate | Owns |
//! |---|---|
//! | [`server`] | HTTP transport, services, runtime state, cluster routing |
//! | [`inference`] | Model execution and retrieval fusion |
//! | [`collections`] | The collection domain object, caching, checkpoints, compaction |
//! | [`embeddings`] | Embedding providers |
//! | [`search`] | Query planning, filtering, ranking |
//! | [`index`] | ANN indexes: flat, HNSW, IVF |
//! | [`storage`] | Records, WAL, sidecars, mmap, vector layout, quantization |
//! | [`compute`] | Distance math and CPU/GPU backend dispatch |
//! | [`gpu`] | Device runtime: contexts, buffers, streams, kernels |
//! | [`stats`] | What the engine measures about itself |
//! | [`observability`] | Where those measurements go: subscriber, OTLP, Prometheus |
//! | [`config`], [`error`], [`mod@metadata`], [`stats`], [`validation`] | `piramid-core` |
//!
//! Two rules worth knowing before adding code:
//!
//! **`compute` and `gpu` are leaves.** They depend on nothing in the workspace. `config` carries
//! an [`ExecutionMode`] but gets it *from* `compute`, not the other way round.
//!
//! **`gpu` owns the device; `compute` and `inference` both borrow it.** Vendor SDK types stay
//! inside `gpu/backends/`. That is what lets retrieval and generation share one
//! [`Device`] so vectors and model weights sit in the same address space.

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
