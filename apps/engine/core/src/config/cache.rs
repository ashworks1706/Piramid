//! Cache limits, applied per collection and across the process.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct CacheConfig {
    pub enabled: bool,

    /// Entry ceiling for the metadata cache.
    pub metadata_entries: usize,

    /// Entry lifetime in seconds. `None` never expires.
    pub ttl_seconds: Option<u64>,

    /// Byte budget for resident vectors, shared across every loaded collection. `None` is unbounded.
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            enabled: true,
            metadata_entries: 10_000,
            ttl_seconds: None,
            max_bytes: None,
        }
    }
}

impl CacheConfig {
    pub fn with_size(size: usize) -> Self {
        CacheConfig {
            enabled: true,
            metadata_entries: size,
            ttl_seconds: None,
            max_bytes: None,
        }
    }
}
