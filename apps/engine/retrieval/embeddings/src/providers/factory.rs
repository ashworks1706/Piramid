//! Provider selection: turn an `EmbeddingConfig` into an `Embedder`.

use std::str::FromStr;
use std::sync::Arc;

use super::local::LocalEmbedder;
use super::ollama::OllamaEmbedder;
use super::openai::OpenAIEmbedder;
use crate::types::{Embedder, EmbeddingConfig, EmbeddingError, EmbeddingResult};

/// Providers this build can construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingProvider {
    OpenAI,
    Ollama,
    Local,
}

impl EmbeddingProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Ollama => "ollama",
            Self::Local => "local",
        }
    }
}

impl FromStr for EmbeddingProvider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "ollama" => Ok(Self::Ollama),
            "local" => Ok(Self::Local),
            _ => Err(()),
        }
    }
}

/// Build the embedder named by `config`, wrapped in whatever caching the provider does.
pub fn create_embedder(config: &EmbeddingConfig) -> EmbeddingResult<Arc<dyn Embedder>> {
    let provider = config.provider.parse::<EmbeddingProvider>().map_err(|_| {
        EmbeddingError::ConfigError(format!("Unknown provider: {}", config.provider))
    })?;

    match provider {
        EmbeddingProvider::OpenAI => {
            let embedder = OpenAIEmbedder::new(config)?;
            Ok(Arc::new(embedder))
        }
        EmbeddingProvider::Ollama => {
            let embedder = OllamaEmbedder::new(config)?;
            Ok(Arc::new(embedder))
        }
        EmbeddingProvider::Local => {
            let embedder = LocalEmbedder::new(config)?;
            Ok(Arc::new(embedder))
        }
    }
}
