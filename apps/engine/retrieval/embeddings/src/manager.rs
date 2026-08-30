//! The embeddings domain entry.

use std::sync::Arc;

use crate::providers::create_embedder;
use crate::retry::RetryEmbedder;
use crate::types::{Embedder, EmbeddingConfig, EmbeddingResult};
use piramid_core::stats::EmbedMetrics;

/// Owns the embedding stack and its throughput counters.
///
/// One of these lives in `AppState`. Anything new the embedding domain holds — a second
/// provider, a failover policy, per-collection models — becomes a field here.
pub struct EmbeddingsManager {
    embedder: Option<Arc<dyn Embedder>>,
    metrics: EmbedMetrics,
}

impl EmbeddingsManager {
    /// A manager with no provider configured; every embed request reports unavailable.
    pub fn disabled() -> Self {
        Self {
            embedder: None,
            metrics: EmbedMetrics::default(),
        }
    }

    /// Build the full stack `config` names: provider, response cache, retries.
    pub fn from_config(config: &EmbeddingConfig) -> EmbeddingResult<Self> {
        let embedder = create_embedder(config)?;
        Ok(Self {
            embedder: Some(Arc::new(RetryEmbedder::new(embedder))),
            metrics: EmbedMetrics::default(),
        })
    }

    /// Whether a provider is configured.
    pub fn is_configured(&self) -> bool {
        self.embedder.is_some()
    }

    /// The configured embedder, if any.
    pub fn embedder(&self) -> Option<&Arc<dyn Embedder>> {
        self.embedder.as_ref()
    }

    /// Throughput counters for the embedding path.
    pub fn metrics(&self) -> &EmbedMetrics {
        &self.metrics
    }
}
