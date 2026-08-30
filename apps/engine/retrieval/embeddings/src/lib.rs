//! Embedding providers.
//!
//! Turns text into vectors so callers can ingest documents without running a model themselves.

pub mod cache;
pub mod providers;
pub mod retry;
mod types;

pub use cache::{CacheStats, CachedEmbedder};
pub use piramid_core::error::embedding::EmbeddingError;
pub use providers::{create_embedder, EmbeddingProvider};
pub use retry::RetryEmbedder;
pub use types::{Embedder, EmbeddingConfig, EmbeddingResponse, EmbeddingResult};
