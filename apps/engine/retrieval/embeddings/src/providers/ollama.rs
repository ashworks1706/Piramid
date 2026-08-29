//! Ollama provider, for local embedding models.
//!
//! Known-good models: `nomic-embed-text` (768), `mxbai-embed-large` (1024), `all-minilm` (384).

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::cache::CachedEmbedder;
use crate::types::{Embedder, EmbeddingConfig, EmbeddingError, EmbeddingResponse, EmbeddingResult};

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
const DEFAULT_CACHE_SIZE: usize = 10000;

struct OllamaEmbedderInner {
    client: Client,
    model: String,
    base_url: String,
}

pub struct OllamaEmbedder {
    cached: CachedEmbedder<OllamaEmbedderInner>,
}

impl OllamaEmbedder {
    pub fn new(config: &EmbeddingConfig) -> EmbeddingResult<Self> {
        let inner = OllamaEmbedderInner::new(config)?;
        Ok(Self {
            cached: CachedEmbedder::new(inner, DEFAULT_CACHE_SIZE),
        })
    }

    pub fn with_cache_size(config: &EmbeddingConfig, cache_size: usize) -> EmbeddingResult<Self> {
        let inner = OllamaEmbedderInner::new(config)?;
        Ok(Self {
            cached: CachedEmbedder::new(inner, cache_size),
        })
    }
}

impl OllamaEmbedderInner {
    fn new(config: &EmbeddingConfig) -> EmbeddingResult<Self> {
        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_OLLAMA_URL.to_string());

        let client = if let Some(timeout_secs) = config.timeout {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .map_err(|e| EmbeddingError::RequestFailed(e.to_string()))?
        } else {
            Client::new()
        };

        Ok(Self {
            client,
            model: config.model.clone(),
            base_url,
        })
    }

    fn get_dimensions(&self) -> Option<usize> {
        match self.model.as_str() {
            "nomic-embed-text" => Some(768),
            "mxbai-embed-large" => Some(1024),
            "all-minilm" => Some(384),
            _ => None,
        }
    }
}

#[async_trait]
impl Embedder for OllamaEmbedderInner {
    async fn embed(&self, text: &str) -> EmbeddingResult<EmbeddingResponse> {
        let request = OllamaEmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let url = format!("{}/api/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbeddingError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(EmbeddingError::ApiError(format!(
                "{}: {}",
                status, error_text
            )));
        }

        let api_response: OllamaEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::InvalidResponse(e.to_string()))?;

        Ok(EmbeddingResponse {
            embedding: api_response.embedding,
            tokens: None, // Ollama doesn't report token usage
            model: self.model.clone(),
        })
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn dimensions(&self) -> Option<usize> {
        self.get_dimensions()
    }
}

#[async_trait]
impl Embedder for OllamaEmbedder {
    async fn embed(&self, text: &str) -> EmbeddingResult<EmbeddingResponse> {
        self.cached.embed(text).await
    }

    fn provider_name(&self) -> &str {
        self.cached.provider_name()
    }

    fn model_name(&self) -> &str {
        self.cached.model_name()
    }

    fn dimensions(&self) -> Option<usize> {
        self.cached.dimensions()
    }
}

#[derive(Debug, Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}
