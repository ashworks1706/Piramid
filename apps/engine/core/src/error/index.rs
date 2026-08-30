use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Index not initialized")]
    NotInitialized,

    #[error("Index corrupted: {0}")]
    Corrupted(String),

    #[error("Invalid index configuration: {0}")]
    InvalidConfig(String),

    #[error("Index build failed: {0}")]
    BuildFailed(String),

    #[error("Search failed: {0}")]
    SearchFailed(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Invalid layer: {0}")]
    InvalidLayer(String),

    #[error("Index persistence failed: {0}")]
    PersistenceFailed(String),

    #[error("Index load failed: {0}")]
    LoadFailed(String),
}
