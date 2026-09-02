//! Configuration, resolved from defaults, an optional file, then the environment.
//!
//! The file has two blocks and the split is by lifecycle: [`StartupConfig`] is fixed at boot,
//! [`RuntimeConfig`] is re-read on reload. See `config.example.yaml` for the whole surface.

mod cache;
mod collection;
mod disk;
mod embedding;
mod file;
mod hardware;
mod index;
mod index_params;
mod inference;
mod limits;
pub mod loader;
mod logging;
mod memory;
mod runtime;
mod search;
mod search_mode;
mod startup;
mod storage;
mod telemetry;
mod wal;

pub use cache::CacheConfig;
pub use collection::CollectionConfig;
pub use disk::DiskConfig;
pub use embedding::EmbeddingConfig;
pub use file::Config;
pub use hardware::{HardwareConfig, HardwareProfile};
pub use index::{AutoIndexConfig, IndexConfig, IndexKind};
pub use index_params::{FlatConfig, HnswConfig, IvfConfig};
pub use inference::{AugmentConfig, InferenceConfig, KvCacheConfig, SamplingConfig};
pub use limits::LimitsConfig;
pub use logging::{LogLevel, LoggingConfig};
pub use memory::MemoryConfig;
pub use runtime::RuntimeConfig;
pub use search::SearchConfig;
pub use search_mode::{RangeSearchParams, SearchMode};
pub use startup::StartupConfig;
pub use storage::StorageConfig;
pub use telemetry::{OtlpConfig, TelemetryConfig};
pub use wal::WalConfig;

// `ExecutionMode` and the quantization types are owned by `compute/` (they name kernel
// behaviour, not app policy). Re-exported here so configuration callers keep one import path.
pub use piramid_hardware::compute::ExecutionMode;
pub use piramid_hardware::quantization::{
    QuantizationConfig, QuantizationLevel, QuantizationStage,
};

/// Shared `#[serde(default = ...)]` target for fields that default to on.
pub(crate) fn default_true() -> bool {
    true
}
