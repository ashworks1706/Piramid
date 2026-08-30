//! Where collection data lives on disk.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Root directory for collection data; each collection gets a subdirectory.
    pub storage_path: String,
}

impl StorageConfig {
    pub fn new(path: String) -> Self {
        Self { storage_path: path }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::new("./data".to_string())
    }
}
