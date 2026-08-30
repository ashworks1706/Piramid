//! Memory limits and mmap settings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Byte ceiling per collection. `None` is unbounded.
    pub max_memory_per_collection: Option<usize>,

    /// Size the data file is first mapped at; it grows from here.
    pub initial_mmap_size: usize,

    /// Map the data file. With this off, records go through ordinary file reads.
    pub use_mmap: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            max_memory_per_collection: None, // Unlimited
            initial_mmap_size: 1024 * 1024,  // 1MB
            use_mmap: true,
        }
    }
}

impl MemoryConfig {
    pub fn with_limit_mb(limit_mb: usize) -> Self {
        MemoryConfig {
            max_memory_per_collection: Some(limit_mb * 1024 * 1024),
            initial_mmap_size: 1024 * 1024,
            use_mmap: true,
        }
    }

    pub fn with_mmap_size_mb(size_mb: usize) -> Self {
        MemoryConfig {
            max_memory_per_collection: None,
            initial_mmap_size: size_mb * 1024 * 1024,
            use_mmap: true,
        }
    }

    pub fn no_mmap() -> Self {
        MemoryConfig {
            max_memory_per_collection: None,
            initial_mmap_size: 0,
            use_mmap: false,
        }
    }
}
