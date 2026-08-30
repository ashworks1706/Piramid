use thiserror::Error;

use super::types::ErrorKind;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Request timeout")]
    Timeout,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl ServerError {
    /// Transport-agnostic classification for this error.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidRequest(_) | Self::ValidationFailed(_) => ErrorKind::BadRequest,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::AlreadyExists(_) => ErrorKind::Conflict,
            Self::AuthenticationFailed(_) => ErrorKind::Unauthenticated,
            Self::AuthorizationFailed(_) => ErrorKind::Forbidden,
            Self::RateLimitExceeded => ErrorKind::RateLimited,
            Self::Timeout => ErrorKind::Timeout,
            Self::Internal(_) => ErrorKind::Internal,
            Self::ServiceUnavailable(_) => ErrorKind::Unavailable,
        }
    }
}
