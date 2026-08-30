//! Embedding providers: turns text into vectors.

pub mod cache;
pub mod providers;
pub mod retry;
mod types;

pub use cache::CachedEmbedder;
pub use piramid_core::error::embedding::EmbeddingError;
pub use providers::{create_embedder, EmbeddingProvider};
pub use retry::RetryEmbedder;
pub use types::{Embedder, EmbeddingConfig, EmbeddingResponse, EmbeddingResult};
