//! Retry wrapper for embedding providers.
//!
//! Retries only errors `EmbeddingError::is_recoverable` accepts, with exponential backoff. A
//! permanent failure — a bad API key, a malformed request — returns immediately rather than
//! burning the retry budget.

// wraps any Embedder implementation and adds retry logic with exponential backoff.
use crate::{Embedder, EmbeddingError, EmbeddingResponse, EmbeddingResult};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryEmbedder {
    inner: Arc<dyn Embedder>,
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
}

// Configuration options for the RetryEmbedder
#[derive(Clone, Debug)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

// Default: 3 retries, starting at 1 second, doubling each time, up to 30 seconds
impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }
}

// Implementation of the RetryEmbedder
impl RetryEmbedder {
    pub fn new(embedder: Arc<dyn Embedder>) -> Self {
        Self::with_options(embedder, RetryConfig::default())
    }

    pub fn with_options(embedder: Arc<dyn Embedder>, options: RetryConfig) -> Self {
        Self {
            inner: embedder,
            max_retries: options.max_retries,
            initial_delay_ms: options.initial_delay_ms,
            max_delay_ms: options.max_delay_ms,
        }
    }
}

#[async_trait]
impl Embedder for RetryEmbedder {
    async fn embed(&self, text: &str) -> EmbeddingResult<EmbeddingResponse> {
        let mut attempts = 0;
        let mut delay_ms = self.initial_delay_ms;

        // Loop to attempt embedding with retries
        loop {
            match self.inner.embed(text).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;

                    if !is_retryable_error(&e) || attempts > self.max_retries {
                        return Err(e);
                    }

                    tracing::warn!(
                        attempt = attempts,
                        max_retries = self.max_retries,
                        delay_ms,
                        error = %e,
                        "embedding_request_retrying"
                    );

                    sleep(Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms * 2).min(self.max_delay_ms);
                }
            }
        }
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }

    fn model_name(&self) -> &str {
        self.inner.model_name()
    }

    fn dimensions(&self) -> Option<usize> {
        self.inner.dimensions()
    }
}

// Determine if an error is retryable
fn is_retryable_error(error: &EmbeddingError) -> bool {
    error.is_recoverable()
}
