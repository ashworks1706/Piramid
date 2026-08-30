//! Embedding provider configuration.

use serde::{Deserialize, Serialize};

/// How to reach an embedding provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Provider name: `openai`, `ollama`, or `local`.
    pub provider: String,

    /// Model identifier as the provider understands it.
    pub model: String,

    /// API key, for providers that require one.
    pub api_key: Option<String>,

    /// Base URL, for self-hosted or proxied endpoints.
    pub base_url: Option<String>,

    /// Provider-specific options passed through verbatim.
    #[serde(default)]
    pub options: serde_json::Value,

    /// Request timeout in seconds.
    #[serde(default)]
    pub timeout: Option<u64>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "text-embedding-3-small".to_string(),
            api_key: None,
            base_url: None,
            options: serde_json::json!({}),
            timeout: None,
        }
    }
}
