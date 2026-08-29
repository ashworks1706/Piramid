use serde::{Deserialize, Serialize};

// Storage configuration for the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Root directory for collection data; each collection gets a subdirectory.
    pub storage_path: String,
}

impl StorageConfig {
    // new StorageConfig with specified storage path
    pub fn new(path: String) -> Self {
        Self { storage_path: path }
    }
}

impl Default for StorageConfig {
    // default storage configuration with a default storage path
    fn default() -> Self {
        Self::new("./data".to_string())
    }
}
