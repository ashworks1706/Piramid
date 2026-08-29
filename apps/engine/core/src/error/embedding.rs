//! Embedding provider errors.
//!
//! `is_recoverable` distinguishes transient failures worth retrying from permanent ones; see
//! `retry::RetryEmbedder`, which is its only consumer.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),
}

impl EmbeddingError {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::RequestFailed(_) => true,
            Self::ApiError(_) => true,
            Self::InvalidResponse(_) => true,
            Self::ConfigError(_) => false,
            Self::RateLimitExceeded => true,
            Self::AuthenticationFailed(_) => false,
            Self::ProviderUnavailable(_) => true,
            Self::Timeout(_) => true,
            Self::InvalidModel(_) => false,
        }
    }
}
