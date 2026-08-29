// Types for embedding system

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use piramid_core::error::embedding::EmbeddingError;

pub type EmbeddingResult<T> = Result<T, EmbeddingError>;

pub use piramid_core::config::EmbeddingConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    // The embedding vector
    pub embedding: Vec<f32>,

    // Number of tokens used if reported by provider
    pub tokens: Option<u32>,

    // Model that generated the embedding
    pub model: String,
}

// Trait for embedding providers
#[async_trait]
pub trait Embedder: Send + Sync {
    // an embedding for a single text
    async fn embed(&self, text: &str) -> EmbeddingResult<EmbeddingResponse>;

    // embeddings for multiple texts in a batch

    fn provider_name(&self) -> &str;

    fn model_name(&self) -> &str;

    // Get the expected dimension of embeddings
    fn dimensions(&self) -> Option<usize>;
}
