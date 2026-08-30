//! Provider selection: turn an `EmbeddingConfig` into an `Embedder`.

use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;

use super::ollama::OllamaEmbedder;
use super::openai::OpenAIEmbedder;
use crate::cache::CachedEmbedder;
use crate::types::{Embedder, EmbeddingConfig, EmbeddingError, EmbeddingResult};

/// Entries kept per embedder. One number, applied here, rather than a constant per provider and
/// a `with_cache_size` constructor nothing called.
const CACHE_CAPACITY: usize = 10_000;

/// Providers this build can construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingProvider {
    /// Anything speaking the OpenAI embeddings format, including a local server.
    OpenAI,
    Ollama,
}

impl EmbeddingProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Ollama => "ollama",
        }
    }
}

impl FromStr for EmbeddingProvider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "openai" => Ok(Self::OpenAI),
            "ollama" => Ok(Self::Ollama),
            _ => Err(()),
        }
    }
}

/// Build the embedder named by `config`, wrapped in the response cache.
pub fn create_embedder(config: &EmbeddingConfig) -> EmbeddingResult<Arc<dyn Embedder>> {
    let provider = config.provider.parse::<EmbeddingProvider>().map_err(|_| {
        EmbeddingError::ConfigError(format!(
            "Unknown provider '{}'. Expected openai or ollama",
            config.provider
        ))
    })?;

    let capacity = NonZeroUsize::new(CACHE_CAPACITY).expect("CACHE_CAPACITY is a nonzero literal");
    Ok(match provider {
        EmbeddingProvider::OpenAI => {
            Arc::new(CachedEmbedder::new(OpenAIEmbedder::new(config)?, capacity))
        }
        EmbeddingProvider::Ollama => {
            Arc::new(CachedEmbedder::new(OllamaEmbedder::new(config)?, capacity))
        }
    })
}
