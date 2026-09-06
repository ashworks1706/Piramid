//! The Embedder contract and what a provider returns.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use piramid_core::error::embedding::EmbeddingError;

/// What a provider call returns.
pub type EmbeddingResult<T> = Result<T, EmbeddingError>;

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
