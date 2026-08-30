//! Cache limits, applied per collection and across the process.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,

    /// Item ceiling for the metadata cache.
    pub max_size: usize,

    /// Entry lifetime in seconds. `None` never expires.
    pub ttl_seconds: Option<u64>,

    /// Byte budget shared across every loaded collection. `None` is unbounded.
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            enabled: true,
            max_size: 10_000,
            ttl_seconds: None,
            max_bytes: None,
        }
    }
}

impl CacheConfig {
    pub fn with_size(size: usize) -> Self {
        CacheConfig {
            enabled: true,
            max_size: size,
            ttl_seconds: None,
            max_bytes: None,
        }
    }
}
