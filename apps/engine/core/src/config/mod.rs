//! Configuration, resolved from defaults, an optional file, then the environment.

mod app;
mod cache;
mod collection;
mod embedding;
mod hardware;
mod index;
mod limits;
pub mod loader;

pub use loader::{ConfigError, RuntimeConfig};
mod logging;
mod memory;
mod parallelism;
mod quantization;
mod search;
mod search_mode;
mod storage;
mod tuning;
mod wal;

pub use app::AppConfig;
pub use cache::CacheConfig;
pub use collection::CollectionConfig;
pub use embedding::EmbeddingConfig;
// `ExecutionMode` is owned by `compute/` (it names a kernel backend, not app policy).
// Re-exported here so configuration callers keep a single import path.
pub use hardware::{HardwareConfig, HardwareProfile};
pub use index::{AutoIndexConfig, IndexConfig, IndexKind};
pub use limits::LimitsConfig;
pub use logging::{LogLevel, LoggingConfig};
pub use memory::MemoryConfig;
pub use parallelism::{ParallelismConfig, ParallelismMode};
pub use piramid_compute::ExecutionMode;
pub use quantization::{QuantizationConfig, QuantizationLevel, QuantizationStage};
pub use search::SearchConfig;
pub use search_mode::{RangeSearchParams, SearchMode};
pub use storage::StorageConfig;
pub use tuning::{AdaptiveTuningConfig, QueryBudgetConfig};
pub use wal::WalConfig;
