//! Embedding provider errors.

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
            Self::RequestFailed(_)
            | Self::ApiError(_)
            | Self::InvalidResponse(_)
            | Self::RateLimitExceeded
            | Self::ProviderUnavailable(_)
            | Self::Timeout(_) => true,
            Self::ConfigError(_) | Self::AuthenticationFailed(_) | Self::InvalidModel(_) => false,
        }
    }
}
