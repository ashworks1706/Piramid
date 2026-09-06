//! The collection manifest: name, dimensionality, counts, timestamps.

use piramid_core::error::{Result, StorageError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub schema_version: u32,
    pub name: String,
    pub created_at: u64, // Unix timestamp (seconds)
    pub updated_at: u64,
    pub dimensions: Option<usize>,
    pub vector_count: usize,
}

pub const SCHEMA_VERSION: u32 = 1;

impl CollectionMetadata {
    pub fn new(name: String) -> Self {
        let now = piramid_core::clock::unix_secs();

        Self {
            schema_version: SCHEMA_VERSION,
            name,
            created_at: now,
            updated_at: now,
            dimensions: None,
            vector_count: 0,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = piramid_core::clock::unix_secs();
    }

    /// Records the vector width of the collection the first time a vector is stored.
    pub fn set_dimensions(&mut self, dimensions: usize) -> Result<()> {
        match self.dimensions {
            None => {
                self.dimensions = Some(dimensions);
                Ok(())
            }
            Some(existing) if existing == dimensions => Ok(()),
            Some(existing) => Err(StorageError::InvalidDimension {
                expected: existing,
                actual: dimensions,
            }
            .into()),
        }
    }

    pub fn update_vector_count(&mut self, count: usize) {
        self.vector_count = count;
        self.touch();
    }
}
