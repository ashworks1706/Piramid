//! Per-collection configuration.

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub index: crate::config::IndexConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub quantization: QuantizationConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
    #[serde(default)]
    pub wal: WalConfig,
    #[serde(default)]
    pub parallelism: ParallelismConfig,
    #[serde(default)]
    pub execution: ExecutionMode,
    #[serde(default)]
    pub hardware: HardwareConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        CollectionConfig {
            index: crate::config::IndexConfig::default(),
            search: SearchConfig::default(),
            quantization: QuantizationConfig::default(),
            memory: MemoryConfig::default(),
            wal: WalConfig::default(),
            parallelism: ParallelismConfig::default(),
            execution: ExecutionMode::Auto,
            hardware: HardwareConfig::default(),
            limits: LimitsConfig::default(),
            cache: CacheConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl CollectionConfig {
    pub fn with_int8_quantization(mut self) -> Self {
        self.quantization = QuantizationConfig::int8();
        self
    }

    pub fn single_threaded(mut self) -> Self {
        self.parallelism = ParallelismConfig::single_threaded();
        self
    }
}
