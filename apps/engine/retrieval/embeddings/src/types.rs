//! The `Embedder` trait and the types providers return.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use piramid_core::error::embedding::EmbeddingError;

pub type EmbeddingResult<T> = Result<T, EmbeddingError>;

pub use piramid_core::config::EmbeddingConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f32>,

    /// Token count, when the provider reports one.
    pub tokens: Option<u32>,

    pub model: String,
}

/// A provider that turns text into a vector.
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> EmbeddingResult<EmbeddingResponse>;

    fn provider_name(&self) -> &'static str;

    fn model_name(&self) -> &str;

    /// Vector width, when the provider declares one.
    fn dimensions(&self) -> Option<usize>;
}
