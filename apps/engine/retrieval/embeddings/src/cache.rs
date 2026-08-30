//! An `Embedder` wrapper that caches by text, evicting least-recently-used entries.

use async_trait::async_trait;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

use super::types::{Embedder, EmbeddingResponse, EmbeddingResult};

pub struct CachedEmbedder<E: Embedder> {
    inner: E,
    cache: Mutex<LruCache<String, Vec<f32>>>,
}

impl<E: Embedder> CachedEmbedder<E> {
    pub fn new(embedder: E, capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(10000).unwrap());
        Self {
            inner: embedder,
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }

    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        CacheStats {
            size: cache.len(),
            capacity: cache.cap().get(),
        }
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

#[async_trait]
impl<E: Embedder> Embedder for CachedEmbedder<E> {
    async fn embed(&self, text: &str) -> EmbeddingResult<EmbeddingResponse> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(embedding) = cache.get(text) {
                return Ok(EmbeddingResponse {
                    embedding: embedding.clone(),
                    tokens: None, // We don't track tokens for cached results
                    model: self.inner.model_name().to_string(),
                });
            }
        }

        let response = self.inner.embed(text).await?;

        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(text.to_string(), response.embedding.clone());
        }

        Ok(response)
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

/// Hit and miss counts for a [`CachedEmbedder`].
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,     // Current number of cached items
    pub capacity: usize, // Maximum capacity
}

impl CacheStats {
    pub fn hit_rate_estimate(&self) -> f32 {
        if self.capacity == 0 {
            0.0
        } else {
            self.size as f32 / self.capacity as f32
        }
    }
}
