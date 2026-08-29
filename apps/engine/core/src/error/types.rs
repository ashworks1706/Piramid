use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PiramidError>;

/// What kind of failure occurred, independent of any wire protocol.
///
/// Transports map these onto their own status codes so no library crate has to know about HTTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The request was malformed or failed validation.
    BadRequest,
    /// The requested resource does not exist.
    NotFound,
    /// The resource already exists.
    Conflict,
    /// Credentials were missing or invalid.
    Unauthenticated,
    /// Credentials were valid but insufficient.
    Forbidden,
    /// The caller exceeded a rate limit.
    RateLimited,
    /// The operation timed out.
    Timeout,
    /// A dependency the server calls out to failed.
    Upstream,
    /// The server is temporarily unable to serve.
    Unavailable,
    /// An unexpected internal failure.
    Internal,
}

#[derive(Error, Debug)]
pub enum PiramidError {
    // Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] super::storage::StorageError),

    // Index errors
    #[error("Index error: {0}")]
    Index(#[from] super::index::IndexError),

    // Server/API errors
    #[error("Server error: {0}")]
    Server(#[from] super::server::ServerError),

    // Embedding errors
    #[error("Embedding error: {0}")]
    Embedding(#[from] super::embedding::EmbeddingError),

    // IO errors
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    // Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    // JSON errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    // Generic errors
    #[error("{0}")]
    Other(String),
}

impl PiramidError {
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }

    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Storage(e) => e.is_recoverable(),
            Self::Index(e) => e.is_recoverable(),
            Self::Server(e) => e.is_recoverable(),
            Self::Embedding(e) => e.is_recoverable(),
            Self::Io(_) => false,
            Self::Serialization(_) => false,
            Self::Json(_) => false,
            Self::Other(_) => false,
        }
    }

    /// Transport-agnostic classification.
    ///
    /// Mapping a kind onto a protocol status is the transport's job — see
    /// `piramid_server::http::ApiError`.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Server(e) => e.kind(),
            Self::Embedding(_) => ErrorKind::Upstream,
            Self::Storage(_) | Self::Index(_) | Self::Io(_) => ErrorKind::Internal,
            Self::Serialization(_) | Self::Json(_) | Self::Other(_) => ErrorKind::Internal,
        }
    }
}
