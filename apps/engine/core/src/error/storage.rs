use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Vector not found: {0}")]
    VectorNotFound(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Collection already exists: {0}")]
    CollectionExists(String),

    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },

    #[error("Invalid vector data: {0}")]
    InvalidVectorData(String),

    #[error("Invalid collection path: {0}")]
    InvalidPath(String),

    #[error("Storage corruption detected: {0}")]
    CorruptedData(String),

    #[error("Storage full: {0}")]
    StorageFull(String),

    #[error("Index file corrupted: {0}")]
    CorruptedIndex(String),

    #[error("Memory map error: {0}")]
    MemoryMapError(String),

    #[error("Lock acquisition failed: {0}")]
    LockFailed(String),

    #[error("Write operation failed: {0}")]
    WriteFailed(String),

    #[error("Read operation failed: {0}")]
    ReadFailed(String),
}
