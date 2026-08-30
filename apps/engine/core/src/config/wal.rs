//! Write-ahead log configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WalConfig {
    pub enabled: bool,

    /// Checkpoint after this many operations.
    pub checkpoint_frequency: usize,

    /// Also checkpoint after this many seconds, if set.
    #[serde(default)]
    pub checkpoint_interval_secs: Option<u64>,

    /// Rotate once the log passes this many bytes.
    pub max_log_size: usize,

    /// `fsync` every write. Durable across power loss, and much slower.
    pub sync_on_write: bool,
}

impl Default for WalConfig {
    fn default() -> Self {
        WalConfig {
            enabled: true,
            checkpoint_frequency: 1000,
            checkpoint_interval_secs: None,
            max_log_size: 100 * 1024 * 1024,
            sync_on_write: false,
        }
    }
}

impl WalConfig {
    pub fn disabled() -> Self {
        WalConfig {
            enabled: false,
            checkpoint_frequency: 0,
            max_log_size: 0,
            sync_on_write: false,
            checkpoint_interval_secs: None,
        }
    }

    pub fn high_durability() -> Self {
        WalConfig {
            enabled: true,
            checkpoint_frequency: 100,
            max_log_size: 50 * 1024 * 1024,
            sync_on_write: true,
            checkpoint_interval_secs: Some(1),
        }
    }

    pub fn fast() -> Self {
        WalConfig {
            enabled: true,
            checkpoint_frequency: 10000,
            max_log_size: 500 * 1024 * 1024,
            sync_on_write: false,
            checkpoint_interval_secs: None,
        }
    }
}
